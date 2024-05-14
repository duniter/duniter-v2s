# Nginx reverse proxy example

In `/etc/nginx/sites-enabled/gdev.YOUR_DOMAIN` put (you can probably do simpler):

```nginx
# see http://nginx.org/en/docs/http/websocket.html
map $http_upgrade $connection_upgrade {
    default upgrade;
    ''      close;
}

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


  location / {
    proxy_pass http://localhost:9944;

    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection $connection_upgrade;
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
