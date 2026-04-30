# VaultShip Distribution Plan

This document tracks binary distribution channels and implementation status.

## Day 1 (implemented)

- GitHub Releases via `.github/workflows/release.yml`
- Install script via:
  - `curl -fsSL https://raw.githubusercontent.com/cyberxdefend/vaultship/main/scripts/install.sh | sh`
- crates.io publish from release pipeline (`vaultship-cli`)
- Docker image release to:
  - `docker.io/cyberxdefend/vaultship`
  - `ghcr.io/cyberxdefend/vaultship`

## Week 1

- Homebrew tap formula (template included under `packaging/homebrew`)

## Week 2

- Snap package metadata (template included under `packaging/snap`)
- AUR package metadata (template included under `packaging/aur`)

## Week 3

- Chocolatey package metadata (template included under `packaging/chocolatey`)
- winget manifests (template included under `packaging/winget`)

## Month 2

- Nix expression (template included under `packaging/nix`)

## Important Notes

- CLI binary crate name is currently `vaultship-cli`.
- `cargo install` command is:
  - `cargo install vaultship-cli`
- If you want `cargo install vaultship`, rename crate package/bin strategy in a dedicated release-breaking change.
