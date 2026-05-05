# VaultShip

VaultShip is a protected container runtime toolkit for hardening, signing, hardware-binding, and encrypting containerized software.

VaultShip: protected container delivery for commercial self-hosted software.

## Why VaultShip

VaultShip helps teams protect containerized software with stronger anti-tamper controls and machine-bound execution controls.

## Scope Boundary

VaultShip handles container-level protection only:

- image encryption/decryption (actual Docker image export or compose configuration)
- runtime hardening
- signature verification (Ed25519)
- hardware binding / policy gating for execution

VaultShip does not implement application business licensing:

- feature flags
- seat counting
- subscriptions/billing
- app user auth/entitlements

## Zero-Install Docker Quickstart

No Rust required. Mount your project into the VaultShip container:

```bash
# Pull VaultShip
docker pull ghcr.io/cyberxdefend/vaultship:latest

# Initialize a config file in your project
docker run --rm -v "$PWD":/workspace -w /workspace \
  ghcr.io/cyberxdefend/vaultship:latest \
  vaultship init --profile baseline

# Generate signing keys
docker run --rm -v "$PWD":/workspace -w /workspace \
  ghcr.io/cyberxdefend/vaultship:latest \
  vaultship keygen --name vaultship

# Harden and encrypt your compose service
# (exports real Docker image layers when Docker socket is mounted)
docker run --rm \
  -v "$PWD":/workspace \
  -v /var/run/docker.sock:/var/run/docker.sock \
  -w /workspace \
  ghcr.io/cyberxdefend/vaultship:latest \
  vaultship build docker-compose.yml

# Inspect the protected artifact
docker run --rm -v "$PWD":/workspace -w /workspace \
  ghcr.io/cyberxdefend/vaultship:latest \
  vaultship inspect api --json
```

## Build from Source

```bash
cargo build --workspace
cargo run -p vaultship-cli -- --help
```

## Binary Distribution

```bash
# crates.io
cargo install vaultship-cli

# install script (from GitHub repo)
curl -fsSL https://raw.githubusercontent.com/cyberxdefend/vaultship/main/scripts/install.sh | sh

# direct binaries
https://github.com/cyberxdefend/vaultship/releases
```

Packaging templates for additional distribution channels are available under `packaging/`:

- Homebrew: `packaging/homebrew/Formula/vaultship.rb`
- Snap: `packaging/snap/snapcraft.yaml`
- AUR: `packaging/aur/PKGBUILD`
- Chocolatey: `packaging/chocolatey/vaultship.nuspec`
- winget: `packaging/winget/vaultship.yaml`
- Nix: `packaging/nix/default.nix`

## Developer Workflow

```bash
# 1) initialize config
vaultship init --profile baseline

# 2) generate signing keys
vaultship keygen --name vaultship

# 3) harden a compose service (generates docker-compose.hardened.yml + seccomp-profile.json)
vaultship harden docker-compose.yml

# 4) build protected artifact
#    - when Docker is available: exports and encrypts the actual image tar
#    - fallback: encrypts the compose configuration
#    - always signs with vaultship.private.key
vaultship build docker-compose.yml

# 5) inspect resulting protection metadata
vaultship inspect api --json
```

## DevOps Workflow

```bash
# push/pull protected artifacts to a local registry directory
vaultship push .vaultship/registry
vaultship pull api

# push/pull through OCI Registry HTTP API
vaultship push http://localhost:5001/vaultship
vaultship pull http://localhost:5001/vaultship/api:latest

# runtime engine parity (docker/podman/nerdctl)
vaultship run api \
  --bind-file vaultship.bind.json \
  --public-key vaultship.public.key \
  --engine docker \
  --dry-run
```

## More Examples

- Developer examples: `examples/dev`
- DevOps examples: `examples/devops`
- SDK embed example: `examples/sdk/main.rs`
- Full index: `examples/README.md`
- Protection profiles: `docs/protection-profiles.md`
- Exit codes: `docs/exit-codes.md`
- Observability: `docs/observability.md`

## Use VaultShip In Your Project

### Option A: Install CLI on Host

```bash
# Install from crates.io
cargo install vaultship-cli

# Protect an existing compose service
vaultship init --profile baseline
vaultship keygen --name vaultship
vaultship build docker-compose.yml
vaultship inspect api --json

# Create and enforce machine binding
vaultship fingerprint > fingerprint.json
vaultship bind \
  --key-file vaultship.layer.key \
  --private-key vaultship.private.key \
  --fingerprint fingerprint.json \
  --output vaultship.bind.json
vaultship run api \
  --bind-file vaultship.bind.json \
  --public-key vaultship.public.key \
  --dry-run
```

### Option B: Use VaultShip Container In CI/CD

VaultShip is packaged as a CLI container. Mount your workspace into it to run any `vaultship` command without installing Rust or the binary. Mount the Docker socket to enable full image encryption.

```bash
# Run vaultship commands against your local workspace
docker run --rm \
  -v "$PWD":/workspace \
  -v /var/run/docker.sock:/var/run/docker.sock \
  -w /workspace \
  ghcr.io/cyberxdefend/vaultship:latest \
  vaultship build docker-compose.yml

# Inspect the result
docker run --rm -v "$PWD":/workspace -w /workspace \
  ghcr.io/cyberxdefend/vaultship:latest \
  vaultship inspect api --json
```

### Option C: Use VaultShip Crates In Rust Code

```toml
[dependencies]
vaultship-encrypt = "0.1.0"
vaultship-harden = "0.1.0"
vaultship-sign = "0.1.0"
```

```rust
let hardened = vaultship_harden::harden_compose_document(compose_yaml, &config)?;
let encrypted = vaultship_encrypt::encrypt::encrypt_layer(bytes, &key)?;
vaultship_sign::verify::verify_signature(signed_ref, "vaultship.public.key")?;
```

## License Management

VaultShip includes a license subsystem for distributing signed, optionally hardware-bound deployment licenses.

```bash
# Vendor side: generate a license keypair
vaultship license keygen --name vaultship

# Vendor side: create a license for a customer (optionally hardware-bound)
vaultship license create \
  --customer "Acme Corp" \
  --product "myapp" \
  --expires 2027-01-01 \
  --seats 5 \
  --features scan,harden

# Customer side: validate a received license
vaultship license validate license-Acme Corp.key \
  --public-key vaultship.public.key

# Customer side: collect hardware fingerprint for hardware-bound license
vaultship license fingerprint
```

## Troubleshooting

**`Private key not found at vaultship.private.key`** — Run `vaultship keygen --name vaultship` first to generate signing keys.

**`Hardware fingerprint mismatch`** — The bind file was created on a different machine. Re-run `vaultship bind` on the target machine.

**`Docker daemon not available`** — `vaultship build` falls back to encrypting the compose configuration. Mount `/var/run/docker.sock` to enable full image encryption.

**Registry auth** — Set `VAULTSHIP_REGISTRY_TOKEN` (bearer) or `VAULTSHIP_REGISTRY_USERNAME`/`VAULTSHIP_REGISTRY_PASSWORD` (basic). For GHCR: `GITHUB_TOKEN` + `GITHUB_ACTOR`. For AWS ECR: `AWS_ECR_PASSWORD`. For Azure ACR: `AZURE_ACR_USERNAME`/`AZURE_ACR_PASSWORD`.

**Signature verification fails** — Verify that `vaultship.public.key` matches the `vaultship.private.key` used during `vaultship build`. Both are generated together by `vaultship keygen`.
