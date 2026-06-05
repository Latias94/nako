# Remote Access Cookbook

Status: Draft operator cookbook

This guide explains remote access shapes Nako supports today. Nako validates
operator-declared network policy, but it does not start tunnel processes, own
DNS, provide endpoint discovery, choose LAN versus remote client URLs, or act
as a relay.

## Supported Shapes

Use one of these shapes:

- Local-only: bind to `127.0.0.1` and do not expose the server.
- Private network: bind on a trusted LAN or VPN with bearer auth enabled.
- Reverse proxy: terminate HTTPS in Caddy, Nginx, or another proxy and forward
  to Nako.
- Tunnel provider: run Tailscale Funnel, Cloudflare Tunnel, ngrok, or another
  external tunnel outside Nako, then declare the public URL for readiness.

Do not expose Nako directly to the public internet with disabled auth,
placeholder tokens, wildcard browser origins, or unreviewed forwarded headers.
`GET /health` is public by design; other routes should require bearer auth in
remote-access deployments.

## Shared Requirements

Keep these rules consistent across every remote-access shape:

- Set `[auth].enabled = true` and keep the bearer token in an environment
  variable.
- Use an HTTPS `external_base_url` for `reverse_proxy` and `tunnel_provider`
  modes.
- Set `allowed_origins` to exact browser origins. Do not use `*`.
- Trust forwarded headers only from reviewed proxy source IPs. Leave
  `trusted_proxy_headers = false` when the proxy or tunnel source boundary is
  unclear.
- Keep tunnel provider credentials outside `nako.toml`. Use `token_env`.
- Treat playback ticket URLs as sensitive. Browser playback tickets are
  short-lived, but reverse-proxy logs, tunnel dashboards, and support bundles
  can still capture them if raw URLs are copied.
- Do not put bearer tokens, tunnel tokens, playback tickets, or provider
  credentials in query strings, committed proxy configs, screenshots, or issue
  attachments.

Nako `config-check --json` redacts raw external URLs, tunnel public URLs,
allowed origins, trusted proxy sources, forwarded header names, bearer token
values, tunnel token values, and local host details from diagnostics. It may
report safe counts, status values, provider IDs, and environment variable
names.

## Reverse Proxy With Caddy

Run Nako on loopback, let Caddy terminate TLS, and trust only the Caddy source:

```toml
listen_addr = "127.0.0.1:3000"

[auth]
enabled = true
token_env = "NAKO_ADMIN_TOKEN"

[network]
exposure_mode = "reverse_proxy"
external_base_url = "https://nako.example.com"
trusted_proxy_headers = true
trusted_proxy_sources = ["127.0.0.1"]
allowed_origins = ["https://nako.example.com"]
```

Minimal Caddy shape:

```caddyfile
nako.example.com {
    reverse_proxy 127.0.0.1:3000 {
        header_up X-Forwarded-Host {host}
        header_up X-Forwarded-Proto {scheme}
    }
}
```

Caddy manages HTTPS certificates for normal public DNS names. If Caddy runs in
a container or on another host, change `trusted_proxy_sources` to the reviewed
source address or CIDR that Nako actually sees.

## Reverse Proxy With Nginx

Run Nginx with TLS enabled and proxy to Nako on a private address:

```nginx
server {
    listen 443 ssl http2;
    server_name nako.example.com;

    ssl_certificate /etc/letsencrypt/live/nako.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/nako.example.com/privkey.pem;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-Host $host;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header X-Request-Id $request_id;
        proxy_set_header Authorization $http_authorization;
        proxy_set_header Range $http_range;
        proxy_request_buffering off;
    }
}
```

Do not rewrite playback paths or strip `Range` requests. Keep access logs and
support bundles redacted because browser playback ticket URLs may appear in
request logs.

## DDNS

DDNS is only a naming layer. It does not replace TLS, auth, CORS policy, or
trusted proxy review.

Use DDNS when a home connection has a changing public IP:

1. Point the DDNS name at the router or proxy host.
2. Terminate HTTPS at Caddy, Nginx, or another reviewed proxy.
3. Set `external_base_url` and `allowed_origins` to the DDNS HTTPS origin.
4. Update Nako config when the hostname changes; do not depend on endpoint
   discovery.

Avoid using raw public IP origins for browser clients. They make certificates,
CORS, and support redaction harder to reason about.

## Tailscale Funnel

Run Tailscale and Funnel outside Nako. Funnel can publish a local Nako listener
over an HTTPS Tailscale-managed hostname:

```toml
[network]
exposure_mode = "tunnel_provider"
external_base_url = "https://nako.tailnet-name.ts.net"
allowed_origins = ["https://nako.tailnet-name.ts.net"]

[[network.tunnel_providers]]
id = "tailscale-funnel"
kind = "tailscale_funnel"
public_url = "https://nako.tailnet-name.ts.net"
token_env = "NAKO_TUNNEL_TOKEN"
```

`token_env` is a readiness declaration for operator-managed tunnel credentials.
Nako does not run `tailscale`, change Funnel settings, or supervise the tunnel.
Leave `trusted_proxy_headers` disabled unless you have reviewed exactly which
local source sends forwarded headers to Nako.

## Cloudflare Tunnel

Run `cloudflared` outside Nako and route the public hostname to Nako's private
listener:

```toml
[network]
exposure_mode = "tunnel_provider"
external_base_url = "https://nako.example.com"
allowed_origins = ["https://nako.example.com"]

[[network.tunnel_providers]]
id = "cloudflared"
kind = "cloudflare_tunnel"
public_url = "https://nako.example.com"
token_env = "NAKO_TUNNEL_TOKEN"
```

Keep Cloudflare Tunnel tokens and origin certificates in the operator secret
store. Disable caching for API and playback routes, preserve `Range` requests,
and redact Cloudflare request logs before sharing support bundles.

## ngrok

Use a reserved HTTPS domain for stable operation:

```toml
[network]
exposure_mode = "tunnel_provider"
external_base_url = "https://nako.example.ngrok.app"
allowed_origins = ["https://nako.example.ngrok.app"]

[[network.tunnel_providers]]
id = "ngrok"
kind = "ngrok"
public_url = "https://nako.example.ngrok.app"
token_env = "NAKO_TUNNEL_TOKEN"
```

Free or ephemeral ngrok URLs require updating `external_base_url` and
`allowed_origins` whenever the hostname changes. Do not paste ngrok authtokens
or inspector URLs into Nako config, logs, or issue attachments.

## Generic External Tunnel

Use `kind = "external"` when the provider is not modeled directly:

```toml
[network]
exposure_mode = "tunnel_provider"
external_base_url = "https://nako.example.net"
allowed_origins = ["https://nako.example.net"]

[[network.tunnel_providers]]
id = "external-tunnel"
kind = "external"
public_url = "https://nako.example.net"
token_env = "NAKO_TUNNEL_TOKEN"
```

The provider must supply HTTPS and must forward requests to Nako without
weakening bearer auth, range streaming, CORS, or playback ticket handling.

## Fixture Gate

The repository includes redaction-safe fixtures:

- `deploy/remote-access/reverse-proxy.nako.toml`
- `deploy/remote-access/tunnel-provider.nako.toml`

Run the gate:

```bash
bash scripts/remote-access-config-gate.sh
```

On Windows:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/remote-access-config-gate.ps1
```

The gate runs `nako-server config-check --json --create-dirs` for a
reverse-proxy fixture and a tunnel-provider fixture. It writes reports under
`target/release-gate/remote-access/` and fails if the reports expose raw URLs,
token values, private origins, trusted proxy sources, forwarded header names,
or host details from the fixtures.
