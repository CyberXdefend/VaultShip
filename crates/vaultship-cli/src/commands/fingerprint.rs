pub async fn run() -> anyhow::Result<()> {
    let fp = vaultship_license::fingerprint::HardwareFingerprint::collect()?;
    println!("{}", serde_json::to_string_pretty(&fp)?);
    Ok(())
}
