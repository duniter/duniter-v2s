# How to deploy a permanent rpc node on ĞDev network

## Publish a node

### Duniter part

- Add this docker-compose template on your server:
  [docker/compose/gdev-rpc.docker-compose.yml](https://git.duniter.org/nodes/rust/duniter-v2s/-/blob/master/docker/compose/gdev-mirror.docker-compose.yml)
- Rename the file : `mv gdev-mirror.docker-compose.yml docker-compose.yml`
- In the same folder, create a `.env` file that defime environment variables `SERVER_DOMAIN` and `PEER_ID`:

```bash
SERVER_DOMAIN=YOUR_DOMAIN
PEER_ID=YOUR_PEER_ID
```

Your `PEER_ID` shoud be generated with this command:

```bash
docker run --rm -it --entrypoint duniter -v $PWD:/var/lib/duniter/  duniter/duniter-v2s:v0.4.0 key generate-node-key --file /var/lib/duniter/node.key
```

- If you have write access errors run in docker-compose.yml folder : `chmod o+rwX -R .`
- Do `docker compose up -d` to start your node

### Reverse-proxy part (with Nginx)

In `/etc/nginx/sites-enabled/gdev.YOUR_DOMAIN` put (you can probably do simpler):

```nginx
server {
  server_name gdev.YOUR_DOMAIN.fr;

  listen 443 ssl http2;
  listen [::]:443 ssl http2;
  ssl_certificate /etc/nginx/ssl/YOUR_DOMAIN.cert;
  ssl_certificate_key /etc/nginx/ssl/YOUR_DOMAIN.key;

  root /nowhere;

  add_header X-Frame-Options SAMEORIGIN;
  add_header X-XSS-Protection "1; mode=block";
  proxy_redirect off;
  proxy_buffering off;
  proxy_set_header Host $host;
  proxy_set_header X-Real-IP $remote_addr;
  proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
  proxy_set_header X-Forwarded-Proto $scheme;
  proxy_set_header X-Forwarded-Port $server_port;
  proxy_read_timeout 90;

  location /http {
    proxy_pass http://localhost:9933;
    proxy_http_version 1.1;
  }
  location /ws {
    proxy_pass http://localhost:9944;

    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
    proxy_http_version 1.1;

    proxy_read_timeout 1200s;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header Host $host;
  }
}
```

and replace `YOUR_DOMAIN` by your domain each time.

- [generate your ssl certificates](https://github.com/acmesh-official/acme.sh) with let's encrypt
  if you don't already have a wildcard certificate.
- `service nginx reload`

Your node is now online as a rpc node. It's fully capable for wallet use.

To go further, read [How to become a (black)smith](./smith.md)

## Upgrade your node with minimal interruption

1. Modify docker image tag on your compose file
2. Run `docker compose pull`, this will pull the new image.
3. Run `docker compose up -d --remove-orphans`, this will recreate the container
4. Verify that your node restarted well `docker compose logs duniter-rpc`
5. Remove the old image `docker images rmi duniter/duniter-v2s:OLD_TAG`
