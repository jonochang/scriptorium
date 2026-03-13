use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse, Redirect};
use bookstore_app::AdminAuthSession;

use crate::AppState;
use crate::admin_intake_ui;
use crate::admin_pages;
use crate::ui::{google_fonts_link, shared_styles, site_footer};
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
      min-height: 52px;
    }
    .intake-brand {
      display: flex;
      align-items: center;
      gap: 0.6rem;
      font-family: "Crimson Pro", serif;
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
      font-weight: 600;
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
      font-family: "Crimson Pro", serif;
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
      font-family: "Crimson Pro", serif;
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
      font-family: "Crimson Pro", serif;
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
      width: 220px;
      min-height: 180px;
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
      object-fit: cover;
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
      display: flex;
      gap: 1.5rem;
    }
    .intake-cover-column {
      width: 140px;
      flex-shrink: 0;
      display: flex;
      flex-direction: column;
      gap: 0.55rem;
    }
    .intake-cover-frame {
      width: 140px;
      height: 200px;
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
      font-family: "Crimson Pro", serif;
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
      flex: 1;
      display: flex;
      flex-direction: column;
      gap: 0.9rem;
    }
    .intake-form-row {
      display: flex;
      gap: 1rem;
    }
    .intake-field {
      flex: 1;
    }
    .intake-field--double {
      flex: 2;
    }
    .intake-field textarea {
      min-height: 84px;
      resize: vertical;
    }
    .intake-pricing-row {
      background: #f7f4ef;
      border-radius: 10px;
      padding: 0.9rem 1rem;
      display: flex;
      gap: 1rem;
      align-items: end;
    }
    .intake-actions {
      display: flex;
      justify-content: flex-end;
      gap: 0.7rem;
      margin-top: 0.15rem;
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
      gap: 1rem;
      color: #8a7e6b;
      font-size: 0.82rem;
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
      .intake-review-layout,
      .intake-scanner-layout,
      .intake-pricing-row,
      .intake-footer {
        flex-direction: column;
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
      .intake-form-row {
        flex-direction: column;
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
    <div class="intake-brand"><span class="intake-brand-mark">✝</span> SCRIPTORIUM</div>
    <nav class="intake-topnav" aria-label="Admin sections">
      <a href="/catalog">Catalog</a>
      <a href="/cart">Cart</a>
      <a href="/admin">Admin</a>
      <span class="is-active">Intake</span>
    </nav>
  </header>
  <main class="intake-main">
    <input id="token" name="token" type="hidden" value=""##,
        &session.token,
        r##"" />
    <input id="tenant-id" name="tenant-id" type="hidden" value=""##,
        &session.tenant_id,
        r##"" />
    <input id="cover-image-key" name="cover-image-key" type="hidden" value="" />
    <div class="intake-header">
      <div>
        <h1>Add New Product</h1>
        <p>Scan or type an ISBN, review the metadata, then save a shelf-ready product record for tenant "##,
        &session.tenant_id,
        r##"."</p>
      </div>
      <div class="intake-steps" aria-label="Intake steps">
        <div class="intake-step is-active" data-step="0"><span class="intake-step-badge">1</span><span>scan</span></div>
        <div class="intake-step-connector" data-step-connector="0"></div>
        <div class="intake-step" data-step="1"><span class="intake-step-badge">2</span><span>review</span></div>
        <div class="intake-step-connector" data-step-connector="1"></div>
        <div class="intake-step" data-step="2"><span class="intake-step-badge">3</span><span>save</span></div>
      </div>
    </div>
    <section class="intake-card">
      <div class="intake-card-head">
        <h2>ISBN &amp; Cover</h2>
        <button type="button" class="intake-reset" id="intake-reset" hidden>Start over</button>
      </div>
      <div class="intake-scanner-layout">
        <div class="intake-camera-panel">
          <video id="camera" autoplay playsinline></video>
          <div id="camera-overlay" class="intake-camera-overlay" hidden>
            <div class="intake-scan-frame"><div class="intake-scan-line"></div></div>
            <span style="font-size:12px;color:#fff;opacity:0.72;">Hold barcode steady</span>
          </div>
          <div id="camera-empty" class="intake-camera-empty">
            <svg width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="#666" stroke-width="1.5">
              <rect x="2" y="4" width="20" height="16" rx="2" />
              <circle cx="12" cy="12" r="3" />
              <path d="M17 4h2a2 2 0 0 1 2 2v2M7 4H5a2 2 0 0 0-2 2v2M17 20h2a2 2 0 0 0 2-2v-2M7 20H5a2 2 0 0 1-2-2v-2" />
            </svg>
            <div style="font-size:12px;margin-top:8px;">Camera off</div>
          </div>
        </div>
        <div class="intake-lookup-panel">
          <div>
            <label class="field-label" for="isbn">ISBN</label>
            <div class="intake-lookup-row">
              <input class="intake-isbn" id="isbn" name="isbn" placeholder="978..." inputmode="numeric" />
              <button class="accent-button" type="button" id="lookup">Fetch</button>
            </div>
          </div>
          <div class="intake-inline-actions">
            <button class="primary-button" type="button" id="camera-start">Start scanner</button>
            <button class="ghost-link ghost-link--ink" type="button" id="camera-stop" hidden>Stop</button>
          </div>
          <div id="scanner-status" class="intake-status-copy" aria-live="polite">Scan a barcode or type an ISBN to begin.</div>
          <div id="intake-auth-status" class="notice-panel notice-panel--success" aria-live="polite">Signed in. Metadata lookup and product save are ready.</div>
          <div id="intake-lookup-status" class="notice-panel">Lookup and save status will appear here.</div>
        </div>
      </div>
    </section>
    <section class="intake-card intake-review" id="intake-review">
      <div class="intake-card-head">
        <h2>Product Details</h2>
      </div>
      <div class="intake-review-layout">
        <div class="intake-cover-column">
          <div id="cover-frame" class="intake-cover-frame">
            <div id="cover-placeholder" class="intake-cover-placeholder">
              <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="#c4b9a8" stroke-width="1.5">
                <rect x="3" y="3" width="18" height="18" rx="2" />
                <circle cx="8.5" cy="8.5" r="1.5" />
                <path d="M21 15l-5-5L5 21" />
              </svg>
              <div style="margin-top:6px;">No cover</div>
            </div>
            <div id="cover-loaded" class="intake-cover-loaded" hidden>
              <div style="font-size:10px;text-transform:uppercase;letter-spacing:1.5px;opacity:0.72;">Cover loaded</div>
              <strong>Cover asset</strong>
              <span>Stored for the product record.</span>
            </div>
            <img id="cover-preview" alt="Uploaded cover preview" hidden />
          </div>
          <label class="intake-cover-upload">Upload cover<input id="cover-file" name="cover-file" type="file" accept="image/*,.svg" /></label>
          <button class="ghost-link ghost-link--ink" type="button" id="upload-cover">Upload selected file</button>
        </div>
        <form id="intake-form" class="intake-form-stack">
          <div class="intake-form-row">
            <div class="intake-field intake-field--double">
              <label class="field-label" for="title">Title</label>
              <input id="title" name="title" placeholder="Book title" />
            </div>
            <div class="intake-field">
              <label class="field-label" for="author">Author</label>
              <input id="author" name="author" placeholder="Author name" />
            </div>
          </div>
          <div class="intake-form-row">
            <div class="intake-field">
              <label class="field-label" for="publisher">Publisher</label>
              <input id="publisher" name="publisher" placeholder="Publisher" />
            </div>
            <div class="intake-field">
              <label class="field-label" for="category">Category</label>
              <select id="category" name="category">
                <option value="Books">Books</option>
                <option value="Icons">Icons</option>
                <option value="Liturgical">Liturgical</option>
                <option value="Gifts">Gifts</option>
              </select>
            </div>
          </div>
          <div class="intake-field">
            <label class="field-label" for="description">Description</label>
            <textarea id="description" name="description" placeholder="Description"></textarea>
          </div>
          <div class="intake-pricing-row">
            <div class="intake-field">
              <label class="field-label" for="cost-cents">Cost ¢</label>
              <input id="cost-cents" name="cost-cents" value="900" inputmode="numeric" />
            </div>
            <div class="intake-field">
              <label class="field-label" for="retail-cents">Retail ¢</label>
              <input id="retail-cents" name="retail-cents" value="1699" inputmode="numeric" />
            </div>
            <div class="intake-field">
              <label class="field-label" for="initial-stock">Stock</label>
              <input id="initial-stock" name="initial-stock" value="5" inputmode="numeric" />
            </div>
            <div class="intake-field">
              <label class="field-label" for="reorder-point">Reorder at</label>
              <input id="reorder-point" name="reorder-point" value="3" inputmode="numeric" />
            </div>
            <div class="intake-field">
              <label class="field-label" for="vendor">Vendor</label>
              <select id="vendor" name="vendor">
                <option value="Church Supplier">Church Supplier</option>
                <option value="Direct Publisher">Direct Publisher</option>
                <option value="Donation">Donation</option>
                <option value="Holy Trinity">Holy Trinity</option>
              </select>
            </div>
          </div>
          <div class="intake-actions">
            <a class="ghost-link ghost-link--ink" href="/admin">Cancel</a>
            <button class="accent-button" type="button" id="save-product">Save Product</button>
          </div>
        </form>
      </div>
    </section>
    <section id="intake-success" class="intake-success" aria-live="polite">
      <div class="intake-success-mark">✓</div>
      <h2 style="margin:0 0 0.35rem;font-family:&quot;Crimson Pro&quot;,serif;font-size:1.45rem;">Product saved</h2>
      <p id="intake-success-copy" style="margin:0;opacity:0.84;">Resetting for next item...</p>
    </section>
    <section class="intake-hint" id="intake-hint">
      <div style="font-size:13px;font-weight:700;color:#8b2635;margin-bottom:4px;">Volunteer flow</div>
      <p style="margin:0;font-size:13px;line-height:1.5;">Start the scanner and hold the book barcode in frame. The ISBN will auto-fill, then press <strong>Fetch</strong> to pull metadata. Confirm the details, optionally upload a cover, and hit <strong>Save Product</strong>.</p>
    </section>
  </main>
  <footer class="intake-footer">
    <span>Scriptorium 2026. Parish browsing, intake, and Sunday-close reconciliation.</span>
    <div class="intake-footer-links">
      <a href="/catalog">Catalog</a>
      <a href="/cart">Cart</a>
      <a href="/admin">Admin</a>
    </div>
  </footer>
  "##,
        site_footer(),
        admin_intake_ui::admin_intake_script(),
    ]
    .concat())
    .into_response()
}
