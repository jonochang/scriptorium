use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse, Redirect};
use bookstore_app::AdminAuthSession;

use crate::AppState;
use crate::admin_intake_ui;
use crate::admin_pages;
use crate::ui::{google_fonts_link, shared_styles};
use crate::web_support::cookie_value;

pub const ADMIN_SESSION_COOKIE: &str = "scriptorium_admin_token";

pub fn sanitize_admin_next(next: Option<&str>) -> String {
    let value = next.unwrap_or("/admin");
    if value.starts_with("/admin") { value.to_string() } else { "/admin".to_string() }
}

pub async fn admin_session_from_cookie(
    state: &AppState,
    headers: &HeaderMap,
) -> Option<AdminAuthSession> {
    let token = cookie_value(headers, ADMIN_SESSION_COOKIE)?;
    state.admin.require_admin(&token).await.ok()
}

pub async fn admin_dashboard_shell(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Html<String> {
    if let Some(session) = admin_session_from_cookie(&state, &headers).await {
        Html(admin_pages::admin_dashboard_shell_html(&session))
    } else {
        Html(admin_pages::admin_login_shell_html(
            &sanitize_admin_next(params.get("next").map(String::as_str)),
            None,
        ))
    }
}

pub async fn admin_orders_shell(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Some(session) = admin_session_from_cookie(&state, &headers).await {
        Html(admin_pages::admin_orders_shell_html(&session)).into_response()
    } else {
        Redirect::to("/admin?next=/admin/orders").into_response()
    }
}

pub async fn admin_logout() -> impl IntoResponse {
    (
        [(
            axum::http::header::SET_COOKIE,
            format!("{ADMIN_SESSION_COOKIE}=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax"),
        )],
        Redirect::to("/admin"),
    )
}

pub async fn admin_intake_shell(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let Some(session) = admin_session_from_cookie(&state, &headers).await else {
        return Redirect::to("/admin?next=/admin/intake").into_response();
    };
    Html([
        r##"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Admin Intake</title>
"##,
        google_fonts_link(),
        r##"<style>
"##,
        shared_styles(),
r##"
    .intake-shell {
      min-height: 100vh;
      background:
        radial-gradient(circle at top right, rgba(139, 38, 53, 0.08), transparent 28%),
        linear-gradient(180deg, #f7f2ea 0%, #f1ebdf 100%);
    }
    .intake-topbar {
      background: #3a2f25;
      color: #f5f1ea;
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 1rem;
      padding: 0 1.75rem;
      min-height: 56px;
    }
    .intake-brand {
      display: flex;
      align-items: center;
      gap: 0.6rem;
      font-family: "Source Serif 4", Georgia, serif;
      font-size: 1rem;
      font-weight: 700;
      letter-spacing: 0.08em;
    }
    .intake-brand-mark {
      font-size: 1.1rem;
    }
    .intake-topnav {
      display: flex;
      flex-wrap: wrap;
      align-items: center;
      gap: 0.75rem;
      font-size: 0.82rem;
    }
    .intake-topnav a,
    .intake-topnav span {
      color: rgba(245, 241, 234, 0.72);
      text-decoration: none;
      font-weight: 700;
      font-size: 0.82rem;
      padding: 0 0.9rem;
      min-height: 32px;
      display: inline-flex;
      align-items: center;
      border-radius: 999px;
    }
    .intake-topnav .is-active {
      color: #fff;
      background: #8b2635;
      padding: 0.28rem 0.9rem;
      border-radius: 999px;
    }
    .intake-main {
      max-width: 860px;
      margin: 0 auto;
      padding: 2rem 1.5rem 3rem;
    }
    .intake-header {
      display: flex;
      justify-content: space-between;
      align-items: flex-start;
      gap: 1.5rem;
      margin-bottom: 1.75rem;
    }
    .intake-header h1 {
      margin: 0 0 0.35rem;
      font-family: "Source Serif 4", Georgia, serif;
      font-size: 2rem;
      line-height: 1.05;
      color: #3a2f25;
    }
    .intake-header p {
      margin: 0;
      max-width: 30rem;
      color: #7d6f60;
    }
    .intake-steps {
      display: flex;
      align-items: center;
      gap: 0;
      flex-wrap: wrap;
    }
    .intake-step {
      display: flex;
      align-items: center;
      gap: 0.55rem;
      color: #8a7e6b;
      font-size: 0.82rem;
      text-transform: capitalize;
      letter-spacing: 0.02em;
    }
    .intake-step-badge {
      width: 28px;
      height: 28px;
      border-radius: 999px;
      border: 2px solid #c4b9a8;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      font-family: "Source Serif 4", Georgia, serif;
      font-weight: 700;
      background: transparent;
      color: #8a7e6b;
    }
    .intake-step.is-active,
    .intake-step.is-done {
      color: #3a2f25;
      font-weight: 700;
    }
    .intake-step.is-active .intake-step-badge {
      border-color: #8b2635;
      background: #8b2635;
      color: #fff;
    }
    .intake-step.is-done .intake-step-badge {
      border-color: #5c6b4f;
      background: #5c6b4f;
      color: #fff;
    }
    .intake-step-connector {
      width: 40px;
      height: 2px;
      margin: 0 0.85rem;
      background: #ddd5c8;
      border-radius: 999px;
    }
    .intake-step-connector.is-done {
      background: #5c6b4f;
    }
    .intake-card {
      background: rgba(255, 255, 255, 0.96);
      border: 1px solid #e7dfd2;
      border-radius: 14px;
      padding: 1.5rem;
      box-shadow: 0 12px 30px rgba(58, 47, 37, 0.05);
      margin-bottom: 1.25rem;
    }
    .intake-card-head {
      display: flex;
      justify-content: space-between;
      align-items: center;
      gap: 1rem;
      margin-bottom: 1rem;
    }
    .intake-card-head h2 {
      margin: 0;
      font-family: "Source Serif 4", Georgia, serif;
      font-size: 1.28rem;
      color: #3a2f25;
    }
    .intake-reset {
      background: none;
      border: none;
      color: #8a7e6b;
      text-decoration: underline;
      cursor: pointer;
      font: inherit;
      padding: 0;
    }
    .intake-scanner-layout {
      display: flex;
      align-items: stretch;
      gap: 1.25rem;
    }
    .intake-camera-panel {
      width: min(360px, 42vw);
      min-height: 260px;
      border-radius: 12px;
      overflow: hidden;
      background: #1a1a1a;
      position: relative;
      flex-shrink: 0;
      display: flex;
      align-items: center;
      justify-content: center;
    }
    .intake-camera-panel video {
      position: absolute;
      inset: 0;
      width: 100%;
      height: 100%;
      object-fit: contain;
      background: #111;
    }
    .intake-camera-overlay,
    .intake-camera-empty {
      position: relative;
      z-index: 1;
      text-align: center;
      padding: 1rem;
    }
    .intake-scan-frame {
      width: 140px;
      height: 80px;
      border: 2px solid rgba(255, 255, 255, 0.55);
      border-radius: 8px;
      margin: 0 auto 0.75rem;
      position: relative;
      animation: intakePulse 1.5s ease-in-out infinite;
    }
    .intake-scan-line {
      position: absolute;
      top: 50%;
      left: 8px;
      right: 8px;
      height: 2px;
      background: #8b2635;
      box-shadow: 0 0 8px #8b2635;
      animation: intakeScanline 1.5s ease-in-out infinite;
    }
    .intake-camera-empty {
      color: #8e8578;
    }
    .intake-camera-empty svg {
      display: block;
      margin: 0 auto;
    }
    .intake-lookup-panel {
      flex: 1;
      display: flex;
      flex-direction: column;
      gap: 0.9rem;
    }
    .intake-lookup-row {
      display: flex;
      gap: 0.7rem;
      align-items: center;
    }
    .intake-isbn {
      font-family: "JetBrains Mono", monospace;
      letter-spacing: 0.08em;
      font-size: 1rem;
    }
    .intake-inline-actions {
      display: flex;
      gap: 0.5rem;
      flex-wrap: wrap;
    }
    .intake-debug-toggle {
      display: inline-flex;
      align-items: center;
      gap: 0.4rem;
      font-size: 0.85rem;
      color: #6b6257;
    }
    .intake-debug-panel {
      display: grid;
      gap: 0.55rem;
      padding: 0.75rem;
      border: 1px solid #ddd5c8;
      border-radius: 10px;
      background: #f7f2ea;
    }
    .intake-debug-panel[hidden] {
      display: none;
    }
    .intake-debug-panel canvas {
      width: 100%;
      max-width: 420px;
      border-radius: 8px;
      background: #151515;
      display: block;
    }
    .intake-debug-meta {
      font-size: 0.82rem;
      color: #6b6257;
      line-height: 1.4;
      font-family: "JetBrains Mono", monospace;
      white-space: pre-wrap;
    }
    .intake-status-copy {
      min-height: 20px;
      font-size: 0.92rem;
      color: #8a7e6b;
    }
    .intake-status-copy.is-success {
      color: #5c6b4f;
    }
    .intake-status-copy.is-busy {
      color: #8b2635;
    }
    .intake-review {
      display: none;
    }
    .intake-review.is-visible {
      display: block;
      animation: intakeFadeUp 0.4s ease;
    }
    .intake-review-layout {
      display: grid;
      grid-template-columns: 170px minmax(0, 1fr);
      gap: 1.5rem;
      align-items: start;
    }
    .intake-cover-column {
      width: 170px;
      display: flex;
      flex-direction: column;
      gap: 0.75rem;
    }
    .intake-cover-frame {
      width: 170px;
      height: 240px;
      border-radius: 8px;
      overflow: hidden;
      background: #ede8df;
      border: 2px dashed #c4b9a8;
      display: flex;
      align-items: center;
      justify-content: center;
      position: relative;
    }
    .intake-cover-frame.has-image {
      border: none;
      background: #8b2635;
    }
    .intake-cover-frame img {
      width: 100%;
      height: 100%;
      object-fit: cover;
      display: block;
    }
    .intake-cover-placeholder,
    .intake-cover-loaded {
      text-align: center;
      padding: 0.9rem;
      color: #8a7e6b;
      font-size: 0.78rem;
    }
    .intake-cover-loaded {
      color: #fff;
      font-family: "Source Serif 4", Georgia, serif;
    }
    .intake-cover-loaded strong {
      display: block;
      margin: 0.35rem 0;
      font-size: 1rem;
    }
    .intake-cover-upload {
      display: block;
      text-align: center;
      padding: 0.45rem 0.75rem;
      border-radius: 6px;
      border: 1px solid #ddd5c8;
      color: #8b2635;
      font-size: 0.82rem;
      font-weight: 600;
      cursor: pointer;
      text-decoration: none;
    }
    .intake-cover-upload input {
      display: none;
    }
    .intake-form-stack {
      display: grid;
      grid-template-columns: 1fr;
      grid-template-areas:
        "meta"
        "pricing"
        "actions";
      row-gap: 1.25rem;
      align-items: start;
      min-width: 0;
    }
    .intake-meta-stack {
      grid-area: meta;
      display: grid;
      gap: 1.25rem;
      min-width: 0;
    }
    .intake-section-label {
      margin: 0;
      padding-bottom: 0.7rem;
      border-bottom: 1px solid #e6ddd1;
      color: #8a7e6b;
      font-size: 0.78rem;
      font-weight: 700;
      letter-spacing: 0.12em;
      text-transform: uppercase;
    }
    .intake-meta-grid {
      display: grid;
      grid-template-columns: minmax(0, 1.45fr) minmax(0, 1fr);
      gap: 1rem;
      align-items: start;
    }
    .intake-field {
      min-width: 0;
    }
    .intake-field input,
    .intake-field select,
    .intake-field textarea {
      width: 100%;
      min-height: 46px;
    }
    .intake-field input,
    .intake-field select {
      font-size: 1rem;
      padding: 0.85rem 0.95rem;
    }
    .intake-field textarea {
      min-height: 112px;
      padding: 0.9rem 0.95rem;
      resize: vertical;
    }
    .intake-pricing-card {
      grid-area: pricing;
      display: grid;
      gap: 1.25rem;
      min-width: 0;
    }
    .intake-pricing-grid {
      display: grid;
      grid-template-columns: repeat(2, minmax(0, 1fr));
      gap: 1rem;
      align-items: end;
    }
    .intake-actions {
      grid-area: actions;
      display: flex;
      justify-content: flex-end;
      align-items: center;
      gap: 0.7rem;
      margin-top: 0.15rem;
    }
    .intake-stock-status {
      margin-right: auto;
      font-size: 0.85rem;
      color: #5c6b4f;
      display: flex;
      align-items: center;
      gap: 0.45rem;
    }
    .intake-stock-dot {
      width: 8px;
      height: 8px;
      border-radius: 50%;
      background: #5c6b4f;
      display: inline-block;
    }
    .intake-category-badge {
      font-size: 0.7rem;
      font-weight: 700;
      letter-spacing: 0.1em;
      text-transform: uppercase;
      background: #ede8df;
      color: #5a5044;
      padding: 0.25rem 0.65rem;
      border-radius: 6px;
      font-family: "Source Sans 3", "Segoe UI", system-ui, sans-serif;
    }
    .intake-menu-btn {
      background: none;
      border: 1px solid #ddd5c8;
      border-radius: 6px;
      color: #8a7e6b;
      cursor: pointer;
      font-size: 1.1rem;
      padding: 0.2rem 0.5rem;
      line-height: 1;
      letter-spacing: 0.15em;
    }
    .intake-price-wrap {
      position: relative;
    }
    .intake-price-wrap::before {
      content: "$";
      position: absolute;
      left: 0.75rem;
      top: 50%;
      transform: translateY(-50%);
      color: #8a7e6b;
      font-size: 1rem;
      pointer-events: none;
    }
    .intake-price-wrap input {
      padding-left: 1.5rem !important;
    }
    .intake-success {
      display: none;
      background: #5c6b4f;
      color: #fff;
      border-radius: 14px;
      padding: 2rem;
      text-align: center;
      animation: intakeFadeUp 0.3s ease;
    }
    .intake-success.is-visible {
      display: block;
    }
    .intake-success-mark {
      font-size: 2rem;
      margin-bottom: 0.4rem;
    }
    .intake-hint {
      background: rgba(139, 38, 53, 0.04);
      border-left: 3px solid #8b2635;
      border-radius: 10px;
      padding: 1rem 1.2rem;
      margin-top: 10px;
      color: #5a5044;
    }
    .intake-hint strong {
      color: #8b2635;
    }
    .intake-footer {
      max-width: 860px;
      margin: 2rem auto 0;
      padding: 1rem 1.5rem 2rem;
      border-top: 1px solid #e0d9cd;
      display: flex;
      justify-content: space-between;
      gap: 0.75rem;
      color: #8a7e6b;
      font-size: 0.82rem;
      flex-wrap: wrap;
    }
    .intake-footer-links {
      display: flex;
      gap: 1rem;
    }
    .intake-footer a {
      color: inherit;
      text-decoration: none;
      font-weight: 600;
    }
    @keyframes intakeScanline {
      0%, 100% { transform: translateY(-15px); }
      50% { transform: translateY(15px); }
    }
    @keyframes intakePulse {
      0%, 100% { opacity: 0.5; }
      50% { opacity: 1; }
    }
    @keyframes intakeFadeUp {
      from { opacity: 0; transform: translateY(12px); }
      to { opacity: 1; transform: translateY(0); }
    }
    @media (max-width: 900px) {
      .intake-header,
      .intake-scanner-layout,
      .intake-footer {
        flex-direction: column;
      }
      .intake-review-layout,
      .intake-meta-grid,
      .intake-pricing-grid {
        grid-template-columns: 1fr;
      }
      .intake-pricing-grid {
        grid-template-columns: 1fr;
      }
      .intake-topbar {
        padding: 0.85rem 1rem;
      }
      .intake-camera-panel,
      .intake-cover-column,
      .intake-cover-frame {
        width: 100%;
      }
      .intake-main {
        padding: 1.5rem 1rem 2.5rem;
      }
      .intake-actions {
        justify-content: stretch;
      }
      .intake-actions > * {
        flex: 1;
        text-align: center;
      }
    }
  </style>
</head>
<body class="intake-shell">
  <header class="intake-topbar">
    <div class="intake-brand"><span class="intake-brand-mark">✝</span> SCRIPTORIUM <span style="font-size:12px;font-weight:600;letter-spacing:1px;background:rgba(245,241,234,0.12);padding:3px 10px;border-radius:12px;margin-left:4px;font-family:'Source Sans 3','Segoe UI',system-ui,sans-serif">ADMIN</span></div>
    <nav class="intake-topnav" aria-label="Admin sections">
      <a href="/admin">Dashboard</a>
      <a href="/admin/orders">Orders</a>
      <span class="is-active">Intake</span>
      <span style="width:1px;height:20px;background:rgba(245,241,234,0.15);margin:0 8px;padding:0;min-height:auto;border-radius:0"></span>
      <a href="/catalog" style="font-size:13px;font-weight:500;color:rgba(245,241,234,0.4);min-height:auto;padding:5px 8px">Store</a>
      <a href="/pos" style="font-size:13px;font-weight:500;color:rgba(245,241,234,0.4);min-height:auto;padding:5px 8px">POS</a>
      <a href="/admin" style="font-size:13px;font-weight:500;color:rgba(245,241,234,0.4);min-height:auto;padding:5px 8px">Sign out</a>
    </nav>
  </header>
  <main class="intake-main">
    <div
      id="intake-root"
      data-token=""##,
        &session.token,
        r##""
      data-tenant-id=""##,
        &session.tenant_id,
        r##""
    ></div>
  </main>
  <footer class="intake-footer">
    <span>Scriptorium 2026. Parish browsing, intake &amp; Sunday-close reconciliation.</span>
    <div class="intake-footer-links">
      <a href="/admin">Dashboard</a>
      <a href="/admin/orders">Orders</a>
      <a href="/admin/intake">Intake</a>
    </div>
  </footer>
  "##,
        admin_intake_ui::admin_intake_script(),
    ]
    .concat())
    .into_response()
}
