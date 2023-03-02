# How to deploy a permanent rpc node on ĞDev network

## Publish a node

### Duniter part

See [docker documentation](../../docker/README.md) to install, configure, and start a node.

### Reverse proxy part

See [nginx reverse proxy](./nginx_reverse_proxy.md).

To go further, read [How to become a (black)smith](./smith.md)

## Upgrade your node with minimal interruption

1. Modify docker image tag on your compose file
2. Run `docker compose pull`, this will pull the new image.
3. Run `docker compose up -d --remove-orphans`, this will recreate the container
4. Verify that your node restarted well `docker compose logs duniter-rpc`
5. Remove the old image `docker rmi duniter/duniter-v2s:OLD_TAG`
