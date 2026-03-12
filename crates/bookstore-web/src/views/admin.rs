use askama::Template;
use bookstore_app::AdminAuthSession;

use crate::admin_ui::admin_dashboard_script;
use crate::ui::shared_styles;

#[derive(Clone)]
pub struct AdminPageContext {
    pub page_title: String,
    pub current: String,
    pub eyebrow: String,
    pub title: String,
    pub lede: String,
    pub badges: Vec<String>,
    pub actions_html: String,
    pub extra_styles: String,
}

impl AdminPageContext {
    pub fn new(
        page_title: impl Into<String>,
        current: impl Into<String>,
        eyebrow: impl Into<String>,
        title: impl Into<String>,
        lede: impl Into<String>,
        badges: &[&str],
        actions_html: impl Into<String>,
    ) -> Self {
        Self {
            page_title: page_title.into(),
            current: current.into(),
            eyebrow: eyebrow.into(),
            title: title.into(),
            lede: lede.into(),
            badges: badges.iter().map(|badge| (*badge).to_string()).collect(),
            actions_html: actions_html.into(),
            extra_styles: String::new(),
        }
    }
}

fn admin_layout_styles() -> &'static str {
    r#"
      .admin-shell {
        min-height: 100vh;
        background:
          radial-gradient(circle at top right, rgba(139, 38, 53, 0.08), transparent 28%),
          linear-gradient(180deg, #f7f2ea 0%, #f1ebdf 100%);
      }
      .admin-topbar {
        background: #3a2f25;
        color: #f5f1ea;
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 1rem;
        padding: 0 1.75rem;
        min-height: 52px;
      }
      .admin-brand {
        display: flex;
        align-items: center;
        gap: 0.6rem;
        font-family: "Crimson Pro", serif;
        font-size: 1rem;
        font-weight: 700;
        letter-spacing: 0.08em;
      }
      .admin-brand-mark { font-size: 1.1rem; }
      .admin-topnav {
        display: flex;
        flex-wrap: wrap;
        align-items: center;
        gap: 0.75rem;
        font-size: 0.82rem;
      }
      .admin-topnav a,
      .admin-topnav span {
        color: rgba(245, 241, 234, 0.72);
        text-decoration: none;
        font-weight: 600;
      }
      .admin-topnav .is-active {
        color: #fff;
        background: #8b2635;
        padding: 0.28rem 0.9rem;
        border-radius: 999px;
      }
      .admin-main {
        max-width: 1220px;
        margin: 0 auto;
        padding: 2rem 1.5rem 3rem;
      }
      .admin-header {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        gap: 1.5rem;
        margin-bottom: 1.5rem;
      }
      .admin-header__eyebrow {
        margin: 0 0 0.3rem;
        font-size: 0.78rem;
        font-weight: 800;
        letter-spacing: 0.12em;
        text-transform: uppercase;
        color: #8b2635;
      }
      .admin-header h1 {
        margin: 0 0 0.45rem;
        font-family: "Crimson Pro", serif;
        font-size: 2.1rem;
        line-height: 1.04;
        color: #3a2f25;
      }
      .admin-header p {
        margin: 0;
        max-width: 40rem;
        color: #7d6f60;
      }
      .admin-header__meta {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
        align-items: flex-end;
      }
      .admin-badges {
        display: flex;
        gap: 0.55rem;
        flex-wrap: wrap;
        justify-content: flex-end;
      }
      .admin-badge {
        display: inline-flex;
        align-items: center;
        min-height: 30px;
        padding: 0 0.8rem;
        border-radius: 999px;
        background: rgba(255, 255, 255, 0.85);
        border: 1px solid #e0d8cc;
        color: #6f6052;
        font-size: 0.8rem;
        font-weight: 700;
      }
      .admin-badge--accent {
        background: rgba(139, 38, 53, 0.08);
        border-color: rgba(139, 38, 53, 0.18);
        color: #8b2635;
      }
      .admin-actions {
        display: flex;
        gap: 0.7rem;
        flex-wrap: wrap;
        justify-content: flex-end;
      }
      .admin-link {
        display: inline-flex;
        align-items: center;
        min-height: 38px;
        padding: 0 0.95rem;
        border-radius: 999px;
        border: 1px solid #d9cfbf;
        background: rgba(255, 255, 255, 0.85);
        color: #5a4a3a;
        text-decoration: none;
        font-weight: 700;
      }
      .admin-link--accent {
        border-color: #8b2635;
        background: #8b2635;
        color: #fff;
      }
      .admin-content {
        display: grid;
        gap: 18px;
      }
      .admin-footer {
        max-width: 1220px;
        margin: 2rem auto 0;
        padding: 1rem 1.5rem 2rem;
        border-top: 1px solid #e0d9cd;
        display: flex;
        justify-content: space-between;
        gap: 1rem;
        color: #8a7e6b;
        font-size: 0.82rem;
      }
      .admin-footer__links {
        display: flex;
        gap: 1rem;
      }
      .admin-footer a {
        color: inherit;
        text-decoration: none;
        font-weight: 600;
      }
      @media (max-width: 900px) {
        .admin-topbar,
        .admin-header,
        .admin-footer {
          flex-direction: column;
          align-items: flex-start;
        }
        .admin-topbar {
          padding: 0.85rem 1rem;
        }
        .admin-main {
          padding: 1.5rem 1rem 2.5rem;
        }
        .admin-header__meta,
        .admin-badges,
        .admin-actions {
          align-items: flex-start;
          justify-content: flex-start;
        }
      }
    "#
}

#[derive(Template)]
#[template(path = "admin/login.html")]
pub struct AdminLoginTemplate {
    pub context: AdminPageContext,
    pub shared_styles: &'static str,
    pub admin_layout_styles: &'static str,
    pub next: String,
    pub message: String,
}

impl AdminLoginTemplate {
    pub fn new(next: &str, message: Option<&str>) -> Self {
        Self {
            context: AdminPageContext::new(
                "Scriptorium Admin Sign-In",
                "dashboard",
                "Admin Office",
                "Admin Sign-In",
                "Unlock the dashboard, intake, and order follow-up tools from one credentialed entry point.",
                &["Protected access", "Dashboard gate", "Session based"],
                r#"<a class="admin-link" href="/catalog">Back to catalog</a><a class="admin-link" href="/pos">POS</a>"#,
            ),
            shared_styles: shared_styles(),
            admin_layout_styles: admin_layout_styles(),
            next: next.to_string(),
            message: message.unwrap_or("Sign in to continue to the admin office.").to_string(),
        }
    }
}

#[derive(Template)]
#[template(path = "admin/dashboard.html")]
pub struct AdminDashboardTemplate {
    pub context: AdminPageContext,
    pub shared_styles: &'static str,
    pub admin_layout_styles: &'static str,
    pub session_script: String,
    pub orders_placeholder: String,
    pub dashboard_script: &'static str,
}

impl AdminDashboardTemplate {
    pub fn new(session: &AdminAuthSession, orders_placeholder: String) -> Self {
        Self {
            context: AdminPageContext::new(
                "Scriptorium Admin",
                "dashboard",
                "Admin Office",
                "Good morning, Father Michael",
                "Reconcile takings, watch the shelves, and settle unpaid tabs before the parish hall empties.",
                &["Treasurer view", "Sunday close", "Pastoral follow-up"],
                r#"<a class="admin-link" href="/admin/orders">Order management</a><a class="admin-link admin-link--accent" href="/admin/intake">Add product</a><a class="admin-link" href="/admin/logout">Sign out</a>"#,
            ),
            shared_styles: shared_styles(),
            admin_layout_styles: admin_layout_styles(),
            session_script: admin_session_script(session),
            orders_placeholder,
            dashboard_script: admin_dashboard_script(),
        }
    }
}

#[derive(Template)]
#[template(path = "admin/orders.html")]
pub struct AdminOrdersTemplate {
    pub context: AdminPageContext,
    pub shared_styles: &'static str,
    pub admin_layout_styles: &'static str,
    pub session_script: String,
    pub orders_placeholder: String,
    pub dashboard_script: &'static str,
}

impl AdminOrdersTemplate {
    pub fn new(session: &AdminAuthSession, orders_placeholder: String) -> Self {
        Self {
            context: AdminPageContext::new(
                "Scriptorium Order Management",
                "orders",
                "Admin Office",
                "Order Management",
                "Track paid orders, open tabs, and follow-up actions from one dedicated table.",
                &["Dedicated orders page", "Export-ready", "IOU follow-up"],
                r#"<a class="admin-link" href="/admin">Dashboard</a><a class="admin-link admin-link--accent" href="/admin/intake">Add product</a><a class="admin-link" href="/admin/logout">Sign out</a>"#,
            ),
            shared_styles: shared_styles(),
            admin_layout_styles: admin_layout_styles(),
            session_script: admin_session_script(session),
            orders_placeholder,
            dashboard_script: admin_dashboard_script(),
        }
    }
}

#[derive(Template)]
#[template(path = "admin/intake.html")]
pub struct AdminIntakeTemplate {
    pub context: AdminPageContext,
    pub shared_styles: &'static str,
    pub admin_layout_styles: &'static str,
    pub intake_script: &'static str,
    pub token: String,
    pub tenant_id: String,
}

impl AdminIntakeTemplate {
    pub fn new(session: &AdminAuthSession, intake_script: &'static str) -> Self {
        let mut context = AdminPageContext::new(
            "Admin Intake",
            "intake",
            "Admin Intake",
            "Add New Product",
            format!(
                "Scan or type an ISBN, review the metadata, then save a shelf-ready product record for tenant {}.",
                session.tenant_id
            ),
            &["Metadata first", "Shelf-ready pricing", "Volunteer friendly"],
            r#"<a class="admin-link" href="/admin">Dashboard</a><a class="admin-link" href="/admin/orders">Orders</a><a class="admin-link" href="/admin/logout">Sign out</a>"#,
        );
        context.extra_styles = intake_extra_styles().to_string();
        Self {
            context,
            shared_styles: shared_styles(),
            admin_layout_styles: admin_layout_styles(),
            intake_script,
            token: session.token.clone(),
            tenant_id: session.tenant_id.clone(),
        }
    }
}

fn intake_extra_styles() -> &'static str {
    r#"
      .admin-main--intake {
        max-width: 860px;
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
      .intake-scanner-layout,
      .intake-review-layout {
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
      .intake-lookup-panel {
        flex: 1;
        display: flex;
        flex-direction: column;
        gap: 0.9rem;
      }
      .intake-lookup-row,
      .intake-inline-actions,
      .intake-form-row,
      .intake-pricing-row,
      .intake-actions {
        display: flex;
        gap: 0.7rem;
      }
      .intake-isbn {
        font-family: "JetBrains Mono", monospace;
        letter-spacing: 0.08em;
        font-size: 1rem;
      }
      .intake-status-copy {
        min-height: 20px;
        font-size: 0.92rem;
        color: #8a7e6b;
      }
      .intake-status-copy.is-success { color: #5c6b4f; }
      .intake-status-copy.is-busy { color: #8b2635; }
      .intake-review { display: none; }
      .intake-review.is-visible {
        display: block;
        animation: intakeFadeUp 0.4s ease;
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
      .intake-cover-upload input { display: none; }
      .intake-form-stack {
        flex: 1;
        display: flex;
        flex-direction: column;
        gap: 0.9rem;
      }
      .intake-field { flex: 1; }
      .intake-field--double { flex: 2; }
      .intake-field textarea {
        min-height: 84px;
        resize: vertical;
      }
      .intake-pricing-row {
        background: #f7f4ef;
        border-radius: 10px;
        padding: 0.9rem 1rem;
        align-items: end;
      }
      .intake-actions {
        justify-content: flex-end;
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
      .intake-success.is-visible { display: block; }
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
        .intake-scanner-layout,
        .intake-review-layout,
        .intake-form-row,
        .intake-pricing-row {
          flex-direction: column;
        }
        .intake-camera-panel,
        .intake-cover-column,
        .intake-cover-frame {
          width: 100%;
        }
        .intake-actions {
          justify-content: stretch;
        }
        .intake-actions > * {
          flex: 1;
          text-align: center;
        }
      }
    "#
}

fn admin_session_script(session: &AdminAuthSession) -> String {
    format!(
        r#"<script>window.SCRIPTORIUM_ADMIN_SESSION = {{ token: {token:?}, tenantId: {tenant:?} }};</script>"#,
        token = session.token,
        tenant = session.tenant_id
    )
}
