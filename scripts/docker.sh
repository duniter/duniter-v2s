#!/usr/bin/env bash
# This script is meant to be run on Unix/Linux based systems
set -e

echo "*** Start Duniter node ***"

cd $(dirname ${BASH_SOURCE[0]})/..

mkdir -p build
cp target/release/duniter build/duniter
docker build -t "duniter/duniter-v2s:local" -f ".maintain/local-docker-test-network/duniter.Dockerfile" .

#docker compose down --remove-orphans
#docker compose run --rm --service-ports dev $@
