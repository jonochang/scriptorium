use axum::extract::Request;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, header};
use axum::middleware::{self, Next};
use axum::routing::{delete, get, post};
use axum::{
    Json, Router,
    response::{Html, IntoResponse, Response},
};
use bookstore_app::{
    AdminOrder, AdminProduct, AdminRole, AdminService, CatalogService, PosCartItem,
    PosCartSnapshot, PosPaymentOutcome, PosService, RequestContext, SalesEvent, StorefrontService,
    WebhookFinalizeStatus,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::time::Instant;

#[derive(Clone, Default)]
pub struct AppState {
    pub catalog: CatalogService,
    pub pos: PosService,
    pub storefront: StorefrontService,
    pub admin: AdminService,
    pub db_pool: Option<SqlitePool>,
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/books", get(list_books))
        .route("/context", get(request_context))
        .route("/admin", get(admin_dashboard_shell))
        .route("/admin/intake", get(admin_intake_shell))
        .route("/catalog", get(storefront_catalog))
        .route("/catalog/items/{book_id}", get(storefront_product_detail))
        .route("/catalog/search", get(storefront_search))
        .route("/cart", get(storefront_cart))
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
        .route("/api/admin/products/isbn-lookup", post(admin_isbn_lookup))
        .route("/api/admin/inventory/receive", post(admin_inventory_receive))
        .route("/api/admin/inventory/adjust", post(admin_inventory_adjust))
        .route("/api/admin/inventory/journal", get(admin_inventory_journal))
        .route("/api/admin/auth/login", post(admin_auth_login))
        .route("/api/admin/products", post(admin_product_upsert).get(admin_product_list))
        .route("/api/admin/products/{product_id}", delete(admin_product_delete))
        .route("/api/admin/categories", get(admin_categories_list))
        .route("/api/admin/vendors", get(admin_vendors_list))
        .route("/api/admin/orders", get(admin_orders_list))
        .route("/api/admin/orders/{order_id}/mark-paid", post(admin_order_mark_paid))
        .route("/api/admin/reports/summary", get(admin_report_summary))
        .route("/api/i18n", get(i18n_lookup))
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

#[derive(Debug, Deserialize, Default)]
struct CatalogQuery {
    q: Option<String>,
}

async fn storefront_catalog(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<CatalogQuery>,
) -> Html<String> {
    let books = state.catalog.list_books().await;
    let items = render_catalog_cards(filter_books(books, query.q.as_deref()));
    let search_value = html_escape(query.q.as_deref().unwrap_or(""));
    Html(
        [
            "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\" /><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" /><title>Scriptorium Catalog</title>",
            google_fonts_link(),
            "<script src=\"https://unpkg.com/htmx.org@2.0.4\"></script><style>",
            shared_styles(),
            "</style></head><body class=\"page-shell\"><main class=\"page-stack\"><section class=\"hero-card\"><div><p class=\"eyebrow\">Storefront</p><h1 class=\"display-title\">Browse the shelves</h1><p class=\"lede\">Search the catalog by title or author. HTMX enhances the results, and plain form submit still works.</p></div><a class=\"ghost-link\" href=\"/checkout\">Checkout</a></section><section class=\"surface-card\"><form class=\"catalog-search\" action=\"/catalog\" method=\"get\" hx-get=\"/catalog/search\" hx-target=\"#results\" hx-push-url=\"true\"><label class=\"field-label\" for=\"catalog-search\">Search catalog</label><div class=\"catalog-search-row\"><input id=\"catalog-search\" name=\"q\" value=\"",
            &search_value,
            "\" placeholder=\"Try Discipline or Foster\" /><button class=\"accent-button\" type=\"submit\">Search</button></div></form><div id=\"results\">",
            &items,
            "</div></section></main></body></html>",
        ]
        .concat(),
    )
}

async fn storefront_search(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Html<String> {
    let query = params.get("q").map_or("", String::as_str).to_ascii_lowercase();
    let books = state.catalog.list_books().await;
    let filtered = render_catalog_cards(filter_books(books, Some(&query)));
    Html(filtered)
}

async fn storefront_product_detail(
    State(state): State<AppState>,
    axum::extract::Path(book_id): axum::extract::Path<String>,
) -> Result<Html<String>, StatusCode> {
    let books = state.catalog.list_books().await;
    let book = books
        .into_iter()
        .find(|book| book.id == book_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    let price = format_money(i64::from(book.price_cents));
    Ok(Html(
        [
            "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\" /><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" /><title>Scriptorium Product</title>",
            google_fonts_link(),
            "<style>",
            shared_styles(),
            "</style></head><body class=\"page-shell\"><main class=\"page-stack\"><section class=\"hero-card\"><div><p class=\"eyebrow\">Product Detail</p><h1 class=\"display-title\">",
            &html_escape(&book.title),
            "</h1><p class=\"lede\">by ",
            &html_escape(&book.author),
            "</p></div><a class=\"ghost-link\" href=\"/cart\">Cart</a></section><section class=\"product-layout\"><article class=\"surface-card\"><div class=\"catalog-cover catalog-cover--detail\">📘</div></article><article class=\"surface-card\"><span class=\"chip\">",
            &html_escape(&book.category),
            "</span><h2 class=\"section-title\">",
            &html_escape(&book.title),
            "</h2><p class=\"catalog-meta\">",
            &html_escape(&book.author),
            "</p><div class=\"detail-price\">",
            &price,
            "</div><p class=\"helper-copy\">This detail page now links the catalog to a cart flow instead of stopping at search results.</p><div class=\"button-row\"><button class=\"primary-button\" type=\"button\" data-add-book-id=\"",
            &html_escape(&book.id),
            "\" data-add-book-title=\"",
            &html_escape(&book.title),
            "\" data-add-book-author=\"",
            &html_escape(&book.author),
            "\" data-add-book-price-cents=\"",
            &i64::from(book.price_cents).to_string(),
            "\">Add to cart</button><a class=\"accent-button\" href=\"/checkout\">Go to checkout</a></div><div id=\"cart-feedback\" class=\"notice-panel\">Ready to add this title to the cart.</div></article></section></main>",
            storefront_cart_script(),
            "</body></html>",
        ]
        .concat(),
    ))
}

async fn storefront_cart(State(state): State<AppState>) -> Html<String> {
    let books = state.catalog.list_books().await;
    let recommendations = books
        .into_iter()
        .take(3)
        .map(|book| {
            format!(
                "<div class=\"list-row\"><div><div class=\"list-title\">{}</div><div class=\"list-meta\">{} · {}</div></div><a class=\"ghost-link ghost-link--ink\" href=\"/catalog/items/{}\">View</a></div>",
                html_escape(&book.title),
                html_escape(&book.author),
                html_escape(&book.category),
                html_escape(&book.id),
            )
        })
        .collect::<Vec<_>>()
        .join("");
    Html(
        [
            "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\" /><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" /><title>Scriptorium Cart</title>",
            google_fonts_link(),
            "<style>",
            shared_styles(),
            "</style></head><body class=\"page-shell\"><main class=\"page-stack\"><section class=\"hero-card\"><div><p class=\"eyebrow\">Cart</p><h1 class=\"display-title\">Review your basket</h1><p class=\"lede\">The cart now persists in browser storage so storefront pages connect to checkout.</p></div><a class=\"ghost-link\" href=\"/checkout\">Checkout</a></section><section class=\"checkout-layout\"><article class=\"surface-card\"><h2 class=\"section-title\">Cart items</h2><div id=\"cart-items\" class=\"stack-list\"><div class=\"empty-inline\">Your cart is empty.</div></div></article><article class=\"surface-card\"><h2 class=\"section-title\">Recommended titles</h2><div class=\"stack-list\">",
            &recommendations,
            "</div><div class=\"notice-panel notice-panel--success\" id=\"cart-summary\">Cart total: $0.00</div></article></section></main>",
            storefront_cart_script(),
            "</body></html>",
        ]
        .concat(),
    )
}

async fn storefront_checkout() -> Html<String> {
    Html(
        [
            "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\" /><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" /><title>Scriptorium Checkout</title>",
            google_fonts_link(),
            "<style>",
            shared_styles(),
            "</style></head><body class=\"page-shell\"><main class=\"page-stack\"><section class=\"hero-card\"><div><p class=\"eyebrow\">Checkout</p><h1 class=\"display-title\">Finish an online order</h1><p class=\"lede\">The backend session and webhook flow is live. This page now creates checkout sessions against the running API.</p></div><a class=\"ghost-link\" href=\"/cart\">Back to cart</a></section><section class=\"checkout-layout\"><article class=\"surface-card\"><h2 class=\"section-title\">Order summary</h2><div id=\"checkout-lines\" class=\"stack-list\"><div class=\"empty-inline\">Your cart is empty.</div></div><div class=\"summary-row\"><span>Shipping</span><strong>$0.00</strong></div><div class=\"summary-row summary-row--total\"><span>Total</span><strong id=\"checkout-total\">$0.00</strong></div></article><article class=\"surface-card\"><h2 class=\"section-title\">Payment</h2><label class=\"field-label\" for=\"checkout-email\">Receipt email</label><input id=\"checkout-email\" placeholder=\"reader@example.com\" value=\"jane@example.com\" /><button class=\"primary-button\" type=\"button\" id=\"create-checkout-session\">Create checkout session</button><p class=\"helper-copy\">The button posts to <code>/api/storefront/checkout/session</code> and surfaces the returned session id.</p><div id=\"checkout-status\" class=\"notice-panel\" aria-live=\"polite\">Ready to create a checkout session.</div></article></section></main><script>const CART_KEY='scriptorium-storefront-cart';function readCart(){try{return JSON.parse(localStorage.getItem(CART_KEY)||'[]');}catch{return [];}}function money(cents){return `$${(Number(cents||0)/100).toFixed(2)}`;}function cartTotal(cart){return cart.reduce((sum,item)=>sum+(Number(item.price_cents||0)*Number(item.quantity||0)),0);}function renderCheckout(){const cart=readCart();const lines=document.getElementById('checkout-lines');const total=cartTotal(cart);document.getElementById('checkout-total').textContent=money(total);if(!cart.length){lines.innerHTML='<div class=\"empty-inline\">Your cart is empty.</div>';return total;}lines.innerHTML=cart.map((item)=>`<div class=\"list-row\"><div><div class=\"list-title\">${item.title}</div><div class=\"list-meta\">${item.author} · Qty ${item.quantity}</div></div><strong>${money(item.price_cents*item.quantity)}</strong></div>`).join('');return total;}async function createCheckoutSession(){const totalCents=renderCheckout();const email=document.getElementById('checkout-email').value;const panel=document.getElementById('checkout-status');panel.textContent='Creating checkout session...';const res=await fetch('/api/storefront/checkout/session',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({total_cents:totalCents,email})});const json=await res.json().catch(()=>({}));if(!res.ok){panel.textContent=json.message||json.error||'Checkout session request failed.';panel.className='notice-panel notice-panel--danger';return;}panel.textContent=`Session created: ${json.session_id}`;panel.className='notice-panel notice-panel--success';}document.getElementById('create-checkout-session').addEventListener('click',createCheckoutSession);renderCheckout();</script></body></html>",
        ]
        .concat(),
    )
}

async fn admin_dashboard_shell() -> Html<String> {
    Html(
        [
            "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\" /><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" /><title>Scriptorium Admin</title>",
            google_fonts_link(),
            "<style>",
            shared_styles(),
            "</style></head><body class=\"page-shell\"><main class=\"page-stack\"><section class=\"hero-card\"><div><p class=\"eyebrow\">Admin Office</p><h1 class=\"display-title\">Dashboard, stock, and reporting</h1><p class=\"lede\">This shell now authenticates against the live admin API and loads report, product, category, vendor, and order data.</p></div></section><section class=\"dashboard-grid\"><article class=\"surface-card\"><h2 class=\"section-title\">Admin sign-in</h2><label class=\"field-label\" for=\"admin-username\">Username</label><input id=\"admin-username\" autocomplete=\"username\" placeholder=\"admin\" /><label class=\"field-label\" for=\"admin-password\">Password</label><input id=\"admin-password\" type=\"password\" autocomplete=\"current-password\" placeholder=\"Password\" /><div class=\"button-row\"><button class=\"primary-button\" type=\"button\" id=\"admin-login\">Login</button><button class=\"accent-button\" type=\"button\" id=\"admin-refresh\">Refresh data</button></div><p class=\"helper-copy\">After login, dashboard widgets load from <code>/api/admin/*</code>.</p><div id=\"admin-status\" class=\"notice-panel\" aria-live=\"polite\">Sign in to load tenant dashboard data.</div></article><article class=\"surface-card\"><h2 class=\"section-title\">Live report summary</h2><div class=\"metric-grid\"><div class=\"metric-card\"><span class=\"metric-label\">Sales</span><strong id=\"metric-sales\">$0.00</strong></div><div class=\"metric-card\"><span class=\"metric-label\">Donations</span><strong id=\"metric-donations\">$0.00</strong></div><div class=\"metric-card\"><span class=\"metric-label\">COGS</span><strong id=\"metric-cogs\">$0.00</strong></div><div class=\"metric-card\"><span class=\"metric-label\">Gross Profit</span><strong id=\"metric-profit\">$0.00</strong></div></div></article></section><section class=\"dashboard-grid\"><article class=\"surface-card\"><h2 class=\"section-title\">Products</h2><div id=\"admin-products\" class=\"stack-list\"><div class=\"empty-inline\">No products loaded yet.</div></div></article><article class=\"surface-card\"><h2 class=\"section-title\">Categories and vendors</h2><div class=\"taxonomy-wrap\"><div><h3 class=\"subheading\">Categories</h3><div id=\"admin-categories\" class=\"chip-wrap\"><span class=\"chip-muted\">Waiting for data</span></div></div><div><h3 class=\"subheading\">Vendors</h3><div id=\"admin-vendors\" class=\"chip-wrap\"><span class=\"chip-muted\">Waiting for data</span></div></div></div></article></section><section class=\"dashboard-grid\"><article class=\"surface-card\"><h2 class=\"section-title\">Recent orders</h2><div id=\"admin-orders\" class=\"stack-list\"><div class=\"empty-inline\">No orders loaded yet.</div></div></article><article class=\"surface-card\"><h2 class=\"section-title\">Open IOUs</h2><div id=\"admin-ious\" class=\"stack-list\"><div class=\"empty-inline\">No open IOUs.</div></div></article></section></main><script>let adminToken='';let adminTenant='church-a';const money=(cents)=>`$${(Number(cents||0)/100).toFixed(2)}`;function setStatus(message,tone=''){const panel=document.getElementById('admin-status');panel.textContent=message;panel.className=`notice-panel${tone?` notice-panel--${tone}`:''}`;}function renderList(containerId,items,emptyMessage,renderer){const node=document.getElementById(containerId);if(!items.length){node.innerHTML=`<div class=\"empty-inline\">${emptyMessage}</div>`;return;}node.innerHTML=items.map(renderer).join('');}async function adminLogin(){const username=document.getElementById('admin-username').value;const password=document.getElementById('admin-password').value;setStatus('Signing in...');const res=await fetch('/api/admin/auth/login',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({username,password})});const json=await res.json().catch(()=>({}));if(!res.ok){setStatus(json.message||'Login failed.','danger');return;}adminToken=json.token||'';adminTenant=json.tenant_id||'church-a';setStatus(`Signed in for ${adminTenant}.`,'success');await refreshAdminData();}async function fetchJson(url,options={}){const headers={...(options.headers||{}),Authorization:`Bearer ${adminToken}`};const res=await fetch(url,{...options,headers});const json=await res.json().catch(()=>({}));if(!res.ok){throw new Error(json.message||json.error||`Request failed for ${url}`);}return json;}async function markOrderPaid(orderId){if(!adminToken){setStatus('Sign in first to manage orders.','danger');return;}try{await fetchJson(`/api/admin/orders/${orderId}/mark-paid?tenant_id=${adminTenant}`,{method:'POST',headers:{Origin:window.location.origin}});setStatus(`Marked ${orderId} paid.`,'success');await refreshAdminData();}catch(error){setStatus(error.message,'danger');}}async function refreshAdminData(){if(!adminToken){setStatus('Sign in first to load dashboard data.','danger');return;}setStatus('Loading dashboard data...');try{const [summary,products,categories,vendors,orders]=await Promise.all([fetchJson(`/api/admin/reports/summary?tenant_id=${adminTenant}`),fetchJson(`/api/admin/products?tenant_id=${adminTenant}`),fetchJson(`/api/admin/categories?tenant_id=${adminTenant}`),fetchJson(`/api/admin/vendors?tenant_id=${adminTenant}`),fetchJson(`/api/admin/orders?tenant_id=${adminTenant}`)]);document.getElementById('metric-sales').textContent=money(summary.sales_cents);document.getElementById('metric-donations').textContent=money(summary.donations_cents);document.getElementById('metric-cogs').textContent=money(summary.cogs_cents);document.getElementById('metric-profit').textContent=money(summary.gross_profit_cents);renderList('admin-products',products,'No products found for this tenant.',(product)=>`<div class=\"list-row\"><div><div class=\"list-title\">${product.title}</div><div class=\"list-meta\">${product.category} · ${product.vendor}</div></div><strong>${money(product.retail_cents)}</strong></div>`);renderList('admin-categories',categories.values||[],'No categories found.',(value)=>`<span class=\"chip\">${value}</span>`);renderList('admin-vendors',vendors.values||[],'No vendors found.',(value)=>`<span class=\"chip\">${value}</span>`);renderList('admin-orders',orders,'No orders found for this tenant.',(order)=>`<div class=\"list-row\"><div><div class=\"list-title\">${order.order_id} · ${order.customer_name}</div><div class=\"list-meta\">${order.channel} · ${order.payment_method} · ${order.created_on}</div></div><strong>${money(order.total_cents)}</strong></div>`);renderList('admin-ious',orders.filter((order)=>order.status==='UnpaidIou'),'No open IOUs.',(order)=>`<div class=\"list-row\"><div><div class=\"list-title\">${order.customer_name}</div><div class=\"list-meta\">${order.order_id} · ${order.created_on}</div></div><div class=\"button-row button-row--compact\"><strong>${money(order.total_cents)}</strong><button class=\"primary-button primary-button--sm\" type=\"button\" onclick=\"markOrderPaid('${order.order_id}')\">Mark Paid</button></div></div>`);setStatus(`Dashboard refreshed for ${adminTenant}.`,'success');}catch(error){setStatus(error.message,'danger');}}document.getElementById('admin-login').addEventListener('click',adminLogin);document.getElementById('admin-refresh').addEventListener('click',refreshAdminData);window.markOrderPaid=markOrderPaid;</script></body></html>",
        ]
        .concat(),
    )
}

fn google_fonts_link() -> &'static str {
    r#"<link href="https://fonts.googleapis.com/css2?family=Crimson+Pro:wght@400;500;600;700&family=DM+Sans:wght@400;500;600;700;800&display=swap" rel="stylesheet">"#
}

fn shared_styles() -> &'static str {
    r#"
      :root {
        --wine: #6B2737;
        --wine-light: #8B3A4A;
        --wine-dark: #4A1A26;
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
      .page-shell { min-height: 100vh; padding: 24px 16px 40px; }
      .page-stack { max-width: 1080px; margin: 0 auto; display: grid; gap: 18px; }
      .hero-card,
      .surface-card {
        background: rgba(255,255,255,0.9);
        border: 1px solid var(--parchment-dark);
        border-radius: var(--radius-lg);
        box-shadow: var(--shadow);
      }
      .hero-card {
        padding: 28px;
        display: flex;
        gap: 18px;
        align-items: end;
        justify-content: space-between;
        background:
          linear-gradient(135deg, rgba(107,39,55,0.98), rgba(74,26,38,0.96)),
          var(--wine);
        color: white;
      }
      .surface-card { padding: 20px; }
      .display-title {
        margin: 0;
        font: 600 2.2rem/1.05 "Crimson Pro", serif;
        letter-spacing: 0.02em;
      }
      .eyebrow {
        margin: 0 0 8px;
        font-size: 0.78rem;
        letter-spacing: 0.18em;
        text-transform: uppercase;
        color: var(--gold-light);
      }
      .lede { margin: 8px 0 0; color: rgba(255,255,255,0.78); max-width: 60ch; }
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
      .primary-button { color: white; background: var(--wine); box-shadow: 0 4px 12px rgba(107,39,55,0.24); }
      .accent-button { color: white; background: var(--gold); }
      .field-label {
        display: block;
        margin: 0 0 8px;
        color: var(--ink-light);
        font-size: 0.92rem;
        font-weight: 600;
      }
      input, textarea {
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
      .catalog-cover {
        display: grid;
        place-items: center;
        min-height: 148px;
        margin-bottom: 12px;
        border-radius: var(--radius);
        background: linear-gradient(135deg, var(--gold-pale), white);
        color: var(--wine);
        font-size: 3rem;
      }
      .catalog-cover--detail { min-height: 320px; font-size: 5rem; }
      .catalog-title {
        margin: 0 0 6px;
        font: 600 1.45rem/1 "Crimson Pro", serif;
      }
      .catalog-meta { margin: 0 0 12px; color: var(--warm-gray); }
      .catalog-price {
        display: inline-flex;
        padding: 6px 10px;
        border-radius: 999px;
        color: var(--wine);
        background: var(--gold-pale);
        font-weight: 700;
      }
      .catalog-empty {
        padding: 22px;
        border-radius: var(--radius);
        background: var(--filled);
        border: 1px solid var(--filled-border);
        color: var(--ink-light);
      }
      .checkout-layout { display: grid; gap: 18px; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); }
      .product-layout { display: grid; gap: 18px; grid-template-columns: minmax(260px, 340px) minmax(0, 1fr); }
      .section-title {
        margin: 0 0 14px;
        font: 600 1.6rem/1 "Crimson Pro", serif;
      }
      .summary-row {
        display: flex;
        justify-content: space-between;
        gap: 12px;
        padding: 12px 0;
        border-bottom: 1px solid var(--parchment-dark);
      }
      .summary-row--total { font-size: 1.05rem; border-bottom: 0; }
      .detail-price {
        margin: 14px 0;
        font-size: 2rem;
        font-weight: 800;
        color: var(--wine);
      }
      .helper-copy { margin: 12px 0 0; color: var(--warm-gray); font-size: 0.92rem; }
      .dashboard-grid {
        display: grid;
        gap: 18px;
        grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
      }
      .button-row {
        display: flex;
        flex-wrap: wrap;
        gap: 10px;
        margin-top: 14px;
      }
      .button-row--compact {
        margin-top: 0;
        align-items: center;
        justify-content: end;
      }
      .primary-button--sm {
        min-height: 34px;
        padding: 0 12px;
        font-size: 0.86rem;
      }
      .notice-panel {
        margin-top: 14px;
        padding: 14px;
        border-radius: var(--radius);
        border: 1px solid var(--parchment-dark);
        background: white;
        color: var(--ink-light);
      }
      .notice-panel--success {
        background: var(--success-light);
        border-color: rgba(90,125,94,0.24);
        color: var(--success);
      }
      .notice-panel--danger {
        background: var(--danger-light);
        border-color: rgba(155,90,90,0.24);
        color: var(--danger);
      }
      .metric-grid {
        display: grid;
        gap: 12px;
        grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
      }
      .metric-card {
        padding: 14px;
        border-radius: var(--radius);
        border: 1px solid var(--parchment-dark);
        background: linear-gradient(180deg, white, var(--filled));
      }
      .metric-label {
        display: block;
        margin-bottom: 8px;
        color: var(--warm-gray);
        font-size: 0.82rem;
        text-transform: uppercase;
        letter-spacing: 0.08em;
      }
      .stack-list { display: grid; gap: 10px; }
      .list-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 12px;
        padding: 12px 14px;
        border-radius: var(--radius);
        border: 1px solid var(--parchment-dark);
        background: white;
      }
      .list-title { font-weight: 700; }
      .list-meta { margin-top: 4px; color: var(--warm-gray); font-size: 0.9rem; }
      .taxonomy-wrap { display: grid; gap: 18px; }
      .subheading {
        margin: 0 0 10px;
        color: var(--ink-light);
        font-size: 0.95rem;
      }
      .chip-wrap {
        display: flex;
        gap: 8px;
        flex-wrap: wrap;
      }
      .chip,
      .chip-muted {
        display: inline-flex;
        align-items: center;
        min-height: 34px;
        padding: 0 12px;
        border-radius: 999px;
        font-weight: 600;
      }
      .chip {
        background: var(--gold-pale);
        color: var(--wine);
      }
      .chip-muted {
        background: var(--filled);
        color: var(--warm-gray);
        border: 1px solid var(--filled-border);
      }
      .empty-inline {
        padding: 14px;
        border-radius: var(--radius);
        background: var(--filled);
        border: 1px solid var(--filled-border);
        color: var(--warm-gray);
      }
      #camera {
        width: 100%;
        min-height: 220px;
        margin-bottom: 14px;
        border-radius: var(--radius);
        background: linear-gradient(135deg, var(--wine-dark), var(--wine));
      }
      #intake-form {
        display: grid;
        gap: 12px;
        grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
      }
      #description { min-height: 96px; }
      @media (max-width: 640px) {
        .hero-card { align-items: start; flex-direction: column; }
        .catalog-search-row { grid-template-columns: 1fr; }
        .product-layout { grid-template-columns: 1fr; }
        #intake-form { grid-template-columns: 1fr; }
      }
    "#
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn filter_books(books: Vec<bookstore_domain::Book>, query: Option<&str>) -> Vec<bookstore_domain::Book> {
    let query = query.unwrap_or("").trim().to_ascii_lowercase();
    if query.is_empty() {
        return books;
    }
    books
        .into_iter()
        .filter(|book| {
            book.title.to_ascii_lowercase().contains(&query)
                || book.author.to_ascii_lowercase().contains(&query)
        })
        .collect()
}

fn render_catalog_cards(books: Vec<bookstore_domain::Book>) -> String {
    if books.is_empty() {
        return "<div class=\"catalog-empty\">No books matched that search.</div>".to_string();
    }
    let items = books
        .into_iter()
        .map(|book| {
            format!(
                r#"<article class="catalog-card">
  <div class="catalog-cover">📚</div>
  <h2 class="catalog-title">{title}</h2>
  <p class="catalog-meta">{author}</p>
  <div class="button-row">
    <span class="catalog-price">{price}</span>
    <a class="ghost-link ghost-link--ink" href="/catalog/items/{book_id}">View</a>
  </div>
</article>"#,
                title = html_escape(&book.title),
                author = html_escape(&book.author),
                price = format_money(i64::from(book.price_cents)),
                book_id = html_escape(&book.id),
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!(r#"<div class="catalog-grid">{items}</div>"#)
}

fn storefront_cart_script() -> &'static str {
    r#"<script>
const CART_KEY = "scriptorium-storefront-cart";
function readCart() {
  try { return JSON.parse(localStorage.getItem(CART_KEY) || "[]"); } catch { return []; }
}
function writeCart(cart) {
  localStorage.setItem(CART_KEY, JSON.stringify(cart));
}
function money(cents) {
  return `$${(Number(cents || 0) / 100).toFixed(2)}`;
}
function addToCartFromDataset(button) {
  const cart = readCart();
  const id = button.dataset.addBookId;
  const price = Number(button.dataset.addBookPriceCents || 0);
  const existing = cart.find((item) => item.id === id);
  if (existing) {
    existing.quantity += 1;
  } else {
    cart.push({
      id,
      title: button.dataset.addBookTitle,
      author: button.dataset.addBookAuthor,
      price_cents: price,
      quantity: 1,
    });
  }
  writeCart(cart);
  const feedback = document.getElementById("cart-feedback");
  if (feedback) {
    feedback.textContent = `Added to cart. Cart now has ${cart.reduce((sum, item) => sum + item.quantity, 0)} item(s).`;
    feedback.className = "notice-panel notice-panel--success";
  }
  renderCartPage();
}
function renderCartPage() {
  const cart = readCart();
  const cartItems = document.getElementById("cart-items");
  const cartSummary = document.getElementById("cart-summary");
  if (cartItems) {
    if (!cart.length) {
      cartItems.innerHTML = '<div class="empty-inline">Your cart is empty.</div>';
    } else {
      cartItems.innerHTML = cart.map((item) => `
        <div class="list-row">
          <div>
            <div class="list-title">${item.title}</div>
            <div class="list-meta">${item.author} · Qty ${item.quantity}</div>
          </div>
          <strong>${money(item.price_cents * item.quantity)}</strong>
        </div>
      `).join("");
    }
  }
  if (cartSummary) {
    const total = cart.reduce((sum, item) => sum + (item.price_cents * item.quantity), 0);
    cartSummary.textContent = `Cart total: ${money(total)}`;
  }
}
document.querySelectorAll("[data-add-book-id]").forEach((button) => {
  button.addEventListener("click", () => addToCartFromDataset(button));
});
renderCartPage();
</script>"#
}

fn format_money(cents: i64) -> String {
    format!("${}.{:02}", cents / 100, (cents % 100).abs())
}

async fn admin_intake_shell() -> Html<String> {
    Html([
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Admin Intake</title>
"#,
        google_fonts_link(),
        r#"<style>"#,
        shared_styles(),
        r#"</style>
</head>
<body class="page-shell">
  <main class="page-stack">
    <section class="hero-card">
      <div>
        <p class="eyebrow">Admin Intake</p>
        <h1 class="display-title">Receive and enrich inventory</h1>
        <p class="lede">Scan ISBNs, authenticate, and pull metadata into the intake form.</p>
      </div>
    </section>
    <section class="surface-card">
      <h2 class="section-title">Inventory intake</h2>
      <video id="camera" autoplay playsinline></video>
      <form id="intake-form">
        <input id="isbn" name="isbn" placeholder="ISBN" />
        <input id="title" name="title" placeholder="Title" />
        <input id="author" name="author" placeholder="Author" />
        <input id="description" name="description" placeholder="Description" />
        <input id="username" name="username" placeholder="Username" autocomplete="username" />
        <input id="password" name="password" type="password" placeholder="Password" autocomplete="current-password" />
        <input id="token" name="token" placeholder="Admin Token" />
        <button class="primary-button" type="button" id="login">Login</button>
        <button class="accent-button" type="button" id="lookup">Lookup</button>
      </form>
    </section>
  </main>
  <script>
    async function bootCamera() {
      if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) return;
      const stream = await navigator.mediaDevices.getUserMedia({ video: { facingMode: "environment" } });
      document.getElementById("camera").srcObject = stream;
      if ("BarcodeDetector" in window) {
        const detector = new BarcodeDetector({ formats: ["ean_13", "upc_a"] });
        setInterval(async () => {
          const video = document.getElementById("camera");
          const barcodes = await detector.detect(video);
          if (barcodes[0] && barcodes[0].rawValue) {
            document.getElementById("isbn").value = barcodes[0].rawValue;
          }
        }, 1000);
      }
    }
    async function login() {
      const username = document.getElementById("username").value;
      const password = document.getElementById("password").value;
      const res = await fetch("/api/admin/auth/login", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ username, password }),
      });
      const json = await res.json();
      document.getElementById("token").value = json.token || "";
    }
    async function lookup() {
      const isbn = document.getElementById("isbn").value;
      const token = document.getElementById("token").value;
      const res = await fetch("/api/admin/products/isbn-lookup", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ token, isbn }),
      });
      const json = await res.json();
      document.getElementById("title").value = json.title || "";
      document.getElementById("author").value = json.author || "";
      document.getElementById("description").value = json.description || "";
    }
    document.getElementById("login").addEventListener("click", login);
    document.getElementById("lookup").addEventListener("click", lookup);
    bootCamera();
  </script>
</body>
</html>"#,
    ]
    .concat())
}

async fn pos_shell() -> Html<&'static str> {
    Html(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Scriptorium POS</title>
  <link href="https://fonts.googleapis.com/css2?family=Crimson+Pro:wght@400;500;600;700&family=DM+Sans:wght@400;500;600;700;800&display=swap" rel="stylesheet">
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
      --success: #5A7D5E;
      --success-light: #EEF3EE;
      --warning: #A07040;
      --warning-light: #F5EDE3;
      --danger: #9B5A5A;
      --danger-light: #F5EDED;
      --radius: 12px;
      --radius-lg: 16px;
      --shadow: 0 4px 18px rgba(44,24,16,.10);
    }
    * { box-sizing: border-box; }
    body {
      margin: 0;
      font-family: "DM Sans", sans-serif;
      background:
        radial-gradient(circle at top, rgba(204,170,94,.18), transparent 28%),
        linear-gradient(180deg, var(--wine-dark), var(--wine) 28%, #55202d 100%);
      color: #fff;
      min-height: 100vh;
      padding: 18px 14px 28px;
    }
    .pos-wrap {
      max-width: 460px;
      margin: 0 auto;
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
    .row { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }
    .hero {
      padding: 18px;
      border-radius: var(--radius-lg);
      background: linear-gradient(135deg, rgba(107,39,55,.96), rgba(139,58,74,.9));
      box-shadow: var(--shadow);
    }
    .hero h1 {
      margin: 0;
      font-family: "Crimson Pro", serif;
      font-size: 2rem;
      color: var(--gold-light);
      letter-spacing: .03em;
    }
    .hero p {
      margin: 8px 0 0;
      color: rgba(255,255,255,.76);
      font-size: .95rem;
    }
    .kicker {
      margin: 0 0 8px;
      color: rgba(255,255,255,.68);
      font-size: .78rem;
      text-transform: uppercase;
      letter-spacing: .16em;
    }
    input {
      width: 100%;
      min-height: 44px;
      border-radius: 10px;
      border: 1px solid var(--parchment-dark);
      padding: 10px 12px;
      box-sizing: border-box;
      background: #fff;
      color: var(--ink);
      font: 500 16px/1.2 "DM Sans", sans-serif;
    }
    .section-title {
      margin: 0 0 12px;
      font-family: "Crimson Pro", serif;
      font-size: 1.45rem;
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
    .cart-list { display: grid; gap: 10px; }
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
    .empty-state {
      padding: 16px;
      border-radius: 12px;
      background: linear-gradient(180deg, #fff, #f7f3ec);
      border: 1px dashed var(--parchment-dark);
      color: var(--ink-light);
      text-align: center;
    }
    .quick-grid {
      display: grid;
      grid-template-columns: repeat(2, minmax(0, 1fr));
      gap: 10px;
    }
    .quick-tile {
      border: 1px solid var(--parchment-dark);
      border-radius: 14px;
      background: linear-gradient(180deg, #fff, var(--gold-pale));
      color: var(--ink);
      min-height: 100px;
      padding: 14px;
      text-align: left;
      font: 700 1rem/1.2 "DM Sans", sans-serif;
    }
    .quick-emoji { font-size: 1.6rem; display: block; margin-bottom: 10px; }
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
    .totals-row strong { font-size: 1.2rem; color: var(--wine); }
    .status-panel {
      min-height: 84px;
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
    .field-label {
      display: block;
      margin: 0 0 8px;
      font-size: .9rem;
      font-weight: 600;
      color: var(--ink-light);
    }
    .actions { display: grid; gap: 10px; }
    .hint { margin: 0; color: var(--warm-gray); font-size: .86rem; }
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
      const [cart, setCart] = useState([]);
      const [total, setTotal] = useState(0);
      const [status, setStatus] = useState({ tone: "warning", title: "Shift not started", detail: "Enter the 4-digit shift PIN to begin taking sales." });
      const [lastSale, setLastSale] = useState(null);

      const money = (cents) => `$${(cents / 100).toFixed(2)}`;

      const applyCart = (payload) => {
        setCart(Array.isArray(payload.items) ? payload.items : []);
        setTotal(Number.isFinite(payload.total_cents) ? payload.total_cents : 0);
      };

      const request = async (url, payload) => {
        const res = await fetch(url, {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify(payload),
        });
        const json = await res.json().catch(() => ({}));
        if (!res.ok) {
          setStatus({
            tone: "danger",
            title: json.error || "Request failed",
            detail: json.message || "The POS endpoint returned an error.",
          });
          return { ok: false, json };
        }
        return { ok: true, json };
      };

      const startShift = async () => {
        const result = await request("/api/pos/login", { pin: "1234" });
        if (!result.ok) return;
        const nextToken = result.json.session_token || "";
        setToken(nextToken);
        setCart([]);
        setTotal(0);
        setLastSale(null);
        setStatus({
          tone: "success",
          title: "Shift started",
          detail: nextToken ? `Session ${nextToken} is ready for scanning and checkout.` : "POS session opened.",
        });
      };

      const scanItem = async () => {
        const result = await request("/api/pos/scan", { session_token: token, isbn: barcode });
        if (!result.ok) return;
        applyCart(result.json);
        setLastSale(null);
        setStatus({
          tone: "success",
          title: "Cart updated",
          detail: result.json.message || "The scanned item was added to the cart.",
        });
      };

      const addQuickItem = async () => {
        const result = await request("/api/pos/cart/items", { session_token: token, item_id: "prayer-card-50c", quantity: 1 });
        if (!result.ok) return;
        applyCart(result.json);
        setLastSale(null);
        setStatus({
          tone: "success",
          title: "Quick item added",
          detail: result.json.message || "Prayer Card was added to the cart.",
        });
      };

      const completePayment = async (url, payload, fallbackTitle) => {
        const result = await request(url, payload);
        if (!result.ok) return;
        setLastSale(result.json);
        setCart([]);
        setTotal(0);
        const tone = result.json.status === "iou" ? "warning" : "success";
        const detailParts = [
          `Total ${money(result.json.total_cents || 0)}`,
          result.json.change_due_cents ? `Change ${money(result.json.change_due_cents)}` : "",
          result.json.donation_cents ? `Donation ${money(result.json.donation_cents)}` : "",
        ].filter(Boolean);
        setStatus({
          tone,
          title: result.json.message || fallbackTitle,
          detail: detailParts.join(" · ") || "Payment completed.",
        });
      };

      const statusClass = `status-panel ${status.tone === "success" ? "status-success" : status.tone === "danger" ? "status-danger" : "status-warning"}`;

      return html`
        <main class="pos-wrap">
          <section class="hero">
            <p class="kicker">Point of Sale</p>
            <h1>Scriptorium POS</h1>
            <p>Large controls, visible totals, and readable outcomes for Sunday-rush operation.</p>
            <div style=${{ marginTop: "14px", display: "flex", gap: "10px", flexWrap: "wrap" }}>
              <span class="session-pill">${token ? `Session ${token}` : "Session offline"}</span>
              <span class="session-pill">Checkout-ready</span>
            </div>
          </section>
          <section class="card">
            <h2 class="section-title">Shift</h2>
            <button class="pos-button--lg pos-button--gold" onClick=${startShift}>Start Shift</button>
          </section>
          <section class="card">
            <h2 class="section-title">Scan by ISBN or barcode</h2>
            <label class="field-label" for="barcode">Barcode</label>
            <input id="barcode" value=${barcode} onInput=${(e) => setBarcode(e.target.value)} />
            <div class="actions" style=${{ marginTop: "10px" }}>
              <button class="pos-button--lg" onClick=${scanItem}>Scan Item</button>
              <p class="hint">The UI now posts `isbn`, and the API accepts both `isbn` and `barcode` for compatibility.</p>
            </div>
          </section>
          <section class="card">
            <h2 class="section-title">Quick items</h2>
            <div class="quick-grid">
              <button class="quick-tile" onClick=${addQuickItem}>
                <span class="quick-emoji">🙏</span>
                Prayer Card
                <div class="cart-meta">$0.50</div>
              </button>
              <button class="quick-tile" onClick=${() => setStatus({ tone: "warning", title: "More quick items pending", detail: "The service already supports quick-item APIs; this screen currently exposes the seeded prayer card tile." })}>
                <span class="quick-emoji">🕯️</span>
                Add more tiles
                <div class="cart-meta">Design parity still open</div>
              </button>
            </div>
          </section>
          <section class="card">
            <h2 class="section-title">Cart</h2>
            ${cart.length ? html`
              <div class="cart-list">
                ${cart.map((item) => html`
                  <div class="cart-row" key=${item.item_id}>
                    <div>
                      <div class="cart-title">${item.title}</div>
                      <div class="cart-meta">Qty ${item.quantity} · ${item.is_quick_item ? "Quick item" : "Scanned item"}</div>
                    </div>
                    <div class="cart-price">${money(item.unit_price_cents * item.quantity)}</div>
                  </div>
                `)}
              </div>
            ` : html`<div class="empty-state">Cart is empty. Scan a book or tap a quick item to start the sale.</div>`}
            <div class="totals" style=${{ marginTop: "12px" }}>
              <div class="totals-row"><span>Current total</span><strong>${money(total)}</strong></div>
            </div>
          </section>
          <section class="card">
            <h2 class="section-title">Payments</h2>
            <div class="row">
              <button class="pos-button--lg pos-button--success" onClick=${() => completePayment("/api/pos/payments/cash", { session_token: token, tendered_cents: 2000, donate_change: true }, "Cash sale complete")}>Pay Cash</button>
              <button class="pos-button--lg" onClick=${() => completePayment("/api/pos/payments/external-card", { session_token: token, external_ref: "square-ui" }, "Card sale complete")}>Pay Card</button>
            </div>
            <div class="row">
              <button class="pos-button--lg" onClick=${() => completePayment("/api/pos/payments/iou", { session_token: token, customer_name: "Walk In" }, "Sale moved to IOU")}>Put on IOU</button>
              <button class="pos-button--lg pos-button--gold" onClick=${() => setBarcode("9780060652937")}>Reload sample ISBN</button>
            </div>
          </section>
          <section class="card">
            <h2 class="section-title">Outcome</h2>
            <div class=${statusClass}>
              <h3>${status.title}</h3>
              <p>${status.detail}</p>
            </div>
            ${lastSale ? html`
              <div class="totals" style=${{ marginTop: "12px" }}>
                <div class="totals-row"><span>Last sale total</span><strong>${money(lastSale.total_cents || 0)}</strong></div>
                <div class="totals-row"><span>Change due</span><span>${money(lastSale.change_due_cents || 0)}</span></div>
                <div class="totals-row"><span>Donation</span><span>${money(lastSale.donation_cents || 0)}</span></div>
              </div>
            ` : null}
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

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    error: String,
    message: String,
}

impl ApiError {
    fn new(status: StatusCode, message: impl Into<String>) -> Self {
        let message = message.into();
        let error = status
            .canonical_reason()
            .unwrap_or("request failed")
            .to_ascii_lowercase()
            .replace(' ', "_");
        Self { status, error, message }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorResponse { error: self.error, message: self.message }),
        )
            .into_response()
    }
}

async fn pos_login(
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

#[derive(Debug, Deserialize)]
struct PosScanRequest {
    session_token: String,
    #[serde(alias = "isbn")]
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
    message: String,
    total_cents: i64,
    change_due_cents: i64,
    donation_cents: i64,
    items: Vec<PosCartItemResponse>,
}

#[derive(Debug, Serialize)]
struct PosCartItemResponse {
    item_id: String,
    title: String,
    unit_price_cents: i64,
    quantity: i64,
    is_quick_item: bool,
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

fn log_checkout_event(
    event: &str,
    status: &str,
    payment_method: &str,
    total_cents: i64,
    started_at: Instant,
) {
    tracing::info!(
        event = event,
        status = status,
        payment_method = payment_method,
        total_cents = total_cents,
        latency_ms = started_at.elapsed().as_millis() as u64,
        "checkout event"
    );
}

fn pos_items(items: Vec<PosCartItem>) -> Vec<PosCartItemResponse> {
    items
        .into_iter()
        .map(|item| PosCartItemResponse {
            item_id: item.item_id,
            title: item.title,
            unit_price_cents: item.unit_price_cents,
            quantity: item.quantity,
            is_quick_item: item.is_quick_item,
        })
        .collect()
}

fn pos_cart_response(snapshot: PosCartSnapshot, message: impl Into<String>) -> PosResponse {
    PosResponse {
        status: "cart_updated",
        message: message.into(),
        total_cents: snapshot.total_cents,
        change_due_cents: 0,
        donation_cents: 0,
        items: pos_items(snapshot.items),
    }
}

async fn storefront_checkout_session(
    State(state): State<AppState>,
    Json(request): Json<StorefrontCheckoutSessionRequest>,
) -> Result<Json<StorefrontCheckoutSessionResponse>, axum::http::StatusCode> {
    let started_at = Instant::now();
    let session = state
        .storefront
        .create_checkout_session(request.total_cents, request.email)
        .await
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    log_checkout_event(
        "storefront_checkout_session",
        "created",
        "online_card",
        session.total_cents,
        started_at,
    );
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

#[derive(Debug, Deserialize)]
struct AdminIsbnLookupRequest {
    token: String,
    isbn: String,
}

#[derive(Debug, Serialize)]
struct AdminIsbnLookupResponse {
    isbn: String,
    title: String,
    author: String,
    description: String,
}

#[derive(Debug, Deserialize)]
struct AdminInventoryReceiveRequest {
    token: String,
    tenant_id: String,
    isbn: String,
    quantity: i64,
}

#[derive(Debug, Serialize)]
struct AdminInventoryReceiveResponse {
    tenant_id: String,
    isbn: String,
    on_hand: i64,
}

#[derive(Debug, Deserialize)]
struct AdminAuthLoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct AdminAuthLoginResponse {
    token: String,
    tenant_id: String,
    role: &'static str,
}

#[derive(Debug, Deserialize)]
struct AdminInventoryAdjustRequest {
    token: String,
    tenant_id: String,
    isbn: String,
    delta: i64,
    reason: String,
}

#[derive(Debug, Serialize)]
struct AdminStockMovementResponse {
    tenant_id: String,
    isbn: String,
    delta: i64,
    reason: String,
}

#[derive(Debug, Deserialize)]
struct AdminProductUpsertRequest {
    token: String,
    tenant_id: String,
    product_id: String,
    title: String,
    isbn: String,
    category: String,
    vendor: String,
    cost_cents: i64,
    retail_cents: i64,
}

#[derive(Debug, Serialize)]
struct AdminProductResponse {
    tenant_id: String,
    product_id: String,
    title: String,
    isbn: String,
    category: String,
    vendor: String,
    cost_cents: i64,
    retail_cents: i64,
}

#[derive(Debug, Serialize)]
struct AdminDeleteResponse {
    status: &'static str,
}

#[derive(Debug, Serialize)]
struct AdminTaxonomyListResponse {
    tenant_id: String,
    values: Vec<String>,
}

#[derive(Debug, Serialize)]
struct AdminReportSummaryResponse {
    tenant_id: String,
    sales_cents: i64,
    donations_cents: i64,
    cogs_cents: i64,
    gross_profit_cents: i64,
    sales_by_payment: Vec<(String, i64)>,
}

#[derive(Debug, Serialize)]
struct AdminOrderResponse {
    order_id: String,
    tenant_id: String,
    customer_name: String,
    channel: String,
    status: String,
    payment_method: String,
    total_cents: i64,
    created_on: String,
}

fn admin_order_response(order: AdminOrder) -> AdminOrderResponse {
    AdminOrderResponse {
        order_id: order.order_id,
        tenant_id: order.tenant_id,
        customer_name: order.customer_name,
        channel: order.channel,
        status: order.status,
        payment_method: order.payment_method,
        total_cents: order.total_cents,
        created_on: order.created_on,
    }
}

async fn payments_webhook(
    State(state): State<AppState>,
    Json(request): Json<PaymentsWebhookRequest>,
) -> Result<Json<PaymentsWebhookResponse>, axum::http::StatusCode> {
    let started_at = Instant::now();
    let result = state
        .storefront
        .finalize_webhook(&request.external_ref, &request.session_id)
        .await
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    if result.status == WebhookFinalizeStatus::Processed {
        state
            .admin
            .create_order(
                "church-a",
                "Online Customer",
                "Online",
                "Paid",
                "online_card",
                0,
                &current_utc_date(),
            )
            .await;
        state
            .admin
            .record_sales_event(SalesEvent {
                tenant_id: "church-a".to_string(),
                payment_method: "online_card".to_string(),
                sales_cents: 0,
                donations_cents: 0,
                cogs_cents: 0,
                occurred_on: current_utc_date(),
            })
            .await;
    }
    log_checkout_event(
        "payment_webhook_finalize",
        match result.status {
            WebhookFinalizeStatus::Processed => "processed",
            WebhookFinalizeStatus::Duplicate => "duplicate",
        },
        "online_card",
        0,
        started_at,
    );
    Ok(Json(PaymentsWebhookResponse {
        status: match result.status {
            WebhookFinalizeStatus::Processed => "processed",
            WebhookFinalizeStatus::Duplicate => "duplicate",
        },
        receipt_sent: result.receipt_sent,
    }))
}

async fn admin_isbn_lookup(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<AdminIsbnLookupRequest>,
) -> Result<Json<AdminIsbnLookupResponse>, axum::http::StatusCode> {
    require_same_origin(&headers)?;
    state
        .admin
        .require_admin(&request.token)
        .await
        .map_err(|_| axum::http::StatusCode::UNAUTHORIZED)?;
    let metadata = state
        .admin
        .lookup_isbn(&request.isbn)
        .await
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    Ok(Json(AdminIsbnLookupResponse {
        isbn: metadata.isbn,
        title: metadata.title,
        author: metadata.author,
        description: metadata.description,
    }))
}

async fn admin_inventory_receive(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<AdminInventoryReceiveRequest>,
) -> Result<Json<AdminInventoryReceiveResponse>, axum::http::StatusCode> {
    require_same_origin(&headers)?;
    let session = state
        .admin
        .require_admin(&request.token)
        .await
        .map_err(|_| axum::http::StatusCode::UNAUTHORIZED)?;
    if session.tenant_id != request.tenant_id {
        return Err(axum::http::StatusCode::FORBIDDEN);
    }
    let receipt = state
        .admin
        .receive_inventory(&request.tenant_id, &request.isbn, request.quantity)
        .await
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    Ok(Json(AdminInventoryReceiveResponse {
        tenant_id: receipt.tenant_id,
        isbn: receipt.isbn,
        on_hand: receipt.on_hand,
    }))
}

async fn admin_inventory_adjust(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<AdminInventoryAdjustRequest>,
) -> Result<Json<AdminInventoryReceiveResponse>, axum::http::StatusCode> {
    require_same_origin(&headers)?;
    let session = state
        .admin
        .require_admin(&request.token)
        .await
        .map_err(|_| axum::http::StatusCode::UNAUTHORIZED)?;
    if session.tenant_id != request.tenant_id {
        return Err(axum::http::StatusCode::FORBIDDEN);
    }
    let receipt = state
        .admin
        .adjust_inventory(&request.tenant_id, &request.isbn, request.delta, &request.reason)
        .await
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    Ok(Json(AdminInventoryReceiveResponse {
        tenant_id: receipt.tenant_id,
        isbn: receipt.isbn,
        on_hand: receipt.on_hand,
    }))
}

async fn admin_inventory_journal(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<AdminStockMovementResponse>>, axum::http::StatusCode> {
    let token = bearer_token(&headers)?;
    let tenant_id = params.get("tenant_id").map_or("default", String::as_str);
    let session = state
        .admin
        .require_admin(&token)
        .await
        .map_err(|_| axum::http::StatusCode::UNAUTHORIZED)?;
    if session.tenant_id != tenant_id {
        return Err(axum::http::StatusCode::FORBIDDEN);
    }
    let items = state
        .admin
        .movement_journal(tenant_id)
        .await
        .into_iter()
        .map(|movement| AdminStockMovementResponse {
            tenant_id: movement.tenant_id,
            isbn: movement.isbn,
            delta: movement.delta,
            reason: movement.reason,
        })
        .collect();
    Ok(Json(items))
}

async fn admin_auth_login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<AdminAuthLoginRequest>,
) -> Result<Json<AdminAuthLoginResponse>, axum::http::StatusCode> {
    require_same_origin(&headers)?;
    let session = state
        .admin
        .login(&request.username, &request.password)
        .await
        .map_err(|_| axum::http::StatusCode::UNAUTHORIZED)?;
    Ok(Json(AdminAuthLoginResponse {
        token: session.token,
        tenant_id: session.tenant_id,
        role: match session.role {
            AdminRole::Admin => "admin",
            AdminRole::Volunteer => "volunteer",
        },
    }))
}

async fn admin_product_upsert(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<AdminProductUpsertRequest>,
) -> Result<Json<AdminProductResponse>, axum::http::StatusCode> {
    require_same_origin(&headers)?;
    let session = state
        .admin
        .require_admin(&request.token)
        .await
        .map_err(|_| axum::http::StatusCode::UNAUTHORIZED)?;
    if session.tenant_id != request.tenant_id {
        return Err(axum::http::StatusCode::FORBIDDEN);
    }
    let product = AdminProduct {
        tenant_id: request.tenant_id,
        product_id: request.product_id,
        title: request.title,
        isbn: request.isbn,
        category: request.category,
        vendor: request.vendor,
        cost_cents: request.cost_cents,
        retail_cents: request.retail_cents,
    };
    state
        .admin
        .upsert_product(product.clone())
        .await
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    Ok(Json(AdminProductResponse {
        tenant_id: product.tenant_id,
        product_id: product.product_id,
        title: product.title,
        isbn: product.isbn,
        category: product.category,
        vendor: product.vendor,
        cost_cents: product.cost_cents,
        retail_cents: product.retail_cents,
    }))
}

async fn admin_product_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<AdminProductResponse>>, axum::http::StatusCode> {
    let token = bearer_token(&headers)?;
    let tenant_id = params.get("tenant_id").map_or("default", String::as_str);
    let session = state
        .admin
        .require_admin(&token)
        .await
        .map_err(|_| axum::http::StatusCode::UNAUTHORIZED)?;
    if session.tenant_id != tenant_id {
        return Err(axum::http::StatusCode::FORBIDDEN);
    }
    let products = state
        .admin
        .list_products(tenant_id)
        .await
        .into_iter()
        .map(|product| AdminProductResponse {
            tenant_id: product.tenant_id,
            product_id: product.product_id,
            title: product.title,
            isbn: product.isbn,
            category: product.category,
            vendor: product.vendor,
            cost_cents: product.cost_cents,
            retail_cents: product.retail_cents,
        })
        .collect();
    Ok(Json(products))
}

async fn admin_product_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Path(product_id): axum::extract::Path<String>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<AdminDeleteResponse>, axum::http::StatusCode> {
    require_same_origin(&headers)?;
    let token = bearer_token(&headers)?;
    let tenant_id = params.get("tenant_id").map_or("default", String::as_str);
    let session = state
        .admin
        .require_admin(&token)
        .await
        .map_err(|_| axum::http::StatusCode::UNAUTHORIZED)?;
    if session.tenant_id != tenant_id {
        return Err(axum::http::StatusCode::FORBIDDEN);
    }
    state
        .admin
        .delete_product(tenant_id, &product_id)
        .await
        .map_err(|_| axum::http::StatusCode::NOT_FOUND)?;
    Ok(Json(AdminDeleteResponse { status: "deleted" }))
}

fn require_same_origin(headers: &HeaderMap) -> Result<(), StatusCode> {
    let origin = headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.is_empty());
    let host = headers.get(header::HOST).and_then(|value| value.to_str().ok()).unwrap_or("");

    if let Some(origin) = origin {
        let expected_http = format!("http://{host}");
        let expected_https = format!("https://{host}");
        if origin != expected_http && origin != expected_https {
            return Err(StatusCode::FORBIDDEN);
        }
    }
    Ok(())
}

async fn admin_categories_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<AdminTaxonomyListResponse>, axum::http::StatusCode> {
    let token = bearer_token(&headers)?;
    let tenant_id = params.get("tenant_id").map_or("default", String::as_str);
    let session = state
        .admin
        .require_admin(&token)
        .await
        .map_err(|_| axum::http::StatusCode::UNAUTHORIZED)?;
    if session.tenant_id != tenant_id {
        return Err(axum::http::StatusCode::FORBIDDEN);
    }
    let values = state.admin.list_categories(tenant_id).await;
    Ok(Json(AdminTaxonomyListResponse { tenant_id: tenant_id.to_string(), values }))
}

async fn admin_vendors_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<AdminTaxonomyListResponse>, axum::http::StatusCode> {
    let token = bearer_token(&headers)?;
    let tenant_id = params.get("tenant_id").map_or("default", String::as_str);
    let session = state
        .admin
        .require_admin(&token)
        .await
        .map_err(|_| axum::http::StatusCode::UNAUTHORIZED)?;
    if session.tenant_id != tenant_id {
        return Err(axum::http::StatusCode::FORBIDDEN);
    }
    let values = state.admin.list_vendors(tenant_id).await;
    Ok(Json(AdminTaxonomyListResponse { tenant_id: tenant_id.to_string(), values }))
}

async fn admin_orders_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<AdminOrderResponse>>, axum::http::StatusCode> {
    let token = bearer_token(&headers)?;
    let tenant_id = params.get("tenant_id").map_or("default", String::as_str);
    let session = state
        .admin
        .require_admin(&token)
        .await
        .map_err(|_| axum::http::StatusCode::UNAUTHORIZED)?;
    if session.tenant_id != tenant_id {
        return Err(axum::http::StatusCode::FORBIDDEN);
    }
    let orders = state
        .admin
        .list_orders(tenant_id)
        .await
        .into_iter()
        .map(admin_order_response)
        .collect();
    Ok(Json(orders))
}

async fn admin_order_mark_paid(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Path(order_id): axum::extract::Path<String>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<AdminOrderResponse>, axum::http::StatusCode> {
    require_same_origin(&headers)?;
    let token = bearer_token(&headers)?;
    let tenant_id = params.get("tenant_id").map_or("default", String::as_str);
    let session = state
        .admin
        .require_admin(&token)
        .await
        .map_err(|_| axum::http::StatusCode::UNAUTHORIZED)?;
    if session.tenant_id != tenant_id {
        return Err(axum::http::StatusCode::FORBIDDEN);
    }
    let order = state
        .admin
        .mark_order_paid(tenant_id, &order_id)
        .await
        .map_err(|_| axum::http::StatusCode::NOT_FOUND)?;
    Ok(Json(admin_order_response(order)))
}

async fn admin_report_summary(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<AdminReportSummaryResponse>, axum::http::StatusCode> {
    let token = bearer_token(&headers)?;
    let tenant_id = params.get("tenant_id").map_or("default", String::as_str);
    let session = state
        .admin
        .require_admin(&token)
        .await
        .map_err(|_| axum::http::StatusCode::UNAUTHORIZED)?;
    if session.tenant_id != tenant_id {
        return Err(axum::http::StatusCode::FORBIDDEN);
    }
    let from = params.get("from").map(String::as_str);
    let to = params.get("to").map(String::as_str);
    if from.is_some_and(|date| !is_valid_iso_date(date))
        || to.is_some_and(|date| !is_valid_iso_date(date))
    {
        return Err(axum::http::StatusCode::BAD_REQUEST);
    }
    let report = state.admin.report_summary_range(tenant_id, from, to).await;
    Ok(Json(AdminReportSummaryResponse {
        tenant_id: report.tenant_id,
        sales_cents: report.sales_cents,
        donations_cents: report.donations_cents,
        cogs_cents: report.cogs_cents,
        gross_profit_cents: report.gross_profit_cents,
        sales_by_payment: report.sales_by_payment,
    }))
}

async fn i18n_lookup(
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Json<std::collections::HashMap<String, String>> {
    let locale = params.get("locale").map_or("en-AU", String::as_str);
    let key = params.get("key").map_or("checkout.complete", String::as_str);
    let value = match (locale, key) {
        ("en-AU", "checkout.complete") => "Sale Complete",
        ("el-GR", "checkout.complete") => "Η πώληση ολοκληρώθηκε",
        ("en-AU", "admin.intake.title") => "Admin Inventory Intake",
        ("el-GR", "admin.intake.title") => "Παραλαβή αποθέματος διαχειριστή",
        ("en-AU", "storefront.checkout.title") => "Checkout",
        ("el-GR", "storefront.checkout.title") => "Ταμείο",
        (_, "checkout.complete") => "Sale Complete",
        _ => key,
    };
    let mut payload = std::collections::HashMap::new();
    payload.insert("locale".to_string(), locale.to_string());
    payload.insert("key".to_string(), key.to_string());
    payload.insert("value".to_string(), value.to_string());
    Json(payload)
}

async fn pos_scan(
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

async fn pos_quick_item(
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

async fn pos_pay_cash(
    State(state): State<AppState>,
    Json(request): Json<PosCashPaymentRequest>,
) -> Result<Json<PosResponse>, ApiError> {
    let started_at = Instant::now();
    let receipt = state
        .pos
        .checkout_cash(&request.session_token, request.tendered_cents, request.donate_change)
        .await
        .map_err(|err| ApiError::new(StatusCode::BAD_REQUEST, err.to_string()))?;
    state
        .admin
        .record_sales_event(SalesEvent {
            tenant_id: "church-a".to_string(),
            payment_method: "cash".to_string(),
            sales_cents: receipt.total_cents,
            donations_cents: receipt.donation_cents,
            cogs_cents: receipt.total_cents / 2,
            occurred_on: current_utc_date(),
        })
        .await;
    state
        .admin
        .create_order(
            "church-a",
            "Walk In",
            "POS",
            "Paid",
            "cash",
            receipt.total_cents,
            &current_utc_date(),
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
        items: Vec::new(),
    }))
}

async fn pos_pay_external_card(
    State(state): State<AppState>,
    Json(request): Json<PosExternalCardRequest>,
) -> Result<Json<PosResponse>, ApiError> {
    let started_at = Instant::now();
    let receipt = state
        .pos
        .checkout_external_card(&request.session_token, &request.external_ref)
        .await
        .map_err(|err| ApiError::new(StatusCode::BAD_REQUEST, err.to_string()))?;
    state
        .admin
        .record_sales_event(SalesEvent {
            tenant_id: "church-a".to_string(),
            payment_method: "external_card".to_string(),
            sales_cents: receipt.total_cents,
            donations_cents: receipt.donation_cents,
            cogs_cents: receipt.total_cents / 2,
            occurred_on: current_utc_date(),
        })
        .await;
    state
        .admin
        .create_order(
            "church-a",
            "Walk In",
            "POS",
            "Paid",
            "external_card",
            receipt.total_cents,
            &current_utc_date(),
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
        items: Vec::new(),
    }))
}

async fn pos_pay_iou(
    State(state): State<AppState>,
    Json(request): Json<PosIouRequest>,
) -> Result<Json<PosResponse>, ApiError> {
    let started_at = Instant::now();
    let receipt = state
        .pos
        .checkout_iou(&request.session_token, &request.customer_name)
        .await
        .map_err(|err| ApiError::new(StatusCode::BAD_REQUEST, err.to_string()))?;
    state
        .admin
        .create_order(
            "church-a",
            &request.customer_name,
            "POS",
            "UnpaidIou",
            "iou",
            receipt.total_cents,
            &current_utc_date(),
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
        items: Vec::new(),
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

fn bearer_token(headers: &HeaderMap) -> Result<String, StatusCode> {
    let raw = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let token = raw.strip_prefix("Bearer ").ok_or(StatusCode::UNAUTHORIZED)?;
    if token.is_empty() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(token.to_string())
}

fn current_utc_date() -> String {
    chrono::Utc::now().date_naive().format("%Y-%m-%d").to_string()
}

fn is_valid_iso_date(input: &str) -> bool {
    chrono::NaiveDate::parse_from_str(input, "%Y-%m-%d").is_ok()
}
