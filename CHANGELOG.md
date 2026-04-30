# Changelog

All notable changes to this project are documented here.

## [0.1.0] - 2026-04-30

### Added

- Workspace crates for CLI, hardening, encryption, signing, bind flow, and SDK compatibility.
- OCI registry push/pull flow for encrypted artifacts.
- Hardware binding command set (`bind`, `fingerprint`) and bind-aware `run`.
- Protection profile initialization (`baseline`, `strict`, `high-assurance`).
- JSON output for `inspect` and `verify`.
- Exit code contract and observability docs.

### Changed

- Scope clarified to container protection, with bind-first model.
- Runtime command supports engine selection (`docker`, `podman`, `nerdctl`).

### Security

- Added negative tests for tamper, replay mismatch, and wrong-key decryption failure.
