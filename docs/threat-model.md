# Threat Model

## Security Goals

- Increase resistance to casual extraction and tampering of shipped container artifacts.
- Enforce signed binding validation before protected runtime startup.
- Reduce container runtime privilege by default.

## Primary Threats Addressed

- Unauthorized re-use of protected images across unbound machines.
- Debugging and memory-inspection attempts inside runtime containers.
- Excessive default Linux capabilities in container deployments.
- Weak deployment defaults that allow mutable root filesystem abuse.

## Controls in Phase 1

- Seccomp profile generation with explicit anti-extraction deny rules.
- `no-new-privileges` and read-only root filesystem hardening.
- Capability drop defaults (`ALL`) with minimal add-back.
- Ed25519-signed bound key files with optional hardware binding.
- Encrypted artifact layer handling with integrity verification.

## Out of Scope (v0.1)

- Trusted Execution Environments and confidential-computing attestation.
- Kernel-compromise or host-root adversary guarantees.
- TPM-backed sealed key release.
- Kubernetes operator-level policy enforcement.
