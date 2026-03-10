# Reverse proxy for a public RPC endpoint

If you want to expose a Duniter RPC endpoint publicly, put it behind a reverse
proxy and expose `443` to users instead of exposing port `9944` directly.

Recommended topology:

```text
client --> https://rpc.example.org or wss://rpc.example.org/ws --> nginx --> duniter RPC on 127.0.0.1:9944
```

## Recommendations

- Prefer `wss://` for public WebSocket access.
- Keep port `9944` private whenever possible.
- If you want Duniter to trust the real client IP forwarded by your reverse
  proxy for RPC rate limiting, add
  `--rpc-rate-limit-trust-proxy-headers` to your Duniter node.
- If Duniter runs in Docker, publish RPC on loopback only:

```yaml
ports:
  - 127.0.0.1:9944:9944
  - 30333:30333
```

- If loopback binding is not possible, use a firewall so only the reverse proxy
  can reach the RPC port.
- Do not expose unsafe RPC methods publicly unless you explicitly need them.

Important: by default, Duniter does **not** trust `X-Real-IP` and
`X-Forwarded-For`, because those headers can be forged by a spammer if the node
is reachable directly. Enable `--rpc-rate-limit-trust-proxy-headers` only when
your node is actually protected behind a reverse proxy and the RPC port is not
publicly reachable directly.

## Nginx example for a `/ws` endpoint

This is the safest default if you want a dedicated public WebSocket endpoint
such as `wss://rpc.example.org/ws`.

```nginx
map $http_upgrade $connection_upgrade {
    default upgrade;
    '' close;
}

server {
    listen 443 ssl http2;
    server_name rpc.example.org;

    ssl_certificate /etc/letsencrypt/live/rpc.example.org/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/rpc.example.org/privkey.pem;

    location /ws {
        proxy_pass http://127.0.0.1:9944;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection $connection_upgrade;
        proxy_read_timeout 86400;
        proxy_send_timeout 86400;
        proxy_buffering off;
    }
}
```

Why these settings matter:

- `proxy_http_version 1.1` is required for WebSocket upgrades.
- `Upgrade` and `Connection` headers are required for WebSocket proxying.
- `proxy_read_timeout 86400` avoids long-lived WebSocket sessions being closed
  too aggressively.

## Nginx example for exposing RPC at `/`

If your clients expect the RPC endpoint at the root URL, use:

```nginx
map $http_upgrade $connection_upgrade {
    default upgrade;
    '' close;
}

server {
    listen 443 ssl http2;
    server_name rpc.example.org;

    ssl_certificate /etc/letsencrypt/live/rpc.example.org/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/rpc.example.org/privkey.pem;

    location / {
        proxy_pass http://127.0.0.1:9944;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection $connection_upgrade;
        proxy_read_timeout 86400;
        proxy_send_timeout 86400;
        proxy_buffering off;
    }
}
```

## Duniter-side checks

Before testing the public endpoint, verify:

- the node is reachable locally on `127.0.0.1:9944`
- the proxy can reach the node
- the node is configured for the RPC methods and CORS policy you actually want
  to expose
- if you rely on proxy-forwarded client IPs for rate limiting, the node is
  started with `--rpc-rate-limit-trust-proxy-headers`

For a public mirror node, a common pattern is:

- keep RPC private on the host
- expose only nginx on `443`
- advertise the public endpoint with `DUNITER_PUBLIC_RPC`

## Quick test

After nginx is reloaded, test the endpoint with a WebSocket-aware client such as
Polkadot.js Apps or Duniter Portal using:

```text
wss://rpc.example.org/ws
```

If you expose RPC at `/`, test:

```text
wss://rpc.example.org
```
