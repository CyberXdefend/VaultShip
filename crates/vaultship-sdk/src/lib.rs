use anyhow::Result;

pub mod check;

pub fn validate_or_exit(license_path: &str, public_key_base64: &[u8]) {
    match validate(license_path, public_key_base64) {
        Ok(_) => tracing::info!("License validated successfully"),
        Err(e) => {
            eprintln!("LICENSE ERROR: {e}");
            eprintln!("Contact support@vaultship.dev for license assistance.");
            std::process::exit(1);
        }
    }
}

pub fn validate(license_path: &str, public_key_base64: &[u8]) -> Result<()> {
    check::validate(license_path, public_key_base64)
}
