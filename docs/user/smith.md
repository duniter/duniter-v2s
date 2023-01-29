# How to become a (black)smith

## Publish a node

### Duniter part

- Add this docker-compose on your server :
  [docker/compose/gdev-validator.docker-compose.yml](https://git.duniter.org/nodes/rust/duniter-v2s/-/blob/master/docker/compose/gdev-validator.docker-compose.yml)
- Create a `.env` file that define environment variable `SERVER_DOMAIN`:

```bash
SERVER_DOMAIN=YOUR_DOMAIN
```

- `docker compose up -d` to start your node

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

## Join the Smith WoT

- add polkadot webextension to be able to authentificate with your account.
- Go to [any node with polkadotjs ui](https://gdev.1000i100.fr/dev-ui/?rpc=wss://gdev.1000i100.fr/ws)
- Ask to join Smith WoT (you need to already be in the main WoT)
  - developer > extrinsics > YOUR_SMITH_ACCOUNT > smithMembership > requestMemberShip(metadata)
  - add your p2p endpoint (optional)
  - add your session key (follow point 1 to 4 from Validate blocks > Generate and publish your session key)
  - Send the query
- Await smith certification : developer > extrinsics > CERTIFIER_SMITH_ACCOUNT > smithCert > addCert(receiver)

When you have at least 3 certifications, your'in!

## Validate blocks (blacksmith work)

- Generate and publish your session keys
  1. create an ssh bridge from your desktop/laptop to your server : `ssh -L 9945:localhost:9945 SSH_USER@YOUR_SERVER`
  2. In your browser go to [polkadotjs : ws://localhost:9945](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2Flocalhost%3A9945#/explorer)
  3. In the UI : developer > appel RPC > author > rotateKey() and run
  4. copy the result in clipboard
  5. In the UI : developer > extrinsics > YOUR_SMITH_ACCOUNT > authorityMembers > setSessionKeys(keys) then copy your session keys and run the query.
  6. **wait 48h to verify you keep sync**
- Join
  - In the UI : developer > extrinsics > YOUR_SMITH_ACCOUNT > authorityMembers > goOnline()

If you're not able to monitor, reboot, act on your node, goOffline() to avoid penality to the blockchain and to you.

## Upgrade your node with minimal interruption

1. Modify docker image tag on your compose file
2. Run `docker compose pull`, this will pull the new image.
3. Run `docker compose up -d --remove-orphans`, this will recreate the container
4. Verify that your node restarted well `docker compose logs duniter-validator`
5. Remove the old image `docker images rmi duniter/duniter-v2s:OLD_TAG`
