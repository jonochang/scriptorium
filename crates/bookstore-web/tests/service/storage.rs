use std::env;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use bookstore_app::{AdminBootstrap, AdminService, CatalogService, PosService, StorefrontService};
use bookstore_web::object_storage::{ObjectStorage, ObjectStorageConfig};
use bookstore_web::{AppState, app};
use reqwest::multipart;
use tempfile::TempDir;
use tokio::time::sleep;

struct TestMinio {
    _dir: TempDir,
    child: Child,
    endpoint: String,
    bucket: String,
    access_key: String,
    secret_key: String,
}

impl Drop for TestMinio {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl TestMinio {
    async fn start() -> anyhow::Result<Self> {
        let dir = tempfile::tempdir()?;
        let port = free_port()?;
        let console_port = free_port()?;
        let access_key = "scriptorium".to_string();
        let secret_key = "scriptorium123".to_string();
        let endpoint = format!("http://127.0.0.1:{port}");
        let bucket = "scriptorium-covers".to_string();
        let mut child = Command::new(minio_executable()?)
            .arg("server")
            .arg(dir.path())
            .arg("--address")
            .arg(format!("127.0.0.1:{port}"))
            .arg("--console-address")
            .arg(format!("127.0.0.1:{console_port}"))
            .env("MINIO_ROOT_USER", &access_key)
            .env("MINIO_ROOT_PASSWORD", &secret_key)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        let client = reqwest::Client::new();
        let health_url = format!("{endpoint}/minio/health/live");
        let mut ready = false;
        for _ in 0..50 {
            if let Ok(response) = client.get(&health_url).send().await {
                if response.status().is_success() {
                    ready = true;
                    break;
                }
            }
            sleep(Duration::from_millis(100)).await;
        }
        if !ready {
            let _ = child.kill();
            anyhow::bail!("minio did not become ready");
        }

        Ok(Self { _dir: dir, child, endpoint, bucket, access_key, secret_key })
    }

    async fn storage(&self) -> anyhow::Result<ObjectStorage> {
        let storage = ObjectStorage::new(ObjectStorageConfig {
            endpoint: self.endpoint.clone(),
            region: "us-east-1".to_string(),
            access_key: self.access_key.clone(),
            secret_key: self.secret_key.clone(),
            bucket: self.bucket.clone(),
        })
        .await?;
        storage.ensure_bucket().await?;
        Ok(storage)
    }
}

fn free_port() -> anyhow::Result<u16> {
    Ok(TcpListener::bind("127.0.0.1:0")?.local_addr()?.port())
}

fn minio_executable() -> anyhow::Result<PathBuf> {
    if let Some(path) = env::var_os("MINIO_EXECUTABLE") {
        return Ok(PathBuf::from(path));
    }
    which_in_path("minio").ok_or_else(|| anyhow::anyhow!("install minio or set MINIO_EXECUTABLE"))
}

fn which_in_path(name: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    env::split_paths(&path).map(|dir| dir.join(name)).find(|candidate| candidate.exists())
}

async fn spawn_app_with_storage(storage: ObjectStorage) -> anyhow::Result<String> {
    let admin = AdminService::with_bootstrap(AdminBootstrap::local_defaults());
    let state = AppState {
        catalog: CatalogService::with_seed(),
        pos: PosService::with_seed(),
        storefront: StorefrontService::new(),
        admin,
        db_pool: None,
        cover_storage: Some(storage),
        isbn_lookup: None,
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    tokio::spawn(async move {
        axum::serve(listener, app(state)).await.expect("run storage test server");
    });
    Ok(format!("http://{addr}"))
}

#[tokio::test]
async fn service_cover_upload_round_trips_through_minio() -> anyhow::Result<()> {
    if minio_executable().is_err() {
        eprintln!("skipping MinIO storage test because minio is not installed");
        return Ok(());
    }
    let minio = TestMinio::start().await?;
    let storage = minio.storage().await?;
    let base = spawn_app_with_storage(storage).await?;
    let client = reqwest::Client::new();

    let login: serde_json::Value = client
        .post(format!("{base}/api/admin/auth/login"))
        .header("Origin", &base)
        .json(&serde_json::json!({ "username": "admin", "password": "admin123" }))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let token = login["token"].as_str().unwrap_or_default();
    anyhow::ensure!(!token.is_empty(), "missing admin token");

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cover-test.svg");
    let svg = std::fs::read_to_string(&fixture)?;
    let form = multipart::Form::new()
        .text("token", token.to_string())
        .text("tenant_id", "church-a".to_string())
        .part(
            "file",
            multipart::Part::text(svg.clone())
                .file_name("cover-test.svg")
                .mime_str("image/svg+xml")?,
        );
    let upload: serde_json::Value = client
        .post(format!("{base}/api/admin/products/cover-upload"))
        .header("Origin", &base)
        .multipart(form)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let asset_url = upload["asset_url"].as_str().unwrap_or_default();
    anyhow::ensure!(asset_url.starts_with("/media/covers/church-a/"));

    let media = client
        .get(format!("{base}{asset_url}"))
        .send()
        .await?
        .error_for_status()?;
    anyhow::ensure!(
        media
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .starts_with("image/svg+xml")
    );
    let body = media.text().await?;
    anyhow::ensure!(body.contains("Scriptorium"));
    Ok(())
}
