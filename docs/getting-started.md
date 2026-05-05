# Getting Started

## Prerequisites

- Rust stable toolchain (for building from source)
- Docker + Docker Compose (for full image encryption and compose hardening flows)

Alternatively, use the pre-built container — no Rust required (see [Zero-Install Docker Quickstart](#zero-install-docker-quickstart)).

## Zero-Install Docker Quickstart

```bash
# Pull VaultShip
docker pull ghcr.io/cyberxdefend/vaultship:latest

# Initialize config, generate keys, build and inspect — all from the container
docker run --rm -v "$PWD":/workspace -w /workspace \
  ghcr.io/cyberxdefend/vaultship:latest vaultship init --profile baseline

docker run --rm -v "$PWD":/workspace -w /workspace \
  ghcr.io/cyberxdefend/vaultship:latest vaultship keygen --name vaultship

# Mount Docker socket for full image encryption
docker run --rm \
  -v "$PWD":/workspace \
  -v /var/run/docker.sock:/var/run/docker.sock \
  -w /workspace \
  ghcr.io/cyberxdefend/vaultship:latest vaultship build docker-compose.yml

docker run --rm -v "$PWD":/workspace -w /workspace \
  ghcr.io/cyberxdefend/vaultship:latest vaultship inspect api --json
```

## Build from Source

```bash
cargo build --workspace
cargo run -p vaultship-cli -- --help
```

## First End-to-End Example

```bash
# 1. Generate signing keys
cargo run -p vaultship-cli -- keygen --name vaultship

# 2. Harden + generate seccomp profile
cargo run -p vaultship-cli -- harden tests/fixtures/sample-app/docker-compose.yml

# 3. Build protected artifact
#    With Docker running: exports and encrypts the actual image layers
#    Without Docker: falls back to encrypting the compose configuration
cargo run -p vaultship-cli -- build tests/fixtures/sample-app/docker-compose.yml

# 4. Inspect artifact protections
cargo run -p vaultship-cli -- inspect sample-app --json
```

> **What does `build` encrypt?**
> When Docker is available and the referenced image can be pulled, `build` exports the full Docker image tar and encrypts it with AES-256-GCM. When Docker is unavailable or the image cannot be found, it falls back to encrypting the compose configuration and emits a warning. The `content_type` field in the manifest JSON distinguishes the two modes.

## OCI Registry Flow

```bash
# Start a local registry for testing
docker run -d --rm --name vaultship-registry -p 5001:5000 registry:2

# Push encrypted OCI artifact to registry
cargo run -p vaultship-cli -- push http://localhost:5001/vaultship

# Pull encrypted OCI artifact from registry and decrypt locally
cargo run -p vaultship-cli -- pull http://localhost:5001/vaultship/api:latest
```

## Runtime Engine Parity

Use `--engine` in `run` to select your local container runtime:

```bash
cargo run -p vaultship-cli -- run api \
  --bind-file vaultship.bind.json \
  --public-key vaultship.public.key \
  --engine docker \
  --dry-run

cargo run -p vaultship-cli -- run api \
  --bind-file vaultship.bind.json \
  --public-key vaultship.public.key \
  --engine podman \
  --dry-run
```

## Hardware Binding

```bash
# Collect fingerprint of the target machine
cargo run -p vaultship-cli -- fingerprint > fingerprint.json

# Create a hardware-bound key file (signed with your private key)
cargo run -p vaultship-cli -- bind \
  --key-file vaultship.layer.key \
  --private-key vaultship.private.key \
  --fingerprint fingerprint.json \
  --output vaultship.bind.json

# Run with hardware binding enforced
cargo run -p vaultship-cli -- run api \
  --bind-file vaultship.bind.json \
  --public-key vaultship.public.key \
  --dry-run
```

## Signature Verification

Every artifact built by `vaultship build` is signed with Ed25519 using `vaultship.private.key`. You can verify at any time:

```bash
# Verify via artifact name (reads manifest + verifies signature)
cargo run -p vaultship-cli -- verify api --public-key vaultship.public.key

# Verify a specific signed ref directly
cargo run -p vaultship-cli -- verify "api@sig:BASE64SIGNATURE" \
  --public-key vaultship.public.key --json
```

## License Management

```bash
# Vendor: generate a license keypair
cargo run -p vaultship-cli -- license keygen --name vaultship

# Vendor: create a signed license for a customer
cargo run -p vaultship-cli -- license create \
  --customer "Acme Corp" \
  --product "myapp" \
  --expires 2027-01-01 \
  --seats 5 \
  --features scan,harden

# Customer: validate a received license file
cargo run -p vaultship-cli -- license validate license-Acme\ Corp.key \
  --public-key vaultship.public.key

# Customer: collect hardware fingerprint for hardware-bound license
cargo run -p vaultship-cli -- license fingerprint
```

## Troubleshooting

| Error | Cause | Fix |
| --- | --- | --- |
| `Private key not found at vaultship.private.key` | Signing keys not generated | Run `vaultship keygen --name vaultship` |
| `Hardware fingerprint mismatch` | Bind file from different machine | Re-run `vaultship bind` on target machine |
| `Docker daemon not available` | Docker not running | Start Docker, or mount `/var/run/docker.sock` in container mode |
| `Public key not found` | Key mismatch | Ensure `vaultship.public.key` matches the keypair used at build time |
| Registry auth failure | Missing credentials | Set `VAULTSHIP_REGISTRY_TOKEN` or username/password env vars |
