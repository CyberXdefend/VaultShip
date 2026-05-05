use vaultship_encrypt::{decrypt::decrypt_layer, encrypt::encrypt_layer, keys::generate_layer_key};

#[test]
fn roundtrip_random_key() {
    let key = generate_layer_key();
    let data = b"hello vaultship integration test";
    let enc = encrypt_layer(data, &key).expect("encrypt");
    let dec = decrypt_layer(&enc, &key).expect("decrypt");
    assert_eq!(dec, data);
}

#[test]
fn tampered_ciphertext_is_rejected() {
    let key = generate_layer_key();
    let data = b"sensitive payload";
    let mut enc = encrypt_layer(data, &key).expect("encrypt");
    enc.ciphertext[0] ^= 0xff;
    assert!(decrypt_layer(&enc, &key).is_err());
}

#[test]
fn wrong_key_is_rejected() {
    let key1 = generate_layer_key();
    let key2 = generate_layer_key();
    let data = b"sensitive payload";
    let enc = encrypt_layer(data, &key1).expect("encrypt");
    assert!(decrypt_layer(&enc, &key2).is_err());
}

#[test]
fn hash_tamper_is_rejected() {
    let key = generate_layer_key();
    let data = b"sensitive payload";
    let mut enc = encrypt_layer(data, &key).expect("encrypt");
    enc.original_hash = "deadbeef".to_string();
    assert!(decrypt_layer(&enc, &key).is_err());
}

#[test]
fn large_payload_roundtrip() {
    let key = generate_layer_key();
    let data: Vec<u8> = (0..65536).map(|i| (i % 256) as u8).collect();
    let enc = encrypt_layer(&data, &key).expect("encrypt large");
    let dec = decrypt_layer(&enc, &key).expect("decrypt large");
    assert_eq!(dec, data);
}
