use bookstore_app::{AdminBootstrap, AdminService, CatalogService, PosService, StorefrontService};
use bookstore_web::isbn_lookup::IsbnLookupClient;
use bookstore_web::{AppState, app};

async fn spawn_mock_isbn_service() -> anyhow::Result<String> {
    let router = axum::Router::new().route(
        "/api/books",
        axum::routing::get(|| async {
            axum::Json(serde_json::json!({
              "ISBN:9780060652937": {
                "title": "The Screwtape Letters",
                "subtitle": "Letters from a senior devil",
                "authors": [{ "name": "C. S. Lewis" }],
                "publishers": [{ "name": "HarperSanFrancisco" }],
                "cover": {
                  "large": "https://covers.example.test/screwtape-large.jpg"
                }
              }
            }))
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    tokio::spawn(async move {
        axum::serve(listener, router).await.expect("run mock isbn server");
    });
    Ok(format!("http://{addr}/api/books"))
}

async fn spawn_app_with_lookup(base_url: String) -> anyhow::Result<String> {
    let admin = AdminService::with_bootstrap(AdminBootstrap::local_defaults());
    let state = AppState {
        catalog: CatalogService::with_seed(),
        pos: PosService::with_seed(),
        storefront: StorefrontService::new(),
        admin,
        db_pool: None,
        cover_storage: None,
        isbn_lookup: Some(IsbnLookupClient::with_base_url(base_url)),
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    tokio::spawn(async move {
        axum::serve(listener, app(state)).await.expect("run isbn lookup app");
    });
    Ok(format!("http://{addr}"))
}

#[tokio::test]
async fn service_isbn_lookup_prefers_online_metadata_when_configured() -> anyhow::Result<()> {
    let isbn_base = spawn_mock_isbn_service().await?;
    let base = spawn_app_with_lookup(isbn_base).await?;
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

    let lookup: serde_json::Value = client
        .post(format!("{base}/api/admin/products/isbn-lookup"))
        .header("Origin", &base)
        .json(&serde_json::json!({ "token": token, "isbn": "9780060652937" }))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    assert_eq!(lookup["title"], "The Screwtape Letters");
    assert_eq!(lookup["author"], "C. S. Lewis");
    assert_eq!(lookup["cover_image_url"], "https://covers.example.test/screwtape-large.jpg");
    Ok(())
}
