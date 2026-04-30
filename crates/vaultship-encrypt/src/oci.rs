use anyhow::Result;

pub fn annotate_manifest_for_encryption(manifest_json: &str) -> Result<String> {
    let mut value: serde_json::Value = serde_json::from_str(manifest_json)?;
    if value.get("annotations").is_none() {
        value["annotations"] = serde_json::json!({});
    }
    value["annotations"]["org.vaultship.encrypted"] = serde_json::json!("true");
    Ok(serde_json::to_string_pretty(&value)?)
}
