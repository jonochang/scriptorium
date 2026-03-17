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
        min-height: 56px;
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
        margin-bottom: 1.25rem;
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
                &[],
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
        let mut context = AdminPageContext::new(
            "Scriptorium Admin",
            "dashboard",
            "Admin Office",
            "Good morning, Father Michael",
            "Review finances, close the table, and track parish follow-up from one dashboard.",
            &[],
            r#"<a class="admin-link" href="/admin/logout">Sign out</a>"#,
        );
        context.extra_styles = dashboard_extra_styles().to_string();
        Self {
            context,
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
        let mut context = AdminPageContext::new(
            "Scriptorium Order Management",
            "orders",
            "Admin Office",
            "Order Management",
            "Track paid orders, open tabs, and follow-up actions from one dedicated table.",
            &[],
            r#"<a class="admin-link" href="/admin/logout">Sign out</a>"#,
        );
        context.extra_styles = orders_extra_styles().to_string();
        Self {
            context,
            shared_styles: shared_styles(),
            admin_layout_styles: admin_layout_styles(),
            session_script: admin_session_script(session),
            orders_placeholder,
            dashboard_script: admin_dashboard_script(),
        }
    }
}

fn orders_extra_styles() -> &'static str {
    r#"
      .office-shell {
        display: grid;
        gap: 1.35rem;
      }
      .office-switcher,
      .office-toolbar,
      .office-toolbar__group,
      .office-switcher__actions,
      .office-metric-row,
      .office-footnote,
      .office-table-card__head,
      .office-legend {
        display: flex;
        align-items: center;
        gap: 0.85rem;
      }
      .office-switcher,
      .office-toolbar,
      .office-table-card,
      .office-metric {
        animation: dashboardFadeUp 0.28s ease both;
      }
      .office-switcher {
        justify-content: space-between;
        align-items: flex-start;
        gap: 1.2rem;
      }
      .office-switcher__lede {
        max-width: 34rem;
        font-size: 1rem;
        color: #5a5044;
      }
      .office-switcher__actions {
        flex-wrap: wrap;
        justify-content: flex-end;
      }
      .office-tab-group,
      .office-chip-group {
        display: inline-flex;
        align-items: center;
        gap: 0.15rem;
        background: #ede8df;
        padding: 0.18rem;
        border-radius: 0.8rem;
      }
      .office-tab,
      .office-chip,
      .office-stock-button {
        border: none;
        border-radius: 0.6rem;
        background: transparent;
        color: #5a5044;
        font: inherit;
        cursor: pointer;
        transition: all 120ms ease;
      }
      .office-tab {
        padding: 0.6rem 1.05rem;
        font-size: 0.86rem;
        font-weight: 700;
      }
      .office-tab.is-active,
      .office-chip--active {
        background: #3a2f25;
        color: #fff;
      }
      .office-chip {
        padding: 0.45rem 0.9rem;
        font-size: 0.78rem;
        font-weight: 700;
        white-space: nowrap;
      }
      .office-pane {
        display: none;
      }
      .office-pane.is-active {
        display: grid;
        gap: 1rem;
      }
      .office-metric-row {
        align-items: stretch;
      }
      .office-metric-row--inventory .office-metric {
        flex: 1 1 0;
      }
      .office-metric {
        flex: 1;
        min-width: 0;
        padding: 1rem 1.1rem;
        border-radius: 0.95rem;
        border: 1px solid #e8e2d8;
        background: #fff;
      }
      .office-metric--dark {
        background: #3a2f25;
        border-color: #3a2f25;
        color: #f5f1ea;
      }
      .office-metric--warn {
        background: rgba(201, 148, 62, 0.08);
        border-color: rgba(201, 148, 62, 0.45);
      }
      .office-metric--danger {
        background: rgba(139, 38, 53, 0.06);
        border-color: rgba(139, 38, 53, 0.35);
      }
      .office-metric__label {
        display: block;
        margin-bottom: 0.2rem;
        font-size: 0.66rem;
        font-weight: 800;
        letter-spacing: 0.11em;
        text-transform: uppercase;
        opacity: 0.72;
      }
      .office-metric__value {
        font-family: "Crimson Pro", serif;
        font-size: 1.65rem;
        color: #3a2f25;
      }
      .office-metric--dark .office-metric__value {
        color: #fff;
      }
      .office-metric__value--green {
        color: #5c6b4f;
      }
      .office-metric__value--amber {
        color: #c9943e;
      }
      .office-metric__value--accent {
        color: #8b2635;
      }
      .office-toolbar {
        justify-content: space-between;
        flex-wrap: wrap;
      }
      .office-toolbar__group {
        flex-wrap: wrap;
      }
      .office-toolbar__group--right {
        justify-content: flex-end;
      }
      .office-toolbar__sep {
        font-size: 0.75rem;
        color: #b0a694;
      }
      .office-search {
        position: relative;
        display: inline-flex;
        align-items: center;
      }
      .office-search__icon {
        position: absolute;
        left: 0.8rem;
        color: #b0a694;
        pointer-events: none;
      }
      .office-search input,
      .office-date {
        border: 1px solid #e8e2d8;
        border-radius: 0.65rem;
        background: #fff;
        color: #3a2f25;
        font: inherit;
      }
      .office-search input {
        min-width: 15rem;
        padding: 0.65rem 0.85rem 0.65rem 2rem;
      }
      .office-date {
        padding: 0.55rem 0.65rem;
        font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
        font-size: 0.78rem;
      }
      .office-table-card {
        padding: 1.1rem 1.15rem 0.95rem;
        border: 1px solid #e8e2d8;
        border-radius: 1rem;
        background: #fff;
      }
      .office-table-card__head {
        justify-content: space-between;
        align-items: flex-start;
        margin-bottom: 0.9rem;
      }
      .office-legend {
        flex-wrap: wrap;
        justify-content: flex-end;
      }
      .office-legend__item {
        padding: 0.25rem 0.55rem;
        border-radius: 999px;
        background: #f9f6f1;
        color: #8a7e6b;
        font-size: 0.72rem;
        font-weight: 700;
      }
      .office-footnote {
        justify-content: space-between;
        margin-top: 1rem;
        padding-top: 0.75rem;
        border-top: 1px solid #ede8df;
        color: #8a7e6b;
      }
      .office-products-wrap .orders-table td:last-child,
      .office-products-wrap .orders-table th:last-child {
        text-align: right;
      }
      .office-stock-cell {
        display: flex;
        align-items: center;
        gap: 0.35rem;
        white-space: nowrap;
      }
      .office-stock-button {
        width: 1.65rem;
        height: 1.65rem;
        border: 1px solid #e8e2d8;
        background: #f9f6f1;
        font-size: 1rem;
        line-height: 1;
      }
      .office-stock-value {
        min-width: 1.8rem;
        text-align: center;
        font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
        font-weight: 700;
      }
      .office-stock-note {
        margin-left: 0.25rem;
        color: #b0a694;
        font-size: 0.72rem;
      }
      .office-inline-badge {
        display: inline-flex;
        align-items: center;
        padding: 0.2rem 0.55rem;
        border-radius: 999px;
        background: #f9f6f1;
        color: #5a5044;
        font-size: 0.72rem;
        font-weight: 700;
      }
      .office-inline-badge--ok {
        background: rgba(92, 107, 79, 0.08);
        color: #5c6b4f;
      }
      .office-inline-badge--low {
        background: rgba(201, 148, 62, 0.1);
        color: #c9943e;
      }
      .office-inline-badge--out {
        background: rgba(139, 38, 53, 0.08);
        color: #8b2635;
      }
      .office-product-meta {
        margin-top: 0.15rem;
        color: #8a7e6b;
        font-size: 0.76rem;
      }
      @media (max-width: 900px) {
        .office-switcher,
        .office-toolbar,
        .office-metric-row,
        .office-table-card__head,
        .office-footnote {
          flex-direction: column;
          align-items: stretch;
        }
        .office-switcher__actions,
        .office-legend,
        .office-toolbar__group--right {
          justify-content: flex-start;
        }
        .office-search input {
          min-width: 0;
          width: 100%;
        }
      }
    "#
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
            &[],
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
        margin-top: 10px;
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

fn dashboard_extra_styles() -> &'static str {
    r#"
      .admin-main--dashboard {
        max-width: 960px;
      }
      .dashboard-shell {
        display: grid;
        gap: 1rem;
      }
      .dashboard-switcher {
        display: flex;
        align-items: flex-end;
        justify-content: space-between;
        gap: 1rem;
        padding-bottom: 1.25rem;
        border-bottom: 1px solid #e0d9cd;
      }
      .dashboard-switcher__lede {
        font-size: 0.82rem;
        color: #8a7e6b;
      }
      .dashboard-tabs {
        display: flex;
        align-items: center;
        gap: 1rem;
        flex-wrap: wrap;
      }
      .dashboard-tab-group {
        display: flex;
        background: #ede8df;
        border-radius: 10px;
        padding: 3px;
        gap: 2px;
      }
      .dashboard-tab {
        padding: 8px 16px;
        font-size: 12px;
        font-weight: 500;
        color: #5a5044;
        background: transparent;
        border: none;
        border-radius: 8px;
        cursor: pointer;
      }
      .dashboard-tab.is-active {
        color: #fff;
        background: #3a2f25;
        font-weight: 700;
      }
      .dashboard-pane { display: none; }
      .dashboard-pane.is-active {
        display: block;
        animation: dashboardFadeUp 0.25s ease both;
      }
      .dashboard-toolbar,
      .dashboard-stat-row,
      .dashboard-payments,
      .dashboard-inventory-stats,
      .dashboard-two-col,
      .dashboard-note-row {
        display: flex;
        gap: 10px;
      }
      .dashboard-toolbar {
        align-items: center;
        margin-bottom: 16px;
      }
      .dashboard-date {
        padding: 6px 10px;
        font-size: 13px;
        font-family: "JetBrains Mono", monospace;
        border: 1px solid #e8e2d8;
        border-radius: 6px;
        background: #fff;
        color: #3a2f25;
        outline: none;
      }
      .dashboard-stat {
        flex: 1;
        min-width: 0;
        padding: 16px 18px;
        background: #fff;
        border-radius: 12px;
        border: 1px solid #e8e2d8;
      }
      .dashboard-stat--accent {
        background: #3a2f25;
        border-color: #3a2f25;
      }
      .dashboard-stat__label {
        font-size: 11px;
        text-transform: uppercase;
        letter-spacing: 1px;
        color: #8a7e6b;
        font-weight: 600;
        margin-bottom: 4px;
      }
      .dashboard-stat--accent .dashboard-stat__label {
        color: rgba(245,241,234,0.5);
      }
      .dashboard-stat__value {
        font-size: 24px;
        font-weight: 700;
        font-family: "Crimson Pro", serif;
        color: #3a2f25;
      }
      .dashboard-stat--accent .dashboard-stat__value {
        color: #f5f1ea;
      }
      .dashboard-payment-card,
      .dashboard-mini-stat {
        flex: 1;
        padding: 14px 16px;
        background: #f9f6f1;
        border-radius: 10px;
        text-align: center;
      }
      .dashboard-payment-card strong,
      .dashboard-mini-stat strong {
        display: block;
        font-size: 20px;
        font-family: "Crimson Pro", serif;
        color: #b0a694;
      }
      .dashboard-payment-card span,
      .dashboard-mini-stat span {
        display: block;
        font-size: 11px;
        color: #8a7e6b;
        margin-bottom: 4px;
      }
      .dashboard-trend-note {
        padding: 12px 18px;
        background: rgba(139,38,53,0.06);
        border-left: 3px solid #8b2635;
        border-radius: 10px;
        font-size: 13px;
        color: #8b2635;
      }
      .dashboard-guidance {
        background: linear-gradient(135deg, #3a2f25 0%, #5c4a38 100%);
        border-radius: 14px;
        padding: 22px 26px;
        margin-bottom: 24px;
        color: #f5f1ea;
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 1.5rem;
      }
      .dashboard-guidance__eyebrow {
        font-size: 10px;
        text-transform: uppercase;
        letter-spacing: 1.4px;
        opacity: 0.5;
        margin-bottom: 6px;
        font-weight: 600;
      }
      .dashboard-guidance h3 {
        margin: 0 0 4px;
        font-family: "Crimson Pro", serif;
        font-size: 20px;
      }
      .dashboard-guidance p {
        margin: 0;
        font-size: 13px;
        opacity: 0.68;
        line-height: 1.5;
        max-width: 500px;
      }
      .dashboard-progress {
        width: 64px;
        height: 64px;
        border-radius: 999px;
        display: flex;
        align-items: center;
        justify-content: center;
        border: 3px solid rgba(245,241,234,0.18);
        font: 700 18px/1 "Crimson Pro", serif;
      }
      .dashboard-checklist {
        display: flex;
        flex-direction: column;
        gap: 10px;
      }
      .dashboard-check-item {
        display: flex;
        gap: 16px;
        align-items: flex-start;
        background: #fff;
        border-radius: 14px;
        border: 1px solid #e8e2d8;
        padding: 18px 20px;
      }
      .dashboard-check-badge {
        width: 26px;
        height: 26px;
        border-radius: 7px;
        border: 2px solid #e8e2d8;
        display: flex;
        align-items: center;
        justify-content: center;
        color: #8a7e6b;
        flex-shrink: 0;
        margin-top: 2px;
      }
      .dashboard-check-title {
        display: flex;
        align-items: center;
        gap: 10px;
        margin-bottom: 4px;
      }
      .dashboard-check-index {
        font-size: 11px;
        color: #b0a694;
        font-weight: 700;
        font-family: "JetBrains Mono", monospace;
      }
      .dashboard-check-item h4 {
        margin: 0;
        font-family: "Crimson Pro", serif;
        font-size: 15px;
      }
      .dashboard-check-item p {
        margin: 0;
        font-size: 13px;
        color: #8a7e6b;
        line-height: 1.4;
      }
      .dashboard-check-addon {
        margin-top: 10px;
      }
      .dashboard-check-input,
      .dashboard-note-input {
        padding: 8px 12px;
        font-size: 14px;
        border: 2px solid #e8e2d8;
        border-radius: 8px;
        background: #fff;
        color: #3a2f25;
        outline: none;
      }
      .dashboard-note-input {
        width: 100%;
        min-height: 84px;
        resize: vertical;
        box-sizing: border-box;
      }
      .dashboard-badge {
        font-size: 11px;
        background: #f9f6f1;
        color: #8a7e6b;
        padding: 2px 10px;
        border-radius: 10px;
        font-weight: 500;
      }
      .dashboard-person-row,
      .dashboard-followup {
        display: flex;
        align-items: flex-start;
        gap: 14px;
        padding: 14px 0;
      }
      .dashboard-person-row + .dashboard-person-row {
        border-top: 1px solid #ede8df;
      }
      .dashboard-avatar {
        width: 38px;
        height: 38px;
        border-radius: 999px;
        background: #f9f6f1;
        display: flex;
        align-items: center;
        justify-content: center;
        font-family: "Crimson Pro", serif;
        font-weight: 700;
        color: #5a5044;
        flex-shrink: 0;
      }
      .dashboard-person-note {
        margin-top: 4px;
        padding: 8px 12px;
        background: #f9f6f1;
        border-radius: 8px;
        font-size: 13px;
        color: #5a5044;
        font-style: italic;
      }
      .dashboard-followup {
        background: #fff;
        border-radius: 10px;
        border: 1px solid #e8e2d8;
        padding: 14px 18px;
      }
      .dashboard-followup--action {
        border-left: 4px solid #8b2635;
      }
      .dashboard-followup--warm {
        border-left: 4px solid #c9943e;
      }
      @keyframes dashboardFadeUp {
        from { opacity: 0; transform: translateY(10px); }
        to { opacity: 1; transform: translateY(0); }
      }
      @media (max-width: 900px) {
        .dashboard-switcher,
        .dashboard-toolbar,
        .dashboard-stat-row,
        .dashboard-payments,
        .dashboard-two-col,
        .dashboard-inventory-stats,
        .dashboard-note-row,
        .dashboard-guidance {
          flex-direction: column;
          align-items: stretch;
        }
        .dashboard-tab-group {
          width: 100%;
          justify-content: stretch;
        }
        .dashboard-tab {
          flex: 1;
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

#[cfg(test)]
mod tests {
    use super::*;
    use bookstore_app::{AdminAuthSession, AdminRole};

    fn admin_session() -> AdminAuthSession {
        AdminAuthSession {
            token: "token-123".to_string(),
            tenant_id: "church-a".to_string(),
            role: AdminRole::Admin,
        }
    }

    #[test]
    fn admin_dashboard_template_renders_all_dashboard_views_and_hooks() {
        let html = AdminDashboardTemplate::new(
            &admin_session(),
            "<div id=\"admin-orders\"></div>".to_string(),
        )
        .render()
        .expect("dashboard should render");

        assert!(html.contains("dashboard-tab is-active"));
        assert!(html.contains("data-dashboard-view=\"sunday\""));
        assert!(html.contains("data-dashboard-view=\"pastoral\""));
        assert!(html.contains("id=\"admin-payment-breakdown\""));
        assert!(html.contains("id=\"admin-products\""));
        assert!(html.contains("id=\"admin-orders\""));
        assert!(html.contains(".dashboard-switcher"));
        assert!(html.contains("window.SCRIPTORIUM_ADMIN_SESSION"));
    }

    #[test]
    fn admin_orders_template_renders_filters_and_orders_placeholder() {
        let html = AdminOrdersTemplate::new(
            &admin_session(),
            "<div id=\"admin-orders\">placeholder</div>".to_string(),
        )
        .render()
        .expect("orders should render");

        assert!(html.contains("Order Management"));
        assert!(html.contains("data-admin-office-tab=\"orders\""));
        assert!(html.contains("data-admin-office-tab=\"inventory\""));
        assert!(html.contains("data-order-filter=\"All\""));
        assert!(html.contains("id=\"admin-orders\""));
        assert!(html.contains("id=\"admin-products-table\""));
        assert!(html.contains("id=\"order-summary-count\""));
        assert!(html.contains("id=\"inventory-total-products\""));
        assert!(html.contains("admin-export-inline"));
    }

    #[test]
    fn admin_intake_template_renders_steps_hidden_fields_and_script() {
        let html =
            AdminIntakeTemplate::new(&admin_session(), "<script>console.log('intake')</script>")
                .render()
                .expect("intake should render");

        assert!(html.contains("id=\"token\""));
        assert!(html.contains("id=\"tenant-id\""));
        assert!(html.contains("intake-step is-active"));
        assert!(html.contains(".admin-main--intake"));
        assert!(html.contains("console.log('intake')"));
    }
}
