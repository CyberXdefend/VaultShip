use serde_json::Value;
use std::path::PathBuf;

pub async fn run(image: &str, json: bool) -> anyhow::Result<()> {
    let manifest = PathBuf::from(".vaultship/artifacts").join(format!("{image}.manifest.json"));
    if !manifest.exists() {
        anyhow::bail!("Manifest not found for `{image}`. Run `vaultship build` first.");
    }

    let value: Value = serde_json::from_str(&std::fs::read_to_string(&manifest)?)?;
    let signed_ref = value["signed_ref"].as_str().unwrap_or_default();
    let signed_ok = vaultship_sign::verify::verify_signature(signed_ref).is_ok();

    if json {
        let out = serde_json::json!({
            "image": image,
            "hardened_compose": value["hardened_compose"].as_str().unwrap_or("n/a"),
            "seccomp_profile": value["seccomp_profile"].as_str().unwrap_or("n/a"),
            "encrypted_artifact": value["artifact"].as_str().unwrap_or("n/a"),
            "signed_ref": signed_ref,
            "signature_ok": signed_ok,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    println!("Image: {image}");
    println!("Hardened compose: {}", value["hardened_compose"].as_str().unwrap_or("n/a"));
    println!("Seccomp profile: {}", value["seccomp_profile"].as_str().unwrap_or("n/a"));
    println!("Encrypted artifact: {}", value["artifact"].as_str().unwrap_or("n/a"));
    println!("Signed ref: {signed_ref} (ok={signed_ok})");
    Ok(())
}
