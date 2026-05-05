use anyhow::{Context, Result, bail};
use base64::Engine as _;
use reqwest::header::{ACCEPT, CONTENT_TYPE, LOCATION};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::encrypt::EncryptedLayer;

const MANIFEST_MEDIA: &str = "application/vnd.oci.image.manifest.v1+json";
const CONFIG_MEDIA: &str = "application/vnd.vaultship.config.v1+json";
const LAYER_MEDIA: &str = "application/vnd.vaultship.layer.v1+json";

#[derive(Debug, Clone)]
pub struct RegistryReference {
    pub registry: String,
    pub repository: String,
    pub tag: String,
    pub scheme: String,
}

#[derive(Debug, Clone)]
enum RegistryAuth {
    Basic { username: String, password: String },
    Bearer(String),
}

#[derive(Debug, Serialize, Deserialize)]
struct OciDescriptor {
    #[serde(rename = "mediaType")]
    media_type: String,
    digest: String,
    size: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct OciManifest {
    #[serde(rename = "schemaVersion")]
    schema_version: u32,
    #[serde(rename = "mediaType")]
    media_type: String,
    config: OciDescriptor,
    layers: Vec<OciDescriptor>,
}

pub fn parse_reference(reference: &str) -> Result<RegistryReference> {
    // Supports:
    // - localhost:5000/team/app:tag
    // - http://localhost:5000/team/app:tag
    let (scheme, rest) = if let Some(stripped) = reference.strip_prefix("http://") {
        ("http".to_string(), stripped)
    } else if let Some(stripped) = reference.strip_prefix("https://") {
        ("https".to_string(), stripped)
    } else {
        ("https".to_string(), reference)
    };

    let mut slash = rest.splitn(2, '/');
    let registry = slash.next().unwrap_or_default().to_string();
    let repo_with_tag = slash.next().unwrap_or_default();
    if registry.is_empty() || repo_with_tag.is_empty() {
        bail!("Invalid registry reference: {reference}");
    }

    let (repository, tag) = if let Some((repo, tag)) = repo_with_tag.rsplit_once(':') {
        (repo.to_string(), tag.to_string())
    } else {
        (repo_with_tag.to_string(), "latest".to_string())
    };

    Ok(RegistryReference {
        registry,
        repository,
        tag,
        scheme,
    })
}

pub async fn push_encrypted_layer(reference: &str, layer: &EncryptedLayer) -> Result<()> {
    let parsed = parse_reference(reference)?;
    let client = reqwest::Client::new();
    let auth = resolve_auth(&parsed);

    let layer_bytes = serde_json::to_vec(layer)?;
    let layer_digest = sha256_digest(&layer_bytes);
    let config_bytes = serde_json::to_vec(&serde_json::json!({
        "created_by": "vaultship",
        "algorithm": layer.algorithm,
    }))?;
    let config_digest = sha256_digest(&config_bytes);

    upload_blob(&client, &parsed, &auth, &config_bytes, &config_digest).await?;
    upload_blob(&client, &parsed, &auth, &layer_bytes, &layer_digest).await?;

    let manifest = OciManifest {
        schema_version: 2,
        media_type: MANIFEST_MEDIA.to_string(),
        config: OciDescriptor {
            media_type: CONFIG_MEDIA.to_string(),
            digest: config_digest.clone(),
            size: config_bytes.len(),
        },
        layers: vec![OciDescriptor {
            media_type: LAYER_MEDIA.to_string(),
            digest: layer_digest.clone(),
            size: layer_bytes.len(),
        }],
    };

    let manifest_json = serde_json::to_string(&manifest)?;
    let annotated_json = crate::oci::annotate_manifest_for_encryption(&manifest_json)?;

    let manifest_url = format!(
        "{}://{}/v2/{}/manifests/{}",
        parsed.scheme, parsed.registry, parsed.repository, parsed.tag
    );
    let response = apply_auth(
        client
            .put(manifest_url)
            .header(CONTENT_TYPE, MANIFEST_MEDIA)
            .body(annotated_json.into_bytes()),
        &auth,
    )
    .send()
    .await?;

    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        bail!("Failed pushing manifest: {body}");
    }

    Ok(())
}

pub async fn pull_encrypted_layer(reference: &str) -> Result<EncryptedLayer> {
    let parsed = parse_reference(reference)?;
    let client = reqwest::Client::new();
    let auth = resolve_auth(&parsed);

    let manifest_url = format!(
        "{}://{}/v2/{}/manifests/{}",
        parsed.scheme, parsed.registry, parsed.repository, parsed.tag
    );
    let response = apply_auth(
        client.get(manifest_url).header(ACCEPT, MANIFEST_MEDIA),
        &auth,
    )
    .send()
    .await?;
    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        bail!("Failed fetching manifest: {body}");
    }

    let manifest: OciManifest = response.json().await?;
    let layer_desc = manifest
        .layers
        .first()
        .context("Manifest has no encrypted layers")?;

    let blob_url = format!(
        "{}://{}/v2/{}/blobs/{}",
        parsed.scheme, parsed.registry, parsed.repository, layer_desc.digest
    );
    let blob_response = apply_auth(client.get(blob_url), &auth).send().await?;
    if !blob_response.status().is_success() {
        let body = blob_response.text().await.unwrap_or_default();
        bail!("Failed fetching blob: {body}");
    }

    let bytes = blob_response.bytes().await?;
    Ok(serde_json::from_slice::<EncryptedLayer>(&bytes)?)
}

async fn upload_blob(
    client: &reqwest::Client,
    parsed: &RegistryReference,
    auth: &Option<RegistryAuth>,
    bytes: &[u8],
    digest: &str,
) -> Result<()> {
    let initiate = format!(
        "{}://{}/v2/{}/blobs/uploads/",
        parsed.scheme, parsed.registry, parsed.repository
    );

    let init_resp = apply_auth(client.post(initiate), auth).send().await?;
    if !init_resp.status().is_success() {
        let body = init_resp.text().await.unwrap_or_default();
        bail!("Failed initiating upload: {body}");
    }

    let location = init_resp
        .headers()
        .get(LOCATION)
        .context("Registry did not return upload location")?
        .to_str()?
        .to_string();

    let upload_url = if location.starts_with("http://") || location.starts_with("https://") {
        location
    } else {
        format!("{}://{}{}", parsed.scheme, parsed.registry, location)
    };

    let final_url = if upload_url.contains('?') {
        format!("{upload_url}&digest={digest}")
    } else {
        format!("{upload_url}?digest={digest}")
    };

    let finish = apply_auth(client.put(final_url).body(bytes.to_vec()), auth)
        .send()
        .await?;
    if !finish.status().is_success() {
        let body = finish.text().await.unwrap_or_default();
        bail!("Failed uploading blob: {body}");
    }

    Ok(())
}

fn sha256_digest(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("sha256:{:x}", hasher.finalize())
}

fn resolve_auth(parsed: &RegistryReference) -> Option<RegistryAuth> {
    // Generic auth
    if let Ok(token) = std::env::var("VAULTSHIP_REGISTRY_TOKEN")
        && !token.trim().is_empty()
    {
        return Some(RegistryAuth::Bearer(token));
    }
    if let (Ok(username), Ok(password)) = (
        std::env::var("VAULTSHIP_REGISTRY_USERNAME"),
        std::env::var("VAULTSHIP_REGISTRY_PASSWORD"),
    ) && !username.trim().is_empty()
    {
        return Some(RegistryAuth::Basic { username, password });
    }

    // Registry-specific convenience envs
    let reg = parsed.registry.to_lowercase();
    if reg.contains("ghcr.io")
        && let Ok(token) = std::env::var("GITHUB_TOKEN")
        && let Ok(username) = std::env::var("GITHUB_ACTOR")
    {
        return Some(RegistryAuth::Basic {
            username,
            password: token,
        });
    }
    if reg.contains(".amazonaws.com")
        && let Ok(password) = std::env::var("AWS_ECR_PASSWORD")
    {
        return Some(RegistryAuth::Basic {
            username: "AWS".to_string(),
            password,
        });
    }
    if reg.contains("azurecr.io")
        && let (Ok(username), Ok(password)) = (
            std::env::var("AZURE_ACR_USERNAME"),
            std::env::var("AZURE_ACR_PASSWORD"),
        )
    {
        return Some(RegistryAuth::Basic { username, password });
    }
    if (reg.contains("gitlab") || reg.contains("registry.gitlab.com"))
        && let Ok(token) = std::env::var("GITLAB_TOKEN")
    {
        return Some(RegistryAuth::Bearer(token));
    }

    None
}

fn apply_auth(
    builder: reqwest::RequestBuilder,
    auth: &Option<RegistryAuth>,
) -> reqwest::RequestBuilder {
    match auth {
        Some(RegistryAuth::Basic { username, password }) => {
            let token =
                base64::engine::general_purpose::STANDARD.encode(format!("{username}:{password}"));
            builder.header("Authorization", format!("Basic {token}"))
        }
        Some(RegistryAuth::Bearer(token)) => {
            builder.header("Authorization", format!("Bearer {token}"))
        }
        None => builder,
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_reference, resolve_auth};

    #[test]
    fn parses_registry_reference() {
        let r = parse_reference("http://localhost:5000/team/app:enc").expect("parse");
        assert_eq!(r.scheme, "http");
        assert_eq!(r.registry, "localhost:5000");
        assert_eq!(r.repository, "team/app");
        assert_eq!(r.tag, "enc");
    }

    #[test]
    fn picks_generic_token_auth() {
        // SAFETY: tests are single-threaded in this crate and we restore env after.
        unsafe {
            std::env::set_var("VAULTSHIP_REGISTRY_TOKEN", "abc123");
        }
        let r = parse_reference("https://ghcr.io/team/app:latest").expect("parse");
        assert!(resolve_auth(&r).is_some());
        unsafe {
            std::env::remove_var("VAULTSHIP_REGISTRY_TOKEN");
        }
    }
}
