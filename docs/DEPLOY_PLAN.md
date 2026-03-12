# Deploy Plan

## Goal

Automate deployment of a staging site on Oracle Cloud Infrastructure (OCI) using:

- a NixOS image for the host
- Pulumi for OCI infrastructure
- NixOS modules for host configuration
- Caddy or Nginx managed by NixOS

The current application shape is:

- one Rust web service binary
- one SQLite database file
- optional object storage for uploaded media

That means staging should be designed as a single-node deployment with persistent storage, not as a horizontally scaled container platform.

## Recommendation

Use:

- **Pulumi** for OCI resource provisioning
- **NixOS** for machine configuration
- **Caddy** as the default reverse proxy for staging

Use Nginx only if we need feature parity with an existing Nginx estate or more custom proxy behavior.

### Why Caddy by default

- simpler NixOS configuration
- simpler TLS setup
- good fit for one app and one staging hostname
- less operational overhead than Nginx for this deployment shape

## Current Gaps In The Repo

Before staging automation is fully usable, the app should be adjusted in a few places:

1. The web server currently binds to `127.0.0.1:8080` in `crates/bookstore-web/src/main.rs`.
2. The repo packages the CLI with Nix today, but not the web service or a NixOS machine configuration.
3. The deployment docs describe a manual process, not an automated rollout.
4. The health endpoint is minimal and not strong enough for deploy gating.

## Target Staging Architecture

### OCI Infrastructure

Provision with Pulumi:

- VCN
- subnet
- network security rules / NSG
- one compute instance using a NixOS custom image
- one block volume for persistent app data
- one reserved public IP
- DNS record for the staging hostname
- optional OCI Object Storage bucket for uploaded covers

### Host Responsibilities

Configure with NixOS:

- reverse proxy via Caddy or Nginx
- TLS termination
- firewall rules
- `systemd` service for `bookstore-web`
- mount persistent volume for SQLite and backups
- application environment variables
- log and restart policy

### Application Responsibilities

The Rust service should:

- listen on a configurable host and port
- use `DATABASE_URL` pointing at the mounted persistent volume
- expose a stronger readiness endpoint for smoke checks

## Deployment Flow

### 1. Base Image

Create or import a NixOS image into OCI and use it as the base image for the compute instance.

This should be done once and then updated only when we need host-level changes that are easier to bake into the image.

### 2. Infrastructure Provisioning

Use Pulumi to provision or update OCI resources:

- networking
- VM instance
- storage
- public IP
- DNS
- optional object storage

Pulumi should manage infrastructure state, not individual application release versions.

### 3. Host Configuration

Manage the machine using NixOS configuration checked into this repo.

Suggested structure:

- `flake.nix`
- `infra/nixos/modules/scriptorium.nix`
- `infra/nixos/hosts/staging.nix`
- `infra/pulumi/oci-staging/`

The NixOS module should define:

- the `bookstore-web` service
- the reverse proxy
- persistent storage mount
- firewall rules
- deploy-time environment file location

### 4. Application Rollout

Use CI to deploy the app artifact to the OCI VM:

1. build the release binary
2. run tests and deploy gates
3. upload the artifact to the host
4. switch the active release symlink
5. take a SQLite backup before promotion
6. restart the systemd service
7. run smoke checks against staging

This keeps application deploys separate from infrastructure changes.

## Proposed Repo Changes

### Application

1. Add configurable bind host and port.
2. Add a readiness endpoint that verifies database connectivity.
3. Keep SQLite on a mounted persistent volume such as `/var/lib/scriptorium`.

### Nix

1. Add a `bookstore-web` package to the flake.
2. Add a NixOS module for running the service.
3. Add a staging host config for OCI.

### Infrastructure

1. Add a Pulumi project for OCI staging resources.
2. Define networking, compute, storage, and optional object storage there.

### Operations

1. Add a deploy script for binary rollout.
2. Add `systemd` unit management via NixOS.
3. Update deployment docs to reflect the automated staging flow.

## Reverse Proxy Choice

### Preferred: Caddy

Choose Caddy if the goal is fast, low-friction staging setup.

Benefits:

- compact configuration
- easy HTTPS setup
- simpler operational model for one site

### Alternative: Nginx

Choose Nginx if we need:

- more custom routing behavior
- fine-grained header or cache control
- consistency with an existing Nginx operational model

## Example NixOS Direction

For staging, the intended shape is:

- `bookstore-web` listens on localhost
- Caddy or Nginx listens publicly on 80/443
- SQLite lives on the mounted persistent volume
- environment variables are supplied via a managed env file or secret path

Representative NixOS approach:

```nix
{
  services.caddy = {
    enable = true;
    virtualHosts."staging.example.com".extraConfig = ''
      reverse_proxy 127.0.0.1:8080
    '';
  };

  networking.firewall.allowedTCPPorts = [ 80 443 ];

  systemd.services.scriptorium = {
    description = "Scriptorium web";
    wantedBy = [ "multi-user.target" ];
    after = [ "network-online.target" ];
    serviceConfig = {
      ExecStart = "/opt/scriptorium/current/bin/bookstore-web";
      WorkingDirectory = "/var/lib/scriptorium";
      Restart = "always";
      EnvironmentFile = "/run/secrets/scriptorium.env";
    };
  };
}
```

## Suggested Rollout Order

1. Make the app bind address configurable.
2. Add Nix packaging for the web service.
3. Add the NixOS module and staging host definition.
4. Add Pulumi OCI infrastructure.
5. Add the deploy script and smoke checks.
6. Update the operational docs and release checklist.

## Notes

- Staging should remain single-node while SQLite is the primary database.
- OCI Object Storage is a reasonable fit for uploaded media if we keep the current S3-compatible storage approach.
- Pulumi should provision the environment, but CI should perform the application rollout.

## Cost Estimate And Platform Options

Because this environment will be lightly used for staging and testing, the cost target should be "cheap, persistent, and easy to reset" rather than "production-grade HA".

### Option A: OCI + NixOS + Caddy

This remains the best fit if we want:

- a NixOS image
- NixOS-managed host configuration
- Pulumi-managed infrastructure
- full control over the machine

Estimated monthly cost:

- around `$0-$5/month` if the tenancy can use OCI Always Free resources effectively
- around `$24-$26/month` for a small paid VM plus about `50 GB` of block storage
- around `$32-$37/month` if we also add an OCI Load Balancer instead of running Caddy directly on the VM

Why the range exists:

- compute is the main cost
- block volume is cheap
- object storage is usually negligible at staging scale
- using Caddy on the VM is materially cheaper than adding a managed load balancer

Recommended OCI staging posture:

- one small VM
- one persistent volume
- Caddy on-host
- no OCI load balancer

### Option B: Fly.io

Fly.io is a strong low-traffic staging option for this app shape.

Why it fits:

- runs a single app instance easily
- supports persistent local volumes for SQLite
- can be much cheaper than a paid OCI VM for a barely-used environment

Estimated monthly cost:

- about `$3-$8/month` for a very small shared machine that is always on
- plus about `$0.15/GB-month` for volume storage
- plus little or no certificate cost for a small number of hostnames

Operational tradeoffs:

- simpler app deployment than OCI
- less aligned with the "NixOS image on the host" goal
- persistent volume is tied to one region and one machine placement model
- lower infra control than OCI

Fit assessment:

- excellent if the primary goal is the cheapest persistent staging site
- not ideal if the goal is specifically to exercise NixOS host management on Oracle

### Option C: Railway

Railway is also a viable lightweight staging option.

Why it fits:

- extremely low setup overhead
- supports persistent volumes
- usage-based pricing is straightforward for small apps

Estimated monthly cost:

- effectively about `$5/month` minimum on the Hobby plan if usage stays within included credits
- modestly above that if the service runs continuously with a volume attached

Operational tradeoffs:

- easiest developer experience of the options
- less infra control than OCI
- no NixOS machine model
- staging becomes platform-managed rather than host-managed

Fit assessment:

- very good if the goal is fast setup and low spend
- not a match for the NixOS-on-OCI direction

### Option D: Cloudflare

Cloudflare is not a strong primary hosting target for the current app architecture.

Reasons:

- the current app is a Rust server binary, not a Workers-native application
- the current persistence model is SQLite on local disk
- Cloudflare Workers does not map cleanly onto "one Rust binary + one SQLite file"

Where Cloudflare does fit:

- DNS and edge proxy in front of another host
- Cloudflare Tunnel to expose a private staging machine
- optional object storage or edge services later if the architecture changes

Estimated monthly cost:

- potentially `$0` or very low if used only for Tunnel and DNS
- but it should be treated as an edge layer, not the primary runtime for this service

Fit assessment:

- useful as an adjunct to OCI, Fly, or a local machine
- not recommended as the primary staging platform for the current application design

## Recommended Decision

There are two sensible paths:

### If the goal is to validate the intended long-term Oracle/NixOS approach

Choose:

- OCI
- NixOS custom image
- Pulumi
- Caddy on the VM

Expected cost:

- roughly `$24-$26/month` for a simple paid setup
- potentially much lower if OCI free resources are available

### If the goal is the cheapest possible low-traffic staging site

Choose:

- Fly.io first

Expected cost:

- roughly `$3-$8/month` plus small storage charges

This is likely the lowest-friction persistent staging option for the current app shape.

### If the goal is "fastest to stand up with low cost"

Choose:

- Railway first

Expected cost:

- roughly `$5/month` minimum for a low-usage Hobby setup

This is probably the easiest platform path, but it does not help with the NixOS host-management direction.

## Current Recommendation

Because the stated preference is to use a NixOS image and manage Caddy or Nginx with NixOS, the primary recommendation remains:

- **OCI + NixOS + Caddy**

But if cost minimization matters more than Oracle alignment for staging, the best fallback is:

- **Fly.io for staging**
- keep OCI + NixOS for later production or pre-production work

## Pricing References

These estimates are based on current provider pricing and product docs:

- OCI compute and storage pricing:
  - https://www.oracle.com/cl/cloud/compute/gpu/pricing/
  - https://www.oracle.com/cloud/storage/block-volumes/pricing/
  - https://www.oracle.com/cloud/data-egress-costs/
- OCI custom images and NixOS on Oracle:
  - https://docs.oracle.com/en-us/iaas/Content/Compute/Tasks/managingcustomimages.htm
  - https://docs.oracle.com/en-us/iaas/Content/Compute/References/bringyourownimage.htm
  - https://wiki.nixos.org/wiki/Install_NixOS_on_Oracle_Cloud
- Fly.io pricing:
  - https://fly.io/pricing/
  - https://fly.io/docs/about/pricing/
- Railway pricing:
  - https://railway.com/pricing
  - https://docs.railway.com/pricing/plans
- Cloudflare pricing and Tunnel:
  - https://developers.cloudflare.com/workers/platform/pricing/
  - https://www.cloudflare.com/products/tunnel/
