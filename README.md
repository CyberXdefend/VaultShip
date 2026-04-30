# VaultShip

VaultShip is a protected container runtime toolkit for hardening, signing, hardware-binding, and encrypting containerized software.

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
