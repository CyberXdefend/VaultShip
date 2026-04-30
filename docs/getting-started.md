# Getting Started

## Prerequisites

- Rust stable toolchain
- Docker + Docker Compose (for compose hardening flows)

## Build

```bash
cargo build --workspace
```

## Run CLI Help

```bash
cargo run -p vaultship-cli -- --help
```

## First End-to-End Example

```bash
# harden + seccomp generation
cargo run -p vaultship-cli -- harden tests/fixtures/sample-app/docker-compose.yml

# build protected local artifact
cargo run -p vaultship-cli -- build tests/fixtures/sample-app/docker-compose.yml

# inspect artifact protections
cargo run -p vaultship-cli -- inspect sample-app
```

## OCI Registry Flow

```bash
# start local registry for testing
docker run -d --rm --name vaultship-registry -p 5001:5000 registry:2

# push encrypted OCI artifact to registry
cargo run -p vaultship-cli -- push http://localhost:5001/vaultship

# pull encrypted OCI artifact from registry and decrypt locally
cargo run -p vaultship-cli -- pull http://localhost:5001/vaultship/api:latest
```

## Runtime Engine Parity

Use `--engine` in `run` to select your local runtime CLI:

```bash
cargo run -p vaultship-cli -- run api --bind-file vaultship.bind.json --public-key vaultship.public.key --engine docker --dry-run
cargo run -p vaultship-cli -- run api --bind-file vaultship.bind.json --public-key vaultship.public.key --engine podman --dry-run
cargo run -p vaultship-cli -- run api --bind-file vaultship.bind.json --public-key vaultship.public.key --engine nerdctl --dry-run
```

## License Example

```bash
cargo run -p vaultship-cli -- keygen --name vaultship
cargo run -p vaultship-cli -- fingerprint > fingerprint.json
cargo run -p vaultship-cli -- bind \
  --key-file vaultship.layer.key \
  --private-key vaultship.private.key \
  --fingerprint fingerprint.json \
  --output vaultship.bind.json
cargo run -p vaultship-cli -- run api --bind-file vaultship.bind.json --public-key vaultship.public.key --dry-run
```
