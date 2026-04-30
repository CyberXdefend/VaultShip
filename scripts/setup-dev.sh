#!/usr/bin/env bash
set -euo pipefail
rustup show >/dev/null
cargo --version
echo "VaultShip dev environment ready."
