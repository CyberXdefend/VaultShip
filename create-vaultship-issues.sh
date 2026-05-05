#!/bin/bash
# VaultShip GitHub Issues - Run this script to create all issues
# Usage: chmod +x create-issues.sh && ./create-issues.sh

REPO="CyberXdefend/VaultShip"

echo "Creating VaultShip GitHub Issues..."
echo "===================================="

# -------------------------------------------------------------------
# CRITICAL / SECURITY
# -------------------------------------------------------------------

gh issue create --repo $REPO \
  --title "🔴 Remove committed private keys and add .gitignore rules" \
  --label "security,priority:critical" \
  --body '## Problem

Private key files (`vaultship.private.key`, `vaultship.layer.key`) and hardware fingerprint data (`vaultship.bind.json`) are committed to the public repo. Even if these are test keys, it creates a bad security posture for a security tool.

Generated output files (`vaultship.toml`, `seccomp-profile.json`, `docker-compose.hardened.yml`) are also tracked but should be gitignored.

## Action Required

```bash
# Remove from tracking
git rm --cached vaultship.private.key vaultship.layer.key vaultship.bind.json
git rm --cached vaultship.toml seccomp-profile.json docker-compose.hardened.yml

# Add to .gitignore
echo "*.private.key" >> .gitignore
echo "*.layer.key" >> .gitignore
echo "vaultship.bind.json" >> .gitignore
echo "vaultship.toml" >> .gitignore
echo "seccomp-profile.json" >> .gitignore
echo "docker-compose.hardened.yml" >> .gitignore

# Rotate keys (old ones are compromised)
cargo run -p vaultship-cli -- keygen --name vaultship
```

## Acceptance Criteria

- [ ] No private keys in git history (consider `git filter-branch` or BFG Repo Cleaner)
- [ ] `.gitignore` updated with all sensitive patterns
- [ ] New keys generated
- [ ] `vaultship.toml.example` remains tracked as reference'

echo "Created: Remove committed private keys"

# -------------------------------------------------------------------
# BINDING - CORE ARCHITECTURE
# -------------------------------------------------------------------

gh issue create --repo $REPO \
  --title "Cluster-level binding for Kubernetes environments" \
  --label "enhancement,binding,kubernetes" \
  --body '## Problem

Current hardware binding uses machine-level fingerprints (MAC, disk serial, BIOS UUID). This works for static on-premise servers but breaks in Kubernetes environments where nodes are ephemeral and autoscaling creates/destroys machines constantly.

## Proposed Solution

Add a new binding level: **cluster binding**. Instead of binding to a specific machine, bind to a Kubernetes cluster identity.

### Cluster Fingerprint Components

```rust
pub struct ClusterFingerprint {
    /// SHA-256 hash of the cluster CA certificate
    pub ca_cert_hash: String,
    /// Allowed namespaces
    pub namespaces: Vec<String>,
    /// Cluster name (from kubeconfig or cloud metadata)
    pub cluster_name: Option<String>,
    /// Cloud provider cluster ID (EKS cluster ARN, GKE resource name, AKS resource ID)
    pub cloud_cluster_id: Option<String>,
}
```

### Binding Flow

```
Autoscaler creates new node
       ↓
Node joins K8s cluster
       ↓
VaultShip init-container reads:
  - Cluster CA cert from /var/run/secrets/kubernetes.io/serviceaccount/ca.crt
  - Namespace from /var/run/secrets/kubernetes.io/serviceaccount/namespace
  - Service account token for identity
       ↓
Validates against cluster bind file
       ↓
Container runs (or rejects)
```

### CLI Interface

```bash
# Vendor creates cluster-level binding
vaultship bind \
  --mode cluster \
  --cluster-ca-hash "sha256:abc123..." \
  --namespaces "cyberxdefend,forensics" \
  --output cluster-bind.json

# Client side (inside K8s pod)
vaultship run api --bind-file cluster-bind.json --public-key vendor.public.key
```

### Configuration

```toml
# vaultship.toml
[binding]
mode = "cluster"    # "machine" | "cluster" | "cloud" | "tee"

[binding.cluster]
verify_ca = true
allowed_namespaces = ["cyberxdefend", "forensics"]
allowed_service_accounts = ["vaultship-runner"]
```

## Binding Hierarchy

```
Machine Binding   → Static servers, on-premise, air-gapped
Cluster Binding   → Kubernetes, managed K8s (EKS/GKE/AKS)
Cloud Binding     → Multi-cluster, org-level (AWS Account, GCP Project)
TEE Binding       → Zero-trust, confidential computing
```

## Acceptance Criteria

- [ ] `vaultship-bind` crate supports `BindingMode::Cluster`
- [ ] Reads K8s service account CA cert and namespace
- [ ] Cluster bind file creation via CLI
- [ ] Validation works inside a K8s pod
- [ ] Falls back gracefully when not running in K8s
- [ ] Integration test with kind or minikube
- [ ] Documentation for K8s deployment'

echo "Created: Cluster-level binding"

# -------------------------------------------------------------------

gh issue create --repo $REPO \
  --title "Cloud identity binding (AWS Account, GCP Project, Azure Tenant)" \
  --label "enhancement,binding,cloud" \
  --body '## Problem

For multi-cluster deployments across regions, neither machine-level nor cluster-level binding is practical. Organizations need to authorize all infrastructure within their cloud account/project/tenant.

## Proposed Solution

Add **cloud identity binding** — bind container execution to a cloud provider identity rather than specific hardware or clusters.

### Supported Cloud Identities

| Provider | Identity Source | How to Retrieve |
|----------|---------------|----------------|
| AWS | Account ID + Region | Instance metadata `http://169.254.169.254/latest/meta-data/identity-credentials/ec2/info` or STS `GetCallerIdentity` |
| GCP | Project ID + Zone | Metadata server `http://metadata.google.internal/computeMetadata/v1/project/project-id` |
| Azure | Tenant ID + Subscription | IMDS `http://169.254.169.254/metadata/instance?api-version=2021-02-01` |
| Hetzner | Server metadata | Hetzner metadata API |

### Data Structure

```rust
pub struct CloudFingerprint {
    pub provider: CloudProvider,
    pub account_id: String,        // AWS Account ID, GCP Project ID, Azure Tenant ID
    pub allowed_regions: Vec<String>,
    pub metadata_hash: String,     // Hash of cloud identity proof
}

pub enum CloudProvider {
    Aws,
    Gcp,
    Azure,
    Hetzner,
    OnPremise,  // Fallback to machine binding
}
```

### CLI Interface

```bash
# Vendor creates cloud-level binding
vaultship bind \
  --mode cloud \
  --provider aws \
  --account-id "123456789012" \
  --regions "eu-west-1,eu-central-1" \
  --output cloud-bind.json

# Client side (on any node in their AWS account)
vaultship run api --bind-file cloud-bind.json --public-key vendor.public.key
```

## Acceptance Criteria

- [ ] `vaultship-bind` supports `BindingMode::Cloud`
- [ ] AWS identity retrieval (STS + instance metadata)
- [ ] GCP identity retrieval (metadata server)
- [ ] Azure identity retrieval (IMDS)
- [ ] Hetzner identity retrieval
- [ ] On-premise fallback to machine binding
- [ ] Region restriction enforcement
- [ ] Integration tests with localstack (AWS) or equivalent'

echo "Created: Cloud identity binding"

# -------------------------------------------------------------------

gh issue create --repo $REPO \
  --title "Tiered binding architecture — machine → cluster → cloud → TEE" \
  --label "enhancement,binding,architecture" \
  --body '## Overview

VaultShip needs a unified binding architecture that supports four levels of trust, from static machine binding to dynamic TEE attestation.

## Binding Hierarchy

```
Level 1: Machine Binding (current, implemented)
  ├── Fingerprint: MAC + disk serial + BIOS UUID + CPU ID
  ├── Best for: on-premise, fixed servers, air-gapped
  ├── Tradeoff: most restrictive, manual per-machine setup
  └── Status: ✅ Implemented

Level 2: Cluster Binding (#issue-cluster)
  ├── Fingerprint: K8s cluster CA hash + namespace + service account
  ├── Best for: managed K8s (EKS, GKE, AKS)
  ├── Tradeoff: nodes can change, cluster identity stays
  └── Status: 🔲 Planned

Level 3: Cloud Binding (#issue-cloud)
  ├── Fingerprint: AWS Account ID / GCP Project / Azure Tenant
  ├── Best for: multi-cluster, multi-region
  ├── Tradeoff: broader authorization, relies on cloud IAM
  └── Status: 🔲 Planned

Level 4: TEE Binding (future)
  ├── Fingerprint: AMD SEV-SNP / Intel TDX attestation report
  ├── Best for: zero-trust, confidential computing
  ├── Tradeoff: requires TEE hardware
  └── Status: 🔲 Phase 3
```

## Design Decisions

### Binding Mode Selection

```toml
# vaultship.toml
[binding]
mode = "auto"   # auto | machine | cluster | cloud | tee

# "auto" detection order:
# 1. Check for TEE attestation device → TEE mode
# 2. Check for K8s service account → Cluster mode
# 3. Check for cloud metadata endpoint → Cloud mode
# 4. Fallback → Machine mode
```

### Multi-level binding (AND/OR logic)

```toml
[binding]
mode = "multi"
require_all = false  # OR logic: any one match is sufficient

[[binding.levels]]
mode = "cluster"
cluster_ca_hash = "sha256:abc..."

[[binding.levels]]
mode = "cloud"
provider = "aws"
account_id = "123456789012"
```

## Acceptance Criteria

- [ ] `BindingMode` enum with all four variants
- [ ] Auto-detection logic for environment
- [ ] Multi-level binding support (AND/OR)
- [ ] Unified `validate_binding()` function that dispatches to correct level
- [ ] Documentation explaining when to use each level
- [ ] Migration guide: machine → cluster for existing deployments'

echo "Created: Tiered binding architecture"

# -------------------------------------------------------------------
# KEY MANAGEMENT
# -------------------------------------------------------------------

gh issue create --repo $REPO \
  --title "Key provider integration — HashiCorp Vault, AWS KMS, GCP KMS, Azure Key Vault" \
  --label "enhancement,key-management,cloud" \
  --body '## Problem

Current key management uses local files (`vaultship.layer.key`, `vaultship.private.key`). This works for development and single-server deployments but does not scale for production environments where:

- Keys must be rotated regularly
- Access to keys must be audited
- Keys must never exist on disk in plaintext
- Multiple environments (staging, production) need separate keys

## Proposed Solution

Add a `KeyProvider` trait and implementations for major secret managers.

### Key Provider Trait

```rust
#[async_trait]
pub trait KeyProvider: Send + Sync {
    /// Retrieve the encryption key for a given image/customer
    async fn get_layer_key(&self, key_id: &str) -> Result<Vec<u8>>;
    
    /// Retrieve the signing private key
    async fn get_signing_key(&self, key_id: &str) -> Result<SigningKey>;
    
    /// Rotate a key (generate new version, keep old for decryption)
    async fn rotate_key(&self, key_id: &str) -> Result<String>;
    
    /// Wrap (encrypt) a key for transport
    async fn wrap_key(&self, key_id: &str, plaintext: &[u8]) -> Result<Vec<u8>>;
    
    /// Unwrap (decrypt) a key
    async fn unwrap_key(&self, key_id: &str, ciphertext: &[u8]) -> Result<Vec<u8>>;
}
```

### Implementations Needed

| Provider | Crate/SDK | Auth Method |
|----------|-----------|-------------|
| Local file (current) | `std::fs` | File path |
| HashiCorp Vault | `vaultrs` | Token, K8s SA, AppRole |
| AWS KMS | `aws-sdk-kms` | IAM role, instance profile |
| GCP KMS | `google-cloud-kms` | Service account, workload identity |
| Azure Key Vault | `azure_security_keyvault` | Managed identity, service principal |
| PKCS#11 / HSM | `cryptoki` | Hardware security module |

### Configuration

```toml
# vaultship.toml

[keys]
provider = "hashicorp-vault"

[keys.local]
layer_key = "vaultship.layer.key"
private_key = "vaultship.private.key"

[keys.hashicorp-vault]
address = "https://vault.internal:8200"
mount = "secret"
path = "vaultship/production"
auth = "kubernetes"    # or "token", "approle"

[keys.aws-kms]
key_id = "arn:aws:kms:eu-west-1:123456789:key/abc-def-ghi"
region = "eu-west-1"

[keys.gcp-kms]
key_ring = "projects/myproject/locations/europe-west1/keyRings/vaultship"
key_name = "layer-key"

[keys.azure-keyvault]
vault_url = "https://myvault.vault.azure.net"
key_name = "vaultship-layer-key"
```

### Key Hierarchy

```
Root Key (offline, HSM or air-gapped)
  └── Signs Distribution Keys
  
Distribution Key (per-customer, stored in KMS)
  └── Encrypts image layers
  └── Rotated quarterly
  
Runtime Key (ephemeral, 1-24h TTL)
  └── Derived from Distribution Key
  └── Released by KBS after attestation
  └── Never persisted to disk
```

## Acceptance Criteria

- [ ] `KeyProvider` trait defined in `vaultship-encrypt`
- [ ] Local file provider (refactor current implementation)
- [ ] HashiCorp Vault provider
- [ ] AWS KMS provider
- [ ] GCP KMS provider
- [ ] Azure Key Vault provider
- [ ] Key rotation support (re-encrypt without rebuilding image)
- [ ] Configuration via `vaultship.toml`
- [ ] Integration tests with vault dev server
- [ ] Documentation for each provider setup'

echo "Created: Key provider integration"

# -------------------------------------------------------------------

gh issue create --repo $REPO \
  --title "Key Broker Service (KBS) for runtime key distribution" \
  --label "enhancement,key-management,architecture" \
  --body '## Problem

In dynamic environments (K8s, autoscaling), pre-distributing key files to every node is impractical. We need a service that releases decryption keys on-demand after validating the requesting node'"'"'s identity.

## Proposed Solution

Build a lightweight **Key Broker Service** — an HTTP API that validates attestation evidence and releases short-lived runtime keys.

### Architecture

```
ISV deploys KBS (their infrastructure or VaultShip Cloud)
       ↓
Stores: encryption keys + binding policies + customer configs
       ↓
Client'"'"'s K8s cluster runs VaultShip operator
       ↓
Pod starts → init-container contacts KBS
       ↓
KBS validates:
  ├── Cluster identity (CA cert hash)
  ├── Cloud identity (AWS account / GCP project)
  ├── Namespace + service account
  ├── TEE attestation (optional)
  └── License status (not expired, not revoked)
       ↓
Returns: Tier 3 runtime key (encrypted, 24h TTL)
       ↓
Init-container decrypts image layers in memory
       ↓
Key expires after TTL — no persistence
```

### KBS API

```
POST /v1/attest
  Request: { cluster_fingerprint, cloud_fingerprint, tee_evidence? }
  Response: { runtime_key (encrypted), ttl_seconds, policy_id }

POST /v1/renew
  Request: { session_token }
  Response: { runtime_key (refreshed), ttl_seconds }

GET /v1/policy/{image_ref}
  Response: { allowed_bindings, regions, namespaces }

POST /v1/revoke
  Request: { customer_id | key_id }
  Response: { revoked: true }
```

### Implementation

```rust
// crates/vaultship-kbs/src/main.rs
// Lightweight Axum HTTP server

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/v1/attest", post(handle_attestation))
        .route("/v1/renew", post(handle_renewal))
        .route("/v1/policy/:image_ref", get(handle_policy))
        .route("/v1/revoke", post(handle_revocation))
        .route("/health", get(health_check));

    axum::serve(listener, app).await.unwrap();
}
```

### Deployment Options

1. **Self-hosted**: ISV runs KBS on their own infrastructure
2. **Sidecar**: KBS runs as a sidecar in the client'"'"'s cluster (air-gap friendly)
3. **VaultShip Cloud**: Managed KBS by CyberXDefend (future commercial offering)

## Acceptance Criteria

- [ ] New crate: `vaultship-kbs`
- [ ] Axum HTTP API with attestation endpoint
- [ ] Validates cluster and cloud fingerprints
- [ ] Issues short-lived runtime keys (configurable TTL)
- [ ] Key revocation support
- [ ] SQLite or PostgreSQL for state (customer configs, issued keys)
- [ ] Helm chart for K8s deployment
- [ ] Docker image for standalone deployment
- [ ] Mutual TLS between client and KBS
- [ ] Rate limiting and abuse protection'

echo "Created: Key Broker Service"

# -------------------------------------------------------------------

gh issue create --repo $REPO \
  --title "Short-lived runtime keys with TTL and no disk persistence" \
  --label "enhancement,key-management,security" \
  --body '## Problem

Current decryption keys are static files with no expiry. If a key is compromised, it works forever. Keys also exist on disk where they can be extracted.

## Proposed Solution

Runtime keys should be:
- **Short-lived**: 1–24 hour TTL, configurable per customer
- **Memory-only**: never written to disk, held in process memory
- **Single-use**: bound to a specific container session
- **Renewable**: can be refreshed before expiry without restart

### Implementation

```rust
pub struct RuntimeKey {
    key_material: zeroize::Zeroizing<Vec<u8>>,  // Auto-zeroed on drop
    issued_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    session_id: Uuid,
    bound_to: BindingEvidence,
}

impl RuntimeKey {
    pub fn is_valid(&self) -> bool {
        Utc::now() < self.expires_at
    }
}

// Key is automatically zeroed from memory when dropped
impl Drop for RuntimeKey {
    fn drop(&mut self) {
        // zeroize::Zeroizing handles this automatically
    }
}
```

### Key Lifecycle

```
KBS issues key → held in memory → used for decryption
       ↓                              ↓
  TTL expires                   Container stops
       ↓                              ↓
  Key zeroed from memory        Key zeroed from memory
       ↓
  Must re-attest to get new key
```

## Dependencies

- `zeroize` crate for secure memory cleanup
- `secrecy` crate for preventing accidental logging of key material

## Acceptance Criteria

- [ ] `RuntimeKey` struct with TTL enforcement
- [ ] `zeroize` integration for secure memory cleanup
- [ ] Periodic TTL check (background task warns before expiry)
- [ ] Re-attestation flow when key expires
- [ ] Key material never logged (use `secrecy::Secret`)
- [ ] Key material never serialized to disk
- [ ] Integration test: key becomes invalid after TTL'

echo "Created: Short-lived runtime keys"

# -------------------------------------------------------------------
# POLICY ENGINE
# -------------------------------------------------------------------

gh issue create --repo $REPO \
  --title "Policy engine — declarative execution constraints embedded in images" \
  --label "enhancement,policy,architecture" \
  --body '## Problem

Currently VaultShip has binary allow/deny based on hardware fingerprint match. Real-world deployments need richer policy: restrict by region, cloud provider, namespace, time window, and container runtime flags.

## Proposed Solution

Embed a declarative policy file in the protected image that VaultShip enforces at runtime.

### Policy Format

```toml
# vaultship-policy.toml (embedded in encrypted image)

[policy]
name = "cyberxdefend-production"
version = "1.0"

[policy.allow]
binding_modes = ["machine", "cluster", "cloud"]
cloud_providers = ["aws", "gcp", "azure", "hetzner"]
regions = ["eu-west-1", "eu-central-1", "europe-west1", "eu-north-1"]
namespaces = ["cyberxdefend", "forensics", "default"]
min_tls_version = "1.3"

[policy.time]
not_before = "2026-01-01T00:00:00Z"
not_after = "2027-12-31T23:59:59Z"

[policy.deny]
cloud_providers = ["alicloud"]
debug_mode = true
privileged = true
host_network = true
host_pid = true

[policy.runtime]
read_only_rootfs = true
no_new_privileges = true
drop_capabilities = ["ALL"]
add_capabilities = ["NET_BIND_SERVICE"]
seccomp_profile = "embedded"    # Use VaultShip-generated profile

[policy.audit]
log_start = true
log_stop = true
log_binding_failures = true
report_endpoint = "https://telemetry.cyberxdefend.com/audit"  # Optional
```

### Enforcement Flow

```
vaultship run
       ↓
Read embedded policy from encrypted image
       ↓
Check each constraint:
  ✓ Cloud provider allowed?
  ✓ Region allowed?
  ✓ Namespace allowed?
  ✓ Time window valid?
  ✓ Not running privileged?
  ✓ Not using host network?
       ↓
All pass → start container
Any fail → exit with specific error code and reason
```

### Policy Signing

The policy file itself must be signed to prevent tampering:

```bash
# Policy is signed with the same key used for image signing
vaultship policy sign --private-key vendor.private.key vaultship-policy.toml
# Produces: vaultship-policy.toml.sig
```

## Acceptance Criteria

- [ ] New crate: `vaultship-policy`
- [ ] TOML policy format parser
- [ ] Allow/deny list enforcement for all constraint types
- [ ] Time window enforcement
- [ ] Runtime flag enforcement (privileged, host_network, etc.)
- [ ] Policy signing and verification
- [ ] Policy embedded in encrypted image during `vaultship build`
- [ ] Clear error messages with specific policy violation details
- [ ] Exit codes per violation type (see docs/exit-codes.md)
- [ ] Audit logging of policy decisions'

echo "Created: Policy engine"

# -------------------------------------------------------------------
# KUBERNETES
# -------------------------------------------------------------------

gh issue create --repo $REPO \
  --title "Kubernetes operator for VaultShip-protected pods" \
  --label "enhancement,kubernetes" \
  --body '## Problem

Currently VaultShip is a CLI tool. For Kubernetes deployments, operators need VaultShip to work as a native K8s component — automatically decrypting and validating containers as part of the pod lifecycle.

## Proposed Solution

Build a **VaultShip Kubernetes Operator** that:

1. Watches for pods with VaultShip annotations
2. Injects an init-container that handles attestation + decryption
3. Enforces VaultShip policies at admission time

### Custom Resource Definition

```yaml
apiVersion: vaultship.cyberxdefend.com/v1alpha1
kind: ProtectedImage
metadata:
  name: cyberxdefend-api
  namespace: forensics
spec:
  image: ghcr.io/cyberxdefend/api:v1.0
  bindingMode: cluster
  kbsEndpoint: https://kbs.cyberxdefend.com
  publicKeySecret: cyberxdefend-public-key
  policy:
    allowedNamespaces: ["forensics", "cyberxdefend"]
    requireReadOnlyRootfs: true
    maxKeyTTL: 24h
```

### Pod Annotation

```yaml
apiVersion: v1
kind: Pod
metadata:
  annotations:
    vaultship.cyberxdefend.com/protected: "true"
    vaultship.cyberxdefend.com/bind-mode: "cluster"
    vaultship.cyberxdefend.com/kbs: "https://kbs.cyberxdefend.com"
spec:
  containers:
    - name: api
      image: ghcr.io/cyberxdefend/api:v1.0
```

### Admission Webhook

A validating webhook that:
- Rejects pods trying to run VaultShip-protected images without proper annotations
- Rejects pods with `privileged: true` or `hostNetwork: true` when policy denies it
- Injects VaultShip init-container automatically

### Components

```
vaultship-operator/
├── src/
│   ├── main.rs
│   ├── controller.rs      # Reconciliation loop
│   ├── webhook.rs          # Admission webhook
│   ├── init_container.rs   # Init-container spec generation
│   └── crd.rs              # Custom Resource Definition
├── charts/
│   └── vaultship-operator/
│       ├── Chart.yaml
│       ├── values.yaml
│       └── templates/
│           ├── deployment.yaml
│           ├── rbac.yaml
│           ├── webhook.yaml
│           └── crd.yaml
└── Cargo.toml
```

### Operator Framework

Use `kube-rs` crate for the Kubernetes client and controller runtime.

## Acceptance Criteria

- [ ] New crate: `vaultship-operator`
- [ ] CRD for ProtectedImage resource
- [ ] Controller watches ProtectedImage resources
- [ ] Init-container injection for annotated pods
- [ ] Validating admission webhook
- [ ] Helm chart for operator installation
- [ ] RBAC with minimal permissions
- [ ] Works with EKS, GKE, AKS, and vanilla K8s
- [ ] Integration test with kind cluster
- [ ] Documentation for operator installation and usage'

echo "Created: Kubernetes operator"

# -------------------------------------------------------------------

gh issue create --repo $REPO \
  --title "Helm chart for VaultShip components" \
  --label "enhancement,kubernetes,packaging" \
  --body '## Overview

Provide Helm charts for deploying VaultShip components in Kubernetes:

1. **vaultship-operator** — controller + admission webhook
2. **vaultship-kbs** — Key Broker Service
3. **vaultship-runner** — DaemonSet for node-level VaultShip runtime

## Chart Repository

Host at: `https://charts.cyberxdefend.com` or use GitHub Pages OCI registry.

```bash
helm repo add vaultship https://charts.cyberxdefend.com
helm install vaultship-operator vaultship/vaultship-operator
helm install vaultship-kbs vaultship/vaultship-kbs
```

## Chart Structure

```
charts/
├── vaultship-operator/
│   ├── Chart.yaml
│   ├── values.yaml
│   ├── templates/
│   │   ├── deployment.yaml
│   │   ├── service.yaml
│   │   ├── rbac.yaml
│   │   ├── serviceaccount.yaml
│   │   ├── webhook.yaml
│   │   ├── crd.yaml
│   │   └── _helpers.tpl
│   └── README.md
├── vaultship-kbs/
│   ├── Chart.yaml
│   ├── values.yaml
│   ├── templates/
│   │   ├── deployment.yaml
│   │   ├── service.yaml
│   │   ├── ingress.yaml
│   │   ├── secret.yaml
│   │   ├── pvc.yaml
│   │   └── _helpers.tpl
│   └── README.md
```

## Acceptance Criteria

- [ ] Helm chart for vaultship-operator
- [ ] Helm chart for vaultship-kbs
- [ ] Configurable via values.yaml
- [ ] Tested with Helm lint and template rendering
- [ ] Chart published to OCI registry or chart repo
- [ ] README with installation instructions'

echo "Created: Helm charts"

# -------------------------------------------------------------------
# PLATFORM FEATURES
# -------------------------------------------------------------------

gh issue create --repo $REPO \
  --title "Windows container support — WDAC policies and Windows fingerprinting" \
  --label "enhancement,windows,platform" \
  --body '## Problem

VaultShip currently generates Linux-only security profiles (seccomp, Linux capabilities). Many EU enterprises run Windows Server infrastructure and need Windows container hardening.

## Scope

### Windows Hardening (vaultship-harden)

| Linux | Windows Equivalent |
|-------|-------------------|
| seccomp profiles | Windows Defender Application Control (WDAC) policies |
| Linux capabilities | Windows privileges and access tokens |
| AppArmor | Windows integrity levels |
| read-only rootfs | Read-only container layer |
| No shell (distroless) | Nano Server base (no PowerShell, no cmd) |
| Anti-ptrace | Anti-debugging via NtSetInformationThread |

### Windows Fingerprinting (vaultship-bind)

```rust
#[cfg(target_os = "windows")]
pub fn collect_fingerprint() -> Result<HardwareFingerprint> {
    // WMI queries:
    // - Win32_BIOS → SerialNumber
    // - Win32_DiskDrive → SerialNumber
    // - Win32_NetworkAdapter → MACAddress
    // - Win32_Processor → ProcessorId
    // - Win32_ComputerSystem → Domain, Name
    // - TPM via tbs.dll
}
```

### Container Base Images

| Linux | Windows |
|-------|---------|
| `gcr.io/distroless/static` | `mcr.microsoft.com/windows/nanoserver:ltsc2022` |
| `scratch` | `mcr.microsoft.com/windows/servercore:ltsc2022` (larger) |

## Acceptance Criteria

- [ ] WDAC policy generation in `vaultship-harden`
- [ ] Hyper-V isolation configuration
- [ ] Windows hardware fingerprinting via WMI
- [ ] Windows TPM binding via `tbs.dll`
- [ ] Nano Server base image detection and enforcement
- [ ] Cross-compilation: build on Linux, target Windows containers
- [ ] CI: Windows runner in GitHub Actions
- [ ] Documentation for Windows container hardening'

echo "Created: Windows container support"

# -------------------------------------------------------------------

gh issue create --repo $REPO \
  --title "Air-gap bundle mode — package everything for offline transfer" \
  --label "enhancement,air-gap" \
  --body '## Problem

Many high-security environments (law firms, government, defense) are air-gapped — no internet access. Currently VaultShip requires network access to pull images and contact registries.

## Proposed Solution

Add a `bundle` command that packages everything needed for an air-gapped deployment into a single encrypted archive.

### Bundle Contents

```
cyberxdefend-v1.0.bundle (encrypted archive)
├── image.tar.enc         # Encrypted OCI image
├── vaultship-cli          # VaultShip binary for target platform
├── bind.json              # Hardware-bound decryption key
├── public.key             # Signature verification key
├── policy.toml            # Execution policy
├── policy.toml.sig        # Signed policy
├── seccomp-profile.json   # Hardening profile
├── compose.yml            # Hardened docker-compose
├── install.sh             # Setup script
├── checksums.sha256       # Integrity verification
└── README.txt             # Human-readable instructions
```

### CLI

```bash
# Vendor: create bundle for a specific client
vaultship bundle create \
  --image ghcr.io/cyberxdefend/api:v1.0 \
  --bind-file client-bind.json \
  --public-key cyberxdefend.public.key \
  --target-platform linux/amd64 \
  --output cyberxdefend-v1.0.bundle

# Transfer via USB drive, secure courier, etc.

# Client: install from bundle (no internet needed)
vaultship bundle install cyberxdefend-v1.0.bundle
vaultship run api
```

### Bundle Encryption

The bundle itself is encrypted with a passphrase or pre-shared key:

```bash
# Create with passphrase
vaultship bundle create --passphrase "shared-secret" ...

# Install with passphrase
vaultship bundle install --passphrase "shared-secret" cyberxdefend-v1.0.bundle
```

## Acceptance Criteria

- [ ] `vaultship bundle create` command
- [ ] `vaultship bundle install` command
- [ ] Bundle includes VaultShip binary for target platform
- [ ] Bundle encryption with passphrase or key
- [ ] SHA-256 checksum verification
- [ ] Cross-platform bundle creation (create on Mac, target Linux)
- [ ] Bundle size optimization (compress with zstd)
- [ ] Documentation for air-gap deployment workflow
- [ ] Example: law firm deployment guide'

echo "Created: Air-gap bundle mode"

# -------------------------------------------------------------------

gh issue create --repo $REPO \
  --title "Audit logging — who ran what, where, when" \
  --label "enhancement,observability" \
  --body '## Problem

For compliance (NIS2, GDPR, SOC 2), organizations need audit trails of:
- Which protected containers were started/stopped
- On which hardware/cluster
- By which identity
- Whether binding validation passed or failed
- Policy decisions and violations

## Proposed Solution

### Structured Audit Events

```rust
#[derive(Serialize)]
pub struct AuditEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: AuditEventType,
    pub image_ref: String,
    pub binding_mode: BindingMode,
    pub fingerprint_hash: String,
    pub policy_result: PolicyResult,
    pub session_id: Uuid,
    pub details: serde_json::Value,
}

pub enum AuditEventType {
    ContainerStart,
    ContainerStop,
    BindingValidationSuccess,
    BindingValidationFailure,
    KeyIssued,
    KeyExpired,
    KeyRevoked,
    PolicyViolation,
    ImageVerified,
    ImageTampered,
}
```

### Output Targets

```toml
[audit]
enabled = true

[audit.stdout]
format = "json"     # or "text"

[audit.file]
path = "/var/log/vaultship/audit.log"
rotation = "daily"
max_files = 90

[audit.syslog]
address = "udp://syslog.internal:514"
facility = "auth"

[audit.webhook]
endpoint = "https://siem.internal/api/events"
auth_header = "Bearer ${SIEM_TOKEN}"
```

## Acceptance Criteria

- [ ] Structured audit events for all security-relevant actions
- [ ] JSON and text output formats
- [ ] File output with rotation
- [ ] Syslog output
- [ ] Webhook output for SIEM integration
- [ ] Failed attempt logging with fingerprint mismatch details
- [ ] NIS2-compatible audit format
- [ ] Integration test verifying all event types are emitted'

echo "Created: Audit logging"

# -------------------------------------------------------------------
# CI/CD & DISTRIBUTION
# -------------------------------------------------------------------

gh issue create --repo $REPO \
  --title "GitHub Actions marketplace action — vaultship-action" \
  --label "enhancement,ci-cd" \
  --body '## Overview

Publish a GitHub Actions action so users can integrate VaultShip into their CI/CD pipelines with minimal configuration.

### Usage

```yaml
- uses: cyberxdefend/vaultship-action@v1
  with:
    command: build
    compose-file: docker-compose.yml
    encrypt: true
    sign: true
    private-key: ${{ secrets.VAULTSHIP_PRIVATE_KEY }}
    layer-key: ${{ secrets.VAULTSHIP_LAYER_KEY }}

- uses: cyberxdefend/vaultship-action@v1
  with:
    command: push
    registry: ghcr.io/myorg/myapp
    tag: ${{ github.ref_name }}
```

## Acceptance Criteria

- [ ] GitHub Action published to marketplace
- [ ] Supports: build, encrypt, sign, push, inspect commands
- [ ] Caches VaultShip binary between runs
- [ ] Works on ubuntu-latest and windows-latest runners
- [ ] README with usage examples
- [ ] Version pinning support'

echo "Created: GitHub Actions marketplace action"

# -------------------------------------------------------------------

gh issue create --repo $REPO \
  --title "Publish multi-platform Docker image to ghcr.io, Docker Hub, and Quay.io" \
  --label "enhancement,packaging,ci-cd" \
  --body '## Problem

Currently 0 packages published on GitHub. VaultShip should be available as a Docker image for CI/CD usage.

### Images to Publish

```
ghcr.io/cyberxdefend/vaultship:latest
ghcr.io/cyberxdefend/vaultship:v0.1.0
docker.io/cyberxdefend/vaultship:latest
quay.io/cyberxdefend/vaultship:latest
```

### Platforms

- `linux/amd64`
- `linux/arm64`

### Release Workflow

Trigger on git tag push. Build multi-platform images. Push to all three registries. Also upload pre-built binaries to GitHub Release.

## Acceptance Criteria

- [ ] Dockerfile for VaultShip CLI image
- [ ] Multi-platform build (amd64 + arm64)
- [ ] Push to ghcr.io on tag
- [ ] Push to Docker Hub on tag
- [ ] Push to Quay.io on tag
- [ ] GitHub Release with pre-built binaries for all platforms
- [ ] `cargo publish` to crates.io in same workflow'

echo "Created: Multi-platform Docker image"

# -------------------------------------------------------------------

gh issue create --repo $REPO \
  --title "Publish crates to crates.io" \
  --label "enhancement,packaging" \
  --body '## Overview

Publish all VaultShip crates to crates.io so Rust developers can use them as libraries.

### Crates to Publish

| Crate | crates.io name | Description |
|-------|---------------|-------------|
| vaultship-cli | `vaultship-cli` | CLI binary (`cargo install vaultship-cli`) |
| vaultship-harden | `vaultship-harden` | Container hardening library |
| vaultship-encrypt | `vaultship-encrypt` | OCI image encryption/decryption |
| vaultship-sign | `vaultship-sign` | Image signing and verification |
| vaultship-bind | `vaultship-bind` | Hardware/cluster/cloud binding |
| vaultship-sdk | `vaultship-sdk` | Embed VaultShip checks in your Rust app |

### Publishing Order (dependency graph)

```
vaultship-harden    (no internal deps)     → publish first
vaultship-encrypt   (no internal deps)     → publish first
vaultship-sign      (no internal deps)     → publish first
vaultship-bind      (no internal deps)     → publish first
vaultship-sdk       (depends on above)     → publish second
vaultship-cli       (depends on all)       → publish last
```

## Acceptance Criteria

- [ ] All crate `Cargo.toml` files have required metadata (description, license, repository, readme)
- [ ] `cargo publish --dry-run` passes for all crates
- [ ] CI workflow publishes on tag push
- [ ] crates.io pages show correct descriptions and links'

echo "Created: Publish crates to crates.io"

# -------------------------------------------------------------------
# DOCUMENTATION & COMMUNITY
# -------------------------------------------------------------------

gh issue create --repo $REPO \
  --title "Add GitHub repo description, website, and topics" \
  --label "documentation,meta" \
  --body '## Problem

The GitHub repo currently shows: "No description, website, or topics provided."

## Fix

```bash
gh repo edit CyberXdefend/VaultShip \
  --description "Encrypt, harden, and protect Docker containers from extraction and reverse engineering. Built in Rust." \
  --homepage "https://cyberxdefend.com" \
  --add-topic rust,docker,containers,security,encryption,oci,container-security,devsecops,image-encryption,software-protection,supply-chain-security
```

## Also Add

- [ ] README badges (CI status, license, crates.io version)
- [ ] Improved README intro with problem statement
- [ ] Demo GIF or terminal recording
- [ ] Comparison table vs alternatives'

echo "Created: GitHub repo metadata"

# -------------------------------------------------------------------

gh issue create --repo $REPO \
  --title "Automated protection verification test suite" \
  --label "enhancement,testing" \
  --body '## Overview

Create an automated test suite that proves each VaultShip protection layer works as claimed.

### Tests

| Test | Verifies | Method |
|------|----------|--------|
| Shell access blocked | Hardening | `docker exec` fails on distroless |
| ptrace blocked | Seccomp | `strace` on container PID returns EPERM |
| Layer extraction blocked | Encryption | `docker save` produces encrypted data |
| Hardware mismatch rejected | Binding | Run with wrong fingerprint fails |
| Tampered image rejected | Signing | Modified layer fails verification |
| Expired key rejected | TTL | Key past TTL returns error |
| Policy violation rejected | Policy | Privileged container blocked |
| Full pipeline | End-to-end | Build → encrypt → sign → bind → push → pull → run |

### Script Location

`tests/verify-protection.sh` — run with `./tests/verify-protection.sh`

Requires Docker to be running.

## Acceptance Criteria

- [ ] Automated test script covering all protection layers
- [ ] Runs in CI (GitHub Actions with Docker)
- [ ] Clear pass/fail output for each test
- [ ] Documentation explaining what each test proves'

echo "Created: Protection verification test suite"

# -------------------------------------------------------------------
# LABELS
# -------------------------------------------------------------------

echo ""
echo "===================================="
echo "Creating labels..."

gh label create "binding" --repo $REPO --color "0E8A16" --description "Hardware/cluster/cloud binding" --force
gh label create "key-management" --repo $REPO --color "1D76DB" --description "Key management and KMS integration" --force
gh label create "kubernetes" --repo $REPO --color "5319E7" --description "Kubernetes operator and Helm charts" --force
gh label create "policy" --repo $REPO --color "FBCA04" --description "Policy engine and execution constraints" --force
gh label create "cloud" --repo $REPO --color "006B75" --description "Cloud provider integration" --force
gh label create "windows" --repo $REPO --color "0052CC" --description "Windows container support" --force
gh label create "air-gap" --repo $REPO --color "B60205" --description "Air-gap and offline deployment" --force
gh label create "ci-cd" --repo $REPO --color "D93F0B" --description "CI/CD integration" --force
gh label create "packaging" --repo $REPO --color "C2E0C6" --description "Distribution and packaging" --force
gh label create "observability" --repo $REPO --color "E4E669" --description "Logging, auditing, and monitoring" --force
gh label create "architecture" --repo $REPO --color "BFD4F2" --description "Architecture and design decisions" --force
gh label create "priority:critical" --repo $REPO --color "B60205" --description "Must fix immediately" --force
gh label create "good first issue" --repo $REPO --color "7057FF" --description "Good for newcomers" --force

echo ""
echo "===================================="
echo "Done! All issues and labels created."
echo ""
echo "Next steps:"
echo "  1. Fix the private keys issue FIRST"
echo "  2. Add repo metadata (description, topics)"  
echo "  3. Update README with badges"
echo "  4. Start working on cluster binding"
