#!/bin/bash
set -e

set -o allexport
source .env
set +o allexport

docker compose -f docker/docker-compose.yml -p charade up -d
