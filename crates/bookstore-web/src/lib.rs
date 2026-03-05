use axum::extract::Request;
use axum::extract::State;
use axum::middleware::{self, Next};
use axum::routing::{get, post};
use axum::{
    Json, Router,
    response::{Html, Response},
};
use bookstore_app::{
    CatalogService, PosPaymentOutcome, PosService, RequestContext, StorefrontService,
    WebhookFinalizeStatus,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Clone, Default)]
pub struct AppState {
    pub catalog: CatalogService,
    pub pos: PosService,
    pub storefront: StorefrontService,
    pub db_pool: Option<SqlitePool>,
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/books", get(list_books))
        .route("/context", get(request_context))
        .route("/catalog", get(storefront_catalog))
        .route("/catalog/search", get(storefront_search))
        .route("/checkout", get(storefront_checkout))
        .route("/pos", get(pos_shell))
        .route("/api/pos/login", post(pos_login))
        .route("/api/pos/scan", post(pos_scan))
        .route("/api/pos/cart/items", post(pos_quick_item))
        .route("/api/pos/payments/cash", post(pos_pay_cash))
        .route("/api/pos/payments/external-card", post(pos_pay_external_card))
        .route("/api/pos/payments/iou", post(pos_pay_iou))
        .route("/api/storefront/checkout/session", post(storefront_checkout_session))
        .route("/api/payments/webhook", post(payments_webhook))
        .layer(middleware::from_fn(request_context_middleware))
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}

#[derive(Debug, Serialize)]
struct ContextResponse {
    tenant_id: String,
    locale: String,
}

async fn request_context(
    axum::extract::Extension(context): axum::extract::Extension<RequestContext>,
) -> Json<ContextResponse> {
    Json(ContextResponse { tenant_id: context.tenant_id, locale: context.locale })
}

async fn list_books(State(state): State<AppState>) -> Json<Vec<bookstore_domain::Book>> {
    Json(state.catalog.list_books().await)
}

async fn storefront_catalog(State(state): State<AppState>) -> Html<String> {
    let books = state.catalog.list_books().await;
    let items = books
        .into_iter()
        .map(|book| format!("<li>{} - {}</li>", book.title, book.author))
        .collect::<Vec<_>>()
        .join("");
    Html(format!(
        "<!doctype html><html><body><h1>Catalog</h1><form hx-get=\"/catalog/search\" hx-target=\"#results\"><input name=\"q\" /></form><ul id=\"results\">{items}</ul></body></html>"
    ))
}

async fn storefront_search(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Html<String> {
    let query = params.get("q").map_or("", String::as_str).to_ascii_lowercase();
    let books = state.catalog.list_books().await;
    let filtered = books
        .into_iter()
        .filter(|book| {
            book.title.to_ascii_lowercase().contains(&query)
                || book.author.to_ascii_lowercase().contains(&query)
        })
        .map(|book| format!("<li>{} - {}</li>", book.title, book.author))
        .collect::<Vec<_>>()
        .join("");
    Html(format!("<ul id=\"results\">{filtered}</ul>"))
}

async fn storefront_checkout() -> Html<&'static str> {
    Html("<!doctype html><html><body><h1>Checkout</h1></body></html>")
}

async fn pos_shell() -> Html<&'static str> {
    Html(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Scriptorium POS</title>
  <style>
    :root {
      --wine: #6B2737;
      --wine-dark: #4A1A26;
      --gold: #B8903A;
      --parchment: #FAF7F2;
      --ink: #2C1810;
      --radius: 12px;
    }
    body {
      margin: 0;
      font-family: "DM Sans", sans-serif;
      background: linear-gradient(180deg, var(--wine-dark), var(--wine));
      color: #fff;
      min-height: 100vh;
      padding: 16px;
    }
    .pos-wrap {
      max-width: 420px;
      margin: 0 auto;
      display: grid;
      gap: 12px;
    }
    .card {
      background: var(--parchment);
      color: var(--ink);
      border-radius: var(--radius);
      padding: 14px;
      box-shadow: 0 4px 18px rgba(0,0,0,.12);
    }
    .pos-button--lg {
      width: 100%;
      min-height: 56px;
      border: 0;
      border-radius: var(--radius);
      font-size: 18px;
      font-weight: 700;
      background: var(--wine);
      color: #fff;
      margin: 6px 0;
    }
    .row { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
    input {
      width: 100%;
      min-height: 44px;
      border-radius: 10px;
      border: 1px solid #ddd;
      padding: 8px 10px;
      box-sizing: border-box;
    }
    #status { font-weight: 700; color: var(--gold); margin-top: 8px; min-height: 24px; }
  </style>
</head>
<body>
  <div id="app"></div>
  <script type="module">
    import { h, render } from "https://esm.sh/preact@10.25.4";
    import htm from "https://esm.sh/htm@3.1.1";
    import { useState } from "https://esm.sh/preact@10.25.4/hooks";

    const html = htm.bind(h);

    function App() {
      const [token, setToken] = useState("");
      const [barcode, setBarcode] = useState("9780060652937");
      const [status, setStatus] = useState("");

      const post = async (url, payload) => {
        const res = await fetch(url, {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify(payload),
        });
        const text = await res.text();
        setStatus(text);
      };

      return html`
        <main class="pos-wrap">
          <section class="card">
            <h2>Scriptorium POS</h2>
            <button class="pos-button--lg" onClick=${async () => {
              const res = await fetch("/api/pos/login", {
                method: "POST",
                headers: { "content-type": "application/json" },
                body: JSON.stringify({ pin: "1234" }),
              });
              const json = await res.json();
              setToken(json.session_token || "");
              setStatus(JSON.stringify(json));
            }}>Start Shift</button>
          </section>
          <section class="card">
            <input value=${barcode} onInput=${(e) => setBarcode(e.target.value)} />
            <button class="pos-button--lg" onClick=${() => post("/api/pos/scan", { session_token: token, barcode })}>Scan Item</button>
            <div class="row">
              <button class="pos-button--lg" onClick=${() => post("/api/pos/cart/items", { session_token: token, item_id: "prayer-card-50c", quantity: 1 })}>Quick Item</button>
              <button class="pos-button--lg" onClick=${() => post("/api/pos/payments/cash", { session_token: token, tendered_cents: 2000, donate_change: true })}>Pay Cash</button>
            </div>
            <div class="row">
              <button class="pos-button--lg" onClick=${() => post("/api/pos/payments/external-card", { session_token: token, external_ref: "square-ui" })}>Pay Card</button>
              <button class="pos-button--lg" onClick=${() => post("/api/pos/payments/iou", { session_token: token, customer_name: "Walk In" })}>Put on IOU</button>
            </div>
            <p id="status">${status}</p>
          </section>
        </main>
      `;
    }

    render(html`<${App} />`, document.getElementById("app"));
  </script>
</body>
</html>"#,
    )
}

#[derive(Debug, Deserialize)]
struct PosLoginRequest {
    pin: String,
}

#[derive(Debug, Serialize)]
struct PosLoginResponse {
    session_token: String,
}

async fn pos_login(
    State(state): State<AppState>,
    Json(request): Json<PosLoginRequest>,
) -> Result<Json<PosLoginResponse>, axum::http::StatusCode> {
    let session_token = state
        .pos
        .login_with_pin(&request.pin)
        .await
        .map_err(|_| axum::http::StatusCode::UNAUTHORIZED)?;
    Ok(Json(PosLoginResponse { session_token }))
}

#[derive(Debug, Deserialize)]
struct PosScanRequest {
    session_token: String,
    barcode: String,
}

#[derive(Debug, Deserialize)]
struct PosQuickItemRequest {
    session_token: String,
    item_id: String,
    quantity: i64,
}

#[derive(Debug, Deserialize)]
struct PosCashPaymentRequest {
    session_token: String,
    tendered_cents: i64,
    donate_change: bool,
}

#[derive(Debug, Deserialize)]
struct PosExternalCardRequest {
    session_token: String,
    external_ref: String,
}

#[derive(Debug, Deserialize)]
struct PosIouRequest {
    session_token: String,
    customer_name: String,
}

#[derive(Debug, Serialize)]
struct PosResponse {
    status: &'static str,
    total_cents: i64,
    change_due_cents: i64,
    donation_cents: i64,
}

#[derive(Debug, Deserialize)]
struct StorefrontCheckoutSessionRequest {
    total_cents: i64,
    email: String,
}

#[derive(Debug, Serialize)]
struct StorefrontCheckoutSessionResponse {
    session_id: String,
}

async fn storefront_checkout_session(
    State(state): State<AppState>,
    Json(request): Json<StorefrontCheckoutSessionRequest>,
) -> Result<Json<StorefrontCheckoutSessionResponse>, axum::http::StatusCode> {
    let session = state
        .storefront
        .create_checkout_session(request.total_cents, request.email)
        .await
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    Ok(Json(StorefrontCheckoutSessionResponse { session_id: session.session_id }))
}

#[derive(Debug, Deserialize)]
struct PaymentsWebhookRequest {
    external_ref: String,
    session_id: String,
}

#[derive(Debug, Serialize)]
struct PaymentsWebhookResponse {
    status: &'static str,
    receipt_sent: bool,
}

async fn payments_webhook(
    State(state): State<AppState>,
    Json(request): Json<PaymentsWebhookRequest>,
) -> Result<Json<PaymentsWebhookResponse>, axum::http::StatusCode> {
    let result = state
        .storefront
        .finalize_webhook(&request.external_ref, &request.session_id)
        .await
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    Ok(Json(PaymentsWebhookResponse {
        status: match result.status {
            WebhookFinalizeStatus::Processed => "processed",
            WebhookFinalizeStatus::Duplicate => "duplicate",
        },
        receipt_sent: result.receipt_sent,
    }))
}

async fn pos_scan(
    State(state): State<AppState>,
    Json(request): Json<PosScanRequest>,
) -> Result<Json<PosResponse>, axum::http::StatusCode> {
    let total = state
        .pos
        .scan_item(&request.session_token, &request.barcode)
        .await
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    Ok(Json(PosResponse {
        status: "cart_updated",
        total_cents: total,
        change_due_cents: 0,
        donation_cents: 0,
    }))
}

async fn pos_quick_item(
    State(state): State<AppState>,
    Json(request): Json<PosQuickItemRequest>,
) -> Result<Json<PosResponse>, axum::http::StatusCode> {
    let total = state
        .pos
        .add_quick_item(&request.session_token, &request.item_id, request.quantity)
        .await
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    Ok(Json(PosResponse {
        status: "cart_updated",
        total_cents: total,
        change_due_cents: 0,
        donation_cents: 0,
    }))
}

async fn pos_pay_cash(
    State(state): State<AppState>,
    Json(request): Json<PosCashPaymentRequest>,
) -> Result<Json<PosResponse>, axum::http::StatusCode> {
    let receipt = state
        .pos
        .checkout_cash(&request.session_token, request.tendered_cents, request.donate_change)
        .await
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    Ok(Json(PosResponse {
        status: if receipt.outcome == PosPaymentOutcome::Paid { "sale_complete" } else { "iou" },
        total_cents: receipt.total_cents,
        change_due_cents: receipt.change_due_cents,
        donation_cents: receipt.donation_cents,
    }))
}

async fn pos_pay_external_card(
    State(state): State<AppState>,
    Json(request): Json<PosExternalCardRequest>,
) -> Result<Json<PosResponse>, axum::http::StatusCode> {
    let receipt = state
        .pos
        .checkout_external_card(&request.session_token, &request.external_ref)
        .await
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    Ok(Json(PosResponse {
        status: if receipt.outcome == PosPaymentOutcome::Paid { "sale_complete" } else { "iou" },
        total_cents: receipt.total_cents,
        change_due_cents: receipt.change_due_cents,
        donation_cents: receipt.donation_cents,
    }))
}

async fn pos_pay_iou(
    State(state): State<AppState>,
    Json(request): Json<PosIouRequest>,
) -> Result<Json<PosResponse>, axum::http::StatusCode> {
    let receipt = state
        .pos
        .checkout_iou(&request.session_token, &request.customer_name)
        .await
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    Ok(Json(PosResponse {
        status: if receipt.outcome == PosPaymentOutcome::UnpaidIou {
            "iou"
        } else {
            "sale_complete"
        },
        total_cents: receipt.total_cents,
        change_due_cents: receipt.change_due_cents,
        donation_cents: receipt.donation_cents,
    }))
}

async fn request_context_middleware(mut request: Request, next: Next) -> Response {
    let tenant_id = request
        .headers()
        .get("x-tenant-id")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.is_empty())
        .unwrap_or("default")
        .to_string();

    let locale = request
        .headers()
        .get("accept-language")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("en-AU")
        .to_string();

    request.extensions_mut().insert(RequestContext { tenant_id, locale });
    next.run(request).await
}
