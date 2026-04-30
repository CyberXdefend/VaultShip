# Exit Codes

VaultShip commands return stable exit codes for CI/CD automation.

- `0`: success
- `1`: generic failure
- `21`: binding/fingerprint validation failure
- `22`: signature verification failure
- `23`: registry push/pull/manifest failure
- `24`: hardening/compose processing failure
- `25`: runtime engine execution failure (`docker`/`podman`/`nerdctl`)

Use `VAULTSHIP_OUTPUT=json` to capture machine-readable failure output.
