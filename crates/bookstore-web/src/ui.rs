pub fn site_nav(current: &str) -> String {
    let nav_link = |href: &str, label: &str, key: &str| {
        if current == key {
            format!("<span class=\"is-active\">{}</span>", label)
        } else {
            format!("<a href=\"{}\">{}</a>", href, label)
        }
    };
    let cart_link = if current == "cart" {
        "<span class=\"is-active\">Cart <span id=\"site-cart-count\" class=\"storefront-topnav__count\">0</span></span>"
            .to_string()
    } else {
        "<a href=\"/cart\">Cart <span id=\"site-cart-count\" class=\"storefront-topnav__count\">0</span></a>"
            .to_string()
    };
    [
        "<header class=\"admin-topbar storefront-topbar\"><a class=\"admin-brand storefront-brand\" href=\"/catalog\"><span class=\"admin-brand-mark storefront-brand-mark\">☦</span><span>Scriptorium</span></a><nav class=\"admin-topnav storefront-topnav\" aria-label=\"Primary\">",
        &nav_link("/catalog", "Catalog", "catalog"),
        &cart_link,
        &nav_link("/checkout", "Checkout", "checkout"),
        &nav_link("/admin", "Dashboard", "dashboard"),
        &nav_link("/admin/orders", "Orders", "orders"),
        &nav_link("/admin/intake", "Intake", "intake"),
        "</nav></header>",
    ]
    .concat()
}

pub fn site_footer() -> &'static str {
    "<footer class=\"admin-footer storefront-footer\"><span>Scriptorium 2026. Parish browsing, intake, and Sunday-close reconciliation.</span><div class=\"admin-footer__links storefront-footer__links\"><a href=\"/catalog\">Catalog</a><a href=\"/cart\">Cart</a><a href=\"/checkout\">Checkout</a><a href=\"/admin\">Dashboard</a><a href=\"/admin/orders\">Orders</a><a href=\"/admin/intake\">Intake</a></div></footer>"
}

pub fn page_header(
    eyebrow: &str,
    title: &str,
    lede: &str,
    badges: &[&str],
    actions_html: &str,
) -> String {
    let badges_html = if badges.is_empty() {
        String::new()
    } else {
        let chips = badges
            .iter()
            .enumerate()
            .map(|(index, badge)| {
                format!(
                    "<span class=\"page-badge{}\">{}</span>",
                    if index == 0 { " page-badge--accent" } else { "" },
                    html_escape(badge)
                )
            })
            .collect::<Vec<_>>()
            .join("");
        format!("<div class=\"page-header__badges\">{chips}</div>")
    };
    format!(
        "<section class=\"page-header admin-header storefront-header\"><div class=\"page-header__content storefront-header__content\"><p class=\"page-header__eyebrow admin-header__eyebrow\">{}</p><h1 class=\"page-header__title\">{}</h1><p class=\"page-header__lede\">{}</p>{}</div><div class=\"page-header__actions storefront-header__meta\">{}</div></section>",
        html_escape(eyebrow),
        html_escape(title),
        html_escape(lede),
        badges_html,
        actions_html,
    )
}

pub fn orders_table_placeholder(message: &str) -> String {
    format!(
        "<div id=\"admin-orders\" class=\"orders-table-wrap\"><table class=\"orders-table\"><thead><tr><th>Order ID</th><th>Date</th><th>Channel</th><th>Customer</th><th>Status</th><th>Method</th><th>Total</th><th>Actions</th></tr></thead><tbody><tr><td colspan=\"8\"><div class=\"empty-inline\">{}</div></td></tr></tbody></table></div>",
        html_escape(message)
    )
}

pub fn google_fonts_link() -> &'static str {
    r#"<link href="https://fonts.googleapis.com/css2?family=Source+Serif+4:opsz,wght@8..60,400;8..60,600;8..60,700&family=Source+Sans+3:wght@400;500;600&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet">"#
}

pub fn shared_styles() -> &'static str {
    r#"
      :root {
        --wine: #6b1c2a;
        --wine-light: #7d2234;
        --wine-dark: #4a1525;
        --wine-muted: #8a7e6b;
        --gold: #8b6914;
        --gold-light: #e8d99b;
        --gold-pale: #faf3dc;
        --parchment: #f5f1ea;
        --parchment-dark: #ede8df;
        --filled: #f0ebe2;
        --filled-border: #e0d9cd;
        --ink: #3a2f25;
        --ink-light: #6b5e4f;
        --warm-gray: #8a7e6b;
        --text-faint: #a89e8e;
        --success: #2d6b3f;
        --success-light: #eaf5ee;
        --success-border: #b4dbc3;
        --warning: #8b6914;
        --warning-light: #faf3dc;
        --danger: #9e2b2b;
        --danger-light: #fdeaea;
        --blue: #5A7A9B;
        --blue-light: #ECF1F5;
        --accent-light: #f9f0f2;
        --radius-sm: 8px;
        --radius: 12px;
        --radius-lg: 16px;
        --shadow: 0 1px 3px rgba(58,47,37,0.04), 0 4px 16px rgba(58,47,37,0.03);
        --shadow-lg: 0 8px 32px rgba(58,47,37,0.10);
      }
      * { margin: 0; padding: 0; box-sizing: border-box; }
      body {
        margin: 0;
        background: var(--filled);
        color: var(--ink);
        font-family: "Source Sans 3", "Segoe UI", system-ui, sans-serif;
      }
      input::placeholder { color: #b0a898; }
      input:focus { outline: none; border-color: var(--wine) !important; box-shadow: 0 0 0 3px rgba(107,28,42,0.08); }
      .page-shell { min-height: 100vh; }
      .page-stack { max-width: 960px; margin: 0 auto; padding: 0 24px 40px; display: grid; gap: 18px; }
      .page-stack--wide { max-width: 1060px; }
      .admin-topbar,
      .storefront-topbar {
        display: flex;
        gap: 16px;
        align-items: center;
        justify-content: space-between;
        padding: 0 32px;
        height: 52px;
        background: #3a2f25;
        color: #f5f1ea;
      }
      .admin-brand,
      .storefront-brand {
        display: inline-flex;
        align-items: center;
        gap: 10px;
        color: #f5f1ea;
        text-decoration: none;
        font: 700 1rem/1 "Source Serif 4", Georgia, serif;
        letter-spacing: 0.5px;
        text-transform: uppercase;
      }
      .admin-brand-mark,
      .storefront-brand-mark {
        font-size: 1.1rem;
      }
      .admin-badge-label {
        font-size: 10px;
        font-weight: 600;
        letter-spacing: 1px;
        background: rgba(245,241,234,0.12);
        padding: 3px 10px;
        border-radius: 12px;
        margin-left: 4px;
      }
      .admin-topnav,
      .admin-footer__links,
      .storefront-topnav,
      .storefront-footer__links {
        display: flex;
        gap: 6px;
        flex-wrap: wrap;
        align-items: center;
      }
      .admin-topnav a,
      .admin-topnav span {
        display: inline-flex;
        align-items: center;
        gap: 5px;
        padding: 5px 14px;
        border-radius: 20px;
        color: rgba(245, 241, 234, 0.5);
        text-decoration: none;
        font-weight: 500;
        font-size: 13px;
        font-family: "Source Sans 3", "Segoe UI", system-ui, sans-serif;
        border: none;
        background: transparent;
      }
      .admin-topnav .is-active {
        color: #f5f1ea;
        font-weight: 600;
        background: rgba(245,241,234,0.13);
      }
      .admin-topnav__separator {
        width: 1px;
        height: 20px;
        background: rgba(245,241,234,0.15);
        margin: 0 8px;
      }
      .admin-topnav__secondary {
        font-size: 12px;
        font-weight: 500;
        color: rgba(245,241,234,0.4);
        background: none;
        border: none;
        cursor: pointer;
        padding: 5px 8px;
        text-decoration: none;
        font-family: "Source Sans 3", "Segoe UI", system-ui, sans-serif;
      }
      .storefront-topnav__count {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        padding: 1px 6px;
        border-radius: 10px;
        background: var(--wine);
        color: #fff;
        font-size: 10px;
        font-weight: 700;
      }
      .site-footer,
      .admin-footer,
      .storefront-footer {
        max-width: 1060px;
        margin: 0 auto;
      }
      .admin-footer,
      .storefront-footer {
        padding: 20px 24px;
        border-top: 1px solid var(--filled-border);
        color: var(--text-faint);
        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: 0.75rem;
        font-size: 12px;
        flex-wrap: wrap;
      }
      .admin-footer a,
      .storefront-footer a {
        color: inherit;
        text-decoration: none;
        font-weight: 500;
        cursor: pointer;
      }
      .admin-footer__links,
      .storefront-footer__links {
        gap: 16px;
      }
      .surface-card {
        background: #ffffff;
        border: 1px solid var(--parchment-dark);
        border-radius: var(--radius);
        box-shadow: none;
      }
      .page-header {
        max-width: 960px;
        margin: 0 auto;
        padding: 40px 24px 32px;
        display: flex;
        gap: 1.5rem;
        align-items: flex-start;
        justify-content: space-between;
      }
      .page-header--centered { text-align: center; justify-content: center; }
      .page-header__content {
        display: grid;
        gap: 6px;
      }
      .admin-header__eyebrow {
        margin: 0 0 0.3rem;
        font-size: 10px;
        font-weight: 600;
        letter-spacing: 1.4px;
        text-transform: uppercase;
        color: var(--warm-gray);
      }
      .surface-card { padding: 24px 28px; }
      .display-title,
      .page-header__title {
        margin: 0;
        font: 700 30px/1.1 "Source Serif 4", Georgia, serif;
        color: var(--ink);
      }
      .eyebrow,
      .page-header__eyebrow {
        margin: 0 0 8px;
        font-size: 0.78rem;
        letter-spacing: 0.18em;
        text-transform: uppercase;
        color: var(--wine-muted);
      }
      .lede,
      .page-header__lede {
        margin: 0;
        color: var(--ink-light);
        max-width: 60ch;
        line-height: 1.6;
      }
      .hero-actions,
      .eyebrow-row,
      .page-header__actions,
      .page-header__badges {
        display: flex;
        flex-wrap: wrap;
        gap: 10px;
        align-items: center;
      }
      .storefront-header__meta {
        flex-direction: column;
        align-items: flex-end;
      }
      .eyebrow-row,
      .page-header__badges { margin-top: 8px; }
      .hero-chip {
        display: inline-flex;
        align-items: center;
        min-height: 34px;
        padding: 0 12px;
        border-radius: 999px;
        color: white;
        background: rgba(255,255,255,0.1);
        border: 1px solid rgba(255,255,255,0.12);
        font-size: 0.85rem;
        font-weight: 600;
      }
      .hero-chip--gold {
        color: var(--wine-dark);
        background: var(--gold-pale);
        border-color: rgba(204,170,94,0.3);
      }
      .admin-badge,
      .page-badge {
        display: inline-flex;
        align-items: center;
        padding: 3px 10px;
        border-radius: 20px;
        background: var(--parchment);
        border: 1px solid var(--filled-border);
        color: var(--warm-gray);
        font-size: 11px;
        font-weight: 600;
      }
      .admin-badge--accent,
      .page-badge--accent {
        background: var(--accent-light);
        border-color: rgba(107, 28, 42, 0.18);
        color: var(--wine);
      }
      .ghost-link,
      .primary-button,
      .accent-button {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        padding: 8px 18px;
        border-radius: 8px;
        border: 0;
        text-decoration: none;
        font: 500 13px/1 "Source Sans 3", "Segoe UI", system-ui, sans-serif;
        cursor: pointer;
        transition: all 0.15s;
        gap: 6px;
      }
      .ghost-link {
        color: white;
        background: rgba(255,255,255,0.08);
        border: 1px solid rgba(255,255,255,0.16);
      }
      .ghost-link--ink {
        color: var(--ink-light);
        background: transparent;
        border: 1px solid var(--filled-border);
      }
      .ghost-link--mini {
        padding: 5px 14px;
        font-size: 12px;
        font-weight: 500;
      }
      .primary-button { color: white; background: var(--wine); border: 1px solid var(--wine); font-weight: 600; }
      .primary-button--sm { padding: 5px 14px; font-size: 12px; }
      .accent-button { color: white; background: var(--wine); border: 1px solid var(--wine); font-weight: 600; }
      .dark-button { color: var(--parchment); background: var(--ink); border: none; font-weight: 600; }
      .gold-button { color: white; background: var(--gold); border: 1px solid var(--gold); font-weight: 600; }
      .field-label {
        display: block;
        margin: 0 0 8px;
        font-size: 13px;
        font-weight: 600;
        color: var(--ink);
      }
      input, textarea, select {
        width: 100%;
        padding: 10px 14px;
        border-radius: 8px;
        border: 1px solid var(--filled-border);
        background: var(--parchment);
        color: var(--ink);
        font: 400 14px/1.4 "Source Sans 3", "Segoe UI", system-ui, sans-serif;
      }
      table { font-family: "Source Sans 3", "Segoe UI", system-ui, sans-serif; }
      .catalog-search { display: grid; gap: 10px; margin-bottom: 14px; }
      .form-grid {
        display: grid;
        gap: 12px;
        grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
      }
      .category-strip {
        display: flex;
        flex-wrap: wrap;
        gap: 4px;
        margin-bottom: 16px;
      }
      .category-chip {
        display: inline-flex;
        align-items: center;
        gap: 6px;
        padding: 5px 14px;
        border-radius: 20px;
        text-decoration: none;
        color: var(--ink-light);
        background: transparent;
        border: 1px solid var(--filled-border);
        font-size: 12px;
        font-weight: 400;
        font-family: "Source Sans 3", "Segoe UI", system-ui, sans-serif;
        cursor: pointer;
      }
      .category-chip span {
        font-size: 12px;
      }
      .category-chip--active {
        color: white;
        font-weight: 600;
        background: var(--ink);
        border-color: var(--ink);
      }
      .category-chip--active span {
        color: rgba(255,255,255,0.88);
      }
      .catalog-results-head {
        display: flex;
        align-items: center;
        justify-content: flex-end;
        gap: 12px;
        margin-bottom: 12px;
      }
      .catalog-search-row { display: grid; gap: 10px; grid-template-columns: minmax(0, 1fr) auto; }
      .catalog-grid {
        display: grid;
        gap: 16px;
        grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
      }
      .catalog-card {
        padding: 0;
        border-radius: var(--radius);
        border: 1px solid var(--parchment-dark);
        background: #ffffff;
        overflow: hidden;
      }
      .catalog-card__link {
        display: block;
        color: inherit;
        text-decoration: none;
        cursor: pointer;
      }
      .catalog-card__body {
        padding: 14px 16px 18px;
      }
      .catalog-kicker {
        font-size: 11px;
        color: var(--warm-gray);
        margin-bottom: 4px;
      }
      .catalog-note {
        font-size: 12px;
        color: var(--warm-gray);
        line-height: 1.5;
        margin-bottom: 14px;
      }
      .catalog-cover {
        height: 140px;
        background: linear-gradient(135deg, rgba(139,105,20,0.2), rgba(107,28,42,0.13));
        position: relative;
        display: flex;
        align-items: end;
        padding: 10px;
        color: white;
      }
      .catalog-cover--detail {
        height: 440px;
        background: linear-gradient(145deg, rgba(139,105,20,0.27), rgba(107,28,42,0.2));
        align-items: flex-end;
        padding: 32px;
      }
      .book-cover-art {
        display: grid;
        gap: 6px;
        max-width: 280px;
        padding: 20px 24px;
        border-radius: 10px;
        background: rgba(58,47,37,0.45);
        backdrop-filter: blur(4px);
        text-align: left;
      }
      .book-cover-art strong {
        font: 700 20px/1.25 "Source Serif 4", Georgia, serif;
        color: var(--parchment);
      }
      .book-cover-art span {
        font-size: 13px;
        color: rgba(245,241,234,0.75);
      }
      .book-cover-art__eyebrow {
        font-size: 9px;
        font-weight: 600;
        letter-spacing: 1.4px;
        text-transform: uppercase;
        color: rgba(245,241,234,0.6);
        margin-bottom: 2px;
      }
      .catalog-tags,
      .button-row,
      .toolbar-chips,
      .chip-wrap {
        display: flex;
        gap: 8px;
        flex-wrap: wrap;
        align-items: center;
      }
      .button-row--compact { gap: 6px; }
      .surface-card > .button-row { margin-top: 14px; }
      .surface-card > .notice-panel { margin-top: 12px; }
      .button-row--flush-start { align-items: end; }
      .catalog-tag,
      .chip,
      .chip-muted,
      .status-badge {
        display: inline-flex;
        align-items: center;
        padding: 3px 10px;
        border-radius: 20px;
        font-size: 11px;
        font-weight: 600;
      }
      .catalog-tag,
      .chip {
        background: var(--gold-pale);
        color: var(--ink-light);
        border: 1px solid var(--parchment-dark);
      }
      .chip-muted {
        background: var(--parchment);
        color: var(--warm-gray);
        border: 1px solid var(--filled-border);
      }
      .catalog-tag--muted {
        background: white;
        color: var(--warm-gray);
        border: 1px solid var(--parchment-dark);
      }
      .stock-badge {
        display: inline-flex;
        align-items: center;
        padding: 3px 10px;
        border-radius: 20px;
        font-size: 11px;
        font-weight: 600;
      }
      .stock-badge--success { color: var(--success); background: var(--success-light); border: 1px solid var(--success-border); }
      .stock-badge--warning { color: var(--gold); background: var(--gold-pale); border: 1px solid var(--gold-light); }
      .stock-badge--danger { color: var(--danger); background: var(--danger-light); border: 1px solid #f0c0c0; }
      .catalog-title,
      .section-title {
        margin: 0;
        font: 700 16px/1.2 "Source Serif 4", Georgia, serif;
        color: var(--ink);
      }
      .catalog-meta,
      .helper-copy,
      .list-meta {
        color: var(--ink-light);
      }
      .catalog-blurb,
      .detail-copy,
      .pilgrim-panel p {
        margin: 0;
        color: var(--ink-light);
        line-height: 1.6;
      }
      .catalog-price-row,
      .detail-price-row,
      .summary-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 10px;
      }
      .catalog-price {
        font-size: 15px;
        font-weight: 700;
        color: var(--ink);
      }
      .detail-price {
        font-size: 22px;
        font-weight: 700;
        font-family: "Source Serif 4", Georgia, serif;
        color: var(--wine);
      }
      .product-layout,
      .checkout-layout,
      .dashboard-grid,
      .intake-grid,
      .fieldset-grid,
      #intake-form {
        display: grid;
        gap: 18px;
      }
      .product-layout { grid-template-columns: minmax(280px, 0.9fr) minmax(0, 1.1fr); }
      .checkout-layout { grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); }
      .dashboard-grid { grid-template-columns: repeat(auto-fit, minmax(260px, 1fr)); }
      .dashboard-grid > .surface-card:only-child { justify-self: center; }
      .dashboard-grid--three { grid-template-columns: repeat(3, minmax(0, 1fr)); }
      .intake-grid { grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); }
      .intake-panel { display: grid; gap: 16px; }
      #intake-form {
        grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
        align-items: start;
      }
      .intake-form__full { grid-column: 1 / -1; }
      .taxonomy-wrap { display: grid; gap: 18px; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); }
      .subheading,
      .detail-heading {
        margin: 0 0 10px;
        font-size: 1rem;
      }
      .detail-section { display: grid; gap: 8px; margin-top: 20px; }
      .detail-table { display: grid; gap: 8px; }
      .detail-table__row,
      .list-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 12px;
      }
      .list-row--soft {
        padding: 12px 14px;
        border-radius: var(--radius-sm);
        background: var(--filled);
      }
      .inline-quantity {
        display: grid;
        gap: 16px;
        grid-template-columns: minmax(120px, 180px) minmax(0, 1fr);
        margin-top: 24px;
      }
      .stack-list { display: grid; gap: 12px; }
      .stack-list--tight { gap: 8px; }
      .list-title { font-weight: 700; }
      .notice-panel {
        padding: 10px 14px;
        border-radius: 8px;
        background: var(--parchment);
        color: var(--warm-gray);
        font-size: 13px;
      }
      .notice-panel--success { background: var(--success-light); color: var(--success); border: 1px solid var(--success-border); }
      .notice-panel--warning { background: var(--gold-pale); color: var(--gold); border: 1px solid var(--gold-light); }
      .notice-panel--danger { background: var(--danger-light); color: var(--danger); }
      .pilgrim-panel {
        padding: 16px 20px;
        border-radius: 10px;
        background: var(--parchment);
        border: none;
      }
      .pilgrim-panel h3 { margin: 0 0 6px; font-size: 14px; font-weight: 700; color: var(--ink); }
      .summary-row {
        padding: 10px 0;
        border-bottom: 1px solid var(--parchment-dark);
      }
      .summary-row--total {
        font-size: 16px;
        font-weight: 700;
        font-family: "Source Serif 4", Georgia, serif;
        border-bottom: 0;
      }
      .empty-inline {
        color: var(--warm-gray);
        font-size: 0.92rem;
      }
      .metric-grid {
        display: grid;
        gap: 12px;
        grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
      }
      .metric-card {
        padding: 18px 20px;
        border-radius: 10px;
        background: #ffffff;
        border: 1px solid var(--parchment-dark);
        display: grid;
        gap: 6px;
      }
      .metric-card--feature {
        background: var(--ink);
        border-color: var(--ink);
        color: white;
      }
      .metric-icon {
        width: 38px;
        height: 38px;
        display: inline-flex;
        align-items: center;
        justify-content: center;
        border-radius: 12px;
        background: rgba(255,255,255,0.18);
        font-size: 1.1rem;
      }
      .metric-label {
        font-size: 10px;
        font-weight: 600;
        letter-spacing: 1.2px;
        text-transform: uppercase;
        color: var(--warm-gray);
      }
      .metric-card--feature .metric-label {
        color: rgba(245,241,234,0.6);
      }
      .divider-title {
        margin: 0 0 8px;
        color: var(--warm-gray);
        font-size: 10px;
        font-weight: 600;
        letter-spacing: 1.4px;
        text-transform: uppercase;
      }
      .divider-title--spaced { margin-top: 28px; }
      .helper-copy--flush { margin: 0; }
      .bar-track {
        width: 100%;
        height: 10px;
        border-radius: 999px;
        background: var(--parchment-dark);
        overflow: hidden;
      }
      .bar-fill {
        height: 100%;
        border-radius: 999px;
        background: linear-gradient(90deg, var(--gold), var(--wine));
      }
      .status-badge--paid {
        background: var(--success-light);
        color: var(--success);
      }
      .status-badge--iou {
        background: var(--warning-light);
        color: var(--warning);
      }
      .pagination,
      .pagination-links {
        display: flex;
        gap: 8px;
        align-items: center;
        flex-wrap: wrap;
      }
      .pagination { justify-content: space-between; margin-top: 18px; }
      .pagination-link {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        width: 28px;
        height: 28px;
        border-radius: 14px;
        border: none;
        background: transparent;
        color: var(--ink-light);
        text-decoration: none;
        font-size: 12px;
        font-weight: 600;
      }
      .pagination-link--active {
        background: var(--ink);
        color: white;
      }
      .stripe-placeholder {
        padding: 16px;
        border-radius: var(--radius);
        border: 1px dashed var(--filled-border);
        background: linear-gradient(180deg, white, var(--filled));
      }
      .stripe-placeholder__card {
        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: 12px;
        padding: 14px 16px;
        border-radius: var(--radius-sm);
        background: var(--ink);
        color: white;
        font-weight: 700;
      }
      .orders-table-wrap { overflow-x: auto; }
      .orders-table {
        width: 100%;
        border-collapse: collapse;
        font-size: 13px;
      }
      .orders-table th,
      .orders-table td {
        padding: 12px;
        border-bottom: 1px solid var(--parchment-dark);
        text-align: left;
        vertical-align: middle;
      }
      .orders-table th {
        color: var(--warm-gray);
        font-size: 11px;
        font-weight: 600;
        letter-spacing: 0.8px;
        text-transform: uppercase;
        border-bottom: 2px solid var(--parchment-dark);
      }
      video#camera {
        width: 100%;
        min-height: 220px;
        border-radius: var(--radius);
        background: linear-gradient(180deg, rgba(44,24,16,0.84), rgba(44,24,16,0.68));
      }
      @media (max-width: 960px) {
        .checkout-layout,
        .product-layout,
        .dashboard-grid--three {
          grid-template-columns: 1fr;
        }
        .admin-topbar,
        .page-header,
        .admin-footer {
          flex-direction: column;
          align-items: flex-start;
        }
        .admin-topbar {
          padding: 0.85rem 1rem;
        }
        .page-header {
          align-items: start;
        }
        .page-stack {
          padding: 0 1rem 2.5rem;
        }
        .storefront-header__meta,
        .page-header__badges,
        .page-header__actions {
          align-items: flex-start;
          justify-content: flex-start;
        }
      }
      @media (max-width: 640px) {
        .page-header { flex-direction: column; }
        .catalog-search-row { grid-template-columns: 1fr; }
        .catalog-results-head { align-items: start; flex-direction: column; }
        .catalog-grid { grid-template-columns: 1fr; }
        .product-layout { grid-template-columns: 1fr; }
        .inline-quantity { grid-template-columns: 1fr; }
        .dashboard-grid--three { grid-template-columns: 1fr; }
        #intake-form { grid-template-columns: 1fr; }
      }
    "#
}

#[cfg(test)]
mod tests {
    use super::{page_header, site_footer, site_nav};

    #[test]
    fn storefront_shell_uses_admin_style_chrome() {
        let nav = site_nav("catalog");
        let header = page_header(
            "Storefront",
            "Feed your soul.",
            "Curated parish titles.",
            &[],
            "",
        );
        let footer = site_footer();

        assert!(nav.contains("admin-topbar"));
        assert!(nav.contains("admin-topnav"));
        assert!(nav.contains("site-cart-count"));
        assert!(header.contains("admin-header"));
        assert!(header.contains("admin-header__eyebrow"));
        assert!(footer.contains("admin-footer"));
    }
}

pub fn html_escape(value: &str) -> String {
    value.replace('&', "&amp;").replace('"', "&quot;").replace('<', "&lt;").replace('>', "&gt;")
}
