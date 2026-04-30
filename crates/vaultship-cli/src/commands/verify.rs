pub async fn run(image_or_ref: &str, json: bool) -> anyhow::Result<()> {
    if image_or_ref.contains("@") {
        vaultship_sign::verify::verify_signature(image_or_ref)?;
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "target": image_or_ref,
                    "verified": true
                }))?
            );
        } else {
            println!("Signature verification passed for {image_or_ref}");
        }
        return Ok(());
    }

    let manifest = std::path::Path::new(".vaultship/artifacts").join(format!("{image_or_ref}.manifest.json"));
    if !manifest.exists() {
        anyhow::bail!("No manifest for `{image_or_ref}`. Provide signed ref or build artifact first.");
    }

    let value: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(manifest)?)?;
    let signed_ref = value["signed_ref"].as_str().unwrap_or_default();
    vaultship_sign::verify::verify_signature(signed_ref)?;
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "target": image_or_ref,
                "signed_ref": signed_ref,
                "verified": true
            }))?
        );
    } else {
        println!("Manifest signature verification passed: {signed_ref}");
    }
    Ok(())
}
