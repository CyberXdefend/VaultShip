use chrono::{DateTime, NaiveDate, Utc};
use clap::Subcommand;
use base64::Engine as _;
use ed25519_dalek::SigningKey;
use vaultship_license::{License, fingerprint::HardwareFingerprint};

#[derive(Subcommand)]
pub enum LicenseCommands {
    Keygen { #[arg(long, default_value = "vaultship")] name: String },
    Create {
        #[arg(long)] customer: String,
        #[arg(long)] product: String,
        #[arg(long)] expires: Option<String>,
        #[arg(long, default_value = "1")] seats: u32,
        #[arg(long)] hardware_bind: bool,
        #[arg(long, value_delimiter = ',')] features: Option<Vec<String>>,
        #[arg(long)] fingerprint: Option<String>,
        #[arg(long, default_value = "vaultship.private.key")] key: String,
    },
    Validate {
        license_file: String,
        #[arg(long, default_value = "vaultship.public.key")] public_key: String,
    },
    Fingerprint,
}

pub async fn run(action: LicenseCommands) -> anyhow::Result<()> {
    match action {
        LicenseCommands::Keygen { name } => keygen(&name),
        LicenseCommands::Create { customer, product, expires, seats, hardware_bind, features, fingerprint, key } => create(&customer, &product, expires, seats, hardware_bind, features, fingerprint, &key),
        LicenseCommands::Validate { license_file, public_key } => validate(&license_file, &public_key),
        LicenseCommands::Fingerprint => fingerprint(),
    }
}

fn keygen(name: &str) -> anyhow::Result<()> {
    let (signing, verifying) = vaultship_license::create::keygen();
    std::fs::write(
        format!("{name}.private.key"),
        base64::engine::general_purpose::STANDARD.encode(signing.to_bytes()),
    )?;
    std::fs::write(
        format!("{name}.public.key"),
        base64::engine::general_purpose::STANDARD.encode(verifying.to_bytes()),
    )?;
    println!("Generated {name}.private.key and {name}.public.key");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn create(customer: &str, product: &str, expires: Option<String>, seats: u32, hardware_bind: bool, features: Option<Vec<String>>, fingerprint: Option<String>, key_path: &str) -> anyhow::Result<()> {
    let key_bytes = base64::engine::general_purpose::STANDARD.decode(std::fs::read_to_string(key_path)?.trim())?;
    let key_arr: [u8; 32] = key_bytes.try_into().map_err(|_| anyhow::anyhow!("Signing key must be 32 bytes"))?;
    let signing_key = SigningKey::from_bytes(&key_arr);
    let expires_at = parse_expiry(expires)?;

    let fp = if hardware_bind {
        if let Some(path) = fingerprint {
            Some(serde_json::from_str::<HardwareFingerprint>(&std::fs::read_to_string(path)?)?)
        } else {
            Some(HardwareFingerprint::collect()?)
        }
    } else { None };

    let license = License::create(&signing_key, customer, product, fp, expires_at, seats, features.unwrap_or_default())?;
    let file_name = format!("license-{customer}.key");
    license.export(&file_name)?;
    println!("Created {file_name}");
    Ok(())
}

fn validate(license_file: &str, public_key_path: &str) -> anyhow::Result<()> {
    let license: License = serde_json::from_str(&std::fs::read_to_string(license_file)?)?;
    let key_bytes = base64::engine::general_purpose::STANDARD.decode(std::fs::read_to_string(public_key_path)?.trim())?;
    let key_arr: [u8; 32] = key_bytes.try_into().map_err(|_| anyhow::anyhow!("Public key must be 32 bytes"))?;
    let public_key = ed25519_dalek::VerifyingKey::from_bytes(&key_arr)?;
    vaultship_license::validate::validate_license(&license, &public_key)?;
    println!("License valid");
    Ok(())
}

fn fingerprint() -> anyhow::Result<()> {
    println!("{}", serde_json::to_string_pretty(&HardwareFingerprint::collect()?)?);
    Ok(())
}

fn parse_expiry(expires: Option<String>) -> anyhow::Result<Option<DateTime<Utc>>> {
    let Some(raw) = expires else { return Ok(None) };
    let date = NaiveDate::parse_from_str(&raw, "%Y-%m-%d")?;
    let dt = date.and_hms_opt(23, 59, 59).ok_or_else(|| anyhow::anyhow!("invalid expiry timestamp"))?;
    Ok(Some(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc)))
}
