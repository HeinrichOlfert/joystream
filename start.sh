#!/usr/bin/env bash
set -e

# Run a complete joystream development network on your machine using docker.
# Make sure to run build.sh prior to running this script.

set -a
. .env
set +a

# Clean start!
docker-compose down -v

function down()
{
    # Stop containers and clear volumes
    docker-compose down -v
}

trap down EXIT

# Run a local development chain
docker-compose up -d joystream-node

## Storage Infrastructure
# Configure a dev storage node and start storage node
DEBUG=joystream:storage-cli:dev yarn storage-cli dev-init
docker-compose up -d colossus

## Query Node Infrastructure
# Initialize a new database for the query node infrastructure
docker-compose up -d db
yarn workspace query-node-root db:migrate

# Startup all query-node infrastructure services
docker-compose up -d graphql-server
docker-compose up -d processor

echo "press Ctrl+C to shutdown"
while true; do 
  read
done
