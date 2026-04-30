# Binding Guide

VaultShip uses signed execution authorization files that can bind container runtime permission to hardware fingerprints.

## Operator Flow

```bash
# 1) generate keys
cargo run -p vaultship-cli -- keygen --name vaultship

# 2) collect fingerprint on target machine
cargo run -p vaultship-cli -- fingerprint > fingerprint.json

# 3) create signed bound key file
cargo run -p vaultship-cli -- bind \
  --key-file vaultship.layer.key \
  --private-key vaultship.private.key \
  --fingerprint fingerprint.json \
  --output vaultship.bind.json

# 4) validate implicitly via runtime execution gate
cargo run -p vaultship-cli -- run api --bind-file vaultship.bind.json --public-key vaultship.public.key --dry-run
```

## Important Scope

This authorization is container-level protection. Business licensing concerns (feature plans, seat limits, subscription billing) remain application responsibilities outside VaultShip.
