# Architecture

Workspace-based Rust design with CLI orchestration and focused crates.

## Crate Responsibilities

- `vaultship-cli`: user/operator entrypoint and command orchestration.
- `vaultship-harden`: compose hardening and seccomp profile generation.
- `vaultship-encrypt`: encryption/decryption primitives for layer payloads.
- `vaultship-bind`: machine-bound execution policy and fingerprint checks.
- `vaultship-sign`: signing and signature verification interfaces.

## Out Of Scope (Deliberate)

- application feature flags
- seat counting / subscriptions / billing
- in-app user entitlement logic

## Command Pipeline (Current Phase)

1. `vaultship build` hardens compose config and encrypts artifacts.
2. `vaultship push` copies signed artifacts to registry storage.
3. `vaultship pull` fetches + decrypts + verifies artifact integrity.
4. `vaultship run` enforces machine/policy binding before runtime start.
