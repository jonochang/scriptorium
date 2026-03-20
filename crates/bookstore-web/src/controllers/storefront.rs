use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Html;
use bookstore_app::{RequestContext, SalesEvent};
use bookstore_domain::{OrderChannel, OrderStatus, PaymentMethod};
use std::time::Instant;

use crate::AppState;
use crate::catalog_ui::{
    book_binding, book_blurb, book_isbn, book_pages, book_publisher, catalog_categories,
    filter_books, format_money, render_catalog_cards, render_catalog_category_chips,
    render_catalog_pagination, stock_hint,
};
use bookstore_app::seed::SeedData;
use crate::models::{
    CatalogQuery, StorefrontCheckoutSessionRequest, StorefrontCheckoutSessionResponse,
};
use crate::storefront_ui;
use crate::ui::{
    google_fonts_link, html_escape, page_header, page_header_centered, shared_styles, site_footer, site_nav,
};
use crate::web_support::{current_utc_datetime, log_checkout_event};

fn storefront_cart_script() -> &'static str {
    storefront_ui::storefront_cart_script()
}

pub async fn storefront_catalog(
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
    let items = render_catalog_cards(&state.seed, paged_books);
    let pagination =
        render_catalog_pagination(page, total_pages, query.q.as_deref(), query.category.as_deref());
    let category_chips = render_catalog_category_chips(
        &categories,
        query.q.as_deref(),
        query.category.as_deref(),
        &filtered_books,
    );
    let search_value = html_escape(query.q.as_deref().unwrap_or(""));
    let active_category =
        query.category.as_deref().filter(|value| !value.trim().is_empty()).unwrap_or("All");
    let catalog_actions = "";
    Html(
        [
            "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\" /><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" /><title>Scriptorium Catalog</title>",
            google_fonts_link(),
            "<script src=\"https://unpkg.com/htmx.org@2.0.4\"></script><style>",
            shared_styles(),
            "</style></head><body class=\"page-shell\">",
            &site_nav("catalog"),
            "<main class=\"page-stack page-stack--wide\">",
            &page_header_centered(
                "Feed your soul.",
                "Find books for parish reading, gifting, and liturgical practice.",
            ),
            "<section class=\"surface-card\"><form class=\"catalog-search\" action=\"/catalog\" method=\"get\" hx-get=\"/catalog/search\" hx-target=\"#results\" hx-push-url=\"true\"><label class=\"field-label\" for=\"catalog-search\">Search catalog</label><input type=\"hidden\" name=\"category\" value=\"",
            &html_escape(query.category.as_deref().unwrap_or("")),
            "\" /><div class=\"catalog-search-row\"><input id=\"catalog-search\" name=\"q\" value=\"",
            &search_value,
            "\" placeholder=\"Try Discipline or Foster\" /><button class=\"accent-button\" type=\"submit\">Search</button></div></form><div class=\"category-strip\">",
            &category_chips,
            "</div><div class=\"catalog-results-head\"><strong>",
            &format!("{} titles", filtered_books.len()),
            "</strong></div><div id=\"results\">",
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

pub async fn storefront_search(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Html<String> {
    let query = params.get("q").map_or("", String::as_str).to_ascii_lowercase();
    let category = params.get("category").map(String::as_str);
    let books = state.catalog.list_books().await;
    let filtered = render_catalog_cards(&state.seed, filter_books(books, Some(&query), category));
    Html(filtered)
}

pub async fn storefront_product_detail(
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
                    "<main class=\"page-stack\">",
                    &page_header(
                        "",
                        "Title not found",
                        "That catalog item is not available in this parish shelf view. Return to browsing and choose another title.",
                        &[],
                        "",
                    ),
                    "<section class=\"surface-card\"><h2 class=\"section-title\">We could not find that product</h2><p class=\"helper-copy helper-copy--flush\">The requested book id does not exist in the seeded catalog. Try the main shelf, search by title, or continue with another selection.</p><div style=\"margin-top:14px\"><a href=\"/catalog\" class=\"ghost-link ghost-link--ink\" style=\"display:inline-flex;align-items:center;gap:5px;font-size:14px\"><svg width=\"14\" height=\"14\" viewBox=\"0 0 16 16\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.5\" stroke-linecap=\"round\"><path d=\"M10 12L6 8l4-4\"/></svg>Back to catalog</a></div></section></main>",
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
    let seed: &SeedData = &state.seed;
    let price = format_money(book.price_cents);
    let (stock_label, stock_class) = stock_hint(seed, &book.id);
    let blurb = book_blurb(seed, &book.id);
    let publisher = book_publisher(seed, &book.id);
    let isbn = book_isbn(seed, &book.id);
    let binding = book_binding(seed, &book.id);
    let pages = book_pages(seed, &book.id);
    let detail_actions = "";
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
            "<main class=\"page-stack page-stack--wide\">",
            "<div style=\"padding:0 0 4px\"><a href=\"/catalog\" class=\"ghost-link ghost-link--ink\" style=\"display:inline-flex;align-items:center;gap:5px;font-size:14px\"><svg width=\"14\" height=\"14\" viewBox=\"0 0 16 16\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.5\" stroke-linecap=\"round\"><path d=\"M10 12L6 8l4-4\"/></svg>Back to catalog</a></div>",
            &page_header(
                "",
                &book.title,
                "",
                &[],
                detail_actions,
            ),
            "<section class=\"product-layout\"><article class=\"surface-card\"><div class=\"catalog-cover catalog-cover--detail\"><div class=\"book-cover-art\"><span class=\"book-cover-art__eyebrow\">Parish shelf edition</span><strong>",
            &html_escape(&book.title),
            "</strong><span>",
            &html_escape(&book.author),
            "</span></div></div></article><article class=\"surface-card product-summary\"><span class=\"chip\">",
            &html_escape(&book.category),
            "</span><h2 class=\"section-title\">",
            &html_escape(&book.title),
            "</h2><p class=\"catalog-meta\">",
            &html_escape(&book.author),
            "</p><div class=\"detail-price-row\"><div class=\"detail-price\">",
            &price,
            "</div><span class=\"",
            &stock_class,
            "\">",
            &stock_label,
            "</span></div><section class=\"detail-section\"><h3 class=\"detail-heading\">Description</h3><p class=\"detail-copy\">",
            &blurb,
            "</p></section><section class=\"detail-section\"><h3 class=\"detail-heading\">Details</h3><div class=\"detail-table\"><div class=\"detail-table__row\"><span>Publisher</span><strong>",
            &publisher,
            "</strong></div><div class=\"detail-table__row\"><span>ISBN</span><strong>",
            &isbn,
            "</strong></div><div class=\"detail-table__row\"><span>Binding</span><strong>",
            &binding,
            "</strong></div><div class=\"detail-table__row\"><span>Pages</span><strong>",
            &pages,
            "</strong></div></div></section><div class=\"inline-quantity\"><div><label class=\"field-label\" for=\"detail-quantity\">Quantity</label><div style=\"display:flex;align-items:center;gap:8px\"><button type=\"button\" class=\"ghost-link ghost-link--ink\" style=\"width:36px;height:36px;padding:0;justify-content:center;font-size:16px\" onclick=\"var i=document.getElementById('detail-quantity');var v=parseInt(i.value)||1;if(v>1){i.value=v-1}\">&#8722;</button><input id=\"detail-quantity\" type=\"number\" min=\"1\" value=\"1\" style=\"width:56px;text-align:center\" /><button type=\"button\" class=\"ghost-link ghost-link--ink\" style=\"width:36px;height:36px;padding:0;justify-content:center;font-size:16px\" onclick=\"var i=document.getElementById('detail-quantity');var v=parseInt(i.value)||1;i.value=v+1\">+</button></div></div><div class=\"stack-list stack-list--tight\"><button class=\"primary-button primary-button--block\" type=\"button\" data-add-book-id=\"",
            &html_escape(&book.id),
            "\" data-add-book-title=\"",
            &html_escape(&book.title),
            "\" data-add-book-author=\"",
            &html_escape(&book.author),
            "\" data-add-book-price-cents=\"",
            &book.price_cents.to_string(),
            "\" data-add-book-quantity-target=\"detail-quantity\" data-feedback-target=\"cart-feedback\">Add to Cart — ",
            &price,
            "</button><a class=\"ghost-link ghost-link--ink\" href=\"/checkout\">Proceed to checkout</a></div></div><div id=\"cart-feedback\" class=\"notice-panel\">Ready to add this title to the cart.</div><div class=\"divider-title divider-title--spaced\">Related titles</div><div class=\"stack-list\">",
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

pub async fn storefront_cart(State(state): State<AppState>) -> Html<String> {
    let books = state.catalog.list_books().await;
    let recommendations = books
        .into_iter()
        .take(3)
        .map(|book| {
            format!(
                "<div class=\"list-row recommendation-row\" data-recommendation-book-id=\"{}\" data-recommendation-title=\"{}\"><div><div class=\"list-title\">{}</div><div class=\"list-meta\">{} · {}</div></div><a class=\"ghost-link ghost-link--ink\" href=\"/catalog/items/{}\">View</a></div>",
                html_escape(&book.id),
                html_escape(&book.title),
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
            "<main class=\"page-stack page-stack--wide\">",
            &page_header(
                "",
                "Review your basket",
                "Confirm quantities, keep gifting simple, and move smoothly into checkout.",
                &[],
                "",
            ),
            "<section class=\"checkout-layout\"><article class=\"surface-card\"><h2 class=\"section-title\">Cart items</h2><div id=\"cart-items\" class=\"stack-list\"><div class=\"empty-inline\">Your cart is empty.</div></div><div class=\"notice-panel notice-panel--success\" id=\"cart-summary\" style=\"margin-top:16px\">Cart total: $0.00</div><div class=\"button-row\" style=\"margin-top:14px\"><button class=\"ghost-link ghost-link--ink\" type=\"button\" id=\"clear-cart\">Clear basket</button><a class=\"primary-button\" href=\"/checkout\">Proceed to checkout</a></div></article><article class=\"surface-card\"><h2 class=\"section-title\">Recommended titles</h2><div id=\"cart-recommendations\" class=\"stack-list\">",
            &recommendations,
            "</div><div id=\"cart-recommendations-empty\" class=\"empty-inline\" hidden>Recommendations update automatically so titles already in the basket are not repeated here.</div></article></section></main>",
            site_footer(),
            storefront_cart_script(),
            "</body></html>",
        ]
        .concat(),
    )
}

pub async fn storefront_checkout() -> Html<String> {
    Html(storefront_ui::storefront_checkout_shell_html(
        google_fonts_link(),
        shared_styles(),
        &site_nav("checkout"),
        &page_header(
            "",
            "Finish your order",
            "Confirm your contact details, choose any extra parish support, and place the order with confidence.",
            &[],
            "",
        ),
        site_footer(),
    ))
}

pub async fn storefront_orders(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Html<String> {
    let placed_id = params.get("placed").cloned().unwrap_or_default();
    let tenant_id = state.admin.default_tenant_id().to_string();
    let orders = state.admin.list_orders(&tenant_id).await;
    let online_orders: Vec<_> =
        orders.into_iter().filter(|o| o.channel == OrderChannel::Online).collect();
    Html(storefront_ui::storefront_orders_shell_html(
        google_fonts_link(),
        shared_styles(),
        &site_nav("orders"),
        &page_header(
            "",
            "Order history",
            "View your recent orders placed through the bookstore.",
            &[],
            "",
        ),
        site_footer(),
        &placed_id,
        &online_orders,
    ))
}

pub async fn storefront_checkout_session(
    State(state): State<AppState>,
    axum::extract::Extension(context): axum::extract::Extension<RequestContext>,
    Json(request): Json<StorefrontCheckoutSessionRequest>,
) -> Result<Json<StorefrontCheckoutSessionResponse>, axum::http::StatusCode> {
    let started_at = Instant::now();
    if request.line_items.is_empty() || request.donation_cents < 0 {
        return Err(axum::http::StatusCode::BAD_REQUEST);
    }
    let price_by_id = state
        .catalog
        .list_books()
        .await
        .into_iter()
        .map(|book| (book.id, book.price_cents))
        .collect::<std::collections::HashMap<_, _>>();
    let mut subtotal_cents = 0_i64;
    for line in &request.line_items {
        if line.quantity <= 0 {
            return Err(axum::http::StatusCode::BAD_REQUEST);
        }
        let unit_price =
            price_by_id.get(&line.item_id).copied().ok_or(axum::http::StatusCode::BAD_REQUEST)?;
        subtotal_cents += unit_price * line.quantity;
    }
    let is_pickup = request.delivery_method == "pickup";
    let shipping_cents = if subtotal_cents > 0 && !is_pickup { 599 } else { 0 };
    let tax_cents = ((subtotal_cents * 7) + 50) / 100;
    let sales_cents = subtotal_cents + shipping_cents + tax_cents;
    let tenant_id = if context.tenant_id == "default" {
        state.admin.default_tenant_id().to_string()
    } else {
        context.tenant_id
    };
    let customer_name = if request.customer_name.trim().is_empty() {
        if request.email.trim().is_empty() {
            "Online Customer".to_string()
        } else {
            request.email.clone()
        }
    } else {
        request.customer_name.clone()
    };
    let session = state
        .storefront
        .create_checkout_session(
            tenant_id.clone(),
            sales_cents,
            shipping_cents,
            tax_cents,
            request.donation_cents,
            request.email,
        )
        .await
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    // Directly create the order and finalize (no external payment processor in demo mode)
    let today = current_utc_datetime();
    let order = state
        .admin
        .create_order(
            &tenant_id,
            &customer_name,
            OrderChannel::Online,
            OrderStatus::Paid,
            PaymentMethod::OnlineCard,
            session.total_cents,
            today,
        )
        .await;
    state
        .admin
        .record_sales_event(SalesEvent {
            tenant_id: tenant_id.clone(),
            payment_method: PaymentMethod::OnlineCard,
            sales_cents: session.sales_cents,
            donations_cents: session.donation_cents,
            cogs_cents: 0,
            occurred_at: today,
        })
        .await;
    // Mark session so webhook won't double-create the order
    state.storefront.mark_order_created(&session.session_id).await;
    log_checkout_event(
        "storefront_checkout_session",
        "created",
        "online_card",
        session.total_cents,
        started_at,
    );
    Ok(Json(StorefrontCheckoutSessionResponse {
        session_id: session.session_id,
        order_id: order.order_id,
        total_cents: session.total_cents,
    }))
}
