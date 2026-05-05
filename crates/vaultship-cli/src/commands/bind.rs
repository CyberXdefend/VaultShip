use base64::Engine as _;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BoundKeyFile {
    pub key_base64: String,
    pub fingerprint: vaultship_license::fingerprint::HardwareFingerprint,
    pub signature_base64: String,
}

#[derive(Debug, Serialize)]
struct BoundKeyPayload<'a> {
    key_base64: &'a str,
    fingerprint: &'a vaultship_license::fingerprint::HardwareFingerprint,
}

pub async fn create(
    key_file: &str,
    private_key: &str,
    fingerprint_path: Option<&str>,
    output: &str,
) -> anyhow::Result<()> {
    let key_base64 = std::fs::read_to_string(key_file)?.trim().to_string();
    let fingerprint = if let Some(path) = fingerprint_path {
        serde_json::from_str(&std::fs::read_to_string(path)?)?
    } else {
        vaultship_license::fingerprint::HardwareFingerprint::collect()?
    };

    let signer = load_signing_key(private_key)?;
    let payload = BoundKeyPayload {
        key_base64: &key_base64,
        fingerprint: &fingerprint,
    };
    let payload_bytes = serde_json::to_vec(&payload)?;
    let signature: Signature = signer.sign(&payload_bytes);

    let bound = BoundKeyFile {
        key_base64,
        fingerprint,
        signature_base64: base64::engine::general_purpose::STANDARD.encode(signature.to_bytes()),
    };
    std::fs::write(output, serde_json::to_string_pretty(&bound)?)?;
    println!("Created bound key file at {output}");
    Ok(())
}

pub fn validate_bound_file(path: &str, public_key_path: &str) -> anyhow::Result<[u8; 32]> {
    let bound: BoundKeyFile = serde_json::from_str(&std::fs::read_to_string(path)?)?;

    let verifier = load_public_key(public_key_path)?;
    let payload = BoundKeyPayload {
        key_base64: &bound.key_base64,
        fingerprint: &bound.fingerprint,
    };
    let payload_bytes = serde_json::to_vec(&payload)?;

    let sig_bytes =
        base64::engine::general_purpose::STANDARD.decode(bound.signature_base64.trim())?;
    let sig_arr: [u8; 64] = sig_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid signature length in bound file"))?;
    let signature = Signature::from_bytes(&sig_arr);
    verifier
        .verify(&payload_bytes, &signature)
        .map_err(|_| anyhow::anyhow!("Invalid bound key signature"))?;

    if !bound.fingerprint.verify_current()? {
        anyhow::bail!("Hardware fingerprint mismatch for bound key")
    }

    let key_bytes = base64::engine::general_purpose::STANDARD.decode(bound.key_base64.trim())?;
    let key: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Bound decryption key must be 32 bytes"))?;
    Ok(key)
}

fn load_signing_key(path: &str) -> anyhow::Result<SigningKey> {
    let key_bytes =
        base64::engine::general_purpose::STANDARD.decode(std::fs::read_to_string(path)?.trim())?;
    let key: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Private key must be 32 bytes"))?;
    Ok(SigningKey::from_bytes(&key))
}

fn load_public_key(path: &str) -> anyhow::Result<VerifyingKey> {
    let key_bytes =
        base64::engine::general_purpose::STANDARD.decode(std::fs::read_to_string(path)?.trim())?;
    let key: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Public key must be 32 bytes"))?;
    Ok(VerifyingKey::from_bytes(&key)?)
}
