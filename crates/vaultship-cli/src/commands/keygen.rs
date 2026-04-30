use base64::Engine as _;

pub async fn run(name: &str) -> anyhow::Result<()> {
    let layer_key = vaultship_encrypt::keys::generate_layer_key();
    std::fs::write(
        format!("{name}.layer.key"),
        base64::engine::general_purpose::STANDARD.encode(layer_key),
    )?;

    let (signing, verifying) = vaultship_license::create::keygen();
    std::fs::write(
        format!("{name}.private.key"),
        base64::engine::general_purpose::STANDARD.encode(signing.to_bytes()),
    )?;
    std::fs::write(
        format!("{name}.public.key"),
        base64::engine::general_purpose::STANDARD.encode(verifying.to_bytes()),
    )?;

    println!("Generated {name}.layer.key, {name}.private.key, {name}.public.key");
    Ok(())
}
