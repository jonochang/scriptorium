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
        &nav_link("/admin", "Admin", "admin"),
        &nav_link("/admin/intake", "Intake", "intake"),
        "</nav></header>",
    ]
    .concat()
}

pub fn site_footer() -> &'static str {
    "<footer class=\"admin-footer storefront-footer\"><span>Scriptorium 2026. Parish browsing, intake, and Sunday-close reconciliation.</span><div class=\"admin-footer__links storefront-footer__links\"><a href=\"/catalog\">Catalog</a><a href=\"/cart\">Cart</a><a href=\"/admin\">Dashboard</a></div></footer>"
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
    r#"<link href="https://fonts.googleapis.com/css2?family=Crimson+Pro:wght@400;500;600;700&family=DM+Sans:wght@400;500;600;700;800&display=swap" rel="stylesheet">"#
}

pub fn shared_styles() -> &'static str {
    r#"
      :root {
        --wine: #6B2737;
        --wine-light: #8B3A4A;
        --wine-dark: #4A1A26;
        --wine-muted: #8B6B74;
        --gold: #B8903A;
        --gold-light: #CCAA5E;
        --gold-pale: #F5ECD7;
        --parchment: #FAF7F2;
        --parchment-dark: #EDE8E0;
        --filled: #F7F3EC;
        --filled-border: #E0D8CC;
        --ink: #2C1810;
        --ink-light: #5A4A3A;
        --warm-gray: #8A7A6A;
        --success: #5A7D5E;
        --success-light: #EEF3EE;
        --warning: #A07040;
        --warning-light: #F5EDE3;
        --danger: #9B5A5A;
        --danger-light: #F5EDED;
        --blue: #5A7A9B;
        --blue-light: #ECF1F5;
        --radius-sm: 8px;
        --radius: 12px;
        --radius-lg: 16px;
        --shadow: 0 2px 12px rgba(44,24,16,0.06);
        --shadow-lg: 0 8px 32px rgba(44,24,16,0.10);
      }
      * { box-sizing: border-box; }
      body {
        margin: 0;
        background:
          radial-gradient(circle at top, rgba(184,144,58,0.16), transparent 28%),
          linear-gradient(180deg, #fdfaf5 0%, var(--parchment) 100%);
        color: var(--ink);
        font-family: "DM Sans", sans-serif;
      }
      .page-shell { min-height: 100vh; }
      .page-stack { max-width: 1080px; margin: 0 auto; padding: 0 1.5rem 3rem; display: grid; gap: 18px; }
      .page-stack--wide { max-width: 1220px; }
      .admin-topbar,
      .storefront-topbar {
        display: flex;
        gap: 16px;
        align-items: center;
        justify-content: space-between;
        margin: 0 auto;
        max-width: 1220px;
        padding: 0 1.75rem;
        min-height: 56px;
        background: #3a2f25;
        color: #f5f1ea;
      }
      .admin-brand,
      .storefront-brand {
        display: inline-flex;
        align-items: center;
        gap: 0.6rem;
        color: #f5f1ea;
        text-decoration: none;
        font: 700 1rem/1 "Crimson Pro", serif;
        letter-spacing: 0.08em;
        text-transform: uppercase;
      }
      .admin-brand-mark,
      .storefront-brand-mark {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        width: 28px;
        height: 28px;
        border-radius: 999px;
        background: rgba(255,255,255,0.14);
        color: #fff;
        font-size: 0.95rem;
      }
      .admin-topnav,
      .admin-footer__links,
      .storefront-topnav,
      .storefront-footer__links {
        display: flex;
        gap: 0.75rem;
        flex-wrap: wrap;
        align-items: center;
      }
      .admin-topnav a,
      .admin-topnav span,
      .admin-footer a {
        display: inline-flex;
        align-items: center;
        gap: 8px;
        min-height: 32px;
        padding: 0 0.9rem;
        border-radius: 999px;
        color: rgba(245, 241, 234, 0.72);
        text-decoration: none;
        font-weight: 700;
        font-size: 0.82rem;
      }
      .admin-topnav .is-active {
        color: #fff;
        background: #8b2635;
      }
      .storefront-topnav__count {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        min-width: 22px;
        min-height: 22px;
        padding: 0 6px;
        border-radius: 999px;
        background: rgba(255,255,255,0.14);
        color: rgba(255,255,255,0.92);
        font-size: 0.78rem;
        font-weight: 800;
      }
      .site-footer,
      .admin-footer,
      .storefront-footer {
        max-width: 1220px;
        margin: 0 auto;
      }
      .admin-footer,
      .storefront-footer {
        padding: 1rem 1.5rem 2rem;
        border-top: 1px solid #e0d9cd;
        color: var(--warm-gray);
        display: flex;
        justify-content: space-between;
        gap: 1rem;
        font-size: 0.82rem;
      }
      .surface-card {
        background: rgba(255,255,255,0.9);
        border: 1px solid var(--parchment-dark);
        border-radius: var(--radius-lg);
        box-shadow: var(--shadow);
      }
      .page-header {
        max-width: 1220px;
        margin: 0 auto;
        padding: 2rem 1.5rem 1.5rem;
        display: flex;
        gap: 1.5rem;
        align-items: flex-start;
        justify-content: space-between;
      }
      .page-header__content {
        display: grid;
        gap: 8px;
      }
      .admin-header__eyebrow {
        margin: 0 0 0.3rem;
        font-size: 0.78rem;
        font-weight: 800;
        letter-spacing: 0.12em;
        text-transform: uppercase;
        color: #8b2635;
      }
      .surface-card { padding: 20px; }
      .display-title,
      .page-header__title {
        margin: 0;
        font: 600 2.2rem/1.05 "Crimson Pro", serif;
        letter-spacing: 0.02em;
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
        min-height: 30px;
        padding: 0 0.8rem;
        border-radius: 999px;
        background: rgba(255, 255, 255, 0.85);
        border: 1px solid #e0d8cc;
        color: #6f6052;
        font-size: 0.8rem;
        font-weight: 700;
      }
      .admin-badge--accent,
      .page-badge--accent {
        background: rgba(139, 38, 53, 0.08);
        border-color: rgba(139, 38, 53, 0.18);
        color: #8b2635;
      }
      .ghost-link,
      .primary-button,
      .accent-button {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        min-height: 46px;
        padding: 0 18px;
        border-radius: var(--radius);
        border: 0;
        text-decoration: none;
        font: 700 0.98rem/1 "DM Sans", sans-serif;
        cursor: pointer;
      }
      .ghost-link {
        color: white;
        background: rgba(255,255,255,0.08);
        border: 1px solid rgba(255,255,255,0.16);
      }
      .ghost-link--ink {
        color: var(--ink);
        background: white;
        border: 1px solid var(--parchment-dark);
      }
      .ghost-link--mini {
        min-height: 34px;
        padding: 0 10px;
        font-size: 0.82rem;
        font-weight: 700;
      }
      .primary-button { color: white; background: var(--wine); box-shadow: 0 4px 12px rgba(107,39,55,0.24); }
      .accent-button { color: white; background: var(--gold); }
      .field-label {
        display: block;
        margin: 0 0 8px;
        color: var(--ink-light);
        font-size: 0.92rem;
        font-weight: 600;
      }
      input, textarea, select {
        width: 100%;
        min-height: 46px;
        padding: 12px 14px;
        border-radius: var(--radius-sm);
        border: 1px solid var(--parchment-dark);
        background: white;
        color: var(--ink);
        font: 500 0.98rem/1.2 "DM Sans", sans-serif;
      }
      .catalog-search { display: grid; gap: 10px; margin-bottom: 18px; }
      .form-grid {
        display: grid;
        gap: 12px;
        grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
      }
      .category-strip {
        display: flex;
        flex-wrap: wrap;
        gap: 8px;
        margin-bottom: 16px;
      }
      .category-chip {
        display: inline-flex;
        align-items: center;
        gap: 8px;
        min-height: 38px;
        padding: 0 14px;
        border-radius: 999px;
        text-decoration: none;
        color: var(--ink-light);
        background: white;
        border: 1px solid var(--parchment-dark);
        font-size: 0.9rem;
        font-weight: 700;
      }
      .category-chip span {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        min-width: 22px;
        min-height: 22px;
        padding: 0 6px;
        border-radius: 999px;
        background: var(--filled);
        color: var(--warm-gray);
        font-size: 0.78rem;
      }
      .category-chip--active {
        color: white;
        background: var(--wine);
        border-color: var(--wine);
      }
      .category-chip--active span {
        background: rgba(255,255,255,0.14);
        color: rgba(255,255,255,0.88);
      }
      .catalog-results-head {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 12px;
        margin-bottom: 16px;
      }
      .catalog-search-row { display: grid; gap: 10px; grid-template-columns: minmax(0, 1fr) auto; }
      .catalog-grid {
        display: grid;
        gap: 14px;
        grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
      }
      .catalog-card {
        padding: 16px;
        border-radius: var(--radius);
        border: 1px solid var(--parchment-dark);
        background: linear-gradient(180deg, white, var(--parchment));
        box-shadow: var(--shadow);
      }
      .catalog-card__link {
        display: grid;
        gap: 12px;
        color: inherit;
        text-decoration: none;
      }
      .catalog-cover {
        min-height: 180px;
        border-radius: calc(var(--radius) - 2px);
        background: linear-gradient(160deg, rgba(107,39,55,0.92), rgba(184,144,58,0.92));
        display: flex;
        align-items: end;
        justify-content: start;
        padding: 16px;
        color: white;
      }
      .catalog-cover--detail {
        min-height: 320px;
        align-items: center;
        justify-content: center;
      }
      .book-cover-art {
        display: grid;
        gap: 10px;
        width: min(100%, 260px);
        padding: 18px;
        border-radius: var(--radius);
        border: 1px solid rgba(255,255,255,0.18);
        background: rgba(255,255,255,0.08);
        box-shadow: inset 0 1px 0 rgba(255,255,255,0.08);
        text-align: left;
      }
      .book-cover-art strong {
        font: 600 1.5rem/1.1 "Crimson Pro", serif;
        letter-spacing: 0.01em;
      }
      .book-cover-art__eyebrow {
        font-size: 0.72rem;
        letter-spacing: 0.18em;
        text-transform: uppercase;
        color: rgba(255,255,255,0.74);
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
      .button-row--flush-start { align-items: end; }
      .catalog-tag,
      .chip,
      .chip-muted,
      .status-badge {
        display: inline-flex;
        align-items: center;
        min-height: 30px;
        padding: 0 10px;
        border-radius: 999px;
        font-size: 0.8rem;
        font-weight: 700;
      }
      .catalog-tag,
      .chip {
        background: var(--gold-pale);
        color: var(--wine-dark);
      }
      .chip-muted {
        background: var(--filled);
        color: var(--warm-gray);
      }
      .catalog-tag--muted {
        background: white;
        color: var(--warm-gray);
        border: 1px solid var(--parchment-dark);
      }
      .stock-badge {
        display: inline-flex;
        align-items: center;
        min-height: 30px;
        padding: 0 10px;
        border-radius: 999px;
        font-size: 0.8rem;
        font-weight: 700;
      }
      .stock-badge--success { color: var(--success); background: var(--success-light); }
      .stock-badge--warning { color: var(--warning); background: var(--warning-light); }
      .stock-badge--danger { color: var(--danger); background: var(--danger-light); }
      .catalog-title,
      .section-title {
        margin: 0;
        font: 600 1.55rem/1.1 "Crimson Pro", serif;
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
      .catalog-price,
      .detail-price {
        font-size: 1.05rem;
        font-weight: 800;
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
      .checkout-layout { grid-template-columns: minmax(0, 1.1fr) minmax(320px, 0.9fr); }
      .dashboard-grid { grid-template-columns: repeat(auto-fit, minmax(260px, 1fr)); }
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
      .detail-section { display: grid; gap: 10px; margin-top: 16px; }
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
        gap: 12px;
        grid-template-columns: minmax(120px, 180px) minmax(0, 1fr);
        margin-top: 18px;
      }
      .stack-list { display: grid; gap: 12px; }
      .stack-list--tight { gap: 8px; }
      .list-title { font-weight: 700; }
      .notice-panel {
        padding: 12px 14px;
        border-radius: var(--radius-sm);
        background: var(--filled);
        color: var(--ink-light);
      }
      .notice-panel--success { background: var(--success-light); color: var(--success); }
      .notice-panel--warning { background: var(--warning-light); color: var(--warning); }
      .notice-panel--danger { background: var(--danger-light); color: var(--danger); }
      .pilgrim-panel {
        padding: 14px 16px;
        border-radius: var(--radius);
        background: linear-gradient(180deg, rgba(184,144,58,0.12), rgba(255,255,255,0.72));
        border: 1px solid rgba(184,144,58,0.24);
      }
      .pilgrim-panel h3 { margin: 0 0 6px; font-size: 1rem; }
      .summary-row {
        padding: 10px 0;
        border-bottom: 1px solid var(--parchment-dark);
      }
      .summary-row--total {
        font-size: 1.08rem;
        font-weight: 800;
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
        padding: 14px;
        border-radius: var(--radius);
        background: var(--filled);
        display: grid;
        gap: 8px;
      }
      .metric-card--feature {
        background: linear-gradient(160deg, rgba(107,39,55,0.96), rgba(139,58,74,0.94));
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
        font-size: 0.82rem;
        color: inherit;
        opacity: 0.84;
        text-transform: uppercase;
        letter-spacing: 0.08em;
      }
      .divider-title {
        margin: 0 0 10px;
        color: var(--warm-gray);
        font-size: 0.78rem;
        letter-spacing: 0.16em;
        text-transform: uppercase;
      }
      .divider-title--spaced { margin-top: 18px; }
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
        min-width: 38px;
        min-height: 38px;
        padding: 0 12px;
        border-radius: 999px;
        border: 1px solid var(--parchment-dark);
        background: white;
        color: var(--ink-light);
        text-decoration: none;
        font-weight: 700;
      }
      .pagination-link--active {
        background: var(--wine);
        border-color: var(--wine);
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
        font-size: 0.94rem;
      }
      .orders-table th,
      .orders-table td {
        padding: 12px 10px;
        border-bottom: 1px solid var(--parchment-dark);
        text-align: left;
        vertical-align: middle;
      }
      .orders-table th {
        color: var(--warm-gray);
        font-size: 0.78rem;
        letter-spacing: 0.12em;
        text-transform: uppercase;
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
