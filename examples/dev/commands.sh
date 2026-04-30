#!/usr/bin/env bash
set -euo pipefail

cargo build --workspace
cargo run -p vaultship-cli -- init
cp examples/dev/vaultship.toml ./vaultship.toml

cargo run -p vaultship-cli -- harden examples/dev/docker-compose.yml
cargo run -p vaultship-cli -- build examples/dev/docker-compose.yml
cargo run -p vaultship-cli -- inspect api

cargo run -p vaultship-cli -- license keygen --name vaultship
cargo run -p vaultship-cli -- license create \
  --customer "Dev-Team" \
  --product "vaultship-demo" \
  --hardware-bind \
  --expires 2027-12-31 \
  --seats 10 \
  --features scan,report,policy \
  --key vaultship.private.key

cargo run -p vaultship-cli -- license validate license-Dev-Team.key --public-key vaultship.public.key
