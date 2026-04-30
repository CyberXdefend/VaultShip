#!/usr/bin/env bash
set -euo pipefail

# Vendor side: generate keys once
cargo run -p vaultship-cli -- license keygen --name vaultship

# Client side: collect fingerprint
cargo run -p vaultship-cli -- license fingerprint > fingerprint.json

# Vendor side: issue license
cargo run -p vaultship-cli -- license create \
  --customer "Prod-Cluster-A" \
  --product "vaultship-production" \
  --hardware-bind \
  --fingerprint fingerprint.json \
  --expires 2027-12-31 \
  --seats 100 \
  --features policy,reporting,audit \
  --key vaultship.private.key

# Runtime side: validate
cargo run -p vaultship-cli -- license validate license-Prod-Cluster-A.key --public-key vaultship.public.key
