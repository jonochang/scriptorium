use std::time::{Duration, Instant};

use axum::Router;
use bookstore_app::{
    AdminBootstrap, AdminService, CatalogService, PosService, StorefrontService,
};
use bookstore_web::{AppState, app};
use futures_util::future::join_all;
use reqwest::Client;

async fn spawn_app() -> anyhow::Result<(String, AdminService)> {
    let admin = AdminService::with_bootstrap(AdminBootstrap::local_defaults());
    let state = AppState {
        catalog: CatalogService::with_seed(),
        pos: PosService::with_seed(),
        storefront: StorefrontService::new(),
        admin: admin.clone(),
        db_pool: None,
    };
    let router: Router = app(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    tokio::spawn(async move {
        axum::serve(listener, router).await.expect("run load test server");
    });
    Ok((format!("http://{addr}"), admin))
}

async fn run_rush_checkout(base: &str, idx: usize) -> anyhow::Result<()> {
    let client = Client::new();

    let login: serde_json::Value = client
        .post(format!("{base}/api/pos/login"))
        .json(&serde_json::json!({ "pin": "1234" }))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let token = login["session_token"].as_str().unwrap_or_default();
    anyhow::ensure!(!token.is_empty(), "missing POS session token");

    client
        .post(format!("{base}/api/pos/cart/items"))
        .json(&serde_json::json!({
            "session_token": token,
            "item_id": "prayer-card-50c",
            "quantity": 2
        }))
        .send()
        .await?
        .error_for_status()?;

    let payment: serde_json::Value = client
        .post(format!("{base}/api/pos/payments/external-card"))
        .json(&serde_json::json!({
            "session_token": token,
            "external_ref": format!("rush-{idx}")
        }))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    anyhow::ensure!(payment["status"] == "sale_complete", "unexpected payment status");
    anyhow::ensure!(payment["total_cents"] == 100, "unexpected total");
    Ok(())
}

#[tokio::test]
async fn concurrent_pos_rush_smoke_stays_consistent() -> anyhow::Result<()> {
    let (base, admin) = spawn_app().await?;
    let start = Instant::now();

    let results = join_all((0..12).map(|idx| run_rush_checkout(&base, idx))).await;
    for result in results {
        result?;
    }

    let elapsed = start.elapsed();
    let orders = admin.list_orders("church-a").await;
    let report = admin.report_summary("church-a").await;

    assert_eq!(orders.len(), 12);
    assert_eq!(report.sales_cents, 1200);
    assert_eq!(report.donations_cents, 0);
    assert!(
        elapsed < Duration::from_secs(10),
        "concurrent POS rush smoke exceeded 10s budget: {:?}",
        elapsed
    );

    Ok(())
}
