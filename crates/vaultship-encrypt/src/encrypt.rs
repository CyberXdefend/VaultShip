use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use anyhow::Result;
use sha2::{Digest, Sha256};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EncryptedLayer {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub original_hash: String,
    pub algorithm: String,
}

pub fn encrypt_layer(layer_data: &[u8], encryption_key: &[u8; 32]) -> Result<EncryptedLayer> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(encryption_key));
    let nonce_bytes: [u8; 12] = rand::random();
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, layer_data)
        .map_err(|e| anyhow::anyhow!("Encryption failed: {e}"))?;

    let mut hasher = Sha256::new();
    hasher.update(layer_data);

    Ok(EncryptedLayer {
        ciphertext,
        nonce: nonce_bytes.to_vec(),
        original_hash: format!("{:x}", hasher.finalize()),
        algorithm: "AES-256-GCM".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::encrypt_layer;
    use crate::decrypt::decrypt_layer;

    #[test]
    fn roundtrip_encrypt_decrypt() {
        let key = [7_u8; 32];
        let input = b"vaultship-layer-bytes";
        let encrypted = encrypt_layer(input, &key).expect("encrypt");
        let output = decrypt_layer(&encrypted, &key).expect("decrypt");
        assert_eq!(output, input);
    }

    #[test]
    fn fails_on_tampered_ciphertext() {
        let key = [9_u8; 32];
        let input = b"vaultship-layer-bytes";
        let mut encrypted = encrypt_layer(input, &key).expect("encrypt");
        encrypted.ciphertext[0] ^= 0x01;
        assert!(decrypt_layer(&encrypted, &key).is_err());
    }

    #[test]
    fn fails_on_stolen_key_mismatch() {
        let right_key = [1_u8; 32];
        let wrong_key = [2_u8; 32];
        let input = b"vaultship-layer-bytes";
        let encrypted = encrypt_layer(input, &right_key).expect("encrypt");
        assert!(decrypt_layer(&encrypted, &wrong_key).is_err());
    }

    #[test]
    fn fails_on_replay_hash_mismatch() {
        let key = [4_u8; 32];
        let input = b"vaultship-layer-bytes";
        let mut encrypted = encrypt_layer(input, &key).expect("encrypt");
        encrypted.original_hash = "deadbeef".to_string();
        assert!(decrypt_layer(&encrypted, &key).is_err());
    }
}
