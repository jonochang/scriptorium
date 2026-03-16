use wasm_bindgen::prelude::*;
use web_sys::{Document, HtmlElement, HtmlInputElement};

fn document() -> Document {
    web_sys::window().expect("window").document().expect("document")
}

fn by_id(id: &str) -> Option<web_sys::Element> {
    document().get_element_by_id(id)
}

fn set_text(id: &str, value: &str) {
    if let Some(el) = by_id(id) {
        el.set_text_content(Some(value));
    }
}

fn money(cents: f64) -> String {
    let value = if cents.abs() < 0.005 { 0.0 } else { cents / 100.0 };
    format!("${:.2}", value)
}

fn get_input_value(id: &str) -> String {
    by_id(id)
        .and_then(|el| el.dyn_into::<HtmlInputElement>().ok())
        .map(|input| input.value())
        .unwrap_or_default()
}

fn set_status(message: &str, tone: &str) {
    if let Some(panel) = by_id("admin-status") {
        panel.set_text_content(Some(message));
        let class = if tone.is_empty() {
            "notice-panel".to_string()
        } else {
            format!("notice-panel notice-panel--{tone}")
        };
        panel.set_class_name(&class);
    }
}

fn render_list(container_id: &str, items_html: &str, empty_message: &str) {
    if let Some(node) = by_id(container_id) {
        if items_html.is_empty() {
            node.set_inner_html(&format!(r#"<div class="empty-inline">{empty_message}</div>"#));
        } else {
            node.set_inner_html(items_html);
        }
    }
}

// ---- Admin session from window global ----

fn admin_token() -> String {
    web_sys::window()
        .and_then(|w| js_sys::Reflect::get(&w, &JsValue::from_str("SCRIPTORIUM_ADMIN_SESSION")).ok())
        .and_then(|session| js_sys::Reflect::get(&session, &JsValue::from_str("token")).ok())
        .and_then(|v| v.as_string())
        .unwrap_or_default()
}

fn admin_tenant() -> String {
    web_sys::window()
        .and_then(|w| js_sys::Reflect::get(&w, &JsValue::from_str("SCRIPTORIUM_ADMIN_SESSION")).ok())
        .and_then(|session| js_sys::Reflect::get(&session, &JsValue::from_str("tenantId")).ok())
        .and_then(|v| v.as_string())
        .unwrap_or_default()
}

// ---- Filter state via window globals ----

fn get_filter(key: &str) -> String {
    web_sys::window()
        .and_then(|w| js_sys::Reflect::get(&w, &JsValue::from_str(key)).ok())
        .and_then(|v| v.as_string())
        .unwrap_or_else(|| "All".to_string())
}

fn set_filter(key: &str, value: &str) {
    if let Some(w) = web_sys::window() {
        let _ = js_sys::Reflect::set(&w, &JsValue::from_str(key), &JsValue::from_str(value));
    }
}

fn get_search(key: &str) -> String {
    web_sys::window()
        .and_then(|w| js_sys::Reflect::get(&w, &JsValue::from_str(key)).ok())
        .and_then(|v| v.as_string())
        .unwrap_or_default()
}

// ---- Snapshot stored on window ----

fn get_snapshot() -> JsValue {
    web_sys::window()
        .and_then(|w| js_sys::Reflect::get(&w, &JsValue::from_str("__adminSnapshot")).ok())
        .unwrap_or(JsValue::NULL)
}

fn set_snapshot(val: &JsValue) {
    if let Some(w) = web_sys::window() {
        let _ = js_sys::Reflect::set(&w, &JsValue::from_str("__adminSnapshot"), val);
    }
}

fn get_admin_orders() -> js_sys::Array {
    web_sys::window()
        .and_then(|w| js_sys::Reflect::get(&w, &JsValue::from_str("__adminOrders")).ok())
        .and_then(|v| v.dyn_into::<js_sys::Array>().ok())
        .unwrap_or_else(js_sys::Array::new)
}

fn set_admin_orders(val: &JsValue) {
    if let Some(w) = web_sys::window() {
        let _ = js_sys::Reflect::set(&w, &JsValue::from_str("__adminOrders"), val);
    }
}

// ---- Helper to extract JS object fields ----

fn js_str(obj: &JsValue, key: &str) -> String {
    js_sys::Reflect::get(obj, &JsValue::from_str(key))
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_default()
}

fn js_f64(obj: &JsValue, key: &str) -> f64 {
    js_sys::Reflect::get(obj, &JsValue::from_str(key))
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0)
}

fn normalize_channel(order: &JsValue) -> String {
    let ch = js_str(order, "channel");
    if ch == "Online" { "Online".to_string() } else { "POS".to_string() }
}

fn order_status_badge(order: &JsValue) -> String {
    if js_str(order, "status") == "Paid" {
        r#"<span class="status-badge status-badge--paid">Paid</span>"#.to_string()
    } else {
        r#"<span class="status-badge status-badge--iou">IOU</span>"#.to_string()
    }
}

fn inventory_status_str(product: &JsValue) -> &'static str {
    let on_hand = js_f64(product, "quantity_on_hand") as i64;
    if on_hand <= 0 {
        "out"
    } else if on_hand <= 3 {
        "low"
    } else {
        "ok"
    }
}

fn inventory_status_badge(product: &JsValue) -> String {
    match inventory_status_str(product) {
        "out" => r#"<span class="office-inline-badge office-inline-badge--out">Out of stock</span>"#.to_string(),
        "low" => r#"<span class="office-inline-badge office-inline-badge--low">Low stock</span>"#.to_string(),
        _ => r#"<span class="office-inline-badge office-inline-badge--ok">In stock</span>"#.to_string(),
    }
}

fn report_query() -> String {
    let from = get_input_value("report-from");
    let to = get_input_value("report-to");
    let tenant = admin_tenant();
    let mut params = format!("tenant_id={}", js_sys::encode_uri_component(&tenant));
    if !from.is_empty() {
        params.push_str(&format!("&from={}", js_sys::encode_uri_component(&from)));
    }
    if !to.is_empty() {
        params.push_str(&format!("&to={}", js_sys::encode_uri_component(&to)));
    }
    params
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

// ---- Render functions ----

fn render_order_actions(order: &JsValue) -> String {
    let order_id = js_str(order, "order_id");
    let eid = escape_html(&order_id);
    let mut actions = format!(
        r#"<button class="ghost-link ghost-link--ink ghost-link--mini" type="button" onclick="viewOrder('{eid}')">View</button><button class="ghost-link ghost-link--ink ghost-link--mini" type="button" onclick="resendReceipt('{eid}')">Resend</button>"#
    );
    if js_str(order, "status") == "UnpaidIou" {
        actions.push_str(&format!(
            r#"<button class="primary-button primary-button--sm" type="button" onclick="markOrderPaid('{eid}')">Mark Paid</button>"#
        ));
    } else {
        actions.push_str(r#"<span class="helper-copy helper-copy--flush">Cleared</span>"#);
    }
    format!(r#"<div class="button-row button-row--compact">{actions}</div>"#)
}

fn render_orders() {
    let orders = get_admin_orders();
    let order_filter = get_filter("__orderFilter");
    let order_search = get_search("__orderSearch").to_lowercase();

    let all_orders: Vec<JsValue> = orders.iter().collect();
    let filtered: Vec<&JsValue> = all_orders
        .iter()
        .filter(|order| {
            let matches_filter = order_filter == "All"
                || (order_filter == "IOU" && js_str(order, "status") == "UnpaidIou")
                || (order_filter != "IOU" && normalize_channel(order) == order_filter);
            if !matches_filter {
                return false;
            }
            if order_search.is_empty() {
                return true;
            }
            [
                js_str(order, "order_id"),
                js_str(order, "customer_name"),
                js_str(order, "payment_method"),
                js_str(order, "channel"),
            ]
            .iter()
            .any(|v| v.to_lowercase().contains(&order_search))
        })
        .collect();

    let iou_count = all_orders
        .iter()
        .filter(|o| js_str(o, "status") == "UnpaidIou")
        .count();

    let paid_revenue: f64 = filtered
        .iter()
        .filter(|o| js_str(o, "status") == "Paid")
        .map(|o| js_f64(o, "total_cents"))
        .sum();

    let open_iou_cents: f64 = filtered
        .iter()
        .filter(|o| js_str(o, "status") == "UnpaidIou")
        .map(|o| js_f64(o, "total_cents"))
        .sum();

    if let Some(iou_btn) = by_id("order-filter-iou-label") {
        iou_btn.set_text_content(Some(&format!("IOU ({iou_count})")));
    }
    set_text("order-summary-count", &filtered.len().to_string());
    set_text("order-summary-revenue", &money(paid_revenue));
    set_text("order-summary-iou", &money(open_iou_cents));
    set_text(
        "order-results-caption",
        &format!("Showing {} of {} orders", filtered.len(), all_orders.len()),
    );

    let node = match by_id("admin-orders") {
        Some(n) => n,
        None => return,
    };

    if filtered.is_empty() {
        node.set_inner_html(r#"<div class="orders-table-wrap"><table class="orders-table"><tbody><tr><td colspan="8"><div class="empty-inline">No orders found for this filter.</div></td></tr></tbody></table></div>"#);
        return;
    }

    let rows: String = filtered
        .iter()
        .map(|order| {
            let oid = escape_html(&js_str(order, "order_id"));
            let date = escape_html(&js_str(order, "created_at"));
            let channel = escape_html(&js_str(order, "channel"));
            let customer = escape_html(&js_str(order, "customer_name"));
            let method = escape_html(&js_str(order, "payment_method"));
            let total = money(js_f64(order, "total_cents"));
            let badge = order_status_badge(order);
            let actions = render_order_actions(order);
            format!(
                r#"<tr><td>{oid}</td><td>{date}</td><td>{channel}</td><td>{customer}</td><td>{badge}</td><td>{method}</td><td><strong>{total}</strong></td><td>{actions}</td></tr>"#
            )
        })
        .collect();

    node.set_inner_html(&format!(
        r#"<table class="orders-table"><thead><tr><th>Order ID</th><th>Date</th><th>Channel</th><th>Customer</th><th>Status</th><th>Method</th><th>Total</th><th>Actions</th></tr></thead><tbody>{rows}</tbody></table>"#
    ));
}

fn filtered_products(snapshot: &JsValue) -> Vec<JsValue> {
    let products = js_sys::Reflect::get(snapshot, &JsValue::from_str("products"))
        .ok()
        .and_then(|v| v.dyn_into::<js_sys::Array>().ok())
        .unwrap_or_else(js_sys::Array::new);

    let category_filter = get_filter("__productCategoryFilter");
    let stock_filter = get_filter("__productStockFilter");
    let search = get_search("__productSearch").to_lowercase();

    products
        .iter()
        .filter(|product| {
            if category_filter != "All" && js_str(&product, "category") != category_filter {
                return false;
            }
            let status = inventory_status_str(&product);
            if stock_filter == "Low" && status != "low" {
                return false;
            }
            if stock_filter == "Out" && status != "out" {
                return false;
            }
            if search.is_empty() {
                return true;
            }
            [
                js_str(&product, "product_id"),
                js_str(&product, "title"),
                js_str(&product, "category"),
                js_str(&product, "vendor"),
                js_str(&product, "isbn"),
            ]
            .iter()
            .any(|v| v.to_lowercase().contains(&search))
        })
        .collect()
}

fn render_category_filters(snapshot: &JsValue) {
    let categories = js_sys::Reflect::get(snapshot, &JsValue::from_str("categories"))
        .ok()
        .and_then(|v| v.dyn_into::<js_sys::Array>().ok())
        .unwrap_or_else(js_sys::Array::new);

    let node = match by_id("admin-category-filters") {
        Some(n) => n,
        None => return,
    };

    let current = get_filter("__productCategoryFilter");
    let mut html = String::new();
    let values: Vec<String> = std::iter::once("All".to_string())
        .chain(categories.iter().filter_map(|v| v.as_string()))
        .collect();

    for value in &values {
        let active = if *value == current { " office-chip--active" } else { "" };
        let ev = escape_html(value);
        html.push_str(&format!(
            r#"<button class="office-chip{active}" type="button" data-product-category="{ev}">{ev}</button>"#
        ));
    }
    node.set_inner_html(&html);
    bind_product_filters();
}

fn render_inventory(snapshot: &JsValue) {
    let all_products = js_sys::Reflect::get(snapshot, &JsValue::from_str("products"))
        .ok()
        .and_then(|v| v.dyn_into::<js_sys::Array>().ok())
        .unwrap_or_else(js_sys::Array::new);

    let all_vec: Vec<JsValue> = all_products.iter().collect();
    let products = filtered_products(snapshot);

    let low_stock_count = all_vec
        .iter()
        .filter(|p| inventory_status_str(p) == "low")
        .count();
    let out_of_stock_count = all_vec
        .iter()
        .filter(|p| inventory_status_str(p) == "out")
        .count();
    let retail_value: f64 = all_vec
        .iter()
        .map(|p| js_f64(p, "retail_cents") * js_f64(p, "quantity_on_hand"))
        .sum();

    set_text("inventory-total-products", &all_vec.len().to_string());
    set_text("inventory-retail-value", &money(retail_value));
    set_text("inventory-low-stock-count", &low_stock_count.to_string());
    set_text("inventory-out-of-stock-count", &out_of_stock_count.to_string());
    set_text(
        "inventory-results-caption",
        &format!("Showing {} of {} products", products.len(), all_vec.len()),
    );

    let node = match by_id("admin-products-table") {
        Some(n) => n,
        None => return,
    };

    if products.is_empty() {
        node.set_inner_html(r#"<div class="empty-inline">No products match your filters.</div>"#);
        return;
    }

    let rows: String = products
        .iter()
        .map(|product| {
            let on_hand = js_f64(product, "quantity_on_hand") as i64;
            let reorder_point = 3;
            let title = escape_html(&js_str(product, "title"));
            let pid = escape_html(&js_str(product, "product_id"));
            let isbn = js_str(product, "isbn");
            let isbn_display = if isbn.is_empty() { "No ISBN".to_string() } else { escape_html(&isbn) };
            let category = escape_html(&js_str(product, "category"));
            let vendor = escape_html(&js_str(product, "vendor"));
            let cost = money(js_f64(product, "cost_cents"));
            let retail = money(js_f64(product, "retail_cents"));
            let badge = inventory_status_badge(product);
            let isbn_escaped = escape_html(&isbn);
            let title_escaped = escape_html(&js_str(product, "title")).replace('\'', "&#39;");
            format!(
                r#"<tr><td><div><div class="list-title">{title}</div><div class="office-product-meta">{pid} · {isbn_display}</div></div></td><td><span class="office-inline-badge">{category}</span></td><td>{cost}</td><td><strong>{retail}</strong></td><td>{vendor}</td><td>{badge}</td><td><div class="office-stock-cell"><button class="office-stock-button" type="button" onclick="adjustInventory('{isbn_escaped}', -1)">−</button><span class="office-stock-value">{on_hand}</span><button class="office-stock-button" type="button" onclick="adjustInventory('{isbn_escaped}', 1)">+</button><span class="office-stock-note">/ {reorder_point} min</span></div></td><td><div class="button-row button-row--compact"><button class="ghost-link ghost-link--ink ghost-link--mini" type="button" onclick="reorderTitle('{title_escaped}')">Prep</button><a class="ghost-link ghost-link--ink ghost-link--mini" href="/admin/intake">Edit</a></div></td></tr>"#
            )
        })
        .collect();

    node.set_inner_html(&format!(
        r#"<div class="orders-table-wrap"><table class="orders-table"><thead><tr><th>Product</th><th>Category</th><th>Cost</th><th>Retail</th><th>Vendor</th><th>Status</th><th>Stock</th><th>Actions</th></tr></thead><tbody>{rows}</tbody></table></div>"#
    ));
}

fn render_payment_breakdown(summary: &JsValue) {
    let sales_by_payment = js_sys::Reflect::get(summary, &JsValue::from_str("sales_by_payment"))
        .ok()
        .unwrap_or(JsValue::NULL);

    let entries = if sales_by_payment.is_object() {
        js_sys::Object::entries(&js_sys::Object::unchecked_from_js(sales_by_payment.clone()))
    } else {
        js_sys::Array::new()
    };

    let total_sales = js_f64(summary, "sales_cents").max(1.0);

    let mut html = String::new();
    for entry in entries.iter() {
        let pair = js_sys::Array::from(&entry);
        let method = pair
            .get(0)
            .as_string()
            .unwrap_or_default()
            .replace('_', " ");
        let cents = pair.get(1).as_f64().unwrap_or(0.0);
        let width = ((cents / total_sales) * 100.0).round().max(8.0) as i64;
        let method_escaped = escape_html(&method);
        html.push_str(&format!(
            r#"<div class="stack-list"><div class="list-row list-row--soft"><div><div class="list-title">{method_escaped}</div><div class="list-meta">Share of report window</div></div><strong>{money}</strong></div><div class="bar-track"><div class="bar-fill" style="width:{width}%"></div></div></div>"#,
            money = money(cents),
        ));
    }

    render_list(
        "admin-payment-breakdown",
        &html,
        "Payment method totals will appear here.",
    );

    if let Some(trend) = by_id("admin-trend-note") {
        let paid = js_f64(summary, "sales_cents") - js_f64(summary, "donations_cents");
        if paid > 0.0 {
            trend.set_text_content(Some(&format!(
                "Paid sales are {} for the selected window, with donations contributing {} on top.",
                money(paid),
                money(js_f64(summary, "donations_cents"))
            )));
            trend.set_class_name("notice-panel notice-panel--success");
        } else {
            trend.set_text_content(Some(
                "No paid sales were recorded in the selected window.",
            ));
            trend.set_class_name("notice-panel notice-panel--success");
        }
    }
}

// ---- Async fetch ----

async fn fetch_json(url: &str) -> Result<JsValue, String> {
    let token = admin_token();
    let opts = web_sys::RequestInit::new();
    opts.set_method("GET");
    let headers = web_sys::Headers::new().map_err(|e| format!("{e:?}"))?;
    headers
        .set("Authorization", &format!("Bearer {token}"))
        .map_err(|e| format!("{e:?}"))?;
    opts.set_headers(&headers);
    let request =
        web_sys::Request::new_with_str_and_init(url, &opts).map_err(|e| format!("{e:?}"))?;
    let window = web_sys::window().ok_or("no window")?;
    let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("{e:?}"))?;
    let resp: web_sys::Response = resp_value.dyn_into().map_err(|e| format!("{e:?}"))?;
    if !resp.ok() {
        let json_promise = resp.json().map_err(|e| format!("{e:?}"))?;
        let json = wasm_bindgen_futures::JsFuture::from(json_promise)
            .await
            .unwrap_or(JsValue::NULL);
        let msg = js_str(&json, "message");
        let err = js_str(&json, "error");
        let message = if !msg.is_empty() {
            msg
        } else if !err.is_empty() {
            err
        } else {
            format!("Request failed for {url}")
        };
        return Err(message);
    }
    let json_promise = resp.json().map_err(|e| format!("{e:?}"))?;
    wasm_bindgen_futures::JsFuture::from(json_promise)
        .await
        .map_err(|e| format!("{e:?}"))
}

async fn fetch_json_post(url: &str, body: Option<&str>) -> Result<JsValue, String> {
    let token = admin_token();
    let opts = web_sys::RequestInit::new();
    opts.set_method("POST");
    let headers = web_sys::Headers::new().map_err(|e| format!("{e:?}"))?;
    headers
        .set("Authorization", &format!("Bearer {token}"))
        .map_err(|e| format!("{e:?}"))?;
    if let Some(b) = body {
        headers
            .set("Content-Type", "application/json")
            .map_err(|e| format!("{e:?}"))?;
        opts.set_body(&JsValue::from_str(b));
    }
    let loc = web_sys::window()
        .and_then(|w| w.location().origin().ok())
        .unwrap_or_default();
    headers.set("Origin", &loc).map_err(|e| format!("{e:?}"))?;
    opts.set_headers(&headers);
    let request =
        web_sys::Request::new_with_str_and_init(url, &opts).map_err(|e| format!("{e:?}"))?;
    let window = web_sys::window().ok_or("no window")?;
    let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("{e:?}"))?;
    let resp: web_sys::Response = resp_value.dyn_into().map_err(|e| format!("{e:?}"))?;
    let json_promise = resp.json().map_err(|e| format!("{e:?}"))?;
    let json = wasm_bindgen_futures::JsFuture::from(json_promise)
        .await
        .unwrap_or(JsValue::NULL);
    if !resp.ok() {
        let msg = js_str(&json, "message");
        let err = js_str(&json, "error");
        let message = if !msg.is_empty() {
            msg
        } else if !err.is_empty() {
            err
        } else {
            format!("Request failed for {url}")
        };
        return Err(message);
    }
    Ok(json)
}

// ---- Refresh ----

async fn refresh_admin_data() {
    let token = admin_token();
    let tenant = admin_tenant();
    if token.is_empty() {
        set_status("Sign in first to load dashboard data.", "danger");
        return;
    }
    set_status("Loading dashboard data...", "");

    let rq = report_query();
    let t = js_sys::encode_uri_component(&tenant);

    let results = futures::future::join_all(vec![
        fetch_json(&format!("/api/admin/reports/summary?{rq}")),
        fetch_json(&format!("/api/admin/products?tenant_id={t}")),
        fetch_json(&format!("/api/admin/categories?tenant_id={t}")),
        fetch_json(&format!("/api/admin/vendors?tenant_id={t}")),
        fetch_json(&format!("/api/admin/orders?tenant_id={t}")),
        fetch_json(&format!("/api/admin/inventory/journal?tenant_id={t}")),
    ])
    .await;

    // Check for first error
    for r in &results {
        if let Err(e) = r {
            set_status(e, "danger");
            return;
        }
    }

    let summary = results[0].as_ref().unwrap().clone();
    let products = results[1].as_ref().unwrap().clone();
    let categories_resp = results[2].as_ref().unwrap().clone();
    let vendors_resp = results[3].as_ref().unwrap().clone();
    let orders = results[4].as_ref().unwrap().clone();
    let journal = results[5].as_ref().unwrap().clone();

    let categories = js_sys::Reflect::get(&categories_resp, &JsValue::from_str("values"))
        .ok()
        .and_then(|v| v.dyn_into::<js_sys::Array>().ok())
        .unwrap_or_else(js_sys::Array::new);
    let vendors = js_sys::Reflect::get(&vendors_resp, &JsValue::from_str("values"))
        .ok()
        .and_then(|v| v.dyn_into::<js_sys::Array>().ok())
        .unwrap_or_else(js_sys::Array::new);

    // Build snapshot object
    let snapshot = js_sys::Object::new();
    let _ = js_sys::Reflect::set(&snapshot, &JsValue::from_str("summary"), &summary);
    let _ = js_sys::Reflect::set(&snapshot, &JsValue::from_str("products"), &products);
    let _ = js_sys::Reflect::set(&snapshot, &JsValue::from_str("categories"), &JsValue::from(categories.clone()));
    let _ = js_sys::Reflect::set(&snapshot, &JsValue::from_str("vendors"), &JsValue::from(vendors.clone()));
    let _ = js_sys::Reflect::set(&snapshot, &JsValue::from_str("orders"), &orders);
    let _ = js_sys::Reflect::set(&snapshot, &JsValue::from_str("journal"), &journal);
    set_snapshot(&snapshot.into());
    set_admin_orders(&orders);

    let orders_arr = js_sys::Array::from(&orders);
    let all_orders: Vec<JsValue> = orders_arr.iter().collect();

    let paid_pos: f64 = all_orders
        .iter()
        .filter(|o| normalize_channel(o) == "POS" && js_str(o, "status") == "Paid")
        .map(|o| js_f64(o, "total_cents"))
        .sum();
    let paid_online: f64 = all_orders
        .iter()
        .filter(|o| normalize_channel(o) == "Online" && js_str(o, "status") == "Paid")
        .map(|o| js_f64(o, "total_cents"))
        .sum();
    let open_ious: Vec<&JsValue> = all_orders
        .iter()
        .filter(|o| js_str(o, "status") == "UnpaidIou")
        .collect();

    set_text("metric-today-sales", &money(js_f64(&summary, "sales_cents")));
    set_text("metric-pos-revenue", &money(paid_pos));
    set_text("metric-online-revenue", &money(paid_online));
    set_text("metric-open-ious", &format!("{} open", open_ious.len()));

    let from_val = get_input_value("report-from");
    let to_val = get_input_value("report-to");
    set_text(
        "report-caption",
        &format!(
            "Showing {} to {}.",
            if from_val.is_empty() { "the start".to_string() } else { from_val },
            if to_val.is_empty() { "today".to_string() } else { to_val },
        ),
    );

    render_payment_breakdown(&summary);

    // Products list (dashboard compact)
    let products_arr = js_sys::Array::from(&products);
    let products_html: String = products_arr
        .iter()
        .map(|product| {
            let title = escape_html(&js_str(&product, "title"));
            let category = escape_html(&js_str(&product, "category"));
            let vendor = escape_html(&js_str(&product, "vendor"));
            let retail = money(js_f64(&product, "retail_cents"));
            format!(
                r#"<div class="list-row list-row--soft"><div><div class="list-title">{title}</div><div class="list-meta">{category} · {vendor}</div></div><strong>{retail}</strong></div>"#
            )
        })
        .collect();
    render_list(
        "admin-products",
        &products_html,
        "No products found for this tenant.",
    );

    // Categories
    let cat_html: String = categories
        .iter()
        .filter_map(|v| v.as_string())
        .map(|v| format!(r#"<span class="chip">{}</span>"#, escape_html(&v)))
        .collect();
    render_list("admin-categories", &cat_html, "No categories found.");

    // Vendors
    let vendor_html: String = vendors
        .iter()
        .filter_map(|v| v.as_string())
        .map(|v| format!(r#"<span class="chip">{}</span>"#, escape_html(&v)))
        .collect();
    render_list("admin-vendors", &vendor_html, "No vendors found.");

    render_orders();
    render_category_filters(&get_snapshot());
    render_inventory(&get_snapshot());

    // IOUs list
    let ious_html: String = open_ious
        .iter()
        .map(|order| {
            let name = escape_html(&js_str(order, "customer_name"));
            let oid = escape_html(&js_str(order, "order_id"));
            let date = escape_html(&js_str(order, "created_at"));
            let total = money(js_f64(order, "total_cents"));
            format!(
                r#"<div class="list-row list-row--soft"><div><div class="list-title">{name}</div><div class="list-meta">{oid} · {date}</div></div><div class="button-row button-row--compact"><strong>{total}</strong><button class="primary-button primary-button--sm" type="button" onclick="markOrderPaid('{oid}')">Mark Paid</button></div></div>"#
            )
        })
        .collect();
    render_list("admin-ious", &ious_html, "No open IOUs.");

    // Low stock
    let low_stock: Vec<JsValue> = products_arr
        .iter()
        .filter(|p| js_f64(&p, "quantity_on_hand") as i64 <= 3)
        .collect();
    let low_html: String = low_stock
        .iter()
        .map(|product| {
            let on_hand = js_f64(product, "quantity_on_hand") as i64;
            let title = escape_html(&js_str(product, "title"));
            let category = escape_html(&js_str(product, "category"));
            let title_escaped = escape_html(&js_str(product, "title")).replace('\'', "&#39;");
            let badge = if on_hand <= 0 {
                r#"<span class="status-badge status-badge--iou">Out of stock</span>"#.to_string()
            } else {
                format!(r#"<span class="status-badge status-badge--iou">{on_hand} left</span>"#)
            };
            format!(
                r#"<div class="list-row list-row--soft"><div><div class="list-title">{title}</div><div class="list-meta">{category} · On hand {on_hand}</div></div><div class="button-row button-row--compact">{badge}<button class="ghost-link ghost-link--ink ghost-link--mini" type="button" onclick="reorderTitle('{title_escaped}')">Prep</button></div></div>"#
            )
        })
        .collect();
    render_list("admin-low-stock", &low_html, "No low-stock titles right now.");

    // Journal
    let journal_arr = js_sys::Array::from(&journal);
    let journal_html: String = journal_arr
        .iter()
        .map(|entry| {
            let isbn = escape_html(&js_str(&entry, "isbn"));
            let reason = escape_html(&js_str(&entry, "reason"));
            let delta = js_f64(&entry, "delta") as i64;
            let delta_str = if delta > 0 {
                format!("+{delta}")
            } else {
                delta.to_string()
            };
            format!(
                r#"<div class="list-row list-row--soft"><div><div class="list-title">{isbn}</div><div class="list-meta">{reason}</div></div><strong>{delta_str}</strong></div>"#
            )
        })
        .collect();
    render_list(
        "admin-journal",
        &journal_html,
        "No inventory movement recorded yet.",
    );

    set_status(
        &format!("Dashboard refreshed for {tenant}."),
        "success",
    );
}

// ---- Global action functions (exposed to onclick handlers) ----

fn view_order_impl(order_id: &str) {
    let orders = get_admin_orders();
    let found = orders.iter().find(|o| js_str(&o, "order_id") == order_id);
    match found {
        Some(order) => {
            let customer = js_str(&order, "customer_name");
            let method = js_str(&order, "payment_method");
            let total = money(js_f64(&order, "total_cents"));
            set_status(
                &format!("Viewing {order_id}: {customer} via {method} for {total}."),
                "success",
            );
        }
        None => {
            set_status(
                &format!("Order {order_id} is no longer available."),
                "danger",
            );
        }
    }
}

fn resend_receipt_impl(order_id: &str) {
    set_status(
        &format!("Receipt resend queued for {order_id}."),
        "success",
    );
}

async fn mark_order_paid_impl(order_id: String) {
    let token = admin_token();
    if token.is_empty() {
        set_status("Sign in first to manage orders.", "danger");
        return;
    }
    let tenant = admin_tenant();
    let t = js_sys::encode_uri_component(&tenant);
    let url = format!("/api/admin/orders/{order_id}/mark-paid?tenant_id={t}");
    match fetch_json_post(&url, None).await {
        Ok(_) => {
            set_status(&format!("Marked {order_id} paid."), "success");
            refresh_admin_data().await;
        }
        Err(e) => {
            set_status(&e, "danger");
        }
    }
}

async fn adjust_inventory_impl(isbn: String, delta: i64) {
    let token = admin_token();
    if token.is_empty() {
        set_status("Sign in first to manage inventory.", "danger");
        return;
    }
    let tenant = admin_tenant();
    let reason = if delta > 0 {
        "manual_adjustment_add"
    } else {
        "manual_adjustment_remove"
    };
    let body = serde_json::json!({
        "token": token,
        "tenant_id": tenant,
        "isbn": isbn,
        "delta": delta,
        "reason": reason,
    });
    match fetch_json_post("/api/admin/inventory/adjust", Some(&body.to_string())).await {
        Ok(_) => {
            set_status(&format!("Adjusted stock for {isbn}."), "success");
            refresh_admin_data().await;
        }
        Err(e) => {
            set_status(&e, "danger");
        }
    }
}

fn export_snapshot_impl() {
    let token = admin_token();
    let snapshot = get_snapshot();
    let summary = js_sys::Reflect::get(&snapshot, &JsValue::from_str("summary"))
        .ok()
        .unwrap_or(JsValue::NULL);
    if token.is_empty() || summary.is_null() || summary.is_undefined() {
        set_status("Load dashboard data before exporting.", "danger");
        return;
    }
    let json_str = js_sys::JSON::stringify_with_replacer_and_space(
        &snapshot,
        &JsValue::NULL,
        &JsValue::from(2),
    )
    .ok()
    .and_then(|v| v.as_string())
    .unwrap_or_else(|| "{}".to_string());

    let blob_parts = js_sys::Array::new();
    blob_parts.push(&JsValue::from_str(&json_str));
    let options = web_sys::BlobPropertyBag::new();
    options.set_type("application/json");
    if let Ok(blob) = web_sys::Blob::new_with_str_sequence_and_options(&blob_parts, &options) {
        if let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) {
            let doc = document();
            if let Ok(el) = doc.create_element("a") {
                if let Some(anchor) = el.dyn_ref::<HtmlElement>() {
                    let tenant = admin_tenant();
                    let _ = el.set_attribute("href", &url);
                    let _ = el.set_attribute(
                        "download",
                        &format!("scriptorium-{tenant}-dashboard.json"),
                    );
                    anchor.click();
                    let _ = web_sys::Url::revoke_object_url(&url);
                    set_status(
                        &format!("Exported dashboard snapshot for {tenant}."),
                        "success",
                    );
                }
            }
        }
    }
}

fn reorder_title_impl(title: &str) {
    set_status(&format!("Open intake to reorder {title}."), "success");
}

// ---- Bind event listeners ----

fn bind_order_filters() {
    let doc = document();
    if let Ok(nodes) = doc.query_selector_all("[data-order-filter]") {
        for i in 0..nodes.length() {
            if let Some(node) = nodes.item(i) {
                if let Some(el) = node.dyn_ref::<HtmlElement>() {
                    let filter_value = el
                        .get_attribute("data-order-filter")
                        .unwrap_or_else(|| "All".to_string());
                    let closure = Closure::wrap(Box::new(move || {
                        set_filter("__orderFilter", &filter_value);
                        let doc = document();
                        if let Ok(chips) = doc.query_selector_all("[data-order-filter]") {
                            for j in 0..chips.length() {
                                if let Some(chip) = chips.item(j) {
                                    if let Some(chip_el) = chip.dyn_ref::<HtmlElement>() {
                                        let _ = chip_el.class_list().remove_2(
                                            "filter-chip--active",
                                            "office-chip--active",
                                        );
                                    }
                                }
                            }
                        }
                        if let Some(btn) = doc
                            .query_selector(&format!("[data-order-filter=\"{}\"]", get_filter("__orderFilter")))
                            .ok()
                            .flatten()
                        {
                            if let Some(btn_el) = btn.dyn_ref::<HtmlElement>() {
                                let _ = btn_el.class_list().add_2(
                                    "filter-chip--active",
                                    "office-chip--active",
                                );
                            }
                        }
                        render_orders();
                    }) as Box<dyn Fn()>);
                    el.set_onclick(Some(closure.as_ref().unchecked_ref()));
                    closure.forget();
                }
            }
        }
    }
}

fn bind_product_filters() {
    let doc = document();
    if let Ok(nodes) = doc.query_selector_all("[data-product-category]") {
        for i in 0..nodes.length() {
            if let Some(node) = nodes.item(i) {
                if let Some(el) = node.dyn_ref::<HtmlElement>() {
                    let cat = el
                        .get_attribute("data-product-category")
                        .unwrap_or_else(|| "All".to_string());
                    let closure = Closure::wrap(Box::new(move || {
                        set_filter("__productCategoryFilter", &cat);
                        let doc = document();
                        if let Ok(chips) = doc.query_selector_all("[data-product-category]") {
                            for j in 0..chips.length() {
                                if let Some(chip) = chips.item(j) {
                                    if let Some(chip_el) = chip.dyn_ref::<HtmlElement>() {
                                        let _ = chip_el
                                            .class_list()
                                            .remove_1("office-chip--active");
                                    }
                                }
                            }
                        }
                        if let Some(btn) = doc
                            .query_selector(&format!("[data-product-category=\"{}\"]", get_filter("__productCategoryFilter")))
                            .ok()
                            .flatten()
                        {
                            if let Some(btn_el) = btn.dyn_ref::<HtmlElement>() {
                                let _ = btn_el.class_list().add_1("office-chip--active");
                            }
                        }
                        render_inventory(&get_snapshot());
                    }) as Box<dyn Fn()>);
                    el.set_onclick(Some(closure.as_ref().unchecked_ref()));
                    closure.forget();
                }
            }
        }
    }
    if let Ok(nodes) = doc.query_selector_all("[data-product-stock]") {
        for i in 0..nodes.length() {
            if let Some(node) = nodes.item(i) {
                if let Some(el) = node.dyn_ref::<HtmlElement>() {
                    let stock = el
                        .get_attribute("data-product-stock")
                        .unwrap_or_else(|| "All".to_string());
                    let closure = Closure::wrap(Box::new(move || {
                        set_filter("__productStockFilter", &stock);
                        let doc = document();
                        if let Ok(chips) = doc.query_selector_all("[data-product-stock]") {
                            for j in 0..chips.length() {
                                if let Some(chip) = chips.item(j) {
                                    if let Some(chip_el) = chip.dyn_ref::<HtmlElement>() {
                                        let _ = chip_el
                                            .class_list()
                                            .remove_1("office-chip--active");
                                    }
                                }
                            }
                        }
                        if let Some(btn) = doc
                            .query_selector(&format!("[data-product-stock=\"{}\"]", get_filter("__productStockFilter")))
                            .ok()
                            .flatten()
                        {
                            if let Some(btn_el) = btn.dyn_ref::<HtmlElement>() {
                                let _ = btn_el.class_list().add_1("office-chip--active");
                            }
                        }
                        render_inventory(&get_snapshot());
                    }) as Box<dyn Fn()>);
                    el.set_onclick(Some(closure.as_ref().unchecked_ref()));
                    closure.forget();
                }
            }
        }
    }
}

// ---- Mount ----

pub fn mount_admin_island() {
    // Only mount if admin-status element exists (dashboard or orders page)
    if by_id("admin-status").is_none() {
        return;
    }

    // Initialize filter state
    set_filter("__orderFilter", "All");
    set_filter("__orderSearch", "");
    set_filter("__productCategoryFilter", "All");
    set_filter("__productStockFilter", "All");
    set_filter("__productSearch", "");

    // Bind refresh button
    if let Some(el) = by_id("admin-refresh").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
        let closure = Closure::wrap(Box::new(|| {
            wasm_bindgen_futures::spawn_local(refresh_admin_data());
        }) as Box<dyn Fn()>);
        el.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // Bind export buttons
    for id in &["admin-export", "admin-export-inline"] {
        if let Some(el) = by_id(id).and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
            let closure = Closure::wrap(Box::new(|| export_snapshot_impl()) as Box<dyn Fn()>);
            el.set_onclick(Some(closure.as_ref().unchecked_ref()));
            closure.forget();
        }
    }

    // Bind date range change → refresh
    for id in &["report-from", "report-to"] {
        if let Some(el) = by_id(id).and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
            let closure = Closure::wrap(Box::new(|| {
                if !admin_token().is_empty() {
                    wasm_bindgen_futures::spawn_local(refresh_admin_data());
                }
            }) as Box<dyn Fn()>);
            el.set_onchange(Some(closure.as_ref().unchecked_ref()));
            closure.forget();
        }
    }

    // Bind order search
    if let Some(el) = by_id("admin-order-search").and_then(|e| e.dyn_into::<HtmlInputElement>().ok())
    {
        let el_clone = el.clone();
        let closure = Closure::wrap(Box::new(move || {
            set_filter("__orderSearch", &el_clone.value());
            render_orders();
        }) as Box<dyn Fn()>);
        el.set_oninput(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // Bind product search
    if let Some(el) =
        by_id("admin-product-search").and_then(|e| e.dyn_into::<HtmlInputElement>().ok())
    {
        let el_clone = el.clone();
        let closure = Closure::wrap(Box::new(move || {
            set_filter("__productSearch", &el_clone.value());
            render_inventory(&get_snapshot());
        }) as Box<dyn Fn()>);
        el.set_oninput(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // Expose global functions for onclick handlers in rendered HTML
    expose_globals();

    bind_order_filters();
    bind_product_filters();

    // Auto-refresh if session is present
    let token = admin_token();
    let tenant = admin_tenant();
    if !token.is_empty() && !tenant.is_empty() {
        wasm_bindgen_futures::spawn_local(async {
            refresh_admin_data().await;
            // Set ready flag for browser tests after initial data load
            if let Some(window) = web_sys::window() {
                let _ = js_sys::Reflect::set(
                    &window,
                    &JsValue::from_str("__SCRIPTORIUM_ADMIN_READY"),
                    &JsValue::TRUE,
                );
            }
        });
    } else {
        set_status("Admin session missing. Sign in again.", "danger");
        // Set ready flag even when not logged in
        if let Some(window) = web_sys::window() {
            let _ = js_sys::Reflect::set(
                &window,
                &JsValue::from_str("__SCRIPTORIUM_ADMIN_READY"),
                &JsValue::TRUE,
            );
        }
    }
}

fn expose_globals() {
    if let Some(window) = web_sys::window() {
        // viewOrder
        let closure = Closure::wrap(Box::new(|order_id: JsValue| {
            if let Some(id) = order_id.as_string() {
                view_order_impl(&id);
            }
        }) as Box<dyn Fn(JsValue)>);
        let _ = js_sys::Reflect::set(&window, &JsValue::from_str("viewOrder"), closure.as_ref());
        closure.forget();

        // resendReceipt
        let closure = Closure::wrap(Box::new(|order_id: JsValue| {
            if let Some(id) = order_id.as_string() {
                resend_receipt_impl(&id);
            }
        }) as Box<dyn Fn(JsValue)>);
        let _ =
            js_sys::Reflect::set(&window, &JsValue::from_str("resendReceipt"), closure.as_ref());
        closure.forget();

        // markOrderPaid
        let closure = Closure::wrap(Box::new(|order_id: JsValue| {
            if let Some(id) = order_id.as_string() {
                wasm_bindgen_futures::spawn_local(mark_order_paid_impl(id));
            }
        }) as Box<dyn Fn(JsValue)>);
        let _ =
            js_sys::Reflect::set(&window, &JsValue::from_str("markOrderPaid"), closure.as_ref());
        closure.forget();

        // adjustInventory
        let closure = Closure::wrap(Box::new(|isbn: JsValue, delta: JsValue| {
            if let (Some(isbn_str), Some(delta_f)) = (isbn.as_string(), delta.as_f64()) {
                wasm_bindgen_futures::spawn_local(adjust_inventory_impl(
                    isbn_str,
                    delta_f as i64,
                ));
            }
        }) as Box<dyn Fn(JsValue, JsValue)>);
        let _ = js_sys::Reflect::set(
            &window,
            &JsValue::from_str("adjustInventory"),
            closure.as_ref(),
        );
        closure.forget();

        // reorderTitle
        let closure = Closure::wrap(Box::new(|title: JsValue| {
            if let Some(t) = title.as_string() {
                reorder_title_impl(&t);
            }
        }) as Box<dyn Fn(JsValue)>);
        let _ =
            js_sys::Reflect::set(&window, &JsValue::from_str("reorderTitle"), closure.as_ref());
        closure.forget();
    }
}
