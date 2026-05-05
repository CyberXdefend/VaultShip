use serde_json::Value;
use std::path::PathBuf;

pub async fn run(image: &str, json: bool) -> anyhow::Result<()> {
    let manifest = PathBuf::from(".vaultship/artifacts").join(format!("{image}.manifest.json"));
    if !manifest.exists() {
        anyhow::bail!("Manifest not found for `{image}`. Run `vaultship build` first.");
    }

    let value: Value = serde_json::from_str(&std::fs::read_to_string(&manifest)?)?;
    let signed_ref = value["signed_ref"].as_str().unwrap_or_default();
    let public_key_path = "vaultship.public.key";
    let signed_ok = vaultship_sign::verify::verify_signature(signed_ref, public_key_path).is_ok();

    if json {
        let out = serde_json::json!({
            "image": image,
            "hardened_compose": value["hardened_compose"].as_str().unwrap_or("n/a"),
            "seccomp_profile": value["seccomp_profile"].as_str().unwrap_or("n/a"),
            "encrypted_artifact": value["artifact"].as_str().unwrap_or("n/a"),
            "content_type": value["content_type"].as_str().unwrap_or("compose-config"),
            "signed_ref": signed_ref,
            "signature_ok": signed_ok,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    let content_type = value["content_type"].as_str().unwrap_or("compose-config");
    println!("Image:             {image}");
    println!("Content type:      {content_type}");
    println!(
        "Hardened compose:  {}",
        value["hardened_compose"].as_str().unwrap_or("n/a")
    );
    println!(
        "Seccomp profile:   {}",
        value["seccomp_profile"].as_str().unwrap_or("n/a")
    );
    println!(
        "Encrypted artifact:{}",
        value["artifact"].as_str().unwrap_or("n/a")
    );
    println!("Signed ref:        {signed_ref}");
    println!("Signature valid:   {signed_ok}");
    Ok(())
}
