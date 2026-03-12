use anyhow::Context;
use aws_config::{BehaviorVersion, Region, meta::region::RegionProviderChain};
use aws_credential_types::Credentials;
use aws_sdk_s3::Client;
use aws_sdk_s3::primitives::ByteStream;

#[derive(Clone, Debug)]
pub struct ObjectStorageConfig {
    pub endpoint: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket: String,
}

#[derive(Clone, Debug)]
pub struct StoredObject {
    pub bytes: Vec<u8>,
    pub content_type: String,
}

#[derive(Clone, Debug)]
pub struct ObjectStorage {
    client: Client,
    bucket: String,
}

impl ObjectStorageConfig {
    pub fn from_env() -> Option<Self> {
        Some(Self {
            endpoint: std::env::var("SCRIPTORIUM_OBJECT_STORAGE_ENDPOINT").ok()?,
            region: std::env::var("SCRIPTORIUM_OBJECT_STORAGE_REGION")
                .unwrap_or_else(|_| "us-east-1".to_string()),
            access_key: std::env::var("SCRIPTORIUM_OBJECT_STORAGE_ACCESS_KEY").ok()?,
            secret_key: std::env::var("SCRIPTORIUM_OBJECT_STORAGE_SECRET_KEY").ok()?,
            bucket: std::env::var("SCRIPTORIUM_OBJECT_STORAGE_BUCKET").ok()?,
        })
    }
}

impl ObjectStorage {
    pub async fn new(config: ObjectStorageConfig) -> anyhow::Result<Self> {
        let region = Region::new(config.region.clone());
        let shared_config = aws_config::defaults(BehaviorVersion::latest())
            .region(RegionProviderChain::first_try(region))
            .credentials_provider(Credentials::new(
                config.access_key,
                config.secret_key,
                None,
                None,
                "scriptorium-object-storage",
            ))
            .load()
            .await;
        let s3_config = aws_sdk_s3::config::Builder::from(&shared_config)
            .endpoint_url(config.endpoint)
            .force_path_style(true)
            .build();
        Ok(Self { client: Client::from_conf(s3_config), bucket: config.bucket })
    }

    pub async fn ensure_bucket(&self) -> anyhow::Result<()> {
        if self.client.head_bucket().bucket(&self.bucket).send().await.is_ok() {
            return Ok(());
        }
        self.client
            .create_bucket()
            .bucket(&self.bucket)
            .send()
            .await
            .context("create storage bucket")?;
        Ok(())
    }

    pub async fn put(&self, key: &str, bytes: Vec<u8>, content_type: &str) -> anyhow::Result<()> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type(content_type)
            .body(ByteStream::from(bytes))
            .send()
            .await
            .with_context(|| format!("upload object {key}"))?;
        Ok(())
    }

    pub async fn get(&self, key: &str) -> anyhow::Result<StoredObject> {
        let object = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .with_context(|| format!("download object {key}"))?;
        let content_type =
            object.content_type.unwrap_or_else(|| "application/octet-stream".to_string());
        let bytes = object.body.collect().await?.into_bytes().to_vec();
        Ok(StoredObject { bytes, content_type })
    }

    pub fn asset_url(&self, key: &str) -> String {
        format!("/media/{key}")
    }

    pub fn key_for_upload(&self, tenant_id: &str, filename: &str) -> String {
        let safe_name =
            filename
                .chars()
                .map(|ch| {
                    if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
                        ch
                    } else {
                        '-'
                    }
                })
                .collect::<String>();
        format!("covers/{tenant_id}/{}-{safe_name}", chrono::Utc::now().format("%Y%m%d%H%M%S%3f"))
    }
}
