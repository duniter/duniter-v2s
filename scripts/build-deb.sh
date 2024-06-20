#!/bin/sh

cd "$(dirname "$0")"
docker buildx build -t build_deb --file ../docker/build-deb.Dockerfile ../
id=$(docker create build_deb)
docker cp $id:/app/debian/ ../target
