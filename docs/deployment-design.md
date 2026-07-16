# Deployment Design

## Goal

Deploy Scriptorium as cheaply and simply as possible for a volunteer-run parish bookshop. Volunteers with minimal ops experience should be able to keep it running.

## What We Are Deploying

| Component | Detail |
|-----------|--------|
| Binary | `bookstore-web` — single statically-linked Rust executable |
| Database | One SQLite file (typically < 50 MB) |
| Config | Two env vars: `DATABASE_URL`, `RUST_LOG` |
| External deps | CDN scripts loaded by browser (HTMX, Google Fonts, Preact) — not by server |
| TLS | Not in binary — provided by platform or reverse proxy |

The app binds to `127.0.0.1:8080` (HTTP only). It has no background workers, no queues, no external database, and no file storage. This makes it an ideal candidate for the simplest possible hosting.

## Usage Model: 200-Member Parish Bookshop

Before comparing platforms, we need realistic traffic estimates. A 200-member parish bookshop is a very low-traffic application.

### Traffic Assumptions

| Metric | Estimate | Rationale |
|--------|----------|-----------|
| Active online browsers | 30–50/month | Not all members shop online regularly |
| POS sessions | 4–6/week | Sunday after service + 1–2 weekday events |
| POS transactions/week | 15–30 | A fraction of attendees buy each Sunday |
| Online orders/month | 5–20 | Casual browsing, occasional purchase |
| Admin sessions/month | 10–20 | 1–2 volunteers checking stock, doing intake |
| Total page views/month | 500–2,000 | Across storefront, POS, and admin |
| API calls/month | 200–500 | POS scans, cart mutations, checkout |

### Resource Consumption

| Resource | Estimate | Notes |
|----------|----------|-------|
| **Server bandwidth** | 60–100 MB/month | ~30 KB average HTML response × 2,000 views. CDN assets (HTMX, fonts, Preact) load from external CDNs, not from our server. |
| **CPU** | Near zero | Rust + tokio handles each request in < 10 ms. 2,000 requests = ~20 seconds of actual compute per month. |
| **Memory** | < 50 MB | Rust binary + SQLite in-memory cache. No garbage collector, no runtime bloat. |
| **Disk** | < 50 MB | SQLite database for a few hundred products and a few thousand orders. |
| **Uptime requirement** | Always-on | POS must respond instantly during Sunday sales. Cold-start delays (2–5 seconds) are unacceptable at the register. |

This is an exceptionally light workload. The binding constraint is not compute or bandwidth — it is **always-on availability** for POS use.

## Platform Comparison

### A. Fly.io (Pay-As-You-Go) — Recommended

Fly.io **no longer has a free tier** for new customers (removed late 2024). New signups get a 2-hour trial, then must add a credit card and pay per use. However, pay-as-you-go pricing for this workload is very cheap.

**Why it is still recommended:** Managed TLS, persistent volumes, single-command deploys (`fly deploy`), and Sydney region availability. No Linux admin skills required.

#### Cost Breakdown (Sydney Region)

| Line Item | Unit Price | Usage | Monthly Cost |
|-----------|-----------|-------|-------------|
| Compute (shared-cpu-1x, 256 MB, always-on) | ~$0.00000078/sec × regional markup | 730 hrs | **~$2.50–3.50** |
| Persistent volume (1 GB) | $0.15/GB | 1 GB | **$0.15** |
| Outbound bandwidth | $0.04/GB (Asia-Pacific) | 0.1 GB | **< $0.01** |
| Volume snapshots | $0.08/GB (first 10 GB free) | < 1 GB | **$0** |
| Shared IPv4 | included | 1 | **$0** |
| **Total** | | | **~$2.70–3.70** |

#### Free Tier Boundary

There is no free tier. Cost begins from first use after adding a payment method. The 2-hour free trial is for evaluation only.

#### Risk of Cost Escalation

**Very low.** Even if traffic doubled or tripled, bandwidth remains negligible at $0.04/GB. Compute is the dominant cost and is fixed at ~$3/month for an always-on machine. There are no surprise scaling charges because there is only one machine. The only way to significantly increase cost is to provision more machines or larger volumes, which requires explicit action.

#### Cost-Saving Option: Auto-Stop

Fly.io can auto-stop machines when idle and wake them on incoming requests. This would reduce compute cost to < $1/month. However, wake time is 2–5 seconds — acceptable for storefront browsing but unacceptable at the POS register. Use auto-stop only if POS is not in use (e.g., online-only bookshop with no physical counter).

### B. Oracle Cloud Free Tier — Cheapest (Truly Free Forever)

Oracle's Always Free tier provides ARM compute, storage, and bandwidth at no cost with no time limit.

#### What You Get (Free)

| Resource | Allowance |
|----------|-----------|
| ARM (Ampere A1) compute | Up to 4 OCPUs, 24 GB RAM (split across 1–4 VMs) |
| Block storage | 200 GB |
| Outbound bandwidth | 10 TB/month |
| Object storage | 20 GB |

For Scriptorium, a 1 OCPU / 6 GB RAM VM is massively overprovisioned.

#### Cost Breakdown

| Line Item | Monthly Cost |
|-----------|-------------|
| Everything | **$0** |

#### Free Tier Boundary

The Always Free tier has **no expiry**. It is not a trial. You keep the resources indefinitely as long as your account remains active and you stay within the Always Free limits (4 OCPUs, 24 GB RAM, 200 GB storage, 10 TB bandwidth). Scriptorium uses a tiny fraction of every limit.

Oracle does reserve the right to reclaim idle Always Free instances after they have been idle for extended periods, but a bookshop with regular Sunday POS traffic is not idle.

#### Caveats

| Risk | Severity | Detail |
|------|----------|--------|
| Provisioning difficulty | **High** | ARM instance capacity is frequently exhausted in popular regions (Sydney included). You may need to retry for days or weeks, or use a less popular region (e.g., Melbourne, if available). Upgrading to a Pay-As-You-Go account (credit card required, but free resources still apply) significantly improves provisioning success. |
| Setup complexity | **Medium** | You must install and configure Caddy or nginx for TLS, create a systemd unit for the binary, manage OS security updates, and open firewall ports. This requires basic Linux admin skills. |
| Support | **Low** | Free tier has no SLA and minimal support. Acceptable for a parish bookshop. |
| Account reclamation | **Low** | Oracle may disable idle Always Free instances. Regular POS usage prevents this. |

#### Verdict

Best choice if a volunteer has Linux experience and can handle initial setup. Once running, it requires minimal maintenance. **$0/month forever** is hard to beat.

### C. Hetzner Cloud VPS — Best Paid Option

#### What You Get

| Plan | vCPU | RAM | Disk | Bandwidth | Price |
|------|------|-----|------|-----------|-------|
| CX22 | 2 | 4 GB | 40 GB | 20 TB | **€3.79/month** (~$4.10 USD) |

Massively overprovisioned for this workload. Price increases to ~€4.50–5.00 after April 2026 adjustment.

#### Cost Breakdown

| Line Item | Monthly Cost |
|-----------|-------------|
| CX22 VM | **€3.79** (~$4.10) |
| Bandwidth | Included (20 TB) |
| Everything else | Included |

#### Free Tier Boundary

None. Hetzner has no free tier. You pay from day one.

#### Caveats

- Same manual setup as Oracle (Caddy, systemd, OS updates).
- EU-only for CX series. Closest region to Australia is... not close. Latency from Sydney to Nuremberg/Helsinki is 250–300 ms. **This matters for POS responsiveness.** Consider this option only if the parish is European, or use the CPX series which is available in the US (Ashburn/Hillsboro).
- Excellent reliability and value, widely recommended for small projects.

### D. DigitalOcean — Simple Paid VPS

#### What You Get

| Plan | vCPU | RAM | Disk | Bandwidth | Price |
|------|------|-----|------|-----------|-------|
| Basic (smallest) | 1 | 512 MB–1 GB | 10 GB | 500 GB | **$4/month** |

DigitalOcean has a Sydney region (`SGP` — Singapore is closest; `SYD` available for some products).

#### Cost Breakdown

| Line Item | Monthly Cost |
|-----------|-------------|
| Basic droplet | **$4** |
| Bandwidth | Included (500 GB) |

#### Free Tier Boundary

No free tier. New users get $200 credit for 60 days (trial only). After trial, $4/month minimum.

#### Caveats

Same manual setup as Oracle/Hetzner. Better documentation and community support than Oracle. Per-second billing since January 2026 means you only pay for uptime.

### E. Railway — Easy But Costs More

#### What You Get

Hobby plan at $5/month includes $5 of resource credits. Docker-based deploy, managed TLS, no infrastructure management.

#### Cost Breakdown

| Line Item | Unit Price | Usage | Monthly Cost |
|-----------|-----------|-------|-------------|
| Plan subscription | $5/month | 1 | $5.00 |
| Compute (tiny Rust binary, always-on) | ~$10/vCPU/month, ~$10/GB/month | < 0.1 vCPU, < 0.1 GB | ~$1.00 |
| Volume storage | ~$0.16/GB/month | 1 GB | ~$0.16 |
| **Total** | | | **$5.00** (usage within $5 credit) |

#### Free Tier Boundary

The "free trial" is $5 credit for 30 days. After that, the Hobby plan is $5/month regardless of how little you use. Even if your actual resource consumption is $1.16, you still pay $5. Unused credit does not roll over.

#### Risk of Cost Escalation

Low. Scriptorium's usage is well under the $5 credit. You would need sustained heavy traffic to exceed it. Overages are billed at ~$10/vCPU/month and ~$10/GB-RAM/month.

#### Caveats

- No free tier — $5/month minimum.
- No persistent disk by default — you need to add a volume.
- Good developer experience but poor value for this workload.

### F. Render — Free Tier Exists But Unusable for POS

#### What You Get (Free)

- 750 instance-hours/month (enough for always-on if it didn't spin down).
- **Spin-down after 15 minutes of inactivity.** Wake time: 30–60 seconds.

#### Why It Doesn't Work

During a Sunday sale, the POS needs to respond instantly. A volunteer scans a book and waits. If the service spun down because no one browsed the website in the past 15 minutes, the first scan takes 30–60 seconds. This is unacceptable.

#### Paid Tier

Render's paid plans start at $7/month per service (Starter instance) or $19/month per user (Professional plan). Both are poor value compared to Fly.io or a VPS.

#### Verdict

Free tier is only suitable for online-only storefronts with no POS component. For Scriptorium with POS, **skip Render**.

## Comparison Summary

| Platform | Monthly Cost | Free Tier? | Setup Difficulty | Always-On? | AU Region? | POS Viable? |
|----------|-------------|-----------|-----------------|-----------|-----------|------------|
| **Oracle Cloud** | **$0** | Yes (forever) | Hard (Linux admin) | Yes | Sydney | Yes |
| **Fly.io** | **~$3** | No (pay-as-you-go) | Easy (one command) | Yes | Sydney | Yes |
| **Hetzner** | **~$4** | No | Hard (Linux admin) | Yes | EU only | Only if EU-based |
| **DigitalOcean** | **$4** | No (60-day trial) | Hard (Linux admin) | Yes | Singapore | Yes (some latency) |
| **Railway** | **$5** | No (30-day trial) | Easy | Yes | US/EU | Yes (some latency) |
| **Render** | $0 / $7+ | Yes (but spins down) | Easy | Free: No | US/EU | **No** |

### Recommendation

1. **If someone can do initial Linux setup:** Oracle Cloud Free Tier. $0/month forever. Set it up once, forget about it.
2. **If ease of deployment matters most:** Fly.io. ~$3/month, deploy with one command, no Linux admin needed. This is the default recommendation.
3. **If in Europe:** Hetzner CX22. ~€3.79/month, best performance per dollar, rock-solid reliability.

### Likelihood of Hitting Paid Tiers

| Platform | Free Limit | Parish Usage | Headroom | Likelihood of Overage |
|----------|-----------|-------------|----------|----------------------|
| Oracle Cloud | 4 OCPU, 24 GB RAM, 10 TB bandwidth | < 0.01 OCPU, < 50 MB RAM, < 0.1 GB bandwidth | 99.99% unused | **None — impossible to exceed** |
| Fly.io | No free tier | ~$3/month baseline | N/A — always paid | **Always paid, but cost is stable and predictable** |
| Railway | $5 credit/month | ~$1.16 usage | ~$3.84 buffer | **Very unlikely** — would need 4× traffic |
| Render (free) | 750 hrs, spins down | N/A | N/A | **Unusable for POS** regardless |

## Recommended Platform: Fly.io (Pay-As-You-Go)

Despite not being free, Fly.io is recommended as the default because:

- **No Linux admin skills required** — critical for volunteer-run orgs.
- **Managed TLS** via Let's Encrypt — no certificate management.
- **Persistent volumes** for the SQLite file — survives redeploys.
- **Single-command deploys** with `fly deploy`.
- **Sydney region** — low latency for an Australian parish.
- **~$3/month** — less than a cup of coffee.

### Architecture

```
                    ┌─────────────────────────────────┐
                    │           Fly.io Edge            │
  Browser ─────▶   │  TLS termination + HTTP routing  │
                    └──────────────┬──────────────────┘
                                   │ :8080
                    ┌──────────────▼──────────────────┐
                    │        bookstore-web             │
                    │   Rust binary (axum + tokio)     │
                    │                                  │
                    │   /data/scriptorium.db  ◄────────┤── Fly persistent volume
                    └─────────────────────────────────┘
```

### Project Files

#### `Dockerfile`

Multi-stage build. Stage 1 compiles the release binary. Stage 2 copies it into a minimal Debian slim image with just the SQLite runtime.

```dockerfile
# --- Build stage ---
FROM rust:1.87-bookworm AS builder

WORKDIR /src
COPY . .
RUN cargo build --release -p bookstore-web

# --- Runtime stage ---
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates sqlite3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /src/target/release/bookstore-web /usr/local/bin/bookstore-web

EXPOSE 8080

CMD ["bookstore-web"]
```

**Note:** The binary must be updated to respect a `PORT` environment variable (Fly.io sets this). The current hardcoded `127.0.0.1:8080` must change to `0.0.0.0:${PORT:-8080}`. See [Required Code Change](#required-code-change) below.

#### `fly.toml`

```toml
app = "scriptorium"
primary_region = "syd"          # Sydney — closest to AU parish

[build]

[env]
  DATABASE_URL = "sqlite:///data/scriptorium.db?mode=rwc"
  RUST_LOG     = "info"

[http_service]
  internal_port = 8080
  force_https   = true

  [http_service.concurrency]
    type       = "connections"
    hard_limit = 50
    soft_limit = 25

[[vm]]
  size   = "shared-cpu-1x"
  memory = "256mb"

[mounts]
  source      = "scriptorium_data"
  destination = "/data"
```

### Deployment Steps

```bash
# 1. Install the Fly CLI
curl -L https://fly.io/install.sh | sh

# 2. Sign up / log in (credit card required for pay-as-you-go)
fly auth login

# 3. Create the app (first time only)
fly apps create scriptorium

# 4. Create the persistent volume for SQLite (first time only)
fly volumes create scriptorium_data --region syd --size 1

# 5. Deploy
fly deploy

# 6. Verify
fly status
curl https://scriptorium.fly.dev/health
```

Subsequent deploys are just `fly deploy` from the repo root.

### Custom Domain

```bash
# Add your parish domain
fly certs add bookshop.stmarys.org.au

# Then point a CNAME record at your Fly app:
#   bookshop.stmarys.org.au  CNAME  scriptorium.fly.dev
```

Fly provisions and renews TLS certificates automatically.

## Required Code Change

The server currently hardcodes `127.0.0.1:8080`. For any deployment (Fly.io, Oracle Cloud, or VPS) it must bind to `0.0.0.0` so it is reachable from outside the machine, and respect a `PORT` env var for platforms that assign a port.

In `crates/bookstore-web/src/main.rs`, change the address binding to:

```rust
let port: u16 = std::env::var("PORT")
    .ok()
    .and_then(|p| p.parse().ok())
    .unwrap_or(8080);
let addr = SocketAddr::from(([0, 0, 0, 0], port));
```

This is the only code change required for deployment. Local development is unaffected (defaults to 8080).

## Backups

SQLite makes backups trivial — it is one file. Three layers of protection:

### 1. Pre-deploy Backup (Manual)

Run before every `fly deploy`:

```bash
fly ssh console -C "sqlite3 /data/scriptorium.db '.backup /data/backup-$(date +%Y%m%d-%H%M%S).db'"
```

### 2. Daily Automated Backup (Cron on Fly)

Add a daily backup script that copies the DB to a second volume or off-site. A minimal approach using Fly's built-in scheduled machines:

```bash
# Download a copy locally
fly ssh sftp get /data/scriptorium.db ./backups/scriptorium-$(date +%Y%m%d).db
```

For a volunteer org, a weekly manual download is likely sufficient. Store it in Google Drive or any free cloud storage the parish already uses.

### 3. Volume Snapshots

Fly automatically snapshots persistent volumes daily. Snapshot storage costs $0.08/GB/month with the first 10 GB free — a 1 GB volume costs nothing to snapshot. This is the safety net if a manual backup is missed.

## Monitoring

Minimal monitoring appropriate for a volunteer-run app:

| What | How | Cost |
|------|-----|------|
| Uptime | Free uptime checker (e.g., UptimeRobot free tier) hitting `/health` | $0 |
| Errors | `fly logs` from CLI when something seems wrong | $0 |
| Metrics | Fly dashboard shows CPU, memory, request count | $0 |
| Alerts | UptimeRobot sends email/SMS when `/health` is down | $0 |

No need for Datadog, Grafana, or any paid observability stack.

## Rollback

```bash
# 1. Roll back to previous release
fly releases
fly deploy --image <previous-image-ref>

# 2. If data rollback is also needed, restore from backup
fly ssh console
  sqlite3 /data/scriptorium.db ".restore /data/backup-YYYYMMDD-HHMMSS.db"
```

## Oracle Cloud Deployment Guide

This section covers deploying Scriptorium to Oracle Cloud's Always Free tier. Two variants are described:

- **Option 1: Ubuntu** — conventional setup with apt-installed Caddy and a manually managed systemd unit. Simpler if unfamiliar with NixOS.
- **Option 2: NixOS** — the entire server (OS, Caddy, Scriptorium service, firewall, backups) declared in a single `configuration.nix`. Reproducible, version-controlled, and easier to maintain long-term.

Both cost $0/month.

### Architecture (Both Variants)

```
                    ┌──────────────────────────────────┐
                    │     Oracle Cloud ARM VM           │
                    │     (VM.Standard.A1.Flex)         │
                    │                                   │
  Browser ─────▶   │  Caddy (:443/:80)                 │
                    │    │ auto-TLS (Let's Encrypt)     │
                    │    │ reverse_proxy localhost:8080  │
                    │    ▼                               │
                    │  bookstore-web (:8080)             │
                    │    │ systemd managed               │
                    │    ▼                               │
                    │  /var/lib/scriptorium/data.db      │
                    └──────────────────────────────────┘
```

### Common Steps (Both Variants)

#### Create an Oracle Cloud Account

1. Go to [cloud.oracle.com](https://cloud.oracle.com) and sign up.
2. Choose a **Home Region** with ARM capacity. Sydney (`ap-sydney-1`) is ideal for an Australian parish. If Sydney capacity is exhausted, try Melbourne (`ap-melbourne-1`).
3. **Upgrade to Pay-As-You-Go** (recommended). This does not cost anything — Always Free resources remain free. But it significantly improves your chances of provisioning an ARM instance, which is often unavailable on free-only accounts due to capacity limits. A credit card is required for verification (a temporary $100 hold is placed and released).

#### Provision the ARM VM

1. In the OCI Console, navigate to **Compute > Instances > Create Instance**.
2. Configure:

| Setting | Value |
|---------|-------|
| Name | `scriptorium` |
| Image | Ubuntu 24.04 Minimal — Aarch64 (for both variants — NixOS replaces it later) |
| Shape | VM.Standard.A1.Flex |
| OCPUs | 1 (minimum — 1 of 4 free) |
| Memory | 6 GB (of 24 GB free) |
| Boot volume | 50 GB (of 200 GB free) |
| Network | Create new VCN + public subnet |
| SSH key | Paste your `~/.ssh/id_rsa.pub` or `~/.ssh/id_ed25519.pub` |

3. Click **Create**. If you get an "Out of Capacity" error, retry later or try a different availability domain. Upgrading to Pay-As-You-Go usually resolves this.

4. Note the **Public IP Address** once the instance is running.

#### Open VCN Security List Ports (Cloud Console)

This step is the same regardless of OS. Oracle's cloud-level firewall must allow HTTP and HTTPS traffic.

1. Navigate to **Networking > Virtual Cloud Networks > your VCN > Security Lists > Default Security List**.
2. Click **Add Ingress Rules** and add two rules:

| Source CIDR | Protocol | Dest Port | Description |
|-------------|----------|-----------|-------------|
| 0.0.0.0/0 | TCP | 80 | HTTP |
| 0.0.0.0/0 | TCP | 443 | HTTPS |

SSH (port 22) is already open by default.

---

### Option 1: Ubuntu Setup

#### Open Instance Firewall (iptables)

Ubuntu on Oracle Cloud has iptables rules that block everything except SSH by default. Open HTTP/HTTPS:

```bash
ssh ubuntu@<PUBLIC_IP>

# Add rules for HTTP and HTTPS BEFORE the REJECT line in iptables
sudo iptables -I INPUT 6 -m state --state NEW -p tcp --dport 80 -j ACCEPT
sudo iptables -I INPUT 7 -m state --state NEW -p tcp --dport 443 -j ACCEPT

# Persist the rules
sudo netfilter-persistent save
```

#### Install Caddy

```bash
sudo apt install -y debian-keyring debian-archive-keyring apt-transport-https curl
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | sudo gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | sudo tee /etc/apt/sources.list.d/caddy-stable.list
sudo apt update
sudo apt install caddy
```

Configure by editing `/etc/caddy/Caddyfile`:

```
bookshop.stmarys.org.au {
    reverse_proxy localhost:8080
}
```

Replace `bookshop.stmarys.org.au` with the actual domain. If you don't have a domain yet, use `:80 { reverse_proxy localhost:8080 }` temporarily (HTTP only for bare IPs).

```bash
sudo systemctl reload caddy
```

#### Build and Upload the Binary

The dev machine is aarch64 (same as the Oracle ARM VM), so no cross-compilation is needed:

```bash
# On dev machine
cargo build --release -p bookstore-web
scp target/release/bookstore-web ubuntu@<PUBLIC_IP>:/home/ubuntu/
```

On the server:

```bash
ssh ubuntu@<PUBLIC_IP>
sudo mv /home/ubuntu/bookstore-web /usr/local/bin/bookstore-web
sudo chmod +x /usr/local/bin/bookstore-web
sudo mkdir -p /var/lib/scriptorium
sudo chown ubuntu:ubuntu /var/lib/scriptorium
```

If the dev machine were x86_64, you would cross-compile:

```bash
rustup target add aarch64-unknown-linux-gnu
sudo apt install gcc-aarch64-linux-gnu
CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
  cargo build --release -p bookstore-web --target aarch64-unknown-linux-gnu
```

#### Create a systemd Service

Create `/etc/systemd/system/scriptorium.service`:

```ini
[Unit]
Description=Scriptorium Bookstore
After=network.target

[Service]
Type=simple
User=ubuntu
Environment=DATABASE_URL=sqlite:///var/lib/scriptorium/data.db?mode=rwc
Environment=RUST_LOG=info
ExecStart=/usr/local/bin/bookstore-web
Restart=on-failure
RestartSec=5
WorkingDirectory=/var/lib/scriptorium

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable scriptorium
sudo systemctl start scriptorium
curl http://localhost:8080/health
```

#### Ubuntu Deployments

```bash
# On dev machine
cargo build --release -p bookstore-web
scp target/release/bookstore-web ubuntu@<PUBLIC_IP>:/home/ubuntu/

# On server
ssh ubuntu@<PUBLIC_IP> << 'EOF'
  sqlite3 /var/lib/scriptorium/data.db ".backup /var/lib/scriptorium/backup-$(date +%Y%m%d-%H%M%S).db"
  sudo mv /home/ubuntu/bookstore-web /usr/local/bin/bookstore-web
  sudo chmod +x /usr/local/bin/bookstore-web
  sudo systemctl restart scriptorium
  sleep 1 && curl -s http://localhost:8080/health
EOF
```

#### Ubuntu OS Maintenance

```bash
sudo apt install unattended-upgrades
sudo dpkg-reconfigure --priority=low unattended-upgrades
```

---

### Option 2: NixOS Setup

NixOS makes the entire server configuration declarative. Caddy, the Scriptorium service, firewall rules, automatic updates, and backups are all defined in one file. Changes are applied atomically with rollback support.

#### Install NixOS on the Oracle Cloud VM

Oracle Cloud does not offer NixOS as a base image. The recommended method is to provision with Ubuntu first, then install NixOS via **netboot.xyz**. This is well-documented:

1. SSH into the Ubuntu instance.
2. Download the netboot EFI file:
   ```bash
   sudo curl -Lo /boot/efi/netboot.efi https://boot.netboot.xyz/ipxe/netboot.xyz-arm64.efi
   ```
3. Open a **Cloud Shell serial console** from the OCI Console (Instance > Console connection > Launch Cloud Shell connection).
4. Reboot the instance (`sudo reboot`) and press **Escape** rapidly in the serial console to enter the EFI boot manager.
5. Select **netboot.efi** from the boot menu, then choose **NixOS** > **NixOS 24.11** (or latest).
6. Once the NixOS live installer boots, set a root password and enable SSH:
   ```bash
   # In the NixOS live environment
   passwd   # set temporary root password
   ```
7. SSH into the live environment and partition the disk, install NixOS:
   ```bash
   # Partition (example using parted)
   parted /dev/sda -- mklabel gpt
   parted /dev/sda -- mkpart ESP fat32 1MB 512MB
   parted /dev/sda -- set 1 esp on
   parted /dev/sda -- mkpart primary ext4 512MB 100%

   mkfs.fat -F 32 /dev/sda1
   mkfs.ext4 /dev/sda2

   mount /dev/sda2 /mnt
   mkdir -p /mnt/boot
   mount /dev/sda1 /mnt/boot

   nixos-generate-config --root /mnt
   # Edit /mnt/etc/nixos/configuration.nix (see below)
   nixos-install
   reboot
   ```

For a detailed walkthrough with screenshots, see: [Install NixOS on a Free Oracle Cloud VM (mtlynch.io)](https://mtlynch.io/notes/nix-oracle-cloud/)

Alternative methods:
- [nixos-infect](https://github.com/elitak/nixos-infect) — in-place conversion from Ubuntu (works but riskier)
- [Terraform automation](https://erikparawell.com/oracle-cloud-nixos.html) — fully automated custom image upload

#### NixOS Configuration

Once NixOS is installed, the entire server is managed by `/etc/nixos/configuration.nix`. Here is a complete configuration for running Scriptorium:

```nix
{ config, pkgs, lib, ... }:

{
  # ── Boot ──────────────────────────────────────────────
  boot.loader.systemd-boot.enable = true;
  boot.loader.efi.canTouchEfiVariables = true;

  # ── Networking ────────────────────────────────────────
  networking.hostName = "scriptorium";

  # DHCP for the primary interface (Oracle Cloud assigns IP via DHCP)
  networking.useDHCP = true;

  # Firewall — NixOS manages iptables declaratively, replacing the
  # manual iptables commands needed on Ubuntu
  networking.firewall = {
    enable = true;
    allowedTCPPorts = [ 22 80 443 ];
  };

  # ── Users ─────────────────────────────────────────────
  users.users.deploy = {
    isNormalUser = true;
    extraGroups = [ "wheel" ];
    openssh.authorizedKeys.keys = [
      # Paste your SSH public key here
      "ssh-ed25519 AAAA... you@machine"
    ];
  };

  security.sudo.wheelNeedsPassword = false;

  # ── SSH ───────────────────────────────────────────────
  services.openssh = {
    enable = true;
    settings = {
      PasswordAuthentication = false;
      PermitRootLogin = "no";
    };
  };

  # ── Caddy (reverse proxy + auto-TLS) ─────────────────
  services.caddy = {
    enable = true;
    virtualHosts."bookshop.stmarys.org.au" = {
      extraConfig = ''
        reverse_proxy localhost:8080
      '';
    };
  };

  # ── Scriptorium service ──────────────────────────────
  systemd.services.scriptorium = {
    description = "Scriptorium Bookstore";
    after = [ "network.target" ];
    wantedBy = [ "multi-user.target" ];

    environment = {
      DATABASE_URL = "sqlite:///var/lib/scriptorium/data.db?mode=rwc";
      RUST_LOG = "info";
    };

    serviceConfig = {
      Type = "simple";
      User = "deploy";
      WorkingDirectory = "/var/lib/scriptorium";
      ExecStart = "/usr/local/bin/bookstore-web";
      Restart = "on-failure";
      RestartSec = 5;

      # Hardening
      NoNewPrivileges = true;
      ProtectSystem = "strict";
      ProtectHome = true;
      ReadWritePaths = [ "/var/lib/scriptorium" ];
    };
  };

  # Create the data directory
  systemd.tmpfiles.rules = [
    "d /var/lib/scriptorium 0750 deploy deploy -"
  ];

  # ── Automatic security updates ───────────────────────
  system.autoUpgrade = {
    enable = true;
    allowReboot = true;
    dates = "04:00";   # reboot window if kernel update requires it
  };

  # ── Daily SQLite backup via cron ─────────────────────
  services.cron = {
    enable = true;
    systemCronJobs = [
      "0 3 * * * deploy sqlite3 /var/lib/scriptorium/data.db '.backup /var/lib/scriptorium/backup-$(date +\\%Y\\%m\\%d).db'"
      "0 4 * * * deploy find /var/lib/scriptorium -name 'backup-*.db' -mtime +30 -delete"
    ];
  };

  # ── System packages ──────────────────────────────────
  environment.systemPackages = with pkgs; [
    sqlite
    vim
    htop
    curl
  ];

  system.stateVersion = "24.11";
}
```

Replace `bookshop.stmarys.org.au` with the actual domain. Replace the SSH key with your own.

#### Applying Configuration Changes

```bash
# Edit the configuration
sudo nano /etc/nixos/configuration.nix

# Apply changes (atomic — rolls back on failure)
sudo nixos-rebuild switch
```

If something goes wrong, reboot and select a previous generation from the boot menu. NixOS keeps every previous configuration, making rollback trivial.

#### NixOS Deployments

Upload the new binary and rebuild:

```bash
# On dev machine
cargo build --release -p bookstore-web
scp target/release/bookstore-web deploy@<PUBLIC_IP>:/home/deploy/

# On server
ssh deploy@<PUBLIC_IP>
sqlite3 /var/lib/scriptorium/data.db ".backup /var/lib/scriptorium/backup-$(date +%Y%m%d-%H%M%S).db"
sudo mv /home/deploy/bookstore-web /usr/local/bin/bookstore-web
sudo chmod +x /usr/local/bin/bookstore-web
sudo systemctl restart scriptorium
curl -s http://localhost:8080/health
```

For a more advanced workflow, the binary could be packaged as a Nix derivation and pulled from a binary cache, but scp is sufficient for a single-maintainer project.

#### Why NixOS Over Ubuntu

| Concern | Ubuntu | NixOS |
|---------|--------|-------|
| Server configuration | Scattered across files, applied imperatively | Single `configuration.nix`, applied atomically |
| Rollback | Restore from backup, hope for the best | Boot into previous generation from boot menu |
| Firewall | Manual iptables rules, easy to forget to persist | Declared in config, always applied |
| Caddy | apt-installed, Caddyfile edited separately | Declared in config alongside everything else |
| systemd service | Manually written unit file | Declared in config, type-checked by Nix |
| Security updates | unattended-upgrades (sometimes breaks) | `system.autoUpgrade` with atomic rollback |
| Reproducibility | Drift over time as packages are added/removed | Rebuild from config produces identical system |
| "What's running on this server?" | `ssh in, poke around` | Read `configuration.nix` |

NixOS is the better choice if the maintainer is comfortable with Nix. The entire server becomes version-controllable — commit `configuration.nix` to the Scriptorium repo and the deployment is fully documented by its own source code.

---

### Common: Point DNS

Add a DNS record for your domain (same for both Ubuntu and NixOS):

| Type | Name | Value |
|------|------|-------|
| A | bookshop | `<PUBLIC_IP>` |

Once DNS propagates, Caddy will automatically issue a TLS certificate. Verify:

```bash
curl https://bookshop.stmarys.org.au/health
```

### Common: Backups

```bash
# Manual backup (run anytime)
sqlite3 /var/lib/scriptorium/data.db ".backup /var/lib/scriptorium/backup-$(date +%Y%m%d-%H%M%S).db"

# Download backups to local machine periodically
scp deploy@<PUBLIC_IP>:/var/lib/scriptorium/backup-*.db ./backups/
```

On Ubuntu, set up a cron job manually. On NixOS, the cron job is already declared in `configuration.nix` above.

For off-site storage, copy backups to Google Drive or any free cloud storage the parish already uses.

### Common: Monitoring

| What | How |
|------|-----|
| Service status | `sudo systemctl status scriptorium` |
| Live logs | `sudo journalctl -u scriptorium -f` |
| Uptime alerts | UptimeRobot free tier hitting `https://bookshop.stmarys.org.au/health` |
| Disk usage | `df -h /var/lib/scriptorium` |

### Common: Idle Instance Reclamation

Oracle may reclaim Always Free instances that are idle for 7 consecutive days (95th percentile CPU < 20%). A parish bookshop with regular POS traffic on Sundays will not be reclaimed. As an extra safeguard, the UptimeRobot health check hitting `/health` every 5 minutes generates minimal but non-zero CPU activity.

## Platforms Not Recommended

| Platform | Why |
|----------|-----|
| AWS / GCP / Azure | Overkill. Complex billing. Easy to accidentally incur costs. Volunteers will not enjoy navigating IAM policies. |
| Kubernetes | Extreme overkill for a single-binary SQLite app. |
| Serverless (Lambda, Cloud Run) | SQLite requires a persistent filesystem. Serverless platforms don't provide one. |
| Render (free tier) | Spins down after 15 minutes of inactivity. POS is unusable with 30–60 second cold starts. |
| Shared PHP hosting | Cannot run a Rust binary. |

## Security Considerations

| Concern | Mitigation |
|---------|-----------|
| TLS | Enforced by Fly (`force_https = true`) or Caddy (auto-HTTPS) |
| Admin access | Token-based auth already in app; no public admin registration |
| POS access | PIN-based auth; POS typically used on LAN during events |
| SSH to server | Fly SSH requires authenticated Fly account; VPS uses SSH keys |
| Database exposure | SQLite file is on a private volume, not web-accessible |
| Secrets in env | `DATABASE_URL` and `RUST_LOG` contain no credentials |
| CSRF | Already implemented for admin mutation routes |
| CDN supply chain | HTMX, Preact, Google Fonts are pinned to specific versions |

## Future Considerations

These are out of scope for the initial deployment but noted for later:

- **Litestream** — continuous SQLite replication to S3-compatible storage ($0 with Cloudflare R2 free tier). Add this when the bookshop reaches a scale where losing even one day of data matters.
- **Staging environment** — deploy a second Fly app (`scriptorium-staging`) when there are multiple contributors.
- **CI/CD** — add a GitHub Actions workflow that runs `cargo test` then `fly deploy` on push to `master`. Free for public repos.
- **PWA manifest** — for POS volunteers to "install" the app on their phone home screen. No app store needed.
- **Payment webhook endpoint** — when a payment provider is chosen (e.g., Square, Stripe), configure their webhook to point at `https://scriptorium.fly.dev/api/payments/webhook`.

## Summary

| | |
|---|---|
| Recommended platform | Fly.io (pay-as-you-go) |
| Cheapest platform | Oracle Cloud (Always Free — $0 forever, requires Linux admin) |
| Region | `syd` (Sydney) |
| Machine | `shared-cpu-1x`, 256 MB RAM |
| Storage | 1 GB persistent volume |
| TLS | Managed by Fly (Let's Encrypt) |
| Deploy | `fly deploy` from repo root |
| Backup | Fly volume snapshots + manual `fly ssh sftp get` |
| Monitoring | UptimeRobot free tier on `/health` |
| Monthly cost | **~$3** (Fly.io) or **$0** (Oracle Cloud) |
| Code changes needed | One: bind to `0.0.0.0:$PORT` instead of `127.0.0.1:8080` |

## Sources

- [Fly.io Resource Pricing](https://fly.io/docs/about/pricing/)
- [Fly.io Free Trial](https://fly.io/docs/about/free-trial/)
- [Fly.io Regions](https://fly.io/docs/reference/regions/)
- [Oracle Cloud Always Free Resources](https://docs.oracle.com/en-us/iaas/Content/FreeTier/freetier_topic-Always_Free_Resources.htm)
- [Oracle Cloud Free Tier FAQ](https://www.oracle.com/cloud/free/faq/)
- [Install NixOS on a Free Oracle Cloud VM (mtlynch.io)](https://mtlynch.io/notes/nix-oracle-cloud/)
- [NixOS on Free Oracle Cloud Arm A1 (NixOS Discourse)](https://discourse.nixos.org/t/nixos-on-free-oracle-cloud-arm-a1/17474)
- [Fully Automated NixOS on Oracle Cloud (erikparawell.com)](https://erikparawell.com/oracle-cloud-nixos.html)
- [nixos-infect (GitHub)](https://github.com/elitak/nixos-infect)
- [NixOS Wiki: Install on Oracle Cloud](https://wiki.nixos.org/wiki/Install_NixOS_on_Oracle_Cloud)
- [NixOS Wiki: Caddy](https://wiki.nixos.org/wiki/Caddy)
- [Railway Pricing Plans](https://docs.railway.com/reference/pricing/plans)
- [Render Deploy for Free](https://render.com/docs/free)
- [Hetzner Cloud Pricing](https://www.hetzner.com/cloud)
- [DigitalOcean Droplet Pricing](https://www.digitalocean.com/pricing/droplets)
