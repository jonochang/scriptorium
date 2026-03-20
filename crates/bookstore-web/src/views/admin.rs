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
        background: #f0ebe2;
      }
      .admin-topbar {
        background: #3a2f25;
        color: #f5f1ea;
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 1rem;
        padding: 0 32px;
        height: 52px;
      }
      .admin-brand {
        display: flex;
        align-items: center;
        gap: 10px;
        font-family: "Source Serif 4", Georgia, serif;
        font-size: 16px;
        font-weight: 700;
        letter-spacing: 0.5px;
        color: #f5f1ea;
        text-decoration: none;
      }
      .admin-brand-mark { font-size: 18px; }
      .admin-topnav {
        display: flex;
        flex-wrap: wrap;
        align-items: center;
        gap: 6px;
      }
      .admin-topnav a,
      .admin-topnav span {
        color: rgba(245, 241, 234, 0.5);
        text-decoration: none;
        font-weight: 500;
        font-size: 14px;
        font-family: "Source Sans 3", "Segoe UI", system-ui, sans-serif;
        padding: 5px 14px;
        border-radius: 20px;
        border: none;
        background: transparent;
      }
      .admin-topnav .is-active {
        color: #f5f1ea;
        font-weight: 600;
        background: rgba(245,241,234,0.13);
      }
      .admin-main {
        max-width: 1060px;
        margin: 0 auto;
        padding: 0 24px 40px;
      }
      .admin-header {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        gap: 1.5rem;
        padding: 32px 0 20px;
        border-bottom: 1px solid #e0d9cd;
        margin-bottom: 24px;
      }
      .admin-header__eyebrow {
        margin: 0 0 0.3rem;
        font-size: 12px;
        font-weight: 600;
        letter-spacing: 1.4px;
        text-transform: uppercase;
        color: #8a7e6b;
      }
      .admin-header h1 {
        margin: 0 0 6px;
        font-family: "Source Serif 4", Georgia, serif;
        font-size: 26px;
        font-weight: 700;
        line-height: 1.1;
        color: #3a2f25;
      }
      .admin-header p {
        margin: 0;
        max-width: 40rem;
        font-size: 14px;
        color: #8a7e6b;
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
        padding: 3px 10px;
        border-radius: 20px;
        background: #f5f1ea;
        border: 1px solid #e0d9cd;
        color: #8a7e6b;
        font-size: 13px;
        font-weight: 600;
      }
      .admin-badge--accent {
        background: #f9f0f2;
        border-color: rgba(107, 28, 42, 0.18);
        color: #6b1c2a;
      }
      .admin-actions {
        display: flex;
        gap: 8px;
        flex-wrap: wrap;
        justify-content: flex-end;
      }
      .admin-link {
        display: inline-flex;
        align-items: center;
        padding: 8px 18px;
        border-radius: 8px;
        border: 1px solid #e0d9cd;
        background: transparent;
        color: #6b5e4f;
        text-decoration: none;
        font-size: 14px;
        font-weight: 500;
        font-family: "Source Sans 3", "Segoe UI", system-ui, sans-serif;
        gap: 6px;
      }
      .admin-link--accent {
        border-color: #6b1c2a;
        background: #6b1c2a;
        color: #fff;
        font-weight: 600;
      }
      .admin-content {
        display: grid;
        gap: 18px;
      }
      .admin-footer {
        max-width: 1060px;
        margin: 0 auto;
        padding: 20px 24px;
        border-top: 1px solid #e0d9cd;
        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: 1rem;
        color: #a89e8e;
        font-size: 13px;
      }
      .admin-footer__links {
        display: flex;
        gap: 16px;
      }
      .admin-footer a {
        color: inherit;
        text-decoration: none;
        font-weight: 500;
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
          height: auto;
        }
        .admin-main {
          padding: 0 1rem 2.5rem;
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
        border-radius: 8px;
        background: transparent;
        color: #6b5e4f;
        font-family: "Source Sans 3", "Segoe UI", system-ui, sans-serif;
        cursor: pointer;
        transition: all 0.15s;
      }
      .office-tab {
        padding: 7px 18px;
        font-size: 14px;
        font-weight: 400;
        white-space: nowrap;
      }
      .office-tab.is-active {
        background: #3a2f25;
        color: #fff;
        font-weight: 600;
      }
      .office-chip--active {
        background: #3a2f25;
        color: #fff;
        font-weight: 600;
      }
      .office-chip {
        padding: 5px 14px;
        font-size: 13px;
        font-weight: 400;
        white-space: nowrap;
        border: 1px solid #e0d9cd;
        border-radius: 20px;
      }
      .office-chip--active {
        border-color: #3a2f25;
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
        flex: 1 1 280px;
        min-width: 0;
        padding: 18px 20px;
        border-radius: 10px;
        border: 1px solid #ede8df;
        background: #fff;
      }
      .office-metric--dark {
        background: #3a2f25;
        border-color: #3a2f25;
        color: #f5f1ea;
      }
      .office-metric--warn {
        background: #fff;
        border-color: #ede8df;
      }
      .office-metric--danger {
        background: #fff;
        border-color: #ede8df;
      }
      .office-metric__label {
        display: block;
        margin-bottom: 6px;
        font-size: 12px;
        font-weight: 600;
        letter-spacing: 1.2px;
        text-transform: uppercase;
        color: #8a7e6b;
      }
      .office-metric--dark .office-metric__label {
        color: rgba(245,241,234,0.6);
      }
      .office-metric__value {
        font-family: "Source Serif 4", Georgia, serif;
        font-size: 22px;
        font-weight: 700;
        color: #3a2f25;
      }
      .office-metric--dark .office-metric__value {
        color: #f5f1ea;
      }
      .office-metric__value--green {
        color: #3a2f25;
      }
      .office-metric__value--amber {
        color: #8b6914;
      }
      .office-metric__value--accent {
        color: #9e2b2b;
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
        border: 1px solid #e0d9cd;
        border-radius: 8px;
        background: #f5f1ea;
        color: #3a2f25;
        font-family: "Source Sans 3", "Segoe UI", system-ui, sans-serif;
        font-size: 14px;
      }
      .office-search input {
        width: auto;
        min-width: 200px;
        padding: 9px 14px 9px 2rem;
      }
      .office-date {
        width: auto;
        padding: 8px 12px;
        font-family: "Source Sans 3", "Segoe UI", system-ui, sans-serif;
        font-size: 14px;
      }
      .office-table-card {
        padding: 24px 28px;
        border: 1px solid #ede8df;
        border-radius: 12px;
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
        padding: 3px 10px;
        border-radius: 20px;
        background: #f5f1ea;
        border: 1px solid #e0d9cd;
        color: #8a7e6b;
        font-size: 13px;
        font-weight: 600;
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
        border: 1px solid #e0d9cd;
        border-radius: 8px;
        background: transparent;
        font-size: 14px;
        line-height: 1;
        cursor: pointer;
        color: #6b5e4f;
      }
      .office-stock-value {
        min-width: 20px;
        text-align: center;
        font-weight: 600;
      }
      .office-stock-note {
        margin-left: 0.25rem;
        color: #b0a694;
        font-size: 0.72rem;
      }
      .office-inline-badge {
        display: inline-flex;
        align-items: center;
        padding: 3px 10px;
        border-radius: 20px;
        font-size: 13px;
        font-weight: 600;
      }
      .office-inline-badge--ok {
        background: #eaf5ee;
        color: #2d6b3f;
        border: 1px solid #b4dbc3;
      }
      .office-inline-badge--low {
        background: #faf3dc;
        color: #8b6914;
        border: 1px solid #e8d99b;
      }
      .office-inline-badge--out {
        background: #fdeaea;
        color: #9e2b2b;
        border: 1px solid #f0c0c0;
      }
      .office-product-meta {
        margin-top: 0.15rem;
        color: #8a7e6b;
        font-size: 0.76rem;
      }
      .order-detail-row td {
        padding: 0 !important;
        border-top: none !important;
      }
      .order-detail {
        background: #faf8f4;
        border-top: 1px solid #e8e2d8;
        border-bottom: 2px solid #e8e2d8;
        padding: 1.1rem 1.4rem;
        animation: dashboardFadeUp 0.18s ease both;
      }
      .order-detail__grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(10rem, 1fr));
        gap: 0.9rem 1.6rem;
      }
      .order-detail__label {
        display: block;
        font-size: 0.72rem;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.04em;
        color: #8a7e6b;
        margin-bottom: 0.2rem;
      }
      .order-detail__value {
        font-size: 0.92rem;
        color: #3a2f25;
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

fn dashboard_extra_styles() -> &'static str {
    r#"
      .admin-main--dashboard {
        max-width: 1060px;
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
      }
      .dashboard-tabs {
        display: flex;
        align-items: center;
        gap: 8px;
        flex-wrap: wrap;
      }
      .dashboard-tab-group {
        display: flex;
        background: #ede8df;
        border-radius: 10px;
        padding: 3px;
        gap: 2px;
        width: fit-content;
      }
      .dashboard-tab {
        padding: 7px 18px;
        font-size: 14px;
        font-weight: 400;
        color: #6b5e4f;
        background: transparent;
        border: none;
        border-radius: 8px;
        cursor: pointer;
        font-family: "Source Sans 3", "Segoe UI", system-ui, sans-serif;
        transition: all 0.15s;
        white-space: nowrap;
      }
      .dashboard-tab.is-active {
        color: #fff;
        background: #3a2f25;
        font-weight: 600;
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
        gap: 12px;
      }
      .dashboard-toolbar {
        align-items: center;
        flex-wrap: wrap;
        margin-bottom: 20px;
      }
      .dashboard-date {
        width: auto;
        padding: 8px 12px;
        font-size: 14px;
        font-family: "Source Sans 3", "Segoe UI", system-ui, sans-serif;
        border: 1px solid #e0d9cd;
        border-radius: 8px;
        background: #f5f1ea;
        color: #3a2f25;
        outline: none;
      }
      .dashboard-stat {
        flex: 1 1 0;
        min-width: 0;
        padding: 18px 20px;
        background: #fff;
        border-radius: 10px;
        border: 1px solid #ede8df;
      }
      .dashboard-stat--accent {
        background: #3a2f25;
        border-color: #3a2f25;
      }
      .dashboard-stat__label {
        font-size: 12px;
        text-transform: uppercase;
        letter-spacing: 1.2px;
        color: #8a7e6b;
        font-weight: 600;
        margin-bottom: 6px;
      }
      .dashboard-stat--accent .dashboard-stat__label {
        color: rgba(245,241,234,0.6);
      }
      .dashboard-stat__value {
        font-size: 22px;
        font-weight: 700;
        font-family: "Source Serif 4", Georgia, serif;
        color: #3a2f25;
      }
      .dashboard-stat--accent .dashboard-stat__value {
        color: #f5f1ea;
      }
      .dashboard-payment-card,
      .dashboard-mini-stat {
        flex: 1;
        padding: 14px 16px;
        background: #f5f1ea;
        border-radius: 8px;
        border: 1px solid #ede8df;
        text-align: center;
      }
      .dashboard-payment-card strong,
      .dashboard-mini-stat strong {
        display: block;
        font-size: 18px;
        font-family: "Source Serif 4", Georgia, serif;
        font-weight: 700;
        color: #3a2f25;
      }
      .dashboard-payment-card span,
      .dashboard-mini-stat span {
        display: block;
        font-size: 13px;
        color: #8a7e6b;
        margin-bottom: 4px;
      }
      .dashboard-trend-note {
        padding: 12px 20px;
        background: #faf3dc;
        border-radius: 8px;
        border: 1px solid #e8d99b;
        font-size: 14px;
        color: #8b6914;
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
        font-size: 12px;
        text-transform: uppercase;
        letter-spacing: 1.4px;
        opacity: 0.5;
        margin-bottom: 6px;
        font-weight: 600;
      }
      .dashboard-guidance h3 {
        margin: 0 0 4px;
        font-family: "Source Serif 4", Georgia, serif;
        font-size: 20px;
      }
      .dashboard-guidance p {
        margin: 0;
        font-size: 14px;
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
        font: 700 18px/1 "Source Serif 4", Georgia, serif;
      }
      .dashboard-checklist {
        display: flex;
        flex-direction: column;
        gap: 10px;
      }
      .dashboard-check-item {
        display: flex;
        gap: 12px;
        align-items: center;
        padding: 14px 0;
        border-bottom: 1px solid #ede8df;
        cursor: pointer;
      }
      .dashboard-check-item:last-child { border-bottom: none; }
      .dashboard-check-badge {
        width: 18px;
        height: 18px;
        border-radius: 4px;
        flex-shrink: 0;
        accent-color: #6b1c2a;
      }
      .dashboard-check-title {
        display: flex;
        align-items: center;
        gap: 10px;
      }
      .dashboard-check-item h4,
      .dashboard-check-item span {
        margin: 0;
        font-size: 14px;
        color: #3a2f25;
        font-weight: 400;
      }
      .dashboard-check-item p {
        margin: 0;
        font-size: 14px;
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
        border: 1px solid #e0d9cd;
        border-radius: 8px;
        background: #f5f1ea;
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
        font-size: 13px;
        background: #f5f1ea;
        color: #8a7e6b;
        padding: 2px 10px;
        border-radius: 10px;
        font-weight: 600;
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
        background: #f5f1ea;
        display: flex;
        align-items: center;
        justify-content: center;
        font-family: "Source Serif 4", Georgia, serif;
        font-weight: 700;
        color: #5a5044;
        flex-shrink: 0;
      }
      .dashboard-person-note {
        margin-top: 4px;
        padding: 8px 12px;
        background: #f5f1ea;
        border-radius: 8px;
        font-size: 14px;
        color: #5a5044;
        font-style: italic;
      }
      .dashboard-followup {
        background: #fff;
        border-radius: 10px;
        border: 1px solid #ede8df;
        padding: 14px 18px;
      }
      .dashboard-followup--action {
        border-left: 4px solid #6b1c2a;
      }
      .dashboard-followup--warm {
        border-left: 4px solid #8b6914;
      }
      .office-chip-group {
        display: inline-flex;
        align-items: center;
        gap: 4px;
      }
      .office-chip {
        padding: 5px 14px;
        font-size: 13px;
        font-weight: 400;
        white-space: nowrap;
        border: 1px solid #e0d9cd;
        border-radius: 20px;
        background: transparent;
        color: #6b5e4f;
        cursor: pointer;
        font-family: "Source Sans 3", "Segoe UI", system-ui, sans-serif;
      }
      .office-chip--active {
        background: #3a2f25;
        color: #fff;
        font-weight: 600;
        border-color: #3a2f25;
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

}
