use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Html;
use bookstore_app::{PosPaymentOutcome, SalesEvent};
use bookstore_domain::{OrderChannel, OrderStatus, PaymentMethod};
use std::time::Instant;

use crate::AppState;
use crate::models::{
    ApiError, PosCartQuantityRequest, PosCashPaymentRequest, PosExternalCardRequest, PosIouRequest,
    PosLoginRequest, PosLoginResponse, PosQuickItemRequest, PosResponse, PosScanRequest,
};
use crate::web_support::{current_utc_datetime, log_checkout_event, pos_cart_response};

pub async fn pos_shell() -> Html<&'static str> {
    Html(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Scriptorium POS</title>
  <link href="https://fonts.googleapis.com/css2?family=Source+Serif+4:opsz,wght@8..60,400;8..60,600;8..60,700&family=Source+Sans+3:wght@400;500;600&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet">
  <style>
    :root {
      --wine: #6B2737;
      --wine-light: #8B3A4A;
      --wine-dark: #4A1A26;
      --gold: #B8903A;
      --gold-light: #CCAA5E;
      --gold-pale: #F5ECD7;
      --parchment: #FAF7F2;
      --parchment-dark: #EDE8E0;
      --ink: #2C1810;
      --ink-light: #5A4A3A;
      --warm-gray: #8A7A6A;
      --warm-gray-light: #B5A898;
      --success: #5A7D5E;
      --success-light: #EEF3EE;
      --warning: #A07040;
      --warning-light: #F5EDE3;
      --danger: #9B5A5A;
      --danger-light: #F5EDED;
      --blue: #5A7A9B;
      --blue-light: #ECF1F5;
      --radius: 12px;
      --radius-lg: 16px;
      --shadow: 0 4px 18px rgba(44,24,16,.10);
      --shadow-lg: 0 10px 32px rgba(44,24,16,.18);
    }
    * { box-sizing: border-box; }
    body {
      margin: 0;
      font-family: "Source Sans 3", sans-serif;
      background:
        radial-gradient(circle at top, rgba(204,170,94,.18), transparent 26%),
        linear-gradient(180deg, var(--wine-dark), var(--wine) 34%, #55202d 100%);
      color: #fff;
      min-height: 100vh;
    }
    .pos-shell {
      min-height: 100vh;
      padding: 18px 14px 28px;
      display: flex;
      justify-content: center;
    }
    .pos-wrap {
      width: 100%;
      max-width: 460px;
      display: grid;
      gap: 14px;
    }
    .card {
      background: var(--parchment);
      color: var(--ink);
      border-radius: var(--radius-lg);
      padding: 16px;
      box-shadow: var(--shadow);
    }
    .pos-header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 14px;
      padding: 16px 18px;
      border-radius: var(--radius-lg);
      background: linear-gradient(135deg, rgba(107,39,55,.96), rgba(139,58,74,.9));
      box-shadow: var(--shadow-lg);
    }
    .pos-header__brand {
      display: grid;
      gap: 4px;
    }
    .pos-header__brand-mark {
      color: rgba(245,236,215,.42);
      font-size: 1.6rem;
      line-height: 1;
    }
    .pos-header__title {
      margin: 0;
      font-family: "Source Serif 4", Georgia, serif;
      font-size: 1.7rem;
      color: var(--gold-light);
      letter-spacing: .05em;
      text-transform: uppercase;
    }
    .pos-header__subtitle {
      color: rgba(255,255,255,.68);
      font-size: .8rem;
      letter-spacing: .24em;
      text-transform: uppercase;
    }
    .pos-header__meta {
      display: flex;
      gap: 10px;
      align-items: center;
      flex-wrap: wrap;
      justify-content: end;
    }
    .pos-header__back {
      min-height: 38px;
      padding: 0 12px;
      border-radius: 999px;
      border: 1px solid rgba(255,255,255,.18);
      background: rgba(255,255,255,.08);
      color: white;
      font: 700 .86rem/1 "Source Sans 3", sans-serif;
    }
    .session-row {
      display: flex;
      gap: 10px;
      flex-wrap: wrap;
    }
    .session-pill {
      display: inline-flex;
      align-items: center;
      gap: 8px;
      padding: 8px 12px;
      border-radius: 999px;
      background: rgba(255,255,255,.12);
      font-size: .85rem;
      color: rgba(255,255,255,.9);
    }
    .section-title {
      margin: 0 0 12px;
      font-family: "Source Serif 4", Georgia, serif;
      font-size: 1.45rem;
    }
    .subtle {
      margin: 6px 0 0;
      color: var(--warm-gray);
      font-size: .9rem;
      line-height: 1.5;
    }
    .center-shell {
      min-height: calc(100vh - 46px);
      display: flex;
      flex-direction: column;
      justify-content: center;
      gap: 18px;
    }
    .pin-head {
      text-align: center;
      padding: 10px 10px 0;
      position: relative;
    }
    .pin-cross {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      width: 40px;
      height: 40px;
      margin-bottom: 14px;
      color: rgba(255,255,255,.35);
      font-size: 2rem;
      line-height: 1;
    }
    .pin-head h1 {
      margin: 0;
      font-family: "Source Serif 4", Georgia, serif;
      font-size: 2.2rem;
      color: var(--gold-light);
      letter-spacing: .05em;
    }
    .pin-head p {
      margin: 6px 0 0;
      color: rgba(255,255,255,.66);
    }
    .pin-links {
      display: flex;
      justify-content: center;
      gap: 14px;
      flex-wrap: wrap;
      margin-top: 14px;
    }
    .pin-links a {
      color: rgba(255,255,255,.78);
      text-decoration: none;
      font-size: .9rem;
    }
    .pin-card {
      background: rgba(250,247,242,.12);
      border: 1px solid rgba(255,255,255,.12);
      border-radius: 24px;
      backdrop-filter: blur(16px);
      box-shadow: var(--shadow-lg);
      padding: 18px;
    }
    .pin-dots {
      display: flex;
      justify-content: center;
      gap: 14px;
      margin: 4px 0 20px;
    }
    .pin-dot {
      width: 18px;
      height: 18px;
      border-radius: 50%;
      border: 2px solid var(--gold-light);
      background: transparent;
      transition: all .16s ease;
    }
    .pin-dot--filled { background: var(--gold-light); }
    .pin-grid {
      display: grid;
      grid-template-columns: repeat(3, 1fr);
      gap: 12px;
    }
    .pin-key {
      min-height: 72px;
      border-radius: 18px;
      border: 0;
      background: rgba(255,255,255,.1);
      color: white;
      font: 700 1.75rem/1 "Source Sans 3", sans-serif;
      box-shadow: inset 0 1px 0 rgba(255,255,255,.08);
    }
    .pin-key--ghost {
      font-size: 1rem;
      color: rgba(255,255,255,.74);
    }
    .status-panel {
      min-height: 86px;
      padding: 14px;
      border-radius: 14px;
      border: 1px solid var(--parchment-dark);
      background: #fff;
    }
    .status-panel h3 {
      margin: 0 0 6px;
      font-size: 1rem;
    }
    .status-panel p {
      margin: 0;
      color: var(--ink-light);
      line-height: 1.45;
    }
    .status-success { background: var(--success-light); border-color: rgba(90,125,94,.25); }
    .status-warning { background: var(--warning-light); border-color: rgba(160,112,64,.22); }
    .status-danger { background: var(--danger-light); border-color: rgba(155,90,90,.24); }
    .toolbar {
      display: flex;
      gap: 8px;
      background: white;
      padding: 6px;
      border-radius: 16px;
      border: 1px solid var(--parchment-dark);
    }
    .toolbar button {
      flex: 1;
      min-height: 42px;
      border-radius: 12px;
      border: 0;
      font: 700 .95rem/1 "Source Sans 3", sans-serif;
      color: var(--warm-gray);
      background: transparent;
    }
    .toolbar button.is-active {
      background: var(--gold-pale);
      color: var(--wine-dark);
      box-shadow: inset 0 -3px 0 var(--gold);
    }
    .field-label {
      display: block;
      margin: 0 0 8px;
      font-size: .9rem;
      font-weight: 600;
      color: var(--ink-light);
    }
    input {
      width: 100%;
      min-height: 46px;
      border-radius: 10px;
      border: 1px solid var(--parchment-dark);
      padding: 10px 12px;
      background: #fff;
      color: var(--ink);
      font: 500 16px/1.2 "Source Sans 3", sans-serif;
    }
    .pos-button--lg {
      width: 100%;
      min-height: 58px;
      border: 0;
      border-radius: var(--radius);
      font-size: 17px;
      font-weight: 700;
      background: var(--wine);
      color: #fff;
      margin: 0;
      box-shadow: 0 4px 12px rgba(107,39,55,.22);
    }
    .pos-button--gold {
      background: var(--gold);
      box-shadow: 0 4px 12px rgba(184,144,58,.22);
    }
    .pos-button--success {
      background: var(--success);
      box-shadow: 0 4px 12px rgba(90,125,94,.24);
    }
    .pos-button--ghost {
      background: white;
      color: var(--ink);
      border: 1px solid var(--parchment-dark);
      box-shadow: none;
    }
    .pos-button--light {
      background: white;
      color: var(--success);
      box-shadow: none;
    }
    .row {
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 10px;
    }
    .scan-frame {
      min-height: 176px;
      border-radius: 18px;
      background: linear-gradient(180deg, rgba(44,24,16,.92), rgba(44,24,16,.78));
      position: relative;
      overflow: hidden;
      margin-bottom: 12px;
    }
    .scan-frame::before {
      content: "";
      position: absolute;
      inset: 24px;
      border-radius: 18px;
      border: 2px solid rgba(204,170,94,.45);
    }
    .scan-frame::after {
      content: "";
      position: absolute;
      left: 18%;
      right: 18%;
      top: 50%;
      height: 2px;
      background: var(--gold);
      box-shadow: 0 0 18px rgba(204,170,94,.48);
      animation: scanline 2.4s ease-in-out infinite;
    }
    @keyframes scanline {
      0%,100% { transform: translateY(-52px); }
      50% { transform: translateY(52px); }
    }
    .scan-caption {
      position: absolute;
      left: 0;
      right: 0;
      bottom: 16px;
      text-align: center;
      color: rgba(255,255,255,.6);
      font-size: .86rem;
    }
    .quick-grid {
      display: grid;
      grid-template-columns: repeat(2, minmax(0, 1fr));
      gap: 10px;
    }
    .quick-tile {
      border: 1px solid var(--parchment-dark);
      border-radius: 16px;
      background: linear-gradient(180deg, #fff, var(--gold-pale));
      color: var(--ink);
      min-height: 112px;
      padding: 14px;
      text-align: left;
      font: 700 1rem/1.2 "Source Sans 3", sans-serif;
      position: relative;
    }
    .quick-emoji {
      display: block;
      font-size: 1.8rem;
      margin-bottom: 10px;
    }
    .quick-price {
      display: inline-flex;
      margin-top: 8px;
      padding: 4px 10px;
      border-radius: 999px;
      background: rgba(255,255,255,.7);
      color: var(--wine);
      font-size: .9rem;
    }
    .basket-card {
      position: sticky;
      bottom: 12px;
    }
    .cart-list {
      display: grid;
      gap: 10px;
    }
    .cart-row {
      display: grid;
      gap: 8px;
      grid-template-columns: 1fr auto;
      padding: 12px;
      border-radius: 12px;
      background: #fff;
      border: 1px solid var(--parchment-dark);
    }
    .cart-title { font-weight: 700; }
    .cart-meta { color: var(--warm-gray); font-size: .9rem; margin-top: 4px; }
    .cart-price { font-weight: 800; color: var(--wine); }
    .cart-controls {
      display: flex;
      align-items: center;
      justify-content: end;
      gap: 8px;
      margin-top: 8px;
    }
    .qty-pill {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      min-width: 32px;
      min-height: 32px;
      padding: 0 10px;
      border-radius: 999px;
      background: var(--filled);
      color: var(--ink);
      font-weight: 700;
    }
    .cart-tag {
      display: inline-flex;
      align-items: center;
      min-height: 26px;
      margin-top: 8px;
      padding: 0 10px;
      border-radius: 999px;
      font-size: .76rem;
      font-weight: 700;
    }
    .cart-tag--quick { color: var(--warning); background: var(--warning-light); }
    .cart-tag--scan { color: var(--blue); background: var(--blue-light); }
    .empty-state {
      padding: 16px;
      border-radius: 12px;
      background: linear-gradient(180deg, #fff, #f7f3ec);
      border: 1px dashed var(--parchment-dark);
      color: var(--ink-light);
      text-align: center;
    }
    .totals {
      display: grid;
      gap: 10px;
      padding: 14px;
      border-radius: 14px;
      background: linear-gradient(180deg, rgba(107,39,55,.06), rgba(184,144,58,.12));
    }
    .totals-row {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 12px;
    }
    .totals-row strong {
      font-size: 1.2rem;
      color: var(--wine);
    }
    .actions { display: grid; gap: 10px; }
    .hint { margin: 0; color: var(--warm-gray); font-size: .86rem; }
    .payment-option {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 14px;
      padding: 16px;
      border-radius: 16px;
      border: 1px solid var(--parchment-dark);
      background: white;
    }
    .payment-option__main {
      display: flex;
      align-items: center;
      gap: 14px;
      min-width: 0;
      flex: 1;
    }
    .payment-icon {
      width: 50px;
      height: 50px;
      border-radius: 50%;
      display: grid;
      place-items: center;
      font-size: 1.35rem;
    }
    .payment-title {
      display: block;
      font-weight: 800;
      color: var(--ink);
    }
    .payment-copy-stack {
      display: grid;
      gap: 4px;
      text-align: left;
    }
    .payment-copy {
      display: block;
      color: var(--warm-gray);
      font-size: .9rem;
    }
    .payment-chevron {
      color: var(--warm-gray);
      font-size: 1.4rem;
      line-height: 1;
    }
    .payment-total-card {
      padding: 18px;
      border-radius: 20px;
      background: rgba(255,255,255,.12);
      border: 1px solid rgba(255,255,255,.14);
      text-align: center;
      box-shadow: var(--shadow-lg);
    }
    .payment-total-card__label {
      color: rgba(255,255,255,.72);
      font-size: .78rem;
      font-weight: 700;
      letter-spacing: .18em;
      text-transform: uppercase;
    }
    .payment-total-card__value {
      margin-top: 10px;
      font-size: 3.2rem;
      font-weight: 800;
      line-height: 1;
      color: white;
    }
    .cash-grid {
      display: grid;
      grid-template-columns: repeat(2, minmax(0, 1fr));
      gap: 10px;
      margin-top: 14px;
    }
    .cash-grid button {
      min-height: 76px;
      border-radius: 14px;
      border: 1px solid var(--parchment-dark);
      background: white;
      color: var(--ink);
      font: 800 1.2rem/1 "Source Sans 3", sans-serif;
    }
    .cash-grid span {
      display: block;
      margin-top: 6px;
      color: var(--warm-gray);
      font-size: .78rem;
      font-weight: 600;
    }
    .round-up-button {
      width: 100%;
      min-height: 56px;
      border-radius: 16px;
      border: 1px dashed rgba(160,112,64,.42);
      background: var(--gold-pale);
      color: var(--warning);
      font: 800 .96rem/1 "Source Sans 3", sans-serif;
    }
    .round-up-button--active {
      background: rgba(184,144,58,.18);
      border-style: solid;
      color: var(--wine);
    }
    .complete-screen {
      min-height: calc(100vh - 46px);
      display: flex;
      flex-direction: column;
      justify-content: center;
      gap: 18px;
      text-align: center;
      padding: 24px;
      border-radius: 28px;
      background: linear-gradient(180deg, #5A7D5E 0%, #6f9a74 100%);
      color: white;
    }
    .complete-mark {
      width: 88px;
      height: 88px;
      margin: 0 auto;
      border-radius: 50%;
      display: grid;
      place-items: center;
      background: rgba(255,255,255,.14);
      box-shadow: var(--shadow-lg);
    }
    .complete-mark span {
      width: 60px;
      height: 60px;
      border-radius: 50%;
      display: grid;
      place-items: center;
      background: white;
      color: var(--success);
      font-size: 2rem;
      font-weight: 800;
    }
    .complete-title {
      margin: 0;
      font: 800 2rem/1 "Source Sans 3", sans-serif;
      letter-spacing: .08em;
    }
    .receipt-card {
      background: rgba(255,255,255,.12);
      border: 1px solid rgba(255,255,255,.14);
      border-radius: 18px;
      padding: 18px;
    }
    .receipt-row {
      display: flex;
      justify-content: space-between;
      gap: 12px;
      padding: 8px 0;
      color: rgba(255,255,255,.78);
    }
    .receipt-row strong { color: white; }
    .receipt-row--big strong {
      font-size: 2.4rem;
      line-height: 1;
    }
    .discount-grid {
      display: grid;
      grid-template-columns: repeat(3, minmax(0, 1fr));
      gap: 10px;
      margin-top: 14px;
    }
    .discount-chip {
      min-height: 56px;
      border-radius: 14px;
      border: 1px dashed rgba(184,144,58,.5);
      background: white;
      color: var(--wine);
      font: 700 .92rem/1.1 "Source Sans 3", sans-serif;
    }
    .discount-chip--active {
      background: var(--gold-pale);
      border-style: solid;
    }
    .top-actions {
      display: flex;
      gap: 10px;
      flex-wrap: wrap;
    }
    .top-actions button {
      flex: 1;
    }
  </style>
</head>
<body>
  <div id="app"></div>
  <script type="module">import init from '/static/wasm/bookstore-cart-wasm.js'; init();</script>
</body>
</html>"#,
    )
}

pub async fn pos_login(
    State(state): State<AppState>,
    Json(request): Json<PosLoginRequest>,
) -> Result<Json<PosLoginResponse>, ApiError> {
    let session_token = state
        .pos
        .login_with_pin(&request.pin)
        .await
        .map_err(|err| ApiError::new(StatusCode::UNAUTHORIZED, err.to_string()))?;
    Ok(Json(PosLoginResponse { session_token }))
}

pub async fn pos_scan(
    State(state): State<AppState>,
    Json(request): Json<PosScanRequest>,
) -> Result<Json<PosResponse>, ApiError> {
    let snapshot = state
        .pos
        .scan_item(&request.session_token, &request.barcode)
        .await
        .map_err(|err| ApiError::new(StatusCode::BAD_REQUEST, err.to_string()))?;
    Ok(Json(pos_cart_response(snapshot, "Item added to cart")))
}

pub async fn pos_quick_item(
    State(state): State<AppState>,
    Json(request): Json<PosQuickItemRequest>,
) -> Result<Json<PosResponse>, ApiError> {
    let snapshot = state
        .pos
        .add_quick_item(&request.session_token, &request.item_id, request.quantity)
        .await
        .map_err(|err| ApiError::new(StatusCode::BAD_REQUEST, err.to_string()))?;
    Ok(Json(pos_cart_response(snapshot, "Quick item added to cart")))
}

pub async fn pos_set_cart_quantity(
    State(state): State<AppState>,
    Json(request): Json<PosCartQuantityRequest>,
) -> Result<Json<PosResponse>, ApiError> {
    let snapshot = state
        .pos
        .set_cart_quantity(&request.session_token, &request.item_id, request.quantity)
        .await
        .map_err(|err| ApiError::new(StatusCode::BAD_REQUEST, err.to_string()))?;
    Ok(Json(pos_cart_response(snapshot, "Basket updated")))
}

pub async fn pos_pay_cash(
    State(state): State<AppState>,
    Json(request): Json<PosCashPaymentRequest>,
) -> Result<Json<PosResponse>, ApiError> {
    let started_at = Instant::now();
    let receipt = state
        .pos
        .checkout_cash(
            &request.session_token,
            request.tendered_cents,
            request.donate_change,
            request.discount_cents,
        )
        .await
        .map_err(|err| ApiError::new(StatusCode::BAD_REQUEST, err.to_string()))?;
    let now = current_utc_datetime();
    state
        .admin
        .record_sales_event(SalesEvent {
            tenant_id: "church-a".to_string(),
            payment_method: PaymentMethod::Cash,
            sales_cents: receipt.total_cents,
            donations_cents: receipt.donation_cents,
            cogs_cents: receipt.total_cents / 2,
            occurred_at: now,
        })
        .await;
    state
        .admin
        .create_order(
            "church-a",
            "Walk In",
            OrderChannel::Pos,
            OrderStatus::Paid,
            PaymentMethod::Cash,
            receipt.total_cents,
            now,
        )
        .await;
    log_checkout_event("pos_checkout", "sale_complete", "cash", receipt.total_cents, started_at);
    Ok(Json(PosResponse {
        status: if receipt.outcome == PosPaymentOutcome::Paid { "sale_complete" } else { "iou" },
        message: if receipt.donation_cents > 0 {
            "Cash sale complete with donated change".to_string()
        } else {
            "Cash sale complete".to_string()
        },
        total_cents: receipt.total_cents,
        change_due_cents: receipt.change_due_cents,
        donation_cents: receipt.donation_cents,
        discount_cents: receipt.discount_cents,
        items: Vec::new(),
    }))
}

pub async fn pos_pay_external_card(
    State(state): State<AppState>,
    Json(request): Json<PosExternalCardRequest>,
) -> Result<Json<PosResponse>, ApiError> {
    let started_at = Instant::now();
    let receipt = state
        .pos
        .checkout_external_card(
            &request.session_token,
            &request.external_ref,
            request.discount_cents,
        )
        .await
        .map_err(|err| ApiError::new(StatusCode::BAD_REQUEST, err.to_string()))?;
    let now = current_utc_datetime();
    state
        .admin
        .record_sales_event(SalesEvent {
            tenant_id: "church-a".to_string(),
            payment_method: PaymentMethod::ExternalCard,
            sales_cents: receipt.total_cents,
            donations_cents: receipt.donation_cents,
            cogs_cents: receipt.total_cents / 2,
            occurred_at: now,
        })
        .await;
    state
        .admin
        .create_order(
            "church-a",
            "Walk In",
            OrderChannel::Pos,
            OrderStatus::Paid,
            PaymentMethod::ExternalCard,
            receipt.total_cents,
            now,
        )
        .await;
    log_checkout_event(
        "pos_checkout",
        "sale_complete",
        "external_card",
        receipt.total_cents,
        started_at,
    );
    Ok(Json(PosResponse {
        status: if receipt.outcome == PosPaymentOutcome::Paid { "sale_complete" } else { "iou" },
        message: "Card sale complete".to_string(),
        total_cents: receipt.total_cents,
        change_due_cents: receipt.change_due_cents,
        donation_cents: receipt.donation_cents,
        discount_cents: receipt.discount_cents,
        items: Vec::new(),
    }))
}

pub async fn pos_pay_iou(
    State(state): State<AppState>,
    Json(request): Json<PosIouRequest>,
) -> Result<Json<PosResponse>, ApiError> {
    let started_at = Instant::now();
    let receipt = state
        .pos
        .checkout_iou(&request.session_token, &request.customer_name, request.discount_cents)
        .await
        .map_err(|err| ApiError::new(StatusCode::BAD_REQUEST, err.to_string()))?;
    state
        .admin
        .create_order(
            "church-a",
            &request.customer_name,
            OrderChannel::Pos,
            OrderStatus::UnpaidIou,
            PaymentMethod::Iou,
            receipt.total_cents,
            current_utc_datetime(),
        )
        .await;
    log_checkout_event("pos_checkout", "iou", "iou", receipt.total_cents, started_at);
    Ok(Json(PosResponse {
        status: if receipt.outcome == PosPaymentOutcome::UnpaidIou {
            "iou"
        } else {
            "sale_complete"
        },
        message: "Sale moved to IOU".to_string(),
        total_cents: receipt.total_cents,
        change_due_cents: receipt.change_due_cents,
        donation_cents: receipt.donation_cents,
        discount_cents: receipt.discount_cents,
        items: Vec::new(),
    }))
}
