#!/usr/bin/env bash
set -euo pipefail

# CI: compile and test
cargo build --workspace
cargo test --workspace

# Build and publish protected artifact to registry path
cargo run -p vaultship-cli -- build examples/dev/docker-compose.yml
cargo run -p vaultship-cli -- push .vaultship/registry

# Simulated deployment host pull + validation
cargo run -p vaultship-cli -- pull api
cargo run -p vaultship-cli -- inspect api
