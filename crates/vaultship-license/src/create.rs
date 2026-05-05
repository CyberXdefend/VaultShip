use anyhow::Result;
use base64::Engine as _;
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey};
use serde::{Deserialize, Serialize};

use crate::fingerprint::HardwareFingerprint;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct License {
    pub id: uuid::Uuid,
    pub customer: String,
    pub product: String,
    pub hardware_fingerprint: Option<HardwareFingerprint>,
    pub issued_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub seats: u32,
    pub features: Vec<String>,
    pub signature: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LicensePayload {
    pub id: uuid::Uuid,
    pub customer: String,
    pub product: String,
    pub hardware_fingerprint: Option<HardwareFingerprint>,
    pub issued_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub seats: u32,
    pub features: Vec<String>,
}

impl From<&License> for LicensePayload {
    fn from(value: &License) -> Self {
        Self {
            id: value.id,
            customer: value.customer.clone(),
            product: value.product.clone(),
            hardware_fingerprint: value.hardware_fingerprint.clone(),
            issued_at: value.issued_at,
            expires_at: value.expires_at,
            seats: value.seats,
            features: value.features.clone(),
        }
    }
}

impl License {
    pub fn create(
        signing_key: &SigningKey,
        customer: &str,
        product: &str,
        fingerprint: Option<HardwareFingerprint>,
        expires_at: Option<DateTime<Utc>>,
        seats: u32,
        features: Vec<String>,
    ) -> Result<Self> {
        let license = Self {
            id: uuid::Uuid::new_v4(),
            customer: customer.to_string(),
            product: product.to_string(),
            hardware_fingerprint: fingerprint,
            issued_at: Utc::now(),
            expires_at,
            seats,
            features,
            signature: String::new(),
        };

        let payload = serde_json::to_string(&LicensePayload::from(&license))?;
        let signature: Signature = signing_key.sign(payload.as_bytes());

        Ok(Self {
            signature: base64::engine::general_purpose::STANDARD.encode(signature.to_bytes()),
            ..license
        })
    }

    pub fn export(&self, path: &str) -> Result<()> {
        std::fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }
}

pub fn keygen() -> (SigningKey, ed25519_dalek::VerifyingKey) {
    let secret: [u8; 32] = rand::random();
    let signing_key = SigningKey::from_bytes(&secret);
    let verifying = signing_key.verifying_key();
    (signing_key, verifying)
}
