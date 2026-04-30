# VaultShip Protection Profiles

VaultShip protection is best applied in profiles. Profiles set predictable defaults so teams can choose the right trade-off between compatibility and security.

## Baseline Profile

Use when onboarding quickly and minimizing breakage risk.

### Config

```toml
[profile]
name = "baseline"

[harden]
read_only = true
drop_capabilities = true
no_new_privileges = true
seccomp_profile = "auto"
anti_debug = true

[bind]
hardware_bind = false
heartbeat_interval_minutes = 240

[signing]
enforce_cosign = false

[attestation]
enabled = false
```

### Security Outcome

- theft cost: medium
- abuse detection: medium
- cloned artifact resistance: medium

### Roadmap Tasks

- complete signed binding + heartbeat alerts
- strengthen compose mutation edge cases
- expand baseline integration tests

---

## Strict Profile

Use for production environments without TEE, where stronger binding enforcement is needed.

### Config

```toml
[profile]
name = "strict"

[harden]
read_only = true
drop_capabilities = true
no_new_privileges = true
seccomp_profile = "strict"
anti_debug = true

[bind]
hardware_bind = true
heartbeat_interval_minutes = 60

[signing]
enforce_cosign = true
cosign_key_path = "/etc/vaultship/cosign.key"

[attestation]
enabled = false
```

### Security Outcome

- theft cost: high
- abuse detection: high
- cloned artifact resistance: high

### Roadmap Tasks

- encrypted-at-rest bound key files + key rotation
- mandatory cosign verify in pull/run policy
- tamper-event telemetry + revocation hooks

---

## High-Assurance Profile

Use when confidential computing infrastructure is available.

### Config

```toml
[profile]
name = "high-assurance"

[harden]
read_only = true
drop_capabilities = true
no_new_privileges = true
seccomp_profile = "strict"
anti_debug = true

[bind]
hardware_bind = true
heartbeat_interval_minutes = 30

[signing]
enforce_cosign = true
cosign_key_path = "/etc/vaultship/cosign.key"

[attestation]
enabled = true
backend = "coco"
require_attested_key_release = true
```

### Security Outcome

- theft cost: very high
- abuse detection: high
- cloned artifact resistance: very high

### Roadmap Tasks

- TEE attestation service integration (SEV-SNP / TDX)
- key broker policy gates tied to attestation
- cross-VM attested launch test suite

---

## Choosing a Profile

- Start with `baseline` for pilot rollout.
- Move to `strict` before broad customer deployment.
- Use `high-assurance` for high-value workloads with supported confidential-compute hardware.
