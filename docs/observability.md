# Observability

VaultShip supports CI-friendly observability primitives:

## Structured Result Output

Set JSON output mode:

```bash
VAULTSHIP_OUTPUT=json vaultship verify api --json
```

On failures, VaultShip emits JSON with `error` and `exit_code`.

## Command-Level JSON

- `vaultship inspect <image> --json`
- `vaultship verify <image-or-ref> --json`

## Runtime Metrics Recommendations

Send command logs to your existing log pipeline and derive counters:

- run success/failure
- binding mismatch failures
- signature verification failures
- registry transfer failures

For system-level collection, ship CLI stdout/stderr to your SIEM and aggregate by `exit_code`.
