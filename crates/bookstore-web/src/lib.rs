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
    category: Option<String>,
    page: Option<usize>,
}

fn site_nav(current: &str) -> String {
    let nav_link = |href: &str, label: &str, key: &str| {
        let class_name = if current == key { "site-nav__link site-nav__link--active" } else { "site-nav__link" };
        format!("<a class=\"{}\" href=\"{}\">{}</a>", class_name, href, label)
    };
    [
        "<header class=\"site-nav\"><div class=\"site-nav__inner\"><a class=\"site-nav__brand\" href=\"/catalog\">Scriptorium</a><nav class=\"site-nav__links\" aria-label=\"Primary\">",
        &nav_link("/catalog", "Catalog", "catalog"),
        &nav_link("/cart", "Cart", "cart"),
        &nav_link("/checkout", "Checkout", "checkout"),
        &nav_link("/admin", "Admin", "admin"),
        &nav_link("/admin/intake", "Intake", "intake"),
        "</nav></div></header>",
    ]
    .concat()
}

fn site_footer() -> &'static str {
    "<footer class=\"site-footer\"><div class=\"site-footer__inner\"><p>Scriptorium supports parish browsing, intake, and Sunday-close reconciliation with one shared surface.</p><div class=\"site-footer__links\"><a href=\"/catalog\">Catalog</a><a href=\"/cart\">Cart</a><a href=\"/admin\">Admin</a></div></div></footer>"
}

fn stock_hint(book_id: &str) -> (&'static str, &'static str) {
    match book_id {
        "bk-106" => ("Only 2 left", "stock-badge stock-badge--warning"),
        "bk-105" => ("Only 3 left", "stock-badge stock-badge--warning"),
        "bk-104" => ("Out of stock", "stock-badge stock-badge--danger"),
        _ => ("In stock", "stock-badge stock-badge--success"),
    }
}

async fn storefront_catalog(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<CatalogQuery>,
) -> Html<String> {
    let books = state.catalog.list_books().await;
    let categories = catalog_categories(&books);
    let filtered_books = filter_books(books, query.q.as_deref(), query.category.as_deref());
    let page = query.page.unwrap_or(1).max(1);
    let per_page = 6usize;
    let total_pages = filtered_books.len().max(1).div_ceil(per_page);
    let page = page.min(total_pages);
    let start = (page - 1) * per_page;
    let paged_books = filtered_books.iter().skip(start).take(per_page).cloned().collect::<Vec<_>>();
    let items = render_catalog_cards(paged_books);
    let pagination = render_catalog_pagination(
        page,
        total_pages,
        query.q.as_deref(),
        query.category.as_deref(),
    );
    let category_chips = render_catalog_category_chips(
        &categories,
        query.q.as_deref(),
        query.category.as_deref(),
        &filtered_books,
    );
    let search_value = html_escape(query.q.as_deref().unwrap_or(""));
    let active_category =
        query.category.as_deref().filter(|value| !value.trim().is_empty()).unwrap_or("All");
    Html(
        [
            "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\" /><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" /><title>Scriptorium Catalog</title>",
            google_fonts_link(),
            "<script src=\"https://unpkg.com/htmx.org@2.0.4\"></script><style>",
            shared_styles(),
            "</style></head><body class=\"page-shell\">",
            &site_nav("catalog"),
            "<main class=\"page-stack page-stack--wide\"><section class=\"hero-card\"><div><p class=\"eyebrow\">Storefront</p><h1 class=\"display-title\">Browse the shelves</h1><p class=\"lede\">Find books for parish reading, gifting, and liturgical practice.</p><div class=\"eyebrow-row\"><span class=\"hero-chip hero-chip--gold\">Parish bookshop</span><span class=\"hero-chip\">Curated titles</span><span class=\"hero-chip\">Warm, accessible checkout</span></div></div><div class=\"hero-actions\"><a class=\"ghost-link\" href=\"/cart\">Cart</a><a class=\"ghost-link\" href=\"/checkout\">Checkout</a></div></section><section class=\"surface-card\"><form class=\"catalog-search\" action=\"/catalog\" method=\"get\" hx-get=\"/catalog/search\" hx-target=\"#results\" hx-push-url=\"true\"><label class=\"field-label\" for=\"catalog-search\">Search catalog</label><input type=\"hidden\" name=\"category\" value=\"",
            &html_escape(query.category.as_deref().unwrap_or("")),
            "\" /><div class=\"catalog-search-row\"><input id=\"catalog-search\" name=\"q\" value=\"",
            &search_value,
            "\" placeholder=\"Try Discipline or Foster\" /><button class=\"accent-button\" type=\"submit\">Search</button></div></form><div class=\"category-strip\">",
            &category_chips,
            "</div><div class=\"catalog-results-head\"><p class=\"helper-copy helper-copy--flush\">Active shelf: ",
            &html_escape(active_category),
            "</p><strong>",
            &format!("{} titles", filtered_books.len()),
            "</strong></div><div id=\"catalog-feedback\" class=\"notice-panel\">Add directly from the shelf cards or open a title for more context.</div><div id=\"results\">",
            &items,
            "</div>",
            &pagination,
            "</section></main>",
            site_footer(),
            storefront_cart_script(),
            "</body></html>",
        ]
        .concat(),
    )
}

async fn storefront_search(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Html<String> {
    let query = params.get("q").map_or("", String::as_str).to_ascii_lowercase();
    let category = params.get("category").map(String::as_str);
    let books = state.catalog.list_books().await;
    let filtered = render_catalog_cards(filter_books(books, Some(&query), category));
    Html(filtered)
}

async fn storefront_product_detail(
    State(state): State<AppState>,
    axum::extract::Path(book_id): axum::extract::Path<String>,
) -> (StatusCode, Html<String>) {
    let books = state.catalog.list_books().await;
    let Some(book) = books.iter().find(|book| book.id == book_id).cloned() else {
        return (
            StatusCode::NOT_FOUND,
            Html(
                [
                    "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\" /><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" /><title>Product Not Found</title>",
                    google_fonts_link(),
                    "<style>",
                    shared_styles(),
                    "</style></head><body class=\"page-shell\">",
                    &site_nav("catalog"),
                    "<main class=\"page-stack\"><section class=\"hero-card\"><div><p class=\"eyebrow\">Product Detail</p><h1 class=\"display-title\">Title not found</h1><p class=\"lede\">That catalog item is not available in this parish shelf view. Return to browsing and choose another title.</p><div class=\"eyebrow-row\"><span class=\"hero-chip hero-chip--gold\">404</span><span class=\"hero-chip\">Friendly fallback</span></div></div><div class=\"hero-actions\"><a class=\"ghost-link\" href=\"/catalog\">Back to catalog</a><a class=\"ghost-link\" href=\"/cart\">Open cart</a></div></section><section class=\"surface-card\"><h2 class=\"section-title\">We could not find that product</h2><p class=\"helper-copy helper-copy--flush\">The requested book id does not exist in the seeded catalog. Try the main shelf, search by title, or continue with another selection.</p></section></main>",
                    site_footer(),
                    "</body></html>",
                ]
                .concat(),
            ),
        );
    };
    let related_books = books
        .iter()
        .filter(|candidate| candidate.id != book.id)
        .filter(|candidate| candidate.category == book.category || candidate.author != book.author)
        .take(2)
        .map(|candidate| {
            format!(
                "<div class=\"list-row list-row--soft\"><div><div class=\"list-title\">{}</div><div class=\"list-meta\">{} · {}</div></div><div class=\"button-row button-row--compact\"><a class=\"ghost-link ghost-link--ink ghost-link--mini\" href=\"/catalog/items/{}\">View</a><button class=\"primary-button primary-button--sm\" type=\"button\" data-add-book-id=\"{}\" data-add-book-title=\"{}\" data-add-book-author=\"{}\" data-add-book-price-cents=\"{}\" data-feedback-target=\"cart-feedback\">Add</button></div></div>",
                html_escape(&candidate.title),
                html_escape(&candidate.author),
                html_escape(&candidate.category),
                html_escape(&candidate.id),
                html_escape(&candidate.id),
                html_escape(&candidate.title),
                html_escape(&candidate.author),
                i64::from(candidate.price_cents),
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let price = format_money(i64::from(book.price_cents));
    let (stock_label, stock_class) = stock_hint(&book.id);
    (
        StatusCode::OK,
        Html(
        [
            "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\" /><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" /><title>Scriptorium Product</title>",
            google_fonts_link(),
            "<style>",
            shared_styles(),
            "</style></head><body class=\"page-shell\">",
            &site_nav("catalog"),
            "<main class=\"page-stack page-stack--wide\"><section class=\"hero-card\"><div><p class=\"eyebrow\">Product Detail</p><h1 class=\"display-title\">",
            &html_escape(&book.title),
            "</h1><p class=\"lede\">by ",
            &html_escape(&book.author),
            "</p><div class=\"eyebrow-row\"><span class=\"hero-chip hero-chip--gold\">Reader favorite</span><span class=\"hero-chip\">Shelf-ready gift</span></div></div><div class=\"hero-actions\"><a class=\"ghost-link\" href=\"/catalog\">Back to catalog</a><a class=\"ghost-link\" href=\"/cart\">Cart</a></div></section><section class=\"product-layout\"><article class=\"surface-card\"><div class=\"catalog-cover catalog-cover--detail\"><div class=\"book-cover-art\"><span class=\"book-cover-art__eyebrow\">Parish shelf edition</span><strong>",
            &html_escape(&book.title),
            "</strong><span>",
            &html_escape(&book.author),
            "</span></div></div><div class=\"pilgrim-panel\"><h3>Pilgrim note</h3><p>Selected for the parish shelf because it is readable, giftable, and easy to recommend after services.</p></div></article><article class=\"surface-card\"><span class=\"chip\">",
            &html_escape(&book.category),
            "</span><div class=\"button-row button-row--compact button-row--flush-start\"><span class=\"",
            stock_class,
            "\">",
            stock_label,
            "</span></div><h2 class=\"section-title\">",
            &html_escape(&book.title),
            "</h2><p class=\"catalog-meta\">",
            &html_escape(&book.author),
            "</p><div class=\"detail-price\">",
            &price,
            "</div><p class=\"helper-copy\">Choose a quantity, add this title to the basket, or continue to checkout when you are ready.</p><div class=\"product-meta-grid\"><div class=\"meta-tile\"><span>Description</span><strong>Curated for parish shelves, gift tables, and post-liturgy recommendation.</strong></div><div class=\"meta-tile\"><span>ISBN</span><strong>",
            &html_escape(&format!("978-0-{}-{}", &book.id.replace("bk-", "84"), &book.price_cents)),
            "</strong></div><div class=\"meta-tile\"><span>Binding</span><strong>Softcover</strong></div><div class=\"meta-tile\"><span>Placement</span><strong>Front display table</strong></div></div><div class=\"inline-quantity\"><div><label class=\"field-label\" for=\"detail-quantity\">Quantity</label><input id=\"detail-quantity\" type=\"number\" min=\"1\" value=\"1\" /></div><div class=\"button-row button-row--compact\"><button class=\"primary-button\" type=\"button\" data-add-book-id=\"",
            &html_escape(&book.id),
            "\" data-add-book-title=\"",
            &html_escape(&book.title),
            "\" data-add-book-author=\"",
            &html_escape(&book.author),
            "\" data-add-book-price-cents=\"",
            &i64::from(book.price_cents).to_string(),
            "\" data-add-book-quantity-target=\"detail-quantity\" data-feedback-target=\"cart-feedback\">Add to cart</button><a class=\"accent-button\" href=\"/checkout\">Go to checkout</a></div></div><div id=\"cart-feedback\" class=\"notice-panel\">Ready to add this title to the cart.</div><div class=\"divider-title divider-title--spaced\">Related titles</div><div class=\"stack-list\">",
            &if related_books.is_empty() {
                "<div class=\"empty-inline\">More curated shelf recommendations will appear here as the catalog grows.</div>".to_string()
            } else {
                related_books
            },
            "</div></article></section></main>",
            site_footer(),
            storefront_cart_script(),
            "</body></html>",
        ]
        .concat(),
        ),
    )
}

async fn storefront_cart(State(state): State<AppState>) -> Html<String> {
    let books = state.catalog.list_books().await;
    let recommendations = books
        .into_iter()
        .take(3)
        .map(|book| {
            format!(
                "<div class=\"list-row recommendation-row\" data-recommendation-book-id=\"{}\"><div><div class=\"list-title\">{}</div><div class=\"list-meta\">{} · {}</div></div><a class=\"ghost-link ghost-link--ink\" href=\"/catalog/items/{}\">View</a></div>",
                html_escape(&book.id),
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
            "</style></head><body class=\"page-shell\">",
            &site_nav("cart"),
            "<main class=\"page-stack page-stack--wide\"><section class=\"hero-card\"><div><p class=\"eyebrow\">Cart</p><h1 class=\"display-title\">Review your basket</h1><p class=\"lede\">Confirm quantities, keep gifting simple, and move smoothly into checkout.</p><div class=\"eyebrow-row\"><span class=\"hero-chip hero-chip--gold\">Gentle checkout</span><span class=\"hero-chip\">Parish-friendly copy</span></div></div><div class=\"hero-actions\"><a class=\"ghost-link\" href=\"/catalog\">Keep browsing</a><a class=\"ghost-link\" href=\"/checkout\">Checkout</a></div></section><section class=\"checkout-layout\"><article class=\"surface-card\"><h2 class=\"section-title\">Cart items</h2><div id=\"cart-items\" class=\"stack-list\"><div class=\"empty-inline\">Your cart is empty.</div></div></article><article class=\"surface-card\"><h2 class=\"section-title\">Recommended titles</h2><div id=\"cart-recommendations\" class=\"stack-list\">",
            &recommendations,
            "</div><div id=\"cart-recommendations-empty\" class=\"empty-inline\" hidden>Recommendations update automatically so titles already in the basket are not repeated here.</div><div class=\"notice-panel notice-panel--success\" id=\"cart-summary\">Cart total: $0.00</div><div class=\"button-row\"><button class=\"accent-button\" type=\"button\" id=\"clear-cart\">Clear basket</button><a class=\"primary-button\" href=\"/checkout\">Proceed to checkout</a></div><div class=\"pilgrim-panel\"><h3>Gift-table guidance</h3><p>Keep the basket light, visible, and easy to confirm. The current flow is optimized for quick parish purchases after liturgy.</p></div></article></section></main>",
            site_footer(),
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
            "</style></head><body class=\"page-shell\">",
            &site_nav("checkout"),
            r#"<main class="page-stack page-stack--wide"><section class="hero-card"><div><p class="eyebrow">Checkout</p><h1 class="display-title">Finish an online order</h1><p class="lede">Review the basket, choose any extra parish support, and hand off payment with confidence.</p><div class="eyebrow-row"><span class="hero-chip hero-chip--gold">Secure handoff</span><span class="hero-chip">Receipt-ready</span><span class="hero-chip">Confirmation state</span></div></div><div class="hero-actions"><a class="ghost-link" href="/cart">Back to cart</a><a class="ghost-link" href="/catalog">Continue shopping</a></div></section><section class="checkout-layout"><article class="surface-card"><h2 class="section-title">Contact and shipping</h2><div class="fieldset-grid"><div><label class="field-label" for="checkout-name">Full name</label><input id="checkout-name" placeholder="Jane Parishioner" value="Jane Parishioner" /></div><div><label class="field-label" for="checkout-email">Receipt email</label><input id="checkout-email" placeholder="reader@example.com" value="jane@example.com" /></div></div><label class="field-label" for="checkout-address">Address</label><textarea id="checkout-address" placeholder="123 Parish Lane, Melbourne VIC">123 Parish Lane, Melbourne VIC 3000</textarea><label class="field-label" for="checkout-note">Order note</label><textarea id="checkout-note" placeholder="Optional note for parish pickup, gifting, or follow-up."></textarea><label class="field-label" for="checkout-donation-select">Optional parish support</label><select id="checkout-donation-select"><option value="0">No extra support</option><option value="200">Round up with $2.00</option><option value="500">Add $5.00 support</option><option value="1000">Add $10.00 support</option></select><div class="pilgrim-panel"><h3>Payment handoff</h3><p>Create a secure payment session, then continue to receipt and fulfilment without losing the parish shopping context.</p></div><button class="primary-button" type="button" id="create-checkout-session">Create checkout session</button><p class="helper-copy">We will confirm the session id, receipt email, and final total here before you move on.</p><div id="checkout-status" class="notice-panel" aria-live="polite">Ready to create a checkout session.</div><div id="checkout-confirmation" class="surface-card" hidden><p class="divider-title">Order confirmation</p><h3 class="section-title">Session ready</h3><div class="stack-list stack-list--tight"><div class="list-row list-row--soft"><span>Session id</span><strong id="checkout-confirmation-session">-</strong></div><div class="list-row list-row--soft"><span>Receipt</span><strong id="checkout-confirmation-email">-</strong></div><div class="list-row list-row--soft"><span>Total handed off</span><strong id="checkout-confirmation-total">-</strong></div></div><div class="button-row"><a class="accent-button" href="/catalog">Keep shopping</a><a class="ghost-link ghost-link--ink" href="/cart">Review cart</a></div></div></article><article class="surface-card"><h2 class="section-title">Order summary</h2><div id="checkout-lines" class="stack-list"><div class="empty-inline">Your cart is empty.</div></div><div class="summary-row"><span>Subtotal</span><strong id="checkout-subtotal">$0.00</strong></div><div class="summary-row"><span>Shipping</span><strong id="checkout-shipping">$5.99</strong></div><div class="summary-row"><span>Tax</span><strong id="checkout-tax">$0.00</strong></div><div class="summary-row"><span>Parish support</span><strong id="checkout-donation">$0.00</strong></div><div class="summary-row summary-row--total"><span>Total</span><strong id="checkout-total">$0.00</strong></div></article></section></main>"#,
            site_footer(),
            r#"<script>const CART_KEY='scriptorium-storefront-cart';function readCart(){try{return JSON.parse(localStorage.getItem(CART_KEY)||'[]');}catch{return [];}}function money(cents){return `$${(Number(cents||0)/100).toFixed(2)}`;}function cartSubtotal(cart){return cart.reduce((sum,item)=>sum+(Number(item.price_cents||0)*Number(item.quantity||0)),0);}function currentDonation(){return Number(document.getElementById('checkout-donation-select')?.value||0);}function shippingCents(subtotal){return subtotal > 0 ? 599 : 0;}function taxCents(subtotal){return Math.round(subtotal * 0.07);}function renderCheckout(){const cart=readCart();const lines=document.getElementById('checkout-lines');const subtotal=cartSubtotal(cart);const shipping=shippingCents(subtotal);const tax=taxCents(subtotal);const donation=currentDonation();document.getElementById('checkout-subtotal').textContent=money(subtotal);document.getElementById('checkout-shipping').textContent=money(shipping);document.getElementById('checkout-tax').textContent=money(tax);document.getElementById('checkout-donation').textContent=money(donation);document.getElementById('checkout-total').textContent=money(subtotal+shipping+tax+donation);if(!cart.length){lines.innerHTML='<div class="empty-inline">Your cart is empty.</div>';return subtotal+shipping+tax+donation;}lines.innerHTML=cart.map((item)=>`<div class="list-row list-row--soft"><div><div class="list-title">${item.title}</div><div class="list-meta">${item.author} · Qty ${item.quantity}</div></div><strong>${money(item.price_cents*item.quantity)}</strong></div>`).join('');return subtotal+shipping+tax+donation;}async function createCheckoutSession(){const totalCents=renderCheckout();const email=document.getElementById('checkout-email').value.trim();const note=document.getElementById('checkout-note').value.trim();const name=document.getElementById('checkout-name').value.trim();const address=document.getElementById('checkout-address').value.trim();const panel=document.getElementById('checkout-status');const confirmation=document.getElementById('checkout-confirmation');if(!totalCents){panel.textContent='Add at least one title before creating a checkout session.';panel.className='notice-panel notice-panel--danger';confirmation.hidden=true;return;}panel.textContent='Creating checkout session...';const res=await fetch('/api/storefront/checkout/session',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({total_cents:totalCents,email})});const json=await res.json().catch(()=>({}));if(!res.ok){panel.textContent=json.message||json.error||'Checkout session request failed.';panel.className='notice-panel notice-panel--danger';confirmation.hidden=true;return;}panel.textContent=`Session created: ${json.session_id}${note?` · Note saved locally: ${note}`:''}${name?` · For ${name}`:''}${address?` · Shipping captured`:''}`;panel.className='notice-panel notice-panel--success';document.getElementById('checkout-confirmation-session').textContent=json.session_id||'-';document.getElementById('checkout-confirmation-email').textContent=email||'No receipt email';document.getElementById('checkout-confirmation-total').textContent=money(totalCents);confirmation.hidden=false;}document.getElementById('create-checkout-session').addEventListener('click',createCheckoutSession);document.getElementById('checkout-donation-select').addEventListener('change',renderCheckout);renderCheckout();</script></body></html>"#,
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
            "</style></head><body class=\"page-shell\">",
            &site_nav("admin"),
            r#"<main class="page-stack page-stack--wide"><section class="hero-card"><div><p class="eyebrow">Admin Office</p><h1 class="display-title">Dashboard, stock, and reporting</h1><p class="lede">Reconcile the day, watch stock pressure, and follow up on open orders before volunteers head home.</p><div class="eyebrow-row"><span class="hero-chip hero-chip--gold">Treasurer view</span><span class="hero-chip">Volunteer-ready operations</span><span class="hero-chip">Sunday rush summary</span></div></div><div class="hero-actions"><a class="ghost-link" href="/admin/intake">Inventory intake</a><a class="ghost-link" href="/catalog">Storefront</a></div></section><section class="dashboard-grid dashboard-grid--three"><article class="surface-card"><h2 class="section-title">Admin sign-in</h2><label class="field-label" for="admin-username">Username</label><input id="admin-username" autocomplete="username" value="admin" aria-describedby="admin-username-help" /><p id="admin-username-help" class="helper-copy helper-copy--flush">Use the seeded <code>admin</code> account for local review, or replace it with another tenant login.</p><label class="field-label" for="admin-password">Password</label><input id="admin-password" type="password" autocomplete="current-password" placeholder="Password" /><div class="form-grid"><div><label class="field-label" for="report-from">From</label><input id="report-from" type="date" value="2026-03-01" /></div><div><label class="field-label" for="report-to">To</label><input id="report-to" type="date" value="2026-03-31" /></div></div><div class="button-row"><button class="primary-button" type="button" id="admin-login">Login</button><button class="accent-button" type="button" id="admin-refresh">Refresh data</button><button class="ghost-link ghost-link--ink" type="button" id="admin-export">Export snapshot</button></div><p class="helper-copy">Sign in once, then refresh the summary, orders, IOUs, and journal panels for the selected reporting window.</p><div id="admin-status" class="notice-panel" aria-live="polite">Sign in to load tenant dashboard data.</div></article><article class="surface-card"><h2 class="section-title">Live report summary</h2><div class="metric-grid"><div class="metric-card metric-card--feature"><div class="metric-icon">💒</div><span class="metric-label">Sales</span><strong id="metric-sales">$0.00</strong></div><div class="metric-card"><div class="metric-icon">🎁</div><span class="metric-label">Donations</span><strong id="metric-donations">$0.00</strong></div><div class="metric-card"><div class="metric-icon">📚</div><span class="metric-label">COGS</span><strong id="metric-cogs">$0.00</strong></div><div class="metric-card"><div class="metric-icon">📈</div><span class="metric-label">Gross Profit</span><strong id="metric-profit">$0.00</strong></div></div><div id="report-caption" class="helper-copy">Showing the selected reporting window.</div><div class="divider-title divider-title--spaced">Payment breakdown</div><div id="admin-payment-breakdown" class="stack-list stack-list--tight"><div class="empty-inline">Payment method totals will appear here.</div></div></article><article class="surface-card"><h2 class="section-title">Ops cadence</h2><div class="pilgrim-panel"><h3>After-liturgy rhythm</h3><p>Use this page to reconcile takings, settle IOUs, export the current snapshot, and spot low-friction follow-up actions before volunteers leave for the day.</p></div><div class="divider-title divider-title--spaced">Trend note</div><div id="admin-trend-note" class="notice-panel">Trend notes will appear after the first refresh.</div></article></section><section class="dashboard-grid"><article class="surface-card"><p class="divider-title">Inventory</p><h2 class="section-title">Products</h2><div id="admin-products" class="stack-list"><div class="empty-inline">No products loaded yet.</div></div></article><article class="surface-card"><p class="divider-title">Taxonomy</p><h2 class="section-title">Categories and vendors</h2><div class="taxonomy-wrap"><div><h3 class="subheading">Categories</h3><div id="admin-categories" class="chip-wrap"><span class="chip-muted">Waiting for data</span></div></div><div><h3 class="subheading">Vendors</h3><div id="admin-vendors" class="chip-wrap"><span class="chip-muted">Waiting for data</span></div></div></div></article></section><section class="dashboard-grid"><article class="surface-card"><p class="divider-title">Orders</p><h2 class="section-title">Recent orders</h2><div class="toolbar-chips"><button class="filter-chip filter-chip--active" type="button" data-order-filter="All">All</button><button class="filter-chip" type="button" data-order-filter="POS">POS</button><button class="filter-chip" type="button" data-order-filter="Online">Online</button><button class="filter-chip" type="button" data-order-filter="IOU">IOU</button></div><div id="admin-orders" class="orders-table-wrap"><div class="empty-inline">No orders loaded yet.</div></div></article><article class="surface-card"><p class="divider-title">Attention queue</p><h2 class="section-title">Open IOUs</h2><div id="admin-ious" class="stack-list"><div class="empty-inline">No open IOUs.</div></div><div class="divider-title divider-title--spaced">Low stock spotlight</div><div id="admin-low-stock" class="stack-list"><div class="empty-inline">Low-stock titles will appear here.</div></div></article></section><section class="dashboard-grid"><article class="surface-card"><p class="divider-title">Stock movement</p><h2 class="section-title">Inventory journal</h2><div id="admin-journal" class="stack-list"><div class="empty-inline">Inventory movement will appear here after login.</div></div></article><article class="surface-card"><p class="divider-title">Reports</p><h2 class="section-title">Readiness actions</h2><div class="stack-list"><div class="list-row list-row--soft"><div><div class="list-title">Export current snapshot</div><div class="list-meta">Download report, product, order, and journal data as JSON.</div></div><button class="primary-button primary-button--sm" type="button" id="admin-export-inline">Export</button></div><div class="list-row list-row--soft"><div><div class="list-title">Open intake workflow</div><div class="list-meta">Jump into screen 9 style intake to receive new stock.</div></div><a class="ghost-link ghost-link--ink ghost-link--mini" href="/admin/intake">Open</a></div></div></article></section></main>"#,
            site_footer(),
            r#"<script>let adminToken='';let adminTenant='church-a';let adminOrders=[];let adminSnapshot={summary:null,products:[],categories:[],vendors:[],orders:[],journal:[]};let orderFilter='All';const money=(cents)=>`$${(Number(cents||0)/100).toFixed(2)}`;function setStatus(message,tone=''){const panel=document.getElementById('admin-status');panel.textContent=message;panel.className=`notice-panel${tone?` notice-panel--${tone}`:''}`;}function renderList(containerId,items,emptyMessage,renderer){const node=document.getElementById(containerId);if(!items.length){node.innerHTML=`<div class="empty-inline">${emptyMessage}</div>`;return;}node.innerHTML=items.map(renderer).join('');}function reportQuery(){const from=document.getElementById('report-from').value;const to=document.getElementById('report-to').value;const params=new URLSearchParams({tenant_id:adminTenant});if(from)params.set('from',from);if(to)params.set('to',to);return params.toString();}function normalizeChannel(order){return order.channel==='Online'?'Online':'POS';}async function adminLogin(){const usernameField=document.getElementById('admin-username');const username=usernameField.value.trim()||'admin';usernameField.value=username;const password=document.getElementById('admin-password').value;setStatus('Signing in...');const res=await fetch('/api/admin/auth/login',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({username,password})});const json=await res.json().catch(()=>({}));if(!res.ok){setStatus(json.message||'Login failed.','danger');return;}adminToken=json.token||'';adminTenant=json.tenant_id||'church-a';setStatus(`Signed in for ${adminTenant}.`,'success');await refreshAdminData();}async function fetchJson(url,options={}){const headers={...(options.headers||{}),Authorization:`Bearer ${adminToken}`};const res=await fetch(url,{...options,headers});const json=await res.json().catch(()=>({}));if(!res.ok){throw new Error(json.message||json.error||`Request failed for ${url}`);}return json;}function orderStatusBadge(order){return order.status==='Paid'?'<span class="status-badge status-badge--paid">Paid</span>':'<span class="status-badge status-badge--iou">IOU</span>';}function renderPaymentBreakdown(summary){const rows=Object.entries(summary?.sales_by_payment||{});renderList('admin-payment-breakdown',rows,'Payment method totals will appear here.',([method,cents])=>{const total=Math.max(1,Number(summary.sales_cents||0));const width=Math.max(8,Math.round((Number(cents||0)/total)*100));return `<div class="stack-list"><div class="list-row list-row--soft"><div><div class="list-title">${method.replaceAll('_',' ')}</div><div class="list-meta">Share of report window</div></div><strong>${money(cents)}</strong></div><div class="bar-track"><div class="bar-fill" style="width:${width}%"></div></div></div>`;});const trend=document.getElementById('admin-trend-note');const paid=Number(summary?.sales_cents||0)-Number(summary?.donations_cents||0);trend.textContent=paid>0?`Paid sales are ${money(paid)} for the selected window, with donations contributing ${money(summary?.donations_cents||0)} on top.`:'No paid sales were recorded in the selected window.';trend.className='notice-panel notice-panel--success';}function renderOrders(){const filtered=adminOrders.filter((order)=>orderFilter==='All'||(orderFilter==='IOU'?order.status==='UnpaidIou':normalizeChannel(order)===orderFilter));const node=document.getElementById('admin-orders');if(!filtered.length){node.innerHTML='<div class="empty-inline">No orders found for this filter.</div>';return;}node.innerHTML=`<table class="orders-table"><thead><tr><th>Order</th><th>Date</th><th>Channel</th><th>Customer</th><th>Status</th><th>Method</th><th>Total</th><th>Actions</th></tr></thead><tbody>${filtered.map((order)=>`<tr><td>${order.order_id}</td><td>${order.created_on}</td><td>${order.channel}</td><td>${order.customer_name}</td><td>${orderStatusBadge(order)}</td><td>${order.payment_method}</td><td><strong>${money(order.total_cents)}</strong></td><td>${order.status==='UnpaidIou'?`<button class="primary-button primary-button--sm" type="button" onclick="markOrderPaid('${order.order_id}')">Mark Paid</button>`:'<span class="helper-copy helper-copy--flush">Cleared</span>'}</td></tr>`).join('')}</tbody></table>`;}function bindOrderFilters(){document.querySelectorAll('[data-order-filter]').forEach((button)=>{button.onclick=()=>{orderFilter=button.dataset.orderFilter||'All';document.querySelectorAll('[data-order-filter]').forEach((chip)=>chip.classList.remove('filter-chip--active'));button.classList.add('filter-chip--active');renderOrders();};});}async function markOrderPaid(orderId){if(!adminToken){setStatus('Sign in first to manage orders.','danger');return;}try{await fetchJson(`/api/admin/orders/${orderId}/mark-paid?tenant_id=${adminTenant}`,{method:'POST',headers:{Origin:window.location.origin}});setStatus(`Marked ${orderId} paid.`,'success');await refreshAdminData();}catch(error){setStatus(error.message,'danger');}}function exportSnapshot(){if(!adminToken||!adminSnapshot.summary){setStatus('Load dashboard data before exporting.','danger');return;}const blob=new Blob([JSON.stringify(adminSnapshot,null,2)],{type:'application/json'});const url=URL.createObjectURL(blob);const link=document.createElement('a');link.href=url;link.download=`scriptorium-${adminTenant}-dashboard.json`;link.click();URL.revokeObjectURL(url);setStatus(`Exported dashboard snapshot for ${adminTenant}.`,'success');}function reorderTitle(title){setStatus(`Open intake to reorder ${title}.`,'success');}async function refreshAdminData(){if(!adminToken){setStatus('Sign in first to load dashboard data.','danger');return;}setStatus('Loading dashboard data...');try{const [summary,products,categories,vendors,orders,journal]=await Promise.all([fetchJson(`/api/admin/reports/summary?${reportQuery()}`),fetchJson(`/api/admin/products?tenant_id=${adminTenant}`),fetchJson(`/api/admin/categories?tenant_id=${adminTenant}`),fetchJson(`/api/admin/vendors?tenant_id=${adminTenant}`),fetchJson(`/api/admin/orders?tenant_id=${adminTenant}`),fetchJson(`/api/admin/inventory/journal?tenant_id=${adminTenant}`)]);adminSnapshot={summary,products,categories:categories.values||[],vendors:vendors.values||[],orders,journal};adminOrders=orders;document.getElementById('metric-sales').textContent=money(summary.sales_cents);document.getElementById('metric-donations').textContent=money(summary.donations_cents);document.getElementById('metric-cogs').textContent=money(summary.cogs_cents);document.getElementById('metric-profit').textContent=money(summary.gross_profit_cents);document.getElementById('report-caption').textContent=`Showing ${document.getElementById('report-from').value||'the start'} to ${document.getElementById('report-to').value||'today'}.`;renderPaymentBreakdown(summary);renderList('admin-products',products,'No products found for this tenant.',(product)=>`<div class="list-row list-row--soft"><div><div class="list-title">${product.title}</div><div class="list-meta">${product.category} · ${product.vendor}</div></div><strong>${money(product.retail_cents)}</strong></div>`);renderList('admin-categories',categories.values||[],'No categories found.',(value)=>`<span class="chip">${value}</span>`);renderList('admin-vendors',vendors.values||[],'No vendors found.',(value)=>`<span class="chip">${value}</span>`);renderOrders();renderList('admin-ious',orders.filter((order)=>order.status==='UnpaidIou'),'No open IOUs.',(order)=>`<div class="list-row list-row--soft"><div><div class="list-title">${order.customer_name}</div><div class="list-meta">${order.order_id} · ${order.created_on}</div></div><div class="button-row button-row--compact"><strong>${money(order.total_cents)}</strong><button class="primary-button primary-button--sm" type="button" onclick="markOrderPaid('${order.order_id}')">Mark Paid</button></div></div>`);const lowStock=(products||[]).filter((product)=>Number(product.quantity_on_hand||0)<=3);renderList('admin-low-stock',lowStock,'No low-stock titles right now.',(product)=>`<div class="list-row list-row--soft"><div><div class="list-title">${product.title}</div><div class="list-meta">${product.category} · On hand ${product.quantity_on_hand}</div></div><div class="button-row button-row--compact"><span class="status-badge status-badge--iou">Reorder</span><button class="ghost-link ghost-link--ink ghost-link--mini" type="button" onclick="reorderTitle('${product.title.replaceAll("'", "&#39;")}')">Prep</button></div></div>`);renderList('admin-journal',journal,'No inventory movement recorded yet.',(entry)=>`<div class="list-row list-row--soft"><div><div class="list-title">${entry.isbn}</div><div class="list-meta">${entry.reason}</div></div><strong>${entry.delta>0?`+${entry.delta}`:entry.delta}</strong></div>`);setStatus(`Dashboard refreshed for ${adminTenant}.`,'success');}catch(error){setStatus(error.message,'danger');}}document.getElementById('admin-login').addEventListener('click',adminLogin);document.getElementById('admin-refresh').addEventListener('click',refreshAdminData);document.getElementById('admin-export').addEventListener('click',exportSnapshot);document.getElementById('admin-export-inline').addEventListener('click',exportSnapshot);document.getElementById('report-from').addEventListener('change',()=>{if(adminToken)refreshAdminData();});document.getElementById('report-to').addEventListener('change',()=>{if(adminToken)refreshAdminData();});window.markOrderPaid=markOrderPaid;window.reorderTitle=reorderTitle;bindOrderFilters();</script></body></html>"#,
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
      .page-shell { min-height: 100vh; padding: 24px 16px 40px; }
      .page-stack { max-width: 1080px; margin: 0 auto; display: grid; gap: 18px; }
      .page-stack--wide { max-width: 1220px; }
      .site-nav {
        max-width: 1220px;
        margin: 0 auto 18px;
      }
      .site-nav__inner,
      .site-footer__inner {
        display: flex;
        gap: 16px;
        align-items: center;
        justify-content: space-between;
        padding: 14px 18px;
        border: 1px solid var(--parchment-dark);
        border-radius: var(--radius-lg);
        background: rgba(255,255,255,0.86);
        box-shadow: var(--shadow);
      }
      .site-nav__brand {
        color: var(--wine);
        text-decoration: none;
        font: 700 1.15rem/1 "Crimson Pro", serif;
        letter-spacing: 0.04em;
        text-transform: uppercase;
      }
      .site-nav__links,
      .site-footer__links {
        display: flex;
        gap: 10px;
        flex-wrap: wrap;
        align-items: center;
      }
      .site-nav__link,
      .site-footer__links a {
        display: inline-flex;
        align-items: center;
        min-height: 38px;
        padding: 0 12px;
        border-radius: 999px;
        color: var(--ink-light);
        text-decoration: none;
        border: 1px solid var(--parchment-dark);
        background: white;
        font-weight: 700;
      }
      .site-nav__link--active {
        color: white;
        border-color: var(--wine);
        background: var(--wine);
      }
      .site-footer {
        max-width: 1220px;
        margin: 18px auto 0;
      }
      .site-footer__inner {
        color: var(--warm-gray);
        font-size: 0.92rem;
      }
      .site-footer__inner p {
        margin: 0;
        max-width: 48ch;
      }
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
        position: relative;
        overflow: hidden;
      }
      .hero-card::after {
        content: "";
        position: absolute;
        inset: auto -6% -30% auto;
        width: 280px;
        height: 280px;
        border-radius: 50%;
        background: radial-gradient(circle, rgba(204,170,94,0.22), transparent 64%);
        pointer-events: none;
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
      .hero-actions,
      .eyebrow-row {
        display: flex;
        flex-wrap: wrap;
        gap: 10px;
        align-items: center;
      }
      .eyebrow-row { margin-top: 16px; }
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
        position: relative;
        overflow: hidden;
      }
      .catalog-card::before {
        content: "";
        position: absolute;
        inset: 0 0 auto;
        height: 4px;
        background: linear-gradient(90deg, var(--wine), var(--gold));
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
      .book-cover-art {
        width: min(100%, 240px);
        min-height: 260px;
        padding: 22px 18px;
        border-radius: var(--radius);
        display: flex;
        flex-direction: column;
        justify-content: space-between;
        background:
          linear-gradient(145deg, rgba(107,39,55,0.94), rgba(74,26,38,0.98)),
          var(--wine);
        color: white;
        box-shadow: var(--shadow-lg);
      }
      .book-cover-art__eyebrow {
        color: rgba(245,236,215,0.92);
        font-size: 0.78rem;
        letter-spacing: 0.12em;
        text-transform: uppercase;
      }
      .book-cover-art strong {
        display: block;
        font: 600 2rem/1.02 "Crimson Pro", serif;
      }
      .book-cover-art span:last-child {
        color: rgba(255,255,255,0.8);
        font-size: 0.96rem;
      }
      .catalog-title {
        margin: 0 0 6px;
        font: 600 1.45rem/1 "Crimson Pro", serif;
      }
      .catalog-kicker {
        display: inline-flex;
        align-items: center;
        gap: 8px;
        margin-bottom: 10px;
        color: var(--warm-gray);
        font-size: 0.75rem;
        text-transform: uppercase;
        letter-spacing: 0.12em;
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
      .catalog-note {
        margin-top: 12px;
        padding-top: 12px;
        border-top: 1px solid var(--parchment-dark);
        color: var(--ink-light);
        font-size: 0.92rem;
      }
      .stock-badge {
        display: inline-flex;
        align-items: center;
        min-height: 28px;
        margin-bottom: 10px;
        padding: 0 10px;
        border-radius: 999px;
        font-size: 0.78rem;
        font-weight: 700;
      }
      .stock-badge--success {
        color: var(--success);
        background: var(--success-light);
      }
      .stock-badge--warning {
        color: var(--warning);
        background: var(--warning-light);
      }
      .stock-badge--danger {
        color: var(--danger);
        background: var(--danger-light);
      }
      .catalog-empty {
        padding: 22px;
        border-radius: var(--radius);
        background: var(--filled);
        border: 1px solid var(--filled-border);
        color: var(--ink-light);
      }
      .pagination {
        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: 12px;
        margin-top: 18px;
        flex-wrap: wrap;
      }
      .pagination-links {
        display: flex;
        gap: 8px;
        flex-wrap: wrap;
      }
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
      .checkout-layout { display: grid; gap: 18px; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); }
      .product-layout { display: grid; gap: 18px; grid-template-columns: minmax(260px, 340px) minmax(0, 1fr); }
      .product-meta-grid {
        display: grid;
        gap: 10px;
        grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
        margin-top: 16px;
      }
      .inline-quantity {
        display: grid;
        gap: 12px;
        grid-template-columns: minmax(120px, 180px) minmax(0, 1fr);
        align-items: end;
        margin-top: 18px;
      }
      .meta-tile {
        padding: 14px;
        border-radius: var(--radius);
        background: linear-gradient(180deg, white, var(--filled));
        border: 1px solid var(--filled-border);
      }
      .meta-tile span {
        display: block;
        color: var(--warm-gray);
        font-size: 0.8rem;
        text-transform: uppercase;
        letter-spacing: 0.08em;
      }
      .meta-tile strong {
        display: block;
        margin-top: 6px;
        color: var(--ink);
      }
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
      .helper-copy--flush { margin: 0; }
      .dashboard-grid {
        display: grid;
        gap: 18px;
        grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
      }
      .dashboard-grid--three {
        grid-template-columns: 1.2fr .8fr .8fr;
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
      .button-row--flush-start {
        justify-content: start;
      }
      .toolbar-chips {
        display: flex;
        gap: 8px;
        flex-wrap: wrap;
        margin-bottom: 14px;
      }
      .filter-chip {
        min-height: 34px;
        padding: 0 12px;
        border-radius: 999px;
        border: 1px solid var(--parchment-dark);
        background: white;
        color: var(--warm-gray);
        font: 700 0.84rem/1 "DM Sans", sans-serif;
        cursor: pointer;
      }
      .filter-chip--active {
        background: var(--wine);
        border-color: var(--wine);
        color: white;
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
      .metric-card--feature {
        background: linear-gradient(180deg, rgba(107,39,55,0.08), rgba(245,236,215,0.42));
      }
      .bar-track {
        height: 10px;
        border-radius: 999px;
        background: var(--parchment-dark);
        overflow: hidden;
      }
      .bar-fill {
        height: 100%;
        border-radius: 999px;
        background: linear-gradient(90deg, var(--wine), var(--gold));
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
      .stack-list--tight { gap: 6px; }
      .orders-table-wrap {
        overflow-x: auto;
        border: 1px solid var(--parchment-dark);
        border-radius: var(--radius);
        background: white;
      }
      .orders-table {
        width: 100%;
        border-collapse: collapse;
        min-width: 760px;
      }
      .orders-table th,
      .orders-table td {
        padding: 12px 14px;
        text-align: left;
        border-bottom: 1px solid var(--parchment-dark);
        vertical-align: middle;
      }
      .orders-table th {
        color: var(--warm-gray);
        font-size: 0.82rem;
        text-transform: uppercase;
        letter-spacing: 0.08em;
        background: var(--filled);
      }
      .orders-table tr:last-child td {
        border-bottom: 0;
      }
      .metric-icon {
        width: 38px;
        height: 38px;
        border-radius: 12px;
        display: grid;
        place-items: center;
        margin-bottom: 8px;
        color: var(--wine);
        background: rgba(184,144,58,0.18);
      }
      .fieldset-grid {
        display: grid;
        gap: 12px;
        grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
      }
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
      .list-row--soft {
        background: linear-gradient(180deg, white, var(--filled));
      }
      .list-title { font-weight: 700; }
      .list-meta { margin-top: 4px; color: var(--warm-gray); font-size: 0.9rem; }
      .status-badge {
        display: inline-flex;
        align-items: center;
        min-height: 30px;
        padding: 0 10px;
        border-radius: 999px;
        font-size: 0.8rem;
        font-weight: 700;
      }
      .status-badge--paid {
        color: var(--success);
        background: var(--success-light);
      }
      .status-badge--iou {
        color: var(--warning);
        background: var(--warning-light);
      }
      .divider-title {
        margin: 0 0 12px;
        color: var(--warm-gray);
        font-size: 0.8rem;
        text-transform: uppercase;
        letter-spacing: 0.12em;
      }
      .divider-title--spaced { margin-top: 18px; }
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
      .pilgrim-panel {
        padding: 18px;
        border-radius: var(--radius-lg);
        background: linear-gradient(180deg, rgba(245,236,215,0.56), white);
        border: 1px solid var(--filled-border);
      }
      .pilgrim-panel h3 {
        margin: 0 0 8px;
        font: 600 1.25rem/1 "Crimson Pro", serif;
        color: var(--wine);
      }
      .pilgrim-panel p {
        margin: 0;
        color: var(--ink-light);
        line-height: 1.6;
      }
      #camera {
        width: 100%;
        min-height: 220px;
        margin-bottom: 14px;
        border-radius: var(--radius);
        background: linear-gradient(135deg, var(--wine-dark), var(--wine));
      }
      .intake-grid {
        display: grid;
        gap: 18px;
        grid-template-columns: minmax(0, 1.1fr) minmax(280px, .9fr);
      }
      .intake-panel {
        display: grid;
        gap: 14px;
      }
      #intake-form {
        display: grid;
        gap: 12px;
        grid-template-columns: repeat(2, minmax(0, 1fr));
      }
      .intake-form__full { grid-column: 1 / -1; }
      #description { min-height: 96px; }
      @media (max-width: 980px) {
        .catalog-grid {
          grid-template-columns: repeat(2, minmax(0, 1fr));
        }
        .intake-grid {
          grid-template-columns: 1fr;
        }
      }
      @media (max-width: 640px) {
        .hero-card { align-items: start; flex-direction: column; }
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

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn filter_books(
    books: Vec<bookstore_domain::Book>,
    query: Option<&str>,
    category: Option<&str>,
) -> Vec<bookstore_domain::Book> {
    let query = query.unwrap_or("").trim().to_ascii_lowercase();
    let category = category.unwrap_or("").trim().to_ascii_lowercase();
    if query.is_empty() {
        if category.is_empty() || category == "all" {
            return books;
        }
        return books
            .into_iter()
            .filter(|book| book.category.to_ascii_lowercase() == category)
            .collect();
    }
    books
        .into_iter()
        .filter(|book| {
            let matches_query = book.title.to_ascii_lowercase().contains(&query)
                || book.author.to_ascii_lowercase().contains(&query);
            let matches_category =
                category.is_empty() || category == "all" || book.category.to_ascii_lowercase() == category;
            matches_query && matches_category
        })
        .collect()
}

fn catalog_categories(books: &[bookstore_domain::Book]) -> Vec<String> {
    let mut categories = books.iter().map(|book| book.category.clone()).collect::<Vec<_>>();
    categories.sort();
    categories.dedup();
    categories
}

fn render_catalog_category_chips(
    categories: &[String],
    query: Option<&str>,
    active_category: Option<&str>,
    filtered_books: &[bookstore_domain::Book],
) -> String {
    let active = active_category.unwrap_or("All");
    let query = query.unwrap_or("").trim();
    std::iter::once("All".to_string())
        .chain(categories.iter().cloned())
        .map(|category| {
            let href = if query.is_empty() {
                format!("/catalog?category={}", urlencoding::encode(&category))
            } else {
                format!(
                    "/catalog?q={}&category={}",
                    urlencoding::encode(query),
                    urlencoding::encode(&category)
                )
            };
            let is_active = category.eq_ignore_ascii_case(active);
            let count = if category == "All" {
                filtered_books.len()
            } else {
                filtered_books
                    .iter()
                    .filter(|book| book.category.eq_ignore_ascii_case(&category))
                    .count()
            };
            format!(
                "<a class=\"category-chip{}\" href=\"{}\">{} <span>{}</span></a>",
                if is_active { " category-chip--active" } else { "" },
                href,
                html_escape(&category),
                count
            )
        })
        .collect::<Vec<_>>()
        .join("")
}

fn render_catalog_cards(books: Vec<bookstore_domain::Book>) -> String {
    if books.is_empty() {
        return "<div class=\"catalog-empty\">No books matched that search.</div>".to_string();
    }
    let items = books
        .into_iter()
        .map(|book| {
            let (stock_label, stock_class) = stock_hint(&book.id);
            format!(
                r#"<article class="catalog-card">
  <div class="catalog-cover">📚</div>
  <div class="{stock_class}">{stock_label}</div>
  <div class="catalog-kicker"><span>{category}</span><span>Front display</span></div>
  <h2 class="catalog-title">{title}</h2>
  <p class="catalog-meta">{author}</p>
  <p class="catalog-note">Selected for parish browsing, gifting, and easy recommendation after services.</p>
  <div class="button-row">
    <span class="catalog-price">{price}</span>
    <button class="primary-button primary-button--sm" type="button" data-add-book-id="{book_id}" data-add-book-title="{title_attr}" data-add-book-author="{author_attr}" data-add-book-price-cents="{price_cents}" data-feedback-target="catalog-feedback">Add</button>
    <a class="ghost-link ghost-link--ink" href="/catalog/items/{book_id}">View</a>
  </div>
</article>"#,
                title = html_escape(&book.title),
                author = html_escape(&book.author),
                category = html_escape(&book.category),
                price = format_money(i64::from(book.price_cents)),
                book_id = html_escape(&book.id),
                title_attr = html_escape(&book.title),
                author_attr = html_escape(&book.author),
                price_cents = i64::from(book.price_cents),
                stock_label = stock_label,
                stock_class = stock_class,
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!(r#"<div class="catalog-grid">{items}</div>"#)
}

fn render_catalog_pagination(
    current_page: usize,
    total_pages: usize,
    query: Option<&str>,
    category: Option<&str>,
) -> String {
    if total_pages <= 1 {
        return String::new();
    }
    let mut items = Vec::new();
    for page in 1..=total_pages {
        let mut params = vec![format!("page={page}")];
        if let Some(q) = query.filter(|value| !value.trim().is_empty()) {
            params.push(format!("q={}", urlencoding::encode(q)));
        }
        if let Some(category) = category.filter(|value| !value.trim().is_empty()) {
            params.push(format!("category={}", urlencoding::encode(category)));
        }
        items.push(format!(
            "<a class=\"pagination-link{}\" href=\"/catalog?{}\">{}</a>",
            if page == current_page { " pagination-link--active" } else { "" },
            params.join("&"),
            page
        ));
    }
    format!(
        "<div class=\"pagination\"><span class=\"helper-copy helper-copy--flush\">Page {} of {}</span><div class=\"pagination-links\">{}</div></div>",
        current_page,
        total_pages,
        items.join("")
    )
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
function readAddQuantity(button) {
  const targetId = button.dataset.addBookQuantityTarget;
  if (!targetId) return 1;
  const input = document.getElementById(targetId);
  const quantity = Math.max(1, Number(input?.value || 1));
  return Number.isFinite(quantity) ? quantity : 1;
}
function addToCartFromDataset(button) {
  const cart = readCart();
  const id = button.dataset.addBookId;
  const price = Number(button.dataset.addBookPriceCents || 0);
  const quantity = readAddQuantity(button);
  const existing = cart.find((item) => item.id === id);
  if (existing) {
    existing.quantity += quantity;
  } else {
    cart.push({
      id,
      title: button.dataset.addBookTitle,
      author: button.dataset.addBookAuthor,
      price_cents: price,
      quantity,
    });
  }
  writeCart(cart);
  const feedback = document.getElementById(button.dataset.feedbackTarget || "cart-feedback");
  if (feedback) {
    feedback.textContent = `Added ${quantity} to cart. Cart now has ${cart.reduce((sum, item) => sum + item.quantity, 0)} item(s).`;
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
          <div class="button-row button-row--compact">
            <button class="ghost-link ghost-link--ink ghost-link--mini" type="button" data-cart-decrement="${item.id}">−</button>
            <button class="ghost-link ghost-link--ink ghost-link--mini" type="button" data-cart-increment="${item.id}">+</button>
            <button class="ghost-link ghost-link--ink ghost-link--mini" type="button" data-cart-remove="${item.id}">Remove</button>
            <strong>${money(item.price_cents * item.quantity)}</strong>
          </div>
        </div>
      `).join("");
    }
  }
  if (cartSummary) {
    const total = cart.reduce((sum, item) => sum + (item.price_cents * item.quantity), 0);
    cartSummary.textContent = `Cart total: ${money(total)}`;
  }
  syncRecommendations(cart);
  bindCartControls();
}
function syncRecommendations(cart) {
  const cartIds = new Set(cart.map((item) => item.id));
  const rows = Array.from(document.querySelectorAll("[data-recommendation-book-id]"));
  let visible = 0;
  rows.forEach((row) => {
    const hidden = cartIds.has(row.dataset.recommendationBookId);
    row.hidden = hidden;
    row.style.display = hidden ? "none" : "";
    if (!hidden) visible += 1;
  });
  const empty = document.getElementById("cart-recommendations-empty");
  if (empty) {
    empty.hidden = visible !== 0;
  }
}
function mutateCart(id, operation) {
  const cart = readCart().map((item) => ({ ...item }));
  const entry = cart.find((item) => item.id === id);
  if (!entry) return;
  if (operation === "increment") entry.quantity += 1;
  if (operation === "decrement") entry.quantity = Math.max(0, entry.quantity - 1);
  const nextCart = operation === "remove" ? cart.filter((item) => item.id !== id) : cart.filter((item) => item.quantity > 0);
  writeCart(nextCart);
  renderCartPage();
}
function bindCartControls() {
  document.querySelectorAll("[data-cart-increment]").forEach((button) => {
    button.onclick = () => mutateCart(button.dataset.cartIncrement, "increment");
  });
  document.querySelectorAll("[data-cart-decrement]").forEach((button) => {
    button.onclick = () => mutateCart(button.dataset.cartDecrement, "decrement");
  });
  document.querySelectorAll("[data-cart-remove]").forEach((button) => {
    button.onclick = () => mutateCart(button.dataset.cartRemove, "remove");
  });
}
document.querySelectorAll("[data-add-book-id]").forEach((button) => {
  button.addEventListener("click", () => addToCartFromDataset(button));
});
const clearCartButton = document.getElementById("clear-cart");
if (clearCartButton) {
  clearCartButton.addEventListener("click", () => {
    writeCart([]);
    renderCartPage();
  });
}
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
  "#,
        &site_nav("intake"),
        r#"
  <main class="page-stack">
    <section class="hero-card">
      <div>
        <p class="eyebrow">Admin Intake</p>
        <h1 class="display-title">Receive and enrich inventory</h1>
        <p class="lede">Scan ISBNs, sign in, and confirm metadata before new stock reaches the shelf.</p>
      </div>
      <div class="hero-actions">
        <a class="ghost-link" href="/admin">Dashboard</a>
        <a class="ghost-link" href="/catalog">Storefront</a>
      </div>
    </section>
    <section class="intake-grid">
      <article class="surface-card intake-panel">
        <div>
          <p class="divider-title">Scanner</p>
          <h2 class="section-title">Camera intake</h2>
          <p class="helper-copy helper-copy--flush">Use the mobile camera when available, then confirm or correct the fields before saving inventory.</p>
        </div>
        <video id="camera" autoplay playsinline></video>
        <div class="button-row">
          <button class="primary-button" type="button" id="camera-start">Start scanner</button>
          <button class="ghost-link ghost-link--ink" type="button" id="camera-stop">Stop</button>
        </div>
        <div id="scanner-status" class="notice-panel" aria-live="polite">Scanner idle. You can still type an ISBN manually below.</div>
        <div class="pilgrim-panel">
          <h3>Volunteer flow</h3>
          <p>Authenticate once, scan the ISBN, run metadata lookup, then review the record before stock is received.</p>
        </div>
      </article>
      <article class="surface-card intake-panel">
        <div>
          <p class="divider-title">Metadata</p>
          <h2 class="section-title">Inventory intake</h2>
        </div>
        <form id="intake-form">
          <div>
            <label class="field-label" for="isbn">ISBN</label>
            <input id="isbn" name="isbn" placeholder="978..." inputmode="numeric" />
          </div>
          <div>
            <label class="field-label" for="title">Title</label>
            <input id="title" name="title" placeholder="Book title" />
          </div>
          <div>
            <label class="field-label" for="author">Author</label>
            <input id="author" name="author" placeholder="Author name" />
          </div>
          <div class="intake-form__full">
            <label class="field-label" for="description">Description</label>
            <textarea id="description" name="description" placeholder="Description"></textarea>
          </div>
          <div>
            <label class="field-label" for="cost-cents">Cost price</label>
            <input id="cost-cents" name="cost-cents" value="900" inputmode="numeric" />
          </div>
          <div>
            <label class="field-label" for="retail-cents">Retail price</label>
            <input id="retail-cents" name="retail-cents" value="1699" inputmode="numeric" />
          </div>
          <div>
            <label class="field-label" for="initial-stock">Initial stock</label>
            <input id="initial-stock" name="initial-stock" value="5" inputmode="numeric" />
          </div>
          <div>
            <label class="field-label" for="reorder-point">Reorder point</label>
            <input id="reorder-point" name="reorder-point" value="3" inputmode="numeric" />
          </div>
          <div>
            <label class="field-label" for="category">Category</label>
            <input id="category" name="category" placeholder="Books / Gifts / Icons" value="Discipleship" />
          </div>
          <div>
            <label class="field-label" for="vendor">Vendor</label>
            <input id="vendor" name="vendor" placeholder="Supplier" value="Church Supplier" />
          </div>
          <div>
            <label class="field-label" for="username">Username</label>
            <input id="username" name="username" placeholder="admin" autocomplete="username" />
          </div>
          <div>
            <label class="field-label" for="password">Password</label>
            <input id="password" name="password" type="password" placeholder="Password" autocomplete="current-password" />
          </div>
          <div class="intake-form__full">
            <label class="field-label" for="token">Admin Token</label>
            <input id="token" name="token" placeholder="Generated after login" />
          </div>
          <div id="intake-lookup-status" class="notice-panel intake-form__full">Lookup and save status will appear here.</div>
          <button class="primary-button" type="button" id="login">Login</button>
          <button class="accent-button" type="button" id="lookup">Lookup</button>
          <button class="ghost-link ghost-link--ink" type="button" id="save-product">Save product</button>
        </form>
      </article>
    </section>
  </main>
  "#,
        site_footer(),
        r#"
  <script>
    let cameraStream = null;
    let scanTimer = null;
    let detector = null;
    let lastScan = "";
    let lastScanAt = 0;
    function setScannerStatus(message, tone = "") {
      const panel = document.getElementById("scanner-status");
      panel.textContent = message;
      panel.className = `notice-panel${tone ? ` notice-panel--${tone}` : ""}`;
    }
    async function ensureDetector() {
      if (!("BarcodeDetector" in window)) return null;
      if (detector) return detector;
      const formats = typeof BarcodeDetector.getSupportedFormats === "function"
        ? await BarcodeDetector.getSupportedFormats().catch(() => [])
        : [];
      const preferredFormats = ["ean_13", "ean_8", "upc_a", "upc_e"];
      const activeFormats = formats.length ? preferredFormats.filter((format) => formats.includes(format)) : preferredFormats;
      detector = new BarcodeDetector({ formats: activeFormats.length ? activeFormats : preferredFormats });
      return detector;
    }
    function stopCamera() {
      if (scanTimer) {
        clearInterval(scanTimer);
        scanTimer = null;
      }
      if (cameraStream) {
        cameraStream.getTracks().forEach((track) => track.stop());
        cameraStream = null;
      }
      document.getElementById("camera").srcObject = null;
      setScannerStatus("Scanner stopped. Manual ISBN entry is still available.");
    }
    async function bootCamera() {
      if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
        setScannerStatus("Camera access is not available in this browser. Enter the ISBN manually.", "warning");
        return;
      }
      try {
        cameraStream = await navigator.mediaDevices.getUserMedia({ video: { facingMode: { ideal: "environment" } } });
        const video = document.getElementById("camera");
        video.srcObject = cameraStream;
        await video.play().catch(() => {});
        const activeDetector = await ensureDetector();
        if (!activeDetector) {
          setScannerStatus("Camera started. Barcode detection is unavailable here, so type the ISBN manually.", "warning");
          return;
        }
        setScannerStatus("Scanner live. Hold the ISBN barcode steady in frame.");
        scanTimer = setInterval(async () => {
          try {
            const barcodes = await activeDetector.detect(video);
            const barcode = barcodes.find((entry) => entry?.rawValue);
            if (!barcode?.rawValue) return;
            const now = Date.now();
            if (barcode.rawValue === lastScan && now - lastScanAt < 2000) return;
            lastScan = barcode.rawValue;
            lastScanAt = now;
            document.getElementById("isbn").value = barcode.rawValue;
            setScannerStatus(`Detected ISBN ${barcode.rawValue}. Review and run lookup when ready.`, "success");
          } catch {
            setScannerStatus("Camera is live, but barcode detection needs a steadier frame or better light.", "warning");
          }
        }, 700);
      } catch {
        setScannerStatus("Camera permission was denied or unavailable. Enter the ISBN manually instead.", "danger");
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
      document.getElementById("intake-lookup-status").textContent = json.token ? "Signed in. You can lookup metadata and save a product." : "Login failed.";
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
      document.getElementById("intake-lookup-status").textContent = json.title ? "Found metadata and auto-filled the product form." : "No metadata found for that ISBN.";
    }
    async function saveProduct() {
      const token = document.getElementById("token").value;
      const isbn = document.getElementById("isbn").value.trim();
      const title = document.getElementById("title").value.trim();
      const category = document.getElementById("category").value.trim() || "Books";
      const vendor = document.getElementById("vendor").value.trim() || "Church Supplier";
      const res = await fetch("/api/admin/products", {
        method: "POST",
        headers: { "content-type": "application/json", Origin: window.location.origin },
        body: JSON.stringify({
          token,
          tenant_id: "church-a",
          product_id: `prd-${isbn || Date.now()}`,
          title,
          isbn,
          category,
          vendor,
          cost_cents: Number(document.getElementById("cost-cents").value || 0),
          retail_cents: Number(document.getElementById("retail-cents").value || 0),
        }),
      });
      const json = await res.json().catch(() => ({}));
      document.getElementById("intake-lookup-status").textContent = res.ok
        ? `Saved ${json.title || title} for ${category}. Initial stock target ${document.getElementById("initial-stock").value}.`
        : (json.message || "Save failed.");
    }
    document.getElementById("login").addEventListener("click", login);
    document.getElementById("lookup").addEventListener("click", lookup);
    document.getElementById("save-product").addEventListener("click", saveProduct);
    document.getElementById("camera-start").addEventListener("click", bootCamera);
    document.getElementById("camera-stop").addEventListener("click", stopCamera);
    window.addEventListener("beforeunload", stopCamera);
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
      font-family: "DM Sans", sans-serif;
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
    .hero {
      padding: 18px;
      border-radius: var(--radius-lg);
      background: linear-gradient(135deg, rgba(107,39,55,.96), rgba(139,58,74,.9));
      box-shadow: var(--shadow-lg);
      position: relative;
      overflow: hidden;
    }
    .hero::after {
      content: "";
      position: absolute;
      width: 180px;
      height: 180px;
      right: -32px;
      bottom: -64px;
      border-radius: 50%;
      background: radial-gradient(circle, rgba(204,170,94,.24), transparent 68%);
      pointer-events: none;
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
      max-width: 30ch;
    }
    .kicker {
      margin: 0 0 8px;
      color: rgba(255,255,255,.68);
      font-size: .78rem;
      text-transform: uppercase;
      letter-spacing: .16em;
    }
    .session-row {
      margin-top: 14px;
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
      font-family: "Crimson Pro", serif;
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
    }
    .pin-head h1 {
      margin: 0;
      font-family: "Crimson Pro", serif;
      font-size: 2.2rem;
      color: var(--gold-light);
      letter-spacing: .05em;
    }
    .pin-head p {
      margin: 6px 0 0;
      color: rgba(255,255,255,.66);
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
      font: 700 1.75rem/1 "DM Sans", sans-serif;
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
      background: rgba(255,255,255,.1);
      padding: 6px;
      border-radius: 16px;
    }
    .toolbar button {
      flex: 1;
      min-height: 42px;
      border-radius: 12px;
      border: 0;
      font: 700 .95rem/1 "DM Sans", sans-serif;
      color: white;
      background: transparent;
    }
    .toolbar button.is-active {
      background: var(--gold-pale);
      color: var(--wine-dark);
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
      font: 500 16px/1.2 "DM Sans", sans-serif;
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
      font: 700 1rem/1.2 "DM Sans", sans-serif;
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
    .quick-note {
      display: block;
      margin-top: 8px;
      font-size: .78rem;
      color: var(--warm-gray);
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
      gap: 14px;
      padding: 16px;
      border-radius: 16px;
      border: 1px solid var(--parchment-dark);
      background: white;
    }
    .payment-icon {
      width: 46px;
      height: 46px;
      border-radius: 14px;
      display: grid;
      place-items: center;
      font-size: 1.35rem;
    }
    .payment-title {
      display: block;
      font-weight: 800;
      color: var(--ink);
    }
    .payment-copy {
      display: block;
      margin-top: 4px;
      color: var(--warm-gray);
      font-size: .9rem;
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
      font: 800 1.2rem/1 "DM Sans", sans-serif;
    }
    .cash-grid span {
      display: block;
      margin-top: 6px;
      color: var(--warm-gray);
      font-size: .78rem;
      font-weight: 600;
    }
    .toggle-row {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 12px;
      min-height: 54px;
      padding: 12px 14px;
      border-radius: 14px;
      border: 1px dashed rgba(160,112,64,.35);
      background: var(--gold-pale);
    }
    .toggle-row button {
      min-width: 86px;
      min-height: 38px;
      border-radius: 999px;
      border: 0;
      font: 700 .88rem/1 "DM Sans", sans-serif;
      background: var(--wine);
      color: white;
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
      font: 800 2rem/1 "DM Sans", sans-serif;
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
      font: 700 .92rem/1.1 "DM Sans", sans-serif;
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
  <script type="module">
    import { h, render } from "https://esm.sh/preact@10.25.4";
    import htm from "https://esm.sh/htm@3.1.1";
    import { useState } from "https://esm.sh/preact@10.25.4/hooks";

    const html = htm.bind(h);

    const QUICK_ITEMS = [
      { itemId: "prayer-card-50c", label: "Prayer Card", emoji: "🙏", priceLabel: "$0.50", live: true, note: "Live quick item" },
      { itemId: "candle-placeholder", label: "Votive Candle", emoji: "🕯️", priceLabel: "$1.00", live: false, note: "UI parity tile" },
      { itemId: "bookmark-placeholder", label: "Bookmark", emoji: "📑", priceLabel: "$1.50", live: false, note: "UI parity tile" },
      { itemId: "greeting-placeholder", label: "Greeting Card", emoji: "✉️", priceLabel: "$3.50", live: false, note: "UI parity tile" },
    ];

    function App() {
      const [screen, setScreen] = useState("login");
      const [mode, setMode] = useState("scan");
      const [pin, setPin] = useState("");
      const [token, setToken] = useState("");
      const [barcode, setBarcode] = useState("9780060652937");
      const [cart, setCart] = useState([]);
      const [total, setTotal] = useState(0);
      const [status, setStatus] = useState({
        tone: "warning",
        title: "Shift not started",
        detail: "Enter the 4-digit PIN to begin the Sunday-rush POS flow.",
      });
      const [paymentMethod, setPaymentMethod] = useState("");
      const [customTendered, setCustomTendered] = useState("20.00");
      const [donateChange, setDonateChange] = useState(true);
      const [iouName, setIouName] = useState("John Doe");
      const [receiptEmail, setReceiptEmail] = useState("jane@example.com");
      const [discountCode, setDiscountCode] = useState("");
      const [lastSale, setLastSale] = useState(null);

      const money = (cents) => `$${(Number(cents || 0) / 100).toFixed(2)}`;
      const discountRate = discountCode === "clergy" ? 0.10 : discountCode === "volunteer" ? 0.15 : discountCode === "bulk" ? 0.20 : 0;
      const discountValue = Math.round(total * discountRate);

      const applyCart = (payload) => {
        setCart(Array.isArray(payload.items) ? payload.items : []);
        setTotal(Number.isFinite(payload.total_cents) ? payload.total_cents : 0);
      };

      const setUiStatus = (tone, title, detail) => {
        setStatus({ tone, title, detail });
      };

      const request = async (url, payload) => {
        const res = await fetch(url, {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify(payload),
        });
        const json = await res.json().catch(() => ({}));
        if (!res.ok) {
          setUiStatus("danger", json.error || "Request failed", json.message || "The POS endpoint returned an error.");
          return { ok: false, json };
        }
        return { ok: true, json };
      };

      const resetSale = () => {
        setCart([]);
        setTotal(0);
        setPaymentMethod("");
        setDonateChange(true);
        setIouName("John Doe");
        setReceiptEmail("jane@example.com");
        setDiscountCode("");
        setLastSale(null);
        setUiStatus("warning", "Ready for next customer", "Scan a title or use a quick tile to build the next basket.");
        setScreen("main");
      };

      const startShift = async (pinValue) => {
        const result = await request("/api/pos/login", { pin: pinValue });
        if (!result.ok) {
          setPin("");
          return;
        }
        const nextToken = result.json.session_token || "";
        setToken(nextToken);
        setPin("");
        resetSale();
        setUiStatus("success", "Shift started", nextToken ? `Session ${nextToken} is ready for scanning and checkout.` : "POS session opened.");
      };

      const pushDigit = (digit) => {
        if (pin.length >= 4) return;
        const next = `${pin}${digit}`;
        setPin(next);
        if (next.length === 4) {
          window.setTimeout(() => startShift(next), 220);
        }
      };

      const removeDigit = () => {
        setPin((current) => current.slice(0, -1));
      };

      const scanItem = async () => {
        if (!token) {
          setUiStatus("danger", "Shift missing", "Start a shift before scanning items.");
          setScreen("login");
          return;
        }
        const result = await request("/api/pos/scan", { session_token: token, isbn: barcode });
        if (!result.ok) return;
        applyCart(result.json);
        setLastSale(null);
        setUiStatus("success", "Scanned to cart", result.json.message || "The item was added to the current sale.");
      };

      const addQuickItem = async (item) => {
        if (!item.live) {
          setUiStatus("warning", "Demo tile only", `${item.label} is shown for parity, but only the seeded Prayer Card quick item is live in this MVP.`);
          return;
        }
        const result = await request("/api/pos/cart/items", { session_token: token, item_id: item.itemId, quantity: 1 });
        if (!result.ok) return;
        applyCart(result.json);
        setLastSale(null);
        setUiStatus("success", "Quick item added", result.json.message || `${item.label} was added to the basket.`);
      };

      const beginCheckout = () => {
        if (!total) {
          setUiStatus("warning", "Basket empty", "Scan an item or tap a quick tile before opening payment options.");
          return;
        }
        setPaymentMethod("");
        setScreen("payment");
      };

      const completeCard = async () => {
        const result = await request("/api/pos/payments/external-card", {
          session_token: token,
          external_ref: "square-ui-posh",
        });
        if (!result.ok) return;
        finalizeSale(result.json, "Card sale complete");
      };

      const completeCash = async (tenderedCents) => {
        const result = await request("/api/pos/payments/cash", {
          session_token: token,
          tendered_cents: tenderedCents,
          donate_change: donateChange,
        });
        if (!result.ok) return;
        finalizeSale(result.json, "Cash sale complete");
      };

      const completeIou = async () => {
        const result = await request("/api/pos/payments/iou", {
          session_token: token,
          customer_name: iouName,
        });
        if (!result.ok) return;
        finalizeSale(result.json, "Sale moved to IOU");
      };

      const finalizeSale = (payload, fallbackTitle) => {
        setLastSale(payload);
        setCart([]);
        setTotal(0);
        const tone = payload.status === "iou" ? "warning" : "success";
        const detailParts = [
          `Total ${money(payload.total_cents || 0)}`,
          payload.change_due_cents ? `Change ${money(payload.change_due_cents)}` : "",
          payload.donation_cents ? `Donation ${money(payload.donation_cents)}` : "",
        ].filter(Boolean);
        setUiStatus(tone, payload.message || fallbackTitle, detailParts.join(" · ") || "Payment completed.");
        setScreen("complete");
      };

      const cashPresets = [
        { label: money(total), cents: total, note: "Exact" },
        { label: "$20.00", cents: 2000, note: "Quick cash" },
        { label: "$50.00", cents: 5000, note: "Notes" },
        { label: "$100.00", cents: 10000, note: "Large note" },
      ].filter((option) => option.cents >= total && total > 0);

      const statusClass = `status-panel ${status.tone === "success" ? "status-success" : status.tone === "danger" ? "status-danger" : "status-warning"}`;

      if (screen === "login") {
        return html`
          <main class="pos-shell">
            <div class="pos-wrap center-shell">
              <section class="pin-head">
                <p class="kicker">Screen 1 · PIN Login</p>
                <h1>SCRIPTORIUM</h1>
                <p>Enter PIN to begin a parish bookshop shift.</p>
              </section>
              <section class="pin-card">
                <div class="pin-dots" aria-label="Enter PIN">
                  ${[0, 1, 2, 3].map((index) => html`<span class=${`pin-dot ${index < pin.length ? "pin-dot--filled" : ""}`}></span>`)}
                </div>
                <div class="pin-grid">
                  ${["1", "2", "3", "4", "5", "6", "7", "8", "9", "", "0", "⌫"].map((key) => {
                    if (!key) return html`<div></div>`;
                    return html`<button class=${`pin-key ${key === "⌫" ? "pin-key--ghost" : ""}`} onClick=${() => key === "⌫" ? removeDigit() : pushDigit(key)}>${key}</button>`;
                  })}
                </div>
              </section>
              <section class=${statusClass}>
                <h3>${status.title}</h3>
                <p>${status.detail}</p>
              </section>
            </div>
          </main>
        `;
      }

      if (screen === "payment") {
        return html`
          <main class="pos-shell">
            <div class="pos-wrap">
              <section class="hero">
                <p class="kicker">Screen 3 · Payment</p>
                <h1>Choose how to finish</h1>
                <p>Large payment targets and clear handoff language keep the volunteer flow calm.</p>
                <div class="session-row">
                  <span class="session-pill">Basket ${money(total)}</span>
                  <span class="session-pill">${cart.length} line item(s)</span>
                </div>
              </section>
              ${!paymentMethod && html`
                <section class="card actions">
                  <button class="payment-option" onClick=${() => setPaymentMethod("card")}>
                    <span class="payment-icon" style=${{ background: "var(--blue-light)" }}>💳</span>
                    <span>
                      <span class="payment-title">Card</span>
                      <span class="payment-copy">External terminal handoff with confirmation back in POS.</span>
                    </span>
                  </button>
                  <button class="payment-option" onClick=${() => setPaymentMethod("cash")}>
                    <span class="payment-icon" style=${{ background: "var(--success-light)" }}>💵</span>
                    <span>
                      <span class="payment-title">Cash</span>
                      <span class="payment-copy">Preset tender buttons, change, and donation round-up.</span>
                    </span>
                  </button>
                  <button class="payment-option" onClick=${() => setPaymentMethod("iou")}>
                    <span class="payment-icon" style=${{ background: "var(--warning-light)" }}>🧾</span>
                    <span>
                      <span class="payment-title">Put on Tab / IOU</span>
                      <span class="payment-copy">Capture the customer name and leave the order unpaid.</span>
                    </span>
                  </button>
                </section>
              `}
              ${paymentMethod === "card" && html`
                <section class="card">
                  <h2 class="section-title">Card handoff</h2>
                  <p class="subtle">Open the terminal, take the card, then record the result here.</p>
                  <div class="totals" style=${{ marginTop: "14px" }}>
                    <div class="totals-row"><span>Total due</span><strong>${money(total)}</strong></div>
                    ${discountCode && html`<div class="totals-row"><span>Discount selected</span><span>${money(discountValue)} (${discountCode})</span></div>`}
                    <div class="totals-row"><span>Provider</span><span>Square handoff</span></div>
                  </div>
                  <div class="actions" style=${{ marginTop: "14px" }}>
                    <button class="pos-button--lg" onClick=${completeCard}>Payment Received</button>
                    <button class="pos-button--lg pos-button--ghost" onClick=${() => setPaymentMethod("")}>Back to methods</button>
                  </div>
                </section>
              `}
              ${paymentMethod === "cash" && html`
                <section class="card">
                  <h2 class="section-title">Cash tendered</h2>
                  <p class="subtle">Use a quick amount or type a custom note denomination.</p>
                  <div class="cash-grid">
                    ${cashPresets.map((option) => html`
                      <button onClick=${() => completeCash(option.cents)}>
                        ${option.label}
                        <span>${option.note}</span>
                      </button>
                    `)}
                  </div>
                  <div style=${{ marginTop: "14px" }}>
                    <label class="field-label" for="custom-tendered">Custom cash amount</label>
                    <input id="custom-tendered" value=${customTendered} onInput=${(event) => setCustomTendered(event.target.value)} />
                  </div>
                  <div class="toggle-row" style=${{ marginTop: "14px" }}>
                    <div>
                      <strong>Donate change</strong>
                      <div class="payment-copy">Use the round-up flow from the design spec.</div>
                    </div>
                    <button onClick=${() => setDonateChange((current) => !current)}>${donateChange ? "On" : "Off"}</button>
                  </div>
                  <div class="actions" style=${{ marginTop: "14px" }}>
                    <button class="pos-button--lg pos-button--success" onClick=${() => completeCash(Math.round(Number(customTendered || 0) * 100))}>Use custom amount</button>
                    <button class="pos-button--lg pos-button--ghost" onClick=${() => setPaymentMethod("")}>Back to methods</button>
                  </div>
                </section>
              `}
              ${paymentMethod === "iou" && html`
                <section class="card">
                  <h2 class="section-title">Record IOU</h2>
                  <p class="subtle">This order will appear in the admin queue until the customer pays.</p>
                  <label class="field-label" for="iou-name">Customer name</label>
                  <input id="iou-name" value=${iouName} onInput=${(event) => setIouName(event.target.value)} />
                  <div class="actions" style=${{ marginTop: "14px" }}>
                    <button class="pos-button--lg pos-button--gold" onClick=${completeIou}>Record IOU</button>
                    <button class="pos-button--lg pos-button--ghost" onClick=${() => setPaymentMethod("")}>Back to methods</button>
                  </div>
                </section>
              `}
              <section class=${statusClass}>
                <h3>${status.title}</h3>
                <p>${status.detail}</p>
              </section>
              <button class="pos-button--lg pos-button--ghost" onClick=${() => setScreen("main")}>Back to basket</button>
            </div>
          </main>
        `;
      }

      if (screen === "complete") {
        const sale = lastSale || {};
        return html`
          <main class="pos-shell">
            <div class="pos-wrap complete-screen">
              <div class="complete-mark"><span>✓</span></div>
              <div>
                <p class="kicker">Screen 4 · Complete</p>
                <h1 class="complete-title">SALE COMPLETE</h1>
              </div>
              <section class="receipt-card">
                <div class="receipt-row"><span>Payment outcome</span><strong>${sale.status === "iou" ? "IOU recorded" : "Paid"}</strong></div>
                <div class="receipt-row"><span>Order total</span><strong>${money(sale.total_cents || 0)}</strong></div>
                <div class=${`receipt-row ${sale.change_due_cents ? "receipt-row--big" : ""}`}><span>Change due</span><strong>${money(sale.change_due_cents || 0)}</strong></div>
                <div class="receipt-row"><span>Donation</span><strong>${money(sale.donation_cents || 0)}</strong></div>
              </section>
              <section class="receipt-card">
                <label class="field-label" for="receipt-email" style=${{ color: "white", textAlign: "left" }}>Email receipt</label>
                <div class="row">
                  <input id="receipt-email" value=${receiptEmail} onInput=${(event) => setReceiptEmail(event.target.value)} />
                  <button class="pos-button--lg" onClick=${() => setUiStatus("success", "Receipt queued", receiptEmail ? `Receipt will be sent to ${receiptEmail}.` : "Add an email to send a receipt.")}>Send receipt</button>
                </div>
              </section>
              <section class=${statusClass}>
                <h3>${status.title}</h3>
                <p>${status.detail}</p>
              </section>
              <button class="pos-button--lg pos-button--success" onClick=${resetSale}>Start next sale</button>
            </div>
          </main>
        `;
      }

      return html`
        <main class="pos-shell">
          <div class="pos-wrap">
            <section class="hero">
              <p class="kicker">Screen 2 · Main POS</p>
              <h1>Scriptorium POS</h1>
              <p>Scan books, tap quick items, review the basket, then move into a payment screen that mirrors the prototype flow.</p>
              <div class="session-row">
                <span class="session-pill">${token ? `Session ${token}` : "Session offline"}</span>
                <span class="session-pill">${cart.length ? `${cart.length} items ready` : "Awaiting first item"}</span>
              </div>
            </section>
            <section class="card">
              <div class="top-actions">
                <button class="pos-button--lg pos-button--gold" onClick=${() => setScreen("login")}>New volunteer login</button>
                <button class="pos-button--lg pos-button--ghost" onClick=${() => setBarcode("9780060652937")}>Reload sample ISBN</button>
              </div>
            </section>
            <section class="card">
              <div class="toolbar">
                <button class=${mode === "scan" ? "is-active" : ""} onClick=${() => setMode("scan")}>Scan Item</button>
                <button class=${mode === "quick" ? "is-active" : ""} onClick=${() => setMode("quick")}>Quick Items</button>
              </div>
              ${mode === "scan" ? html`
                <div style=${{ marginTop: "14px" }}>
                  <div class="scan-frame"><div class="scan-caption">Point camera at ISBN, EAN-13, or typed barcode</div></div>
                  <label class="field-label" for="barcode">ISBN / barcode</label>
                  <input id="barcode" value=${barcode} onInput=${(event) => setBarcode(event.target.value)} />
                  <div class="actions" style=${{ marginTop: "10px" }}>
                    <button class="pos-button--lg" onClick=${scanItem}>Scan to cart</button>
                    <p class="hint">The UI posts the isbn field, and the API accepts both isbn and barcode to stay compatible with the BDD flow.</p>
                  </div>
                </div>
              ` : html`
                <div class="quick-grid" style=${{ marginTop: "14px" }}>
                  ${QUICK_ITEMS.map((item) => html`
                    <button class="quick-tile" onClick=${() => addQuickItem(item)}>
                      <span class="quick-emoji">${item.emoji}</span>
                      ${item.label}
                      <span class="quick-price">${item.priceLabel}</span>
                      <span class="quick-note">${item.note}</span>
                    </button>
                  `)}
                </div>
              `}
            </section>
            <section class="card">
              <h2 class="section-title">Basket</h2>
              ${cart.length ? html`
                <div class="cart-list">
                  ${cart.map((item) => html`
                    <div class="cart-row" key=${item.item_id}>
                      <div>
                        <div class="cart-title">${item.title}</div>
                        <div class="cart-meta">Qty ${item.quantity}</div>
                        <span class=${`cart-tag ${item.is_quick_item ? "cart-tag--quick" : "cart-tag--scan"}`}>${item.is_quick_item ? "Quick item" : "Scanned item"}</span>
                      </div>
                      <div class="cart-price">${money(item.unit_price_cents * item.quantity)}</div>
                    </div>
                  `)}
                </div>
              ` : html`<div class="empty-state">Cart empty. Scan an item or use a quick tile to start the sale.</div>`}
              <div class="totals" style=${{ marginTop: "12px" }}>
                <div class="totals-row"><span>Current total</span><strong>${money(total)}</strong></div>
                ${discountCode && html`<div class="totals-row"><span>Discount selected</span><span>${money(discountValue)} (${discountCode})</span></div>`}
                <div class="totals-row"><span>Checkout path</span><span>Card, cash, or IOU</span></div>
              </div>
              <div class="discount-grid">
                ${[
                  ["", "No discount"],
                  ["clergy", "10% Clergy"],
                  ["volunteer", "15% Volunteer"],
                  ["bulk", "20% Bulk"],
                ].map(([code, label]) => html`<button class=${`discount-chip ${discountCode===code?"discount-chip--active":""}`} onClick=${() => setDiscountCode(code)}>${label}</button>`)}
              </div>
            </section>
            <section class=${statusClass}>
              <h3>${status.title}</h3>
              <p>${status.detail}</p>
            </section>
            <button class="pos-button--lg pos-button--success" onClick=${beginCheckout}>Checkout · ${money(total)}</button>
          </div>
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
