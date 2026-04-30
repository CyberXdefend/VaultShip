# VaultShip

VaultShip is a protected container runtime toolkit for hardening, signing, hardware-binding, and encrypting containerized software.

VaultShip: protected container delivery for commercial self-hosted software.

## Why VaultShip

VaultShip helps teams protect containerized software with stronger anti-tamper controls and machine-bound execution controls.

## Scope Boundary

VaultShip handles container-level protection only:

- image encryption/decryption
- runtime hardening
- signature verification
- hardware binding / policy gating for execution

VaultShip does not implement application business licensing:

- feature flags
- seat counting
- subscriptions/billing
- app user auth/entitlements

## Quick Start

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
cargo run -p vaultship-cli -- init

# 2) harden a compose service
cargo run -p vaultship-cli -- harden tests/fixtures/sample-app/docker-compose.yml

# 3) build local protected artifact
cargo run -p vaultship-cli -- build tests/fixtures/sample-app/docker-compose.yml

# 4) inspect resulting protection metadata
cargo run -p vaultship-cli -- inspect sample-app
```

## DevOps Workflow

```bash
# push/pull protected artifacts to a registry directory
cargo run -p vaultship-cli -- push .vaultship/registry
cargo run -p vaultship-cli -- pull sample-app

# push/pull through OCI Registry HTTP API (real registry path)
# (example local registry reference)
cargo run -p vaultship-cli -- push http://localhost:5001/vaultship
cargo run -p vaultship-cli -- pull http://localhost:5001/vaultship/api:latest

# runtime engine parity (docker/podman/nerdctl)
cargo run -p vaultship-cli -- run api --bind-file vaultship.bind.json --public-key vaultship.public.key --engine docker --dry-run
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
vaultship build docker-compose.yml
vaultship inspect api --json

# Create and enforce machine binding
vaultship keygen --name vaultship
vaultship fingerprint > fingerprint.json
vaultship bind --key-file vaultship.layer.key --private-key vaultship.private.key --fingerprint fingerprint.json --output vaultship.bind.json
vaultship run api --bind-file vaultship.bind.json --public-key vaultship.public.key --dry-run
```

### Option B: Use VaultShip Container In CI/CD

```bash
docker run --rm -v "$PWD":/workspace -w /workspace ghcr.io/cyberxdefend/vaultship:latest \
  vaultship build docker-compose.yml

docker run --rm -v "$PWD":/workspace -w /workspace ghcr.io/cyberxdefend/vaultship:latest \
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
vaultship_sign::verify::verify_signature(signed_ref)?;
```
