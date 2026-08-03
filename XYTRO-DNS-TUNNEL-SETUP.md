# Xytro Labs — DNS & Cloudflare Tunnel Setup Guide

## Overview

All services run on a single Linux server. Cloudflare Tunnel exposes them to the internet.

```
                      ┌──────────────────────────────────┐
                      │       Cloudflare                 │
                      │  (DNS + Tunnel endpoint)         │
                      └──────┬───────────────────────────┘
                             │ cloudflared tunnel
                      ┌──────▼───────────────────────────┐
                      │    cloudflared (tunnel client)    │
                      │    ingress rules route by         │
                      │    hostname → localhost:PORT      │
                      └──────────────────────────────────┘
```

---

## 1. DNS Records

Create these A records in your Cloudflare dashboard for **xytro.site** and **xytro.site**.

### Zone: `xytro.site`

| Type | Name | Content | Proxy |
|------|------|---------|-------|
| **CNAME** | `@` (root) | `{tunnel-uuid}.cfargotunnel.com` | Proxied (orange cloud) |
| **CNAME** | `www` | `{tunnel-uuid}.cfargotunnel.com` | Proxied |
| **CNAME** | `mail` | `{tunnel-uuid}.cfargotunnel.com` | Proxied |
| **CNAME** | `one` | `{tunnel-uuid}.cfargotunnel.com` | Proxied |
| **CNAME** | `cloud` | `{tunnel-uuid}.cfargotunnel.com` | Proxied |
| **CNAME** | `vault` | `{tunnel-uuid}.cfargotunnel.com` | Proxied |
| **CNAME** | `nova` | `{tunnel-uuid}.cfargotunnel.com` | Proxied |
| **CNAME** | `admin` | `{tunnel-uuid}.cfargotunnel.com` | Proxied |
| **CNAME** | `aether` | `{tunnel-uuid}.cfargotunnel.com` | Proxied |
| **CNAME** | `balance` | `{tunnel-uuid}.cfargotunnel.com` | Proxied |

### Zone: `xytro.site`

| Type | Name | Content | Proxy |
|------|------|---------|-------|
| **CNAME** | `@` (root) | `{tunnel-uuid}.cfargotunnel.com` | Proxied |
| **CNAME** | `www` | `{tunnel-uuid}.cfargotunnel.com` | Proxied |

### MX Records (if you want inbound email on xytro.site)

| Type | Name | Priority | Content | Proxy |
|------|------|----------|---------|-------|
| **MX** | `@` | 10 | `{tunnel-uuid}.cfargotunnel.com` | DNS only (grey cloud) |
| **TXT** | `@` | — | `v=spf1 a mx include:_spf.xytro.site ~all` | DNS only |
| **TXT** | `_dmarc` | — | `v=DMARC1; p=quarantine; rua=mailto:dmarc@xytro.site` | DNS only |

---

## 2. Local Services Map

| Port | Hostname(s) | Service | Type |
|------|------------|---------|------|
| **3000** | — | **XytroNetwork** (Express — games, news, tools) | Node.js |
| **4000** | `mail.xytro.site`, `one.xytro.site`, `cloud.xytro.site`, `vault.xytro.site`, `nova.xytro.site`, `admin.xytro.site`, `xytro.site`, `www.xytro.site` | **XytroMailing** (Express — email, auth, cloud, admin, NOVA social) | Node.js |
| **4020** | `xytro.site` (root), `www.xytro.site` | **Landing page** (site-root/index.html) | Python HTTP |
| **4021** | `indent.xytro.site` | **Indent language site** | Python HTTP |
| **4022** | `balance.xytro.site` | **Balance/GEMs site** | Python HTTP |
| **4010** | — | **Legacy reports** (internal only) | Python HTTP |
| **11435** | — | **Ollama AI** (for XytroMail AI features) | Ollama |

---

## 3. Cloudflare Tunnel Setup

### 3a. Install cloudflared

**cloudflared is not currently installed.** Choose one of these methods:

```bash
# Option A: Download the binary directly (works on any distro)
curl -L https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64 -o /tmp/cloudflared
sudo install -m 755 /tmp/cloudflared /usr/local/bin/cloudflared

# Option B: Using your package manager
# (check what's available for your distro)
```

Verify it installed:
```bash
cloudflared version
```

### 3b. Authenticate

```bash
cloudflared tunnel login
```
This opens a browser to authorize with your Cloudflare account. If you're on a headless server, use `--token` instead.

### 3c. Create the Tunnel

```bash
cloudflared tunnel create xytro-labs
```

This creates a tunnel with a UUID and generates a credentials file at:
`~/.cloudflared/{tunnel-uuid}.json`

### 3d. Route DNS to the Tunnel

```bash
# Main domain subdomains
cloudflared tunnel route dns xytro-labs mail.xytro.site
cloudflared tunnel route dns xytro-labs one.xytro.site
cloudflared tunnel route dns xytro-labs cloud.xytro.site
cloudflared tunnel route dns xytro-labs vault.xytro.site
cloudflared tunnel route dns xytro-labs nova.xytro.site
cloudflared tunnel route dns xytro-labs admin.xytro.site
cloudflared tunnel route dns xytro-labs indent.xytro.site
cloudflared tunnel route dns xytro-labs balance.xytro.site

# Root domain
cloudflared tunnel route dns xytro-labs xytro.site
cloudflared tunnel route dns xytro-labs www.xytro.site

# Mail domain
cloudflared tunnel route dns xytro-labs xytro.site
cloudflared tunnel route dns xytro-labs www.xytro.site
```

*Alternatively*, create the CNAME records manually in the Cloudflare dashboard (see table above).

### 3e. Create the Config File

Create `~/.cloudflared/config.yml`:

```yaml
tunnel: xytro-labs
credentials-file: /home/raf/.cloudflared/<your-tunnel-uuid>.json

ingress:
  # ─── XytroMailing (port 4000) ───
  - hostname: mail.xytro.site
    service: http://localhost:4000
  - hostname: one.xytro.site
    service: http://localhost:4000
  - hostname: cloud.xytro.site
    service: http://localhost:4000
  - hostname: vault.xytro.site
    service: http://localhost:4000
  - hostname: nova.xytro.site
    service: http://localhost:4000
  - hostname: admin.xytro.site
    service: http://localhost:4000
  - hostname: xytro.site
    service: http://localhost:4000
  - hostname: www.xytro.site
    service: http://localhost:4000

  # ─── Static Sites (Python HTTP servers) ───
  - hostname: xytro.site
    service: http://localhost:4020
  - hostname: www.xytro.site
    service: http://localhost:4020
  - hostname: indent.xytro.site
    service: http://localhost:4021
  - hostname: balance.xytro.site
    service: http://localhost:4022

  # ─── Fallback ───
  - service: http_status:404
```

### 3f. Install as a System Service

```bash
sudo cloudflared service install
```

Or run it manually via the startup script (already in `start_all.sh`):

```bash
nohup cloudflared tunnel --config ~/.cloudflared/config.yml --no-autoupdate run > /tmp/cloudflared_tunnel.log 2>&1 &
```

---

## 4. Verifying It Works

### 4a. Check tunnel is running

```bash
cloudflared tunnel list
cloudflared tunnel info xytro-labs
```

### 4b. Test each endpoint

```bash
curl -H "Host: xytro.site" http://localhost:4020          # → landing page
curl -H "Host: mail.xytro.site" http://localhost:4000      # → XytroMail login
curl -H "Host: one.xytro.site" http://localhost:4000       # → Xytro One
curl -H "Host: cloud.xytro.site" http://localhost:4000     # → The Vault
curl -H "Host: nova.xytro.site" http://localhost:4000      # → Nova AI
curl -H "Host: admin.xytro.site" http://localhost:4000     # → Admin
curl -H "Host: indent.xytro.site" http://localhost:4021    # → Aether site
curl -H "Host: balance.xytro.site" http://localhost:4022   # → Balance site
```

### 4c. XytroNetwork (games/news — port 3000)

The XytroNetwork Express app on port 3000 currently isn't mapped to a tunnel hostname. If you want it accessible, you can add a subdomain like `network.xytro.site`:

1. Add DNS CNAME: `network → {tunnel-uuid}.cfargotunnel.com`
2. Add ingress rule: `- hostname: network.xytro.site` → `service: http://localhost:3000`
3. Or keep it accessible only via LAN for now.

---

## 5. Quick Reference: Previous → New

| Old Domain | New Domain | Service |
|-----------|-----------|---------|
| prismtech.site | **xytro.site** | Landing page (port 4020) |
| mail.prismtech.site | **mail.xytro.site** | XytroMail (port 4000) |
| one.prismtech.site | **one.xytro.site** | Xytro One (port 4000) |
| cloud.prismtech.site | **cloud.xytro.site** | The Vault (port 4000) |
| nova.prismtech.site | **nova.xytro.site** | Nova AI (port 4000) |
| admin.prismtech.site | **admin.xytro.site** | Admin (port 4000) |
| aether.prismtech.site | **indent.xytro.site** | Indent lang (ex-Aether) (port 4021) |
| balance.prismtech.site | **balance.xytro.site** | Balance (port 4022) |
| stratosmail.site | **xytro.site** | Mail domain (port 4000) |
| stratoslabs.site | _(merged into xytro.site)_ | — |

---

## 6. Environment Variables to Update

In your `.env` file for XytroMailing:

```env
# Old                            # New
PUBLIC_BASE_URL=http://localhost:4001   # (keep as-is for local dev)
# Make sure SESSION_COOKIE_DOMAIN is correct:
SESSION_COOKIE_DOMAIN=.xytro.site
MAIL_DOMAIN=xytro.site
```

---

## 7. Quick Start (after DNS + Tunnel)

```bash
cd /run/media/raf/Z/PrismTechnologies
bash start_all.sh
```

This starts everything: XytroMailing (port 4000), XytroNetwork (port 3000), all static sites, Ollama AI, and the Cloudflare tunnel.
