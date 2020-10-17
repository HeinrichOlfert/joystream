#!/usr/bin/env node
/* es-lint disable  */

'use strict'

// Node requires
const path = require('path')

// npm requires
const meow = require('meow')
const chalk = require('chalk')
const figlet = require('figlet')
const _ = require('lodash')
const { sleep } = require('@joystream/storage-utils/sleep')

const debug = require('debug')('joystream:colossus')

// Project root
const PROJECT_ROOT = path.resolve(__dirname, '..')

// Number of milliseconds to wait between synchronization runs.
const SYNC_PERIOD_MS = 120000 // 2min

// Parse CLI
const FLAG_DEFINITIONS = {
  port: {
    type: 'number',
    alias: 'p',
    default: 3000,
  },
  keyFile: {
    type: 'string',
    isRequired: (flags, input) => {
      // Only required if running server command and not in dev mode
      const serverCmd = input[0] === 'server'
      return !flags.dev && serverCmd
    },
  },
  publicUrl: {
    type: 'string',
    alias: 'u',
    isRequired: (flags, input) => {
      // Only required if running server command and not in dev mode
      const serverCmd = input[0] === 'server'
      return !flags.dev && serverCmd
    },
  },
  passphrase: {
    type: 'string',
  },
  wsProvider: {
    type: 'string',
    default: 'ws://localhost:9944',
  },
  providerId: {
    type: 'number',
    alias: 'i',
    isRequired: (flags, input) => {
      // Only required if running server command and not in dev mode
      const serverCmd = input[0] === 'server'
      return !flags.dev && serverCmd
    },
  },
  ipfsHost: {
    type: 'string',
    default: 'localhost',
  },
}

const cli = meow(
  `
  Usage:
    $ colossus [command] [arguments]

  Commands:
    server        Runs a production server instance. (discovery and storage services)
                  This is the default command if not specified.
    discovery     Run the discovery service only.

  Arguments (required for server. Ignored if running server with --dev option):
    --provider-id ID, -i ID     StorageProviderId assigned to you in working group.
    --key-file FILE             JSON key export file to use as the storage provider (role account).
    --public-url=URL, -u URL    API Public URL to announce.

  Arguments (optional):
    --dev                   Runs server with developer settings.
    --passphrase            Optional passphrase to use to decrypt the key-file.
    --port=PORT, -p PORT    Port number to listen on, defaults to 3000.
    --ws-provider WS_URL    Joystream-node websocket provider, defaults to ws://localhost:9944
    --ipfs-host   hostname  ipfs host to use, default to 'localhost'. Default port 5001 is always used
  `,
  { flags: FLAG_DEFINITIONS }
)

// All-important banner!
function banner() {
  console.log(chalk.blue(figlet.textSync('joystream', 'Speed')))
}

function startExpressApp(app, port) {
  const http = require('http')
  const server = http.createServer(app)

  return new Promise((resolve, reject) => {
    server.on('error', reject)
    server.on('close', (...args) => {
      console.log('Server closed, shutting down...')
      resolve(...args)
    })
    server.on('listening', () => {
      console.log('API server started.', server.address())
    })
    server.listen(port, '::')
    console.log('Starting API server...')
  })
}

// Start app
function startAllServices({ store, api, port, discoveryClient, ipfsHttpGatewayUrl }) {
  const app = require('../lib/app')(PROJECT_ROOT, store, api, discoveryClient, ipfsHttpGatewayUrl)
  return startExpressApp(app, port)
}

// Start discovery service app only
function startDiscoveryService({ port, discoveryClient }) {
  const app = require('../lib/discovery')(PROJECT_ROOT, discoveryClient)
  return startExpressApp(app, port)
}

// Get an initialized storage instance
function getStorage(runtimeApi, { ipfsHost }) {
  // TODO at some point, we can figure out what backend-specific connection
  // options make sense. For now, just don't use any configuration.
  const { Storage } = require('@joystream/storage-node-backend')

  const options = {
    resolve_content_id: async (contentId) => {
      // Resolve via API
      const obj = await runtimeApi.assets.getDataObject(contentId)
      if (!obj || obj.isNone) {
        return
      }
      // if obj.liaison_judgement !== Accepted .. throw ?
      return obj.unwrap().ipfs_content_id.toString()
    },
    ipfsHost,
  }

  return Storage.create(options)
}

async function initApiProduction({ wsProvider, providerId, keyFile, passphrase }) {
  // Load key information
  const { RuntimeApi } = require('@joystream/storage-runtime-api')

  const api = await RuntimeApi.create({
    account_file: keyFile,
    passphrase,
    provider_url: wsProvider,
    storageProviderId: providerId,
  })

  if (!api.identities.key) {
    throw new Error('Failed to unlock storage provider account')
  }

  await api.untilChainIsSynced()

  // We allow the node to startup without correct provider id and account, but syncing and
  // publishing of identity will be skipped.
  if (!(await api.providerIsActiveWorker())) {
    debug('storage provider role account and storageProviderId are not associated with a worker')
  }

  return api
}

async function initApiDevelopment({ wsProvider }) {
  // Load key information
  const { RuntimeApi } = require('@joystream/storage-runtime-api')

  const api = await RuntimeApi.create({
    provider_url: wsProvider,
  })

  const dev = require('../../cli/dist/commands/dev')

  api.identities.useKeyPair(dev.roleKeyPair(api))

  // Wait until dev provider is added to role
  while (true) {
    try {
      api.storageProviderId = await dev.check(api)
      break
    } catch (err) {
      debug(err)
    }

    await sleep(10000)
  }

  return api
}

function getServiceInformation(publicUrl) {
  // For now assume we run all services on the same endpoint
  return {
    asset: {
      version: 1, // spec version
      endpoint: publicUrl,
    },
    discover: {
      version: 1, // spec version
      endpoint: publicUrl,
    },
  }
}

// TODO: instead of recursion use while/async-await and use promise/setTimout based sleep
// or cleaner code with generators?
async function announcePublicUrl(api, publicUrl, publisherClient) {
  // re-announce in future
  const reannounce = function (timeoutMs) {
    setTimeout(announcePublicUrl, timeoutMs, api, publicUrl, publisherClient)
  }

  const chainIsSyncing = await api.chainIsSyncing()
  if (chainIsSyncing) {
    debug('Chain is syncing. Postponing announcing public url.')
    return reannounce(10 * 60 * 1000)
  }

  // postpone if provider not active
  if (!(await api.providerIsActiveWorker())) {
    debug('storage provider role account and storageProviderId are not associated with a worker')
    return reannounce(10 * 60 * 1000)
  }

  const sufficientBalance = await api.providerHasMinimumBalance(1)
  if (!sufficientBalance) {
    debug('Provider role account does not have sufficient balance. Postponing announcing public url.')
    return reannounce(10 * 60 * 1000)
  }

  debug('announcing public url')

  try {
    const serviceInformation = getServiceInformation(publicUrl)

    const keyId = await publisherClient.publish(serviceInformation)

    await api.discovery.setAccountInfo(keyId)

    debug('publishing complete, scheduling next update')

    // >> sometimes after tx is finalized.. we are not reaching here!

    // Reannounce before expiery. Here we are concerned primarily
    // with keeping the account information refreshed and 'available' in
    // the ipfs network. our record on chain is valid for 24hr
    reannounce(50 * 60 * 1000) // in 50 minutes
  } catch (err) {
    debug(`announcing public url failed: ${err.stack}`)

    // On failure retry sooner
    debug(`announcing failed, retrying in: 2 minutes`)
    reannounce(120 * 1000)
  }
}

// Simple CLI commands
let command = cli.input[0]
if (!command) {
  command = 'server'
}

const commands = {
  server: async () => {
    banner()
    let publicUrl, port, api

    if (cli.flags.dev) {
      const dev = require('../../cli/dist/commands/dev')
      api = await initApiDevelopment(cli.flags)
      port = dev.developmentPort()
      publicUrl = `http://localhost:${port}/`
    } else {
      api = await initApiProduction(cli.flags)
      publicUrl = cli.flags.publicUrl
      port = cli.flags.port
    }

    // TODO: check valid url, and valid port number
    const store = getStorage(api, cli.flags)

    const ipfsHost = cli.flags.ipfsHost
    const ipfs = require('ipfs-http-client')(ipfsHost, '5001', { protocol: 'http' })
    const { PublisherClient, DiscoveryClient } = require('@joystream/service-discovery')
    const publisherClient = new PublisherClient(ipfs)
    const discoveryClient = new DiscoveryClient({ ipfs, api })
    const ipfsHttpGatewayUrl = `http://${ipfsHost}:8080/`

    const { startSyncing } = require('../lib/sync')
    startSyncing(api, { syncPeriod: SYNC_PERIOD_MS }, store)

    announcePublicUrl(api, publicUrl, publisherClient)

    return startAllServices({ store, api, port, discoveryClient, ipfsHttpGatewayUrl })
  },
  discovery: async () => {
    banner()
    debug('Starting Joystream Discovery Service')
    const { RuntimeApi } = require('@joystream/storage-runtime-api')
    const wsProvider = cli.flags.wsProvider
    const api = await RuntimeApi.create({ provider_url: wsProvider })
    const port = cli.flags.port
    const ipfsHost = cli.flags.ipfsHost
    const ipfs = require('ipfs-http-client')(ipfsHost, '5001', { protocol: 'http' })
    const { DiscoveryClient } = require('@joystream/service-discovery')
    const discoveryClient = new DiscoveryClient({ ipfs, api })
    await api.untilChainIsSynced()
    await startDiscoveryService({ api, port, discoveryClient })
  },
}

async function main() {
  // Simple CLI commands
  let command = cli.input[0]
  if (!command) {
    command = 'server'
  }

  if (Object.prototype.hasOwnProperty.call(commands, command)) {
    // Command recognized
    const args = _.clone(cli.input).slice(1)
    await commands[command](...args)
  } else {
    throw new Error(`Command '${command}' not recognized, aborting!`)
  }
}

main()
  .then(() => {
    process.exit(0)
  })
  .catch((err) => {
    console.error(chalk.red(err.stack))
    process.exit(-1)
  })
