use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use anyhow::{Result, bail};
use sha2::{Digest, Sha256};

use crate::encrypt::EncryptedLayer;

pub fn decrypt_layer(layer: &EncryptedLayer, encryption_key: &[u8; 32]) -> Result<Vec<u8>> {
    if layer.nonce.len() != 12 {
        bail!("Invalid nonce length")
    }

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(encryption_key));
    let nonce = Nonce::from_slice(&layer.nonce);
    let plaintext = cipher
        .decrypt(nonce, layer.ciphertext.as_ref())
        .map_err(|e| anyhow::anyhow!("Decryption failed: {e}"))?;

    let mut hasher = Sha256::new();
    hasher.update(&plaintext);
    if format!("{:x}", hasher.finalize()) != layer.original_hash {
        bail!("Layer integrity mismatch")
    }

    Ok(plaintext)
}
