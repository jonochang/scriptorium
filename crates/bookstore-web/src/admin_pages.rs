use bookstore_app::AdminAuthSession;

use crate::admin_ui::admin_dashboard_script;
use crate::ui::{google_fonts_link, orders_table_placeholder, shared_styles};

fn admin_session_script(session: &AdminAuthSession) -> String {
    format!(
        r#"<script>window.SCRIPTORIUM_ADMIN_SESSION = {{ token: {token:?}, tenantId: {tenant:?} }};</script>"#,
        token = session.token,
        tenant = session.tenant_id
    )
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

fn admin_topnav(current: &str) -> String {
    let link = |href: &str, label: &str, key: &str| {
        if current == key {
            format!(r#"<span class="is-active">{label}</span>"#)
        } else {
            format!(r#"<a href="{href}">{label}</a>"#)
        }
    };
    [
        r#"<header class="admin-topbar"><div class="admin-brand"><span class="admin-brand-mark">✝</span> SCRIPTORIUM</div><nav class="admin-topnav" aria-label="Admin sections">"#,
        &link("/catalog", "Catalog", "catalog"),
        &link("/cart", "Cart", "cart"),
        &link("/admin", "Dashboard", "dashboard"),
        &link("/admin/orders", "Orders", "orders"),
        &link("/admin/intake", "Intake", "intake"),
        r#"</nav></header>"#,
    ]
    .concat()
}

fn admin_header(
    eyebrow: &str,
    title: &str,
    lede: &str,
    badges: &[&str],
    actions_html: &str,
) -> String {
    let badges_html = badges
        .iter()
        .enumerate()
        .map(|(index, badge)| {
            format!(
                r#"<span class="admin-badge{}">{}</span>"#,
                if index == 0 { " admin-badge--accent" } else { "" },
                badge
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        r#"<section class="admin-header"><div><p class="admin-header__eyebrow">{}</p><h1>{}</h1><p>{}</p></div><div class="admin-header__meta"><div class="admin-badges">{}</div><div class="admin-actions">{}</div></div></section>"#,
        eyebrow, title, lede, badges_html, actions_html
    )
}

fn admin_page_shell(
    page_title: &str,
    current: &str,
    eyebrow: &str,
    title: &str,
    lede: &str,
    badges: &[&str],
    actions_html: &str,
    body_html: &str,
    trailing_html: &str,
) -> String {
    [
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\" /><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />",
        &format!("<title>{page_title}</title>"),
        google_fonts_link(),
        "<style>",
        shared_styles(),
        admin_layout_styles(),
        "</style></head><body class=\"admin-shell\">",
        &admin_topnav(current),
        "<main class=\"admin-main\">",
        &admin_header(eyebrow, title, lede, badges, actions_html),
        "<div class=\"admin-content\">",
        body_html,
        "</div></main>",
        "<footer class=\"admin-footer\"><span>Scriptorium 2026. Parish browsing, intake, and Sunday-close reconciliation.</span><div class=\"admin-footer__links\"><a href=\"/catalog\">Catalog</a><a href=\"/cart\">Cart</a><a href=\"/admin\">Dashboard</a></div></footer>",
        trailing_html,
        "</body></html>",
    ]
    .concat()
}

pub fn admin_login_shell_html(next: &str, message: Option<&str>) -> String {
    let safe_message = message.unwrap_or("Sign in to continue to the admin office.");
    let body_html = format!(
        r#"<section class="dashboard-grid"><article class="surface-card" style="max-width:640px;"><p class="divider-title">Secure access</p><h2 class="section-title">Continue to dashboard</h2><p class="helper-copy helper-copy--flush">Use the parish admin credentials to open reporting, intake, and order management.</p><form id="admin-login-form" class="form-grid" style="margin-top:1rem;"><input type="hidden" id="admin-next" value="{next}" /><div><label class="field-label" for="admin-username">Username</label><input id="admin-username" autocomplete="username" placeholder="Username" /></div><div><label class="field-label" for="admin-password">Password</label><input id="admin-password" type="password" autocomplete="current-password" placeholder="Password" /></div></form><div class="button-row"><button class="primary-button" type="button" id="admin-login">Sign in</button><a class="ghost-link ghost-link--ink" href="/catalog">Cancel</a></div><div id="admin-login-status" class="notice-panel" aria-live="polite">{safe_message}</div></article></section>"#
    );
    let trailing = format!(
        r#"<script>
const loginStatus = document.getElementById('admin-login-status');
const loginButton = document.getElementById('admin-login');
if (loginButton) {{
  loginButton.addEventListener('click', async () => {{
    const username = (document.getElementById('admin-username')?.value || '').trim();
    const password = document.getElementById('admin-password')?.value || '';
    const next = document.getElementById('admin-next')?.value || '/admin';
    if (!username) {{
      loginStatus.textContent = 'Enter your username before signing in.';
      loginStatus.className = 'notice-panel notice-panel--danger';
      return;
    }}
    loginStatus.textContent = 'Signing in...';
    loginStatus.className = 'notice-panel';
    const res = await fetch('/api/admin/auth/login', {{
      method: 'POST',
      headers: {{ 'content-type': 'application/json' }},
      body: JSON.stringify({{ username, password }}),
    }});
    const json = await res.json().catch(() => ({{}}));
    if (!res.ok) {{
      loginStatus.textContent = json.message || 'Login failed.';
      loginStatus.className = 'notice-panel notice-panel--danger';
      return;
    }}
    window.location.assign(next.startsWith('/admin') ? next : '/admin');
  }});
}}
</script>"#
    );
    admin_page_shell(
        "Scriptorium Admin Sign-In",
        "dashboard",
        "Admin Office",
        "Admin Sign-In",
        "Unlock the dashboard, intake, and order follow-up tools from one credentialed entry point.",
        &["Protected access", "Dashboard gate", "Session based"],
        r#"<a class="admin-link" href="/catalog">Back to catalog</a><a class="admin-link" href="/pos">POS</a>"#,
        &body_html,
        &trailing,
    )
}

pub fn admin_dashboard_shell_html(session: &AdminAuthSession) -> String {
    let body_html = format!(
        r#"<section class="dashboard-grid dashboard-grid--three"><article class="surface-card"><p class="divider-title">Reporting window</p><h2 class="section-title">Today's snapshot</h2><div class="form-grid"><div><label class="field-label" for="report-from">From</label><input id="report-from" type="date" value="2026-03-01" /></div><div><label class="field-label" for="report-to">To</label><input id="report-to" type="date" value="2026-03-31" /></div></div><div class="button-row"><button class="accent-button" type="button" id="admin-refresh">Refresh data</button><button class="ghost-link ghost-link--ink" type="button" id="admin-export">Export</button></div><div id="admin-status" class="notice-panel" aria-live="polite">Loading dashboard…</div></article><article class="surface-card"><p class="divider-title">At a glance</p><h2 class="section-title">Today's snapshot</h2><div class="metric-grid"><div class="metric-card metric-card--feature"><div class="metric-icon">💒</div><span class="metric-label">Today's Sales</span><strong id="metric-today-sales">$0.00</strong></div><div class="metric-card"><div class="metric-icon">🛒</div><span class="metric-label">POS Revenue</span><strong id="metric-pos-revenue">$0.00</strong></div><div class="metric-card"><div class="metric-icon">📦</div><span class="metric-label">Online Revenue</span><strong id="metric-online-revenue">$0.00</strong></div><div class="metric-card"><div class="metric-icon">🧾</div><span class="metric-label">Open IOUs</span><strong id="metric-open-ious">0 open</strong></div></div><div id="report-caption" class="helper-copy">Showing the selected reporting window.</div><div class="divider-title divider-title--spaced">Payment breakdown</div><div id="admin-payment-breakdown" class="stack-list stack-list--tight"><div class="empty-inline">Payment method totals will appear here.</div></div></article><article class="surface-card"><p class="divider-title">Pastoral rhythm</p><h2 class="section-title">After-liturgy cadence</h2><div class="pilgrim-panel"><h3>Closing the table</h3><p>Review today’s totals, settle open IOUs, and hand the next volunteer a clearer shelf than the one you inherited.</p></div><div class="divider-title divider-title--spaced">Trend note</div><div id="admin-trend-note" class="notice-panel">Trend notes will appear after the first refresh.</div></article></section><section class="dashboard-grid"><article class="surface-card"><p class="divider-title">Inventory</p><h2 class="section-title">Products</h2><div id="admin-products" class="stack-list"><div class="empty-inline">No products loaded yet.</div></div></article><article class="surface-card"><p class="divider-title">Taxonomy</p><h2 class="section-title">Categories and vendors</h2><div class="taxonomy-wrap"><div><h3 class="subheading">Categories</h3><div id="admin-categories" class="chip-wrap"><span class="chip-muted">Waiting for data</span></div></div><div><h3 class="subheading">Vendors</h3><div id="admin-vendors" class="chip-wrap"><span class="chip-muted">Waiting for data</span></div></div></div></article></section><section class="dashboard-grid"><article class="surface-card"><div class="button-row button-row--compact" style="justify-content:space-between;margin-top:0;"><div><p class="divider-title">Orders</p><h2 class="section-title" style="margin:0;">Recent orders</h2></div><a class="ghost-link ghost-link--ink" href="/admin/orders">Open full page</a></div><div class="toolbar-chips"><button class="filter-chip filter-chip--active" type="button" data-order-filter="All">All</button><button class="filter-chip" type="button" data-order-filter="POS">POS</button><button class="filter-chip" type="button" data-order-filter="Online">Online</button><button class="filter-chip" type="button" data-order-filter="IOU" id="order-filter-iou-label">IOU (0)</button></div><div id="admin-orders">{}</div></article><article class="surface-card"><p class="divider-title">Attention queue</p><h2 class="section-title">Open IOUs</h2><div id="admin-ious" class="stack-list"><div class="empty-inline">No open IOUs.</div></div><div class="divider-title divider-title--spaced">Low stock spotlight</div><div id="admin-low-stock" class="stack-list"><div class="empty-inline">Low-stock titles will appear here.</div></div></article></section><section class="dashboard-grid"><article class="surface-card"><p class="divider-title">Stock movement</p><h2 class="section-title">Inventory journal</h2><div id="admin-journal" class="stack-list"><div class="empty-inline">Inventory movement will appear here after login.</div></div></article><article class="surface-card"><p class="divider-title">Next steps</p><h2 class="section-title">Readiness actions</h2><div class="stack-list"><div class="list-row list-row--soft"><div><div class="list-title">Order management</div><div class="list-meta">Open the dedicated order view for filtering, exports, and follow-up.</div></div><a class="ghost-link ghost-link--ink ghost-link--mini" href="/admin/orders">Open</a></div><div class="list-row list-row--soft"><div><div class="list-title">Receive new stock</div><div class="list-meta">Move into intake to fetch metadata, price a title, and prepare it for the shelf.</div></div><a class="ghost-link ghost-link--ink ghost-link--mini" href="/admin/intake">Open</a></div></div></article></section>"#,
        orders_table_placeholder("No orders loaded yet.")
    );
    admin_page_shell(
        "Scriptorium Admin",
        "dashboard",
        "Admin Office",
        "Good morning, Father Michael",
        "Reconcile takings, watch the shelves, and settle unpaid tabs before the parish hall empties.",
        &["Treasurer view", "Sunday close", "Pastoral follow-up"],
        r#"<a class="admin-link" href="/admin/orders">Order management</a><a class="admin-link admin-link--accent" href="/admin/intake">Add product</a><a class="admin-link" href="/admin/logout">Sign out</a>"#,
        &body_html,
        &[admin_session_script(session), admin_dashboard_script().to_string()].concat(),
    )
}

pub fn admin_orders_shell_html(session: &AdminAuthSession) -> String {
    let body_html = format!(
        r#"<section class="dashboard-grid dashboard-grid--three"><article class="surface-card"><p class="divider-title">Reporting window</p><h2 class="section-title">Order filters</h2><div class="form-grid"><div><label class="field-label" for="report-from">From</label><input id="report-from" type="date" value="2026-03-01" /></div><div><label class="field-label" for="report-to">To</label><input id="report-to" type="date" value="2026-03-31" /></div></div><div class="button-row"><button class="accent-button" type="button" id="admin-refresh">Refresh data</button><button class="ghost-link ghost-link--ink" type="button" id="admin-export">Export</button></div><div id="admin-status" class="notice-panel" aria-live="polite">Loading orders…</div></article><article class="surface-card" style="grid-column: span 2;"><div class="button-row button-row--compact" style="justify-content:space-between;margin-top:0;"><div><p class="divider-title">Orders</p><h2 class="section-title" style="margin:0;">Order Management</h2></div><button class="ghost-link ghost-link--ink" type="button" id="admin-export-inline">Export</button></div><div class="toolbar-chips"><button class="filter-chip filter-chip--active" type="button" data-order-filter="All">All</button><button class="filter-chip" type="button" data-order-filter="POS">POS</button><button class="filter-chip" type="button" data-order-filter="Online">Online</button><button class="filter-chip" type="button" data-order-filter="IOU" id="order-filter-iou-label">IOU (0)</button></div><div id="admin-orders">{}</div><div class="pagination"><span class="helper-copy helper-copy--flush">Page 1 of 1</span><div class="pagination-links"><a class="pagination-link pagination-link--active" href="/admin/orders">1</a></div></div></article></section>"#,
        orders_table_placeholder("No orders loaded yet.")
    );
    admin_page_shell(
        "Scriptorium Order Management",
        "orders",
        "Admin Office",
        "Order Management",
        "Track paid orders, open tabs, and follow-up actions from one dedicated table.",
        &["Dedicated orders page", "Export-ready", "IOU follow-up"],
        r#"<a class="admin-link" href="/admin">Dashboard</a><a class="admin-link admin-link--accent" href="/admin/intake">Add product</a><a class="admin-link" href="/admin/logout">Sign out</a>"#,
        &body_html,
        &[admin_session_script(session), admin_dashboard_script().to_string()].concat(),
    )
}
