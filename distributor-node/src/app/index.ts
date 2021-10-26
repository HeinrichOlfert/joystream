import { ReadonlyConfig } from '../types'
import { NetworkingService } from '../services/networking'
import { LoggingService } from '../services/logging'
import { StateCacheService } from '../services/cache/StateCacheService'
import { ContentService } from '../services/content/ContentService'
import { HttpApiService } from '../services/httpApi/HttpApiService'
import { Logger } from 'winston'
import fs from 'fs'
import nodeCleanup from 'node-cleanup'
import { AppIntervals } from '../types/app'

export class App {
  private config: ReadonlyConfig
  private content: ContentService
  private stateCache: StateCacheService
  private networking: NetworkingService
  private httpApi: HttpApiService
  private logging: LoggingService
  private logger: Logger
  private intervals: AppIntervals | undefined

  constructor(config: ReadonlyConfig) {
    this.config = config
    this.logging = LoggingService.withAppConfig(config)
    this.stateCache = new StateCacheService(config, this.logging)
    this.networking = new NetworkingService(config, this.stateCache, this.logging)
    this.content = new ContentService(config, this.logging, this.networking, this.stateCache)
    this.httpApi = new HttpApiService(config, this.stateCache, this.content, this.logging, this.networking)
    this.logger = this.logging.createLogger('App')
  }

  private setIntervals() {
    this.intervals = {
      saveCacheState: setInterval(() => this.stateCache.save(), this.config.intervals.saveCacheState * 1000),
      checkStorageNodeResponseTimes: setInterval(
        () => this.networking.checkActiveStorageNodeEndpoints(),
        this.config.intervals.checkStorageNodeResponseTimes * 1000
      ),
      cacheCleanup: setInterval(() => this.content.cacheCleanup(), this.config.intervals.cacheCleanup * 1000),
    }
  }

  private clearIntervals() {
    if (this.intervals) {
      Object.values(this.intervals).forEach((interval) => clearInterval(interval))
    }
  }

  private checkConfigDirectories(): void {
    Object.entries(this.config.directories).forEach(([name, path]) => {
      if (path === undefined) {
        return
      }
      const dirInfo = `${name} directory (${path})`
      if (!fs.existsSync(path)) {
        try {
          fs.mkdirSync(path, { recursive: true })
        } catch (e) {
          throw new Error(`${dirInfo} doesn't exist and cannot be created!`)
        }
      }
      try {
        fs.accessSync(path, fs.constants.R_OK)
      } catch (e) {
        throw new Error(`${dirInfo} is not readable`)
      }
      try {
        fs.accessSync(path, fs.constants.W_OK)
      } catch (e) {
        throw new Error(`${dirInfo} is not writable`)
      }
    })
  }

  public async start(): Promise<void> {
    this.logger.info('Starting the app', { config: this.config })
    try {
      this.checkConfigDirectories()
      this.stateCache.load()
      await this.content.startupInit()
      this.setIntervals()
      this.httpApi.start()
    } catch (err) {
      this.logger.error('Node initialization failed!', { err })
      process.exit(-1)
    }
    nodeCleanup(this.exitHandler.bind(this))
  }

  private async exitGracefully(): Promise<void> {
    // Async exit handler - ideally should not take more than 10 sec
    // We can try to wait until some pending downloads are finished here etc.
    this.logger.info('Graceful exit initialized')

    // Try to process remaining downloads
    const MAX_RETRY_ATTEMPTS = 3
    let retryCounter = 0
    while (retryCounter < MAX_RETRY_ATTEMPTS && this.stateCache.getPendingDownloadsCount()) {
      const pendingDownloadsCount = this.stateCache.getPendingDownloadsCount()
      this.logger.info(`${pendingDownloadsCount} pending downloads in progress... Retrying exit in 5 sec...`, {
        retryCounter,
        pendingDownloadsCount,
      })
      await new Promise((resolve) => setTimeout(resolve, 5000))
      this.stateCache.saveSync()
      ++retryCounter
    }

    if (this.stateCache.getPendingDownloadsCount()) {
      this.logger.warn('Limit reached: Could not finish all pending downloads.', {
        pendingDownloadsCount: this.stateCache.getPendingDownloadsCount(),
      })
    }

    this.logger.info('Graceful exit finished')
    await this.logging.end()
  }

  private exitCritically(): void {
    // Some additional synchronous work if required...
    this.logger.info('Critical exit finished')
  }

  private exitHandler(exitCode: number | null, signal: string | null): boolean | undefined {
    this.logger.info('Exiting...')
    // Clear intervals
    this.clearIntervals()
    // Stop the http api
    this.httpApi.stop()
    // Save cache
    this.stateCache.saveSync()
    if (signal) {
      // Async exit can be executed
      this.exitGracefully()
        .then(() => {
          process.kill(process.pid, signal)
        })
        .catch((err) => {
          this.logger.error('Graceful exit error', { err })
          this.logging.end().finally(() => {
            process.kill(process.pid, signal)
          })
        })
      nodeCleanup.uninstall()
      return false
    } else {
      // Only synchronous work can be done here
      this.exitCritically()
    }
  }
}
