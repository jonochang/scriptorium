use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::Document;

// ---- Helpers ----

fn document() -> Document {
    web_sys::window().expect("window").document().expect("document")
}

fn by_id(id: &str) -> Option<web_sys::Element> {
    document().get_element_by_id(id)
}

fn money(cents: i64) -> String {
    let value = if cents.abs() == 0 { 0.0 } else { cents as f64 / 100.0 };
    format!("${:.2}", value)
}

// ---- Window-global state ----

fn win_get_str(key: &str) -> String {
    web_sys::window()
        .and_then(|w| js_sys::Reflect::get(&w, &JsValue::from_str(key)).ok())
        .and_then(|v| v.as_string())
        .unwrap_or_default()
}

fn win_set_str(key: &str, value: &str) {
    if let Some(w) = web_sys::window() {
        let _ = js_sys::Reflect::set(&w, &JsValue::from_str(key), &JsValue::from_str(value));
    }
}

fn win_get_i64(key: &str) -> i64 {
    web_sys::window()
        .and_then(|w| js_sys::Reflect::get(&w, &JsValue::from_str(key)).ok())
        .and_then(|v| v.as_f64())
        .map(|v| v as i64)
        .unwrap_or(0)
}

fn win_set_i64(key: &str, value: i64) {
    if let Some(w) = web_sys::window() {
        let _ = js_sys::Reflect::set(&w, &JsValue::from_str(key), &JsValue::from(value as f64));
    }
}

fn win_get_bool(key: &str) -> bool {
    web_sys::window()
        .and_then(|w| js_sys::Reflect::get(&w, &JsValue::from_str(key)).ok())
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

fn win_set_bool(key: &str, value: bool) {
    if let Some(w) = web_sys::window() {
        let _ = js_sys::Reflect::set(&w, &JsValue::from_str(key), &JsValue::from(value));
    }
}

// State keys
const SCREEN: &str = "__posScreen";
const MODE: &str = "__posMode";
const PIN: &str = "__posPin";
const TOKEN: &str = "__posToken";
const BARCODE: &str = "__posBarcode";
const CART_JSON: &str = "__posCartJson";
const TOTAL: &str = "__posTotal";
const STATUS_TONE: &str = "__posStatusTone";
const STATUS_TITLE: &str = "__posStatusTitle";
const STATUS_DETAIL: &str = "__posStatusDetail";
const PAYMENT_METHOD: &str = "__posPaymentMethod";
const CUSTOM_TENDERED: &str = "__posCustomTendered";
const DONATE_CHANGE: &str = "__posDonateChange";
const IOU_NAME: &str = "__posIouName";
const RECEIPT_EMAIL: &str = "__posReceiptEmail";
const DISCOUNT_CODE: &str = "__posDiscountCode";
const LAST_SALE_JSON: &str = "__posLastSaleJson";
const POS_CONFIG_JSON: &str = "__posConfigJson";

// ---- Data types ----

#[derive(Serialize, Deserialize, Clone, Default)]
struct CartItem {
    item_id: String,
    title: String,
    #[serde(default)]
    is_quick_item: bool,
    #[serde(default)]
    quantity: i64,
    #[serde(default)]
    unit_price_cents: i64,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct SaleResult {
    #[serde(default)]
    status: String,
    #[serde(default)]
    message: String,
    #[serde(default)]
    total_cents: i64,
    #[serde(default)]
    change_due_cents: i64,
    #[serde(default)]
    donation_cents: i64,
    #[serde(default)]
    discount_cents: i64,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct QuickItem {
    item_id: String,
    #[serde(alias = "title")]
    label: String,
    emoji: String,
    price_label: String,
    note: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct DiscountCode {
    code: String,
    label: String,
    rate: f64,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct PosConfig {
    quick_items: Vec<QuickItem>,
    discount_codes: Vec<DiscountCode>,
}

fn read_pos_config() -> PosConfig {
    let json = win_get_str(POS_CONFIG_JSON);
    if json.is_empty() {
        return PosConfig::default();
    }
    serde_json::from_str(&json).unwrap_or_default()
}

fn write_pos_config(config: &PosConfig) {
    let json = serde_json::to_string(config).unwrap_or_else(|_| "{}".to_string());
    win_set_str(POS_CONFIG_JSON, &json);
}

// ---- State accessors ----

fn read_cart() -> Vec<CartItem> {
    let json = win_get_str(CART_JSON);
    if json.is_empty() {
        return Vec::new();
    }
    serde_json::from_str(&json).unwrap_or_default()
}

fn write_cart(items: &[CartItem]) {
    let json = serde_json::to_string(items).unwrap_or_else(|_| "[]".to_string());
    win_set_str(CART_JSON, &json);
}

fn read_last_sale() -> Option<SaleResult> {
    let json = win_get_str(LAST_SALE_JSON);
    if json.is_empty() {
        return None;
    }
    serde_json::from_str(&json).ok()
}

fn write_last_sale(sale: Option<&SaleResult>) {
    match sale {
        Some(s) => {
            let json = serde_json::to_string(s).unwrap_or_default();
            win_set_str(LAST_SALE_JSON, &json);
        }
        None => win_set_str(LAST_SALE_JSON, ""),
    }
}

fn set_ui_status(tone: &str, title: &str, detail: &str) {
    win_set_str(STATUS_TONE, tone);
    win_set_str(STATUS_TITLE, title);
    win_set_str(STATUS_DETAIL, detail);
}

fn discount_rate() -> f64 {
    let code = win_get_str(DISCOUNT_CODE);
    if code.is_empty() {
        return 0.0;
    }
    let config = read_pos_config();
    config.discount_codes.iter().find(|dc| dc.code == code).map(|dc| dc.rate).unwrap_or(0.0)
}

fn discount_value() -> i64 {
    let total = win_get_i64(TOTAL);
    (total as f64 * discount_rate()).round() as i64
}

fn amount_due() -> i64 {
    let total = win_get_i64(TOTAL);
    let dv = discount_value();
    (total - dv).max(0)
}

// ---- API ----

async fn request(url: &str, payload: &serde_json::Value) -> Result<(bool, serde_json::Value), String> {
    let opts = web_sys::RequestInit::new();
    opts.set_method("POST");
    let headers = web_sys::Headers::new().map_err(|e| format!("{e:?}"))?;
    headers.set("content-type", "application/json").map_err(|e| format!("{e:?}"))?;
    opts.set_headers(&headers);
    opts.set_body(&JsValue::from_str(&payload.to_string()));

    let request = web_sys::Request::new_with_str_and_init(url, &opts).map_err(|e| format!("{e:?}"))?;
    let window = web_sys::window().ok_or("no window")?;
    let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("{e:?}"))?;
    let resp: web_sys::Response = resp_value.dyn_into().map_err(|e| format!("{e:?}"))?;
    let ok = resp.ok();
    let json_value = match resp.json() {
        Ok(p) => wasm_bindgen_futures::JsFuture::from(p)
            .await
            .unwrap_or(JsValue::NULL),
        Err(_) => JsValue::NULL,
    };

    // Convert JsValue to serde_json::Value
    let json_str = js_sys::JSON::stringify(&json_value)
        .map(|s| s.as_string().unwrap_or_default())
        .unwrap_or_else(|_| "{}".to_string());
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap_or(serde_json::Value::Object(Default::default()));

    if !ok {
        let error = json.get("error").and_then(|v| v.as_str()).unwrap_or("Request failed");
        let message = json.get("message").and_then(|v| v.as_str()).unwrap_or("The POS endpoint returned an error.");
        set_ui_status("danger", error, message);
    }

    Ok((ok, json))
}

async fn fetch_pos_config() -> Result<PosConfig, String> {
    let opts = web_sys::RequestInit::new();
    opts.set_method("GET");
    let req = web_sys::Request::new_with_str_and_init("/api/pos/config", &opts)
        .map_err(|e| format!("{e:?}"))?;
    let window = web_sys::window().ok_or("no window")?;
    let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&req))
        .await
        .map_err(|e| format!("{e:?}"))?;
    let resp: web_sys::Response = resp_value.dyn_into().map_err(|e| format!("{e:?}"))?;
    if !resp.ok() {
        return Err("Failed to fetch POS config".to_string());
    }
    let json_value = resp.json().map_err(|e| format!("{e:?}"))?;
    let json_value = wasm_bindgen_futures::JsFuture::from(json_value)
        .await
        .map_err(|e| format!("{e:?}"))?;
    let json_str = js_sys::JSON::stringify(&json_value)
        .map(|s| s.as_string().unwrap_or_default())
        .unwrap_or_else(|_| "{}".to_string());
    serde_json::from_str(&json_str).map_err(|e| format!("{e:?}"))
}

fn apply_cart(json: &serde_json::Value) {
    if let Some(items) = json.get("items").and_then(|v| v.as_array()) {
        let cart: Vec<CartItem> = items
            .iter()
            .filter_map(|v| serde_json::from_value(v.clone()).ok())
            .collect();
        write_cart(&cart);
    } else {
        write_cart(&[]);
    }
    if let Some(total) = json.get("total_cents").and_then(|v| v.as_i64()) {
        win_set_i64(TOTAL, total);
    } else {
        win_set_i64(TOTAL, 0);
    }
}

// ---- Actions (async) ----

async fn start_shift(pin_value: &str) {
    let result = request("/api/pos/login", &serde_json::json!({ "pin": pin_value })).await;
    match result {
        Err(_) | Ok((false, _)) => {
            win_set_str(PIN, "");
            render_pos();
            return;
        }
        Ok((true, json)) => {
            let next_token = json.get("session_token").and_then(|v| v.as_str()).unwrap_or("");
            win_set_str(TOKEN, next_token);
            win_set_str(PIN, "");
            reset_sale_state();
            set_ui_status(
                "success",
                "Shift started",
                &if next_token.is_empty() {
                    "POS session opened.".to_string()
                } else {
                    format!("Session {next_token} is ready for scanning, baskets, and payment.")
                },
            );
            render_pos();
        }
    }
}

async fn scan_item() {
    let token = win_get_str(TOKEN);
    if token.is_empty() {
        set_ui_status("danger", "Shift missing", "Start a shift before scanning items.");
        win_set_str(SCREEN, "login");
        render_pos();
        return;
    }
    let barcode = win_get_str(BARCODE);
    let result = request("/api/pos/scan", &serde_json::json!({ "session_token": token, "isbn": barcode })).await;
    match result {
        Ok((true, json)) => {
            apply_cart(&json);
            write_last_sale(None);
            let msg = json.get("message").and_then(|v| v.as_str()).unwrap_or("The item was added to the current sale.");
            set_ui_status("success", "Scanned to cart", msg);
        }
        _ => {}
    }
    render_pos();
}

async fn add_quick_item(item_id: &str, label: &str) {
    let token = win_get_str(TOKEN);
    let result = request("/api/pos/cart/items", &serde_json::json!({
        "session_token": token,
        "item_id": item_id,
        "quantity": 1
    })).await;
    match result {
        Ok((true, json)) => {
            apply_cart(&json);
            write_last_sale(None);
            let msg = json.get("message").and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("{label} was added to the basket."));
            set_ui_status("success", "Quick item added", &msg);
        }
        _ => {}
    }
    render_pos();
}

async fn change_cart_quantity(item_id: &str, title: &str, current_qty: i64, delta: i64) {
    let token = win_get_str(TOKEN);
    let next_quantity = (current_qty + delta).max(0);
    let result = request("/api/pos/cart/quantity", &serde_json::json!({
        "session_token": token,
        "item_id": item_id,
        "quantity": next_quantity,
    })).await;
    match result {
        Ok((true, json)) => {
            apply_cart(&json);
            let detail = if next_quantity > 0 {
                format!("{title} quantity is now {next_quantity}.")
            } else {
                format!("{title} was removed from the basket.")
            };
            set_ui_status("success", "Basket updated", &detail);
        }
        _ => {}
    }
    render_pos();
}

async fn complete_card() {
    let token = win_get_str(TOKEN);
    let dv = discount_value();
    let result = request("/api/pos/payments/external-card", &serde_json::json!({
        "session_token": token,
        "external_ref": "square-ui-posh",
        "discount_cents": dv,
    })).await;
    match result {
        Ok((true, json)) => {
            finalize_sale(&json, "Card sale complete");
        }
        _ => { render_pos(); }
    }
}

async fn complete_cash(tendered_cents: i64) {
    let token = win_get_str(TOKEN);
    let donate = win_get_bool(DONATE_CHANGE);
    let dv = discount_value();
    let result = request("/api/pos/payments/cash", &serde_json::json!({
        "session_token": token,
        "tendered_cents": tendered_cents,
        "donate_change": donate,
        "discount_cents": dv,
    })).await;
    match result {
        Ok((true, json)) => {
            finalize_sale(&json, "Cash sale complete");
        }
        _ => { render_pos(); }
    }
}

async fn complete_iou() {
    let token = win_get_str(TOKEN);
    let iou_name = win_get_str(IOU_NAME);
    let dv = discount_value();
    let result = request("/api/pos/payments/iou", &serde_json::json!({
        "session_token": token,
        "customer_name": iou_name,
        "discount_cents": dv,
    })).await;
    match result {
        Ok((true, json)) => {
            finalize_sale(&json, "Sale moved to IOU");
        }
        _ => { render_pos(); }
    }
}

fn finalize_sale(json: &serde_json::Value, fallback_title: &str) {
    let sale: SaleResult = serde_json::from_value(json.clone()).unwrap_or_default();
    write_last_sale(Some(&sale));
    write_cart(&[]);
    win_set_i64(TOTAL, 0);

    let tone = if sale.status == "iou" { "warning" } else { "success" };
    let mut detail_parts = Vec::new();
    detail_parts.push(format!("Total {}", money(sale.total_cents)));
    if sale.discount_cents > 0 {
        detail_parts.push(format!("Discount {}", money(sale.discount_cents)));
    }
    if sale.change_due_cents > 0 {
        detail_parts.push(format!("Change {}", money(sale.change_due_cents)));
    }
    if sale.donation_cents > 0 {
        detail_parts.push(format!("Donation {}", money(sale.donation_cents)));
    }

    let msg = if sale.message.is_empty() { fallback_title.to_string() } else { sale.message.clone() };
    let detail = if detail_parts.is_empty() { "Payment completed.".to_string() } else { detail_parts.join(" \u{00B7} ") };
    set_ui_status(tone, &msg, &detail);
    win_set_str(SCREEN, "complete");
    render_pos();
}

fn reset_sale_state() {
    write_cart(&[]);
    win_set_i64(TOTAL, 0);
    win_set_str(PAYMENT_METHOD, "");
    win_set_bool(DONATE_CHANGE, true);
    win_set_str(IOU_NAME, "John Doe");
    win_set_str(RECEIPT_EMAIL, "jane@example.com");
    win_set_str(DISCOUNT_CODE, "");
    write_last_sale(None);
    win_set_str(SCREEN, "main");
}

// ---- HTML rendering ----

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn status_html() -> String {
    let tone = win_get_str(STATUS_TONE);
    let title = win_get_str(STATUS_TITLE);
    let detail = win_get_str(STATUS_DETAIL);
    let class = match tone.as_str() {
        "success" => "status-panel status-success",
        "danger" => "status-panel status-danger",
        _ => "status-panel status-warning",
    };
    format!(
        r#"<section class="{class}"><h3>{title}</h3><p>{detail}</p></section>"#,
        class = class,
        title = html_escape(&title),
        detail = html_escape(&detail),
    )
}

fn render_login_screen() -> String {
    let pin = win_get_str(PIN);
    let dots: String = (0..4)
        .map(|i| {
            let filled = if i < pin.len() { " pin-dot--filled" } else { "" };
            format!(r#"<span class="pin-dot{filled}"></span>"#)
        })
        .collect();

    let keys = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "", "0", "\u{232B}"];
    let key_buttons: String = keys
        .iter()
        .map(|key| {
            if key.is_empty() {
                "<div></div>".to_string()
            } else if *key == "\u{232B}" {
                r#"<button class="pin-key pin-key--ghost" data-pos-key="backspace">⌫</button>"#.to_string()
            } else {
                format!(r#"<button class="pin-key" data-pos-key="{key}">{key}</button>"#)
            }
        })
        .collect();

    format!(
        r#"<main class="pos-shell">
  <div class="pos-wrap center-shell">
    <section class="pin-head">
      <div class="pin-cross">✠</div>
      <h1>SCRIPTORIUM</h1>
      <p>Point of Sale</p>
    </section>
    <section class="pin-card">
      <div class="pin-dots" aria-label="Enter PIN">{dots}</div>
      <div class="pin-grid">{keys}</div>
    </section>
    {status}
    <div class="pin-links">
      <button type="button" class="ghost-link" id="pos-forgot-pin">Forgot PIN?</button>
      <a href="/admin">Admin login</a>
    </div>
  </div>
</main>"#,
        dots = dots,
        keys = key_buttons,
        status = status_html(),
    )
}

fn render_help_screen() -> String {
    format!(
        r#"<main class="pos-shell">
  <div class="pos-wrap center-shell">
    <section class="pin-head">
      <div class="pin-cross">✠</div>
      <h1>SCRIPTORIUM</h1>
      <p>PIN recovery</p>
    </section>
    <section class="pin-card">
      <div class="pilgrim-panel">
        <h3>Forgot the shift PIN?</h3>
        <p>For local testing, the demo PIN is <strong>1234</strong>.</p>
        <p>For live parish use, open the admin area to rotate volunteer access before the next shift begins.</p>
      </div>
      <div class="button-row">
        <button class="primary-button" type="button" id="pos-back-to-keypad">Back to keypad</button>
        <a class="ghost-link ghost-link--ink" href="/admin">Open admin sign-in</a>
      </div>
    </section>
    {status}
  </div>
</main>"#,
        status = status_html(),
    )
}

fn render_main_screen() -> String {
    let token = win_get_str(TOKEN);
    let mode = win_get_str(MODE);
    let cart = read_cart();
    let total = win_get_i64(TOTAL);
    let discount_code = win_get_str(DISCOUNT_CODE);
    let dv = discount_value();
    let due = amount_due();

    let token_pill = if token.is_empty() {
        "Shift offline".to_string()
    } else {
        format!("Shift {}", html_escape(&token))
    };
    let items_pill = if cart.is_empty() {
        "Awaiting first item".to_string()
    } else {
        format!("{} item(s)", cart.len())
    };

    let scan_active = if mode == "scan" { " is-active" } else { "" };
    let quick_active = if mode == "quick" { " is-active" } else { "" };

    let barcode = win_get_str(BARCODE);

    let mode_content = if mode == "quick" {
        let config = read_pos_config();
        let tiles: String = config.quick_items
            .iter()
            .map(|item| {
                format!(
                    r#"<button class="quick-tile" data-pos-quick="{id}"><span class="quick-emoji">{emoji}</span>{label}<span class="quick-price">{price}</span></button>"#,
                    id = item.item_id,
                    emoji = item.emoji,
                    label = item.label,
                    price = item.price_label,
                )
            })
            .collect();
        format!(r#"<div class="quick-grid" style="margin-top:14px">{tiles}</div>"#)
    } else {
        format!(
            r#"<div style="margin-top:14px">
  <div class="scan-frame"><div class="scan-caption">Point camera at ISBN, EAN-13, or typed barcode</div></div>
  <label class="field-label" for="barcode">ISBN / barcode</label>
  <input id="barcode" value="{barcode}" />
  <div class="actions" style="margin-top:10px">
    <button class="pos-button--lg" id="pos-scan-item">Scan to cart</button>
    <p class="hint">Use the camera lane or type the barcode when labels are faint.</p>
  </div>
</div>"#,
            barcode = html_escape(&barcode),
        )
    };

    let cart_html = if cart.is_empty() {
        r#"<div class="empty-state">Cart empty. Scan an item or use a quick tile to start the sale.</div>"#.to_string()
    } else {
        let rows: String = cart
            .iter()
            .map(|item| {
                let tag_class = if item.is_quick_item { "cart-tag--quick" } else { "cart-tag--scan" };
                let tag_label = if item.is_quick_item { "Quick item" } else { "Scanned item" };
                let meta = if item.is_quick_item { "Quick item" } else { "Scanned title" };
                let line_total = money(item.unit_price_cents * item.quantity);
                format!(
                    r#"<div class="cart-row">
  <div>
    <div class="cart-title">{title}</div>
    <div class="cart-meta">{meta}</div>
    <div class="cart-controls">
      <button class="ghost-link ghost-link--ink ghost-link--mini" data-pos-qty-dec="{id}" data-pos-qty-title="{title_esc}" data-pos-qty-current="{qty}">−</button>
      <span class="qty-pill">Qty {qty}</span>
      <button class="ghost-link ghost-link--ink ghost-link--mini" data-pos-qty-inc="{id}" data-pos-qty-title="{title_esc}" data-pos-qty-current="{qty}">+</button>
    </div>
    <span class="{tag_class} cart-tag">{tag_label}</span>
  </div>
  <div class="cart-price">{line_total}</div>
</div>"#,
                    title = html_escape(&item.title),
                    title_esc = html_escape(&item.title),
                    meta = meta,
                    id = html_escape(&item.item_id),
                    qty = item.quantity,
                    tag_class = tag_class,
                    tag_label = tag_label,
                    line_total = line_total,
                )
            })
            .collect();
        format!(r#"<div class="cart-list">{rows}</div>"#)
    };

    let discount_row = if !discount_code.is_empty() {
        format!(
            r#"<div class="totals-row"><span>Discount selected</span><span>{dv} ({code})</span></div>"#,
            dv = money(dv),
            code = html_escape(&discount_code),
        )
    } else {
        String::new()
    };

    let config = read_pos_config();
    let mut discount_chips = {
        let active = if discount_code.is_empty() { " discount-chip--active" } else { "" };
        format!(r#"<button class="discount-chip{active}" data-pos-discount="">No discount</button>"#)
    };
    for dc in &config.discount_codes {
        let active = if discount_code == dc.code { " discount-chip--active" } else { "" };
        discount_chips.push_str(&format!(
            r#"<button class="discount-chip{active}" data-pos-discount="{code}">{label}</button>"#,
            code = html_escape(&dc.code),
            label = html_escape(&dc.label),
        ));
    }

    format!(
        r#"<main class="pos-shell">
  <div class="pos-wrap">
    <section class="pos-header">
      <div class="pos-header__brand">
        <span class="pos-header__brand-mark">☦</span>
        <h1 class="pos-header__title">Scriptorium</h1>
        <span class="pos-header__subtitle">Point of Sale</span>
      </div>
      <div class="pos-header__meta">
        <span class="session-pill">{token_pill}</span>
        <span class="session-pill">{items_pill}</span>
      </div>
    </section>
    <section class="card">
      <div class="toolbar">
        <button class="{scan_active}" id="pos-mode-scan">Scan Item</button>
        <button class="{quick_active}" id="pos-mode-quick">Quick Items</button>
      </div>
      {mode_content}
    </section>
    <section class="card basket-card">
      <h2 class="section-title">Basket</h2>
      {cart_html}
      <div class="totals" style="margin-top:12px">
        <div class="totals-row"><span>Current total</span><strong>{total}</strong></div>
        {discount_row}
        <div class="totals-row"><span>Amount due</span><strong>{due}</strong></div>
        <div class="totals-row"><span>Checkout path</span><span>Card, cash, or IOU</span></div>
      </div>
      <div class="discount-grid">{discount_chips}</div>
    </section>
    {status}
    <button class="pos-button--lg" id="pos-checkout">Checkout · {total}</button>
  </div>
</main>"#,
        token_pill = token_pill,
        items_pill = items_pill,
        scan_active = scan_active,
        quick_active = quick_active,
        mode_content = mode_content,
        cart_html = cart_html,
        total = money(total),
        discount_row = discount_row,
        due = money(due),
        discount_chips = discount_chips,
        status = status_html(),
    )
}

fn render_payment_screen() -> String {
    let cart = read_cart();
    let total = win_get_i64(TOTAL);
    let discount_code = win_get_str(DISCOUNT_CODE);
    let dv = discount_value();
    let due = amount_due();
    let payment_method = win_get_str(PAYMENT_METHOD);
    let custom_tendered = win_get_str(CUSTOM_TENDERED);
    let donate_change = win_get_bool(DONATE_CHANGE);
    let iou_name = win_get_str(IOU_NAME);

    let header = format!(
        r#"<section class="pos-header">
  <div class="pos-header__brand">
    <span class="pos-header__brand-mark">☦</span>
    <h1 class="pos-header__title">Scriptorium</h1>
    <span class="pos-header__subtitle">Payment</span>
  </div>
  <div class="pos-header__meta">
    <button class="pos-header__back" id="pos-back-to-basket">← Basket</button>
    <span class="session-pill">{count} line item(s)</span>
  </div>
</section>"#,
        count = cart.len(),
    );

    let discount_pill = if !discount_code.is_empty() {
        format!(
            r#"<div class="session-row" style="justify-content:center;margin-top:14px"><span class="session-pill">Discount selected {dv} ({code})</span></div>"#,
            dv = money(dv),
            code = html_escape(&discount_code),
        )
    } else {
        String::new()
    };

    let total_card = format!(
        r#"<section class="payment-total-card">
  <div class="payment-total-card__label">Total Due</div>
  <div class="payment-total-card__value">{due}</div>
  {discount_pill}
</section>"#,
        due = money(due),
        discount_pill = discount_pill,
    );

    let method_section = if payment_method.is_empty() {
        r#"<section class="card actions">
  <button class="payment-option" data-pos-payment="card">
    <span class="payment-option__main">
      <span class="payment-icon" style="background:var(--blue-light)">💳</span>
      <span class="payment-copy-stack">
        <span class="payment-title">Credit / Debit Card</span>
        <span class="payment-copy">Use the external terminal, then confirm the sale back at the counter.</span>
      </span>
    </span>
    <span class="payment-chevron">›</span>
  </button>
  <button class="payment-option" data-pos-payment="cash">
    <span class="payment-option__main">
      <span class="payment-icon" style="background:var(--success-light)">💵</span>
      <span class="payment-copy-stack">
        <span class="payment-title">Cash</span>
        <span class="payment-copy">Use quick tender buttons, calculate change, and invite a round-up gift.</span>
      </span>
    </span>
    <span class="payment-chevron">›</span>
  </button>
  <button class="payment-option" data-pos-payment="iou">
    <span class="payment-option__main">
      <span class="payment-icon" style="background:var(--warning-light)">🧾</span>
      <span class="payment-copy-stack">
        <span class="payment-title">Put on Tab / IOU</span>
        <span class="payment-copy">Record the customer name and follow up later from the admin queue.</span>
      </span>
    </span>
    <span class="payment-chevron">›</span>
  </button>
</section>"#.to_string()
    } else if payment_method == "card" {
        let discount_detail = if !discount_code.is_empty() {
            format!(
                r#"<div class="totals-row"><span>Discount selected</span><span>{} ({})</span></div>"#,
                money(dv),
                html_escape(&discount_code),
            )
        } else {
            String::new()
        };
        format!(
            r#"<section class="card">
  <h2 class="section-title">Card handoff</h2>
  <p class="subtle">Open the terminal, take the card, then return here to confirm the payment.</p>
  <div class="totals" style="margin-top:14px">
    <div class="totals-row"><span>Cart subtotal</span><strong>{subtotal}</strong></div>
    {discount_detail}
    <div class="totals-row"><span>Amount due</span><strong>{due}</strong></div>
    <div class="totals-row"><span>Provider</span><span>Square handoff</span></div>
  </div>
  <div class="actions" style="margin-top:14px">
    <button class="pos-button--lg" id="pos-complete-card">Payment Received</button>
    <button class="pos-button--lg pos-button--ghost" data-pos-payment-back>Back to methods</button>
  </div>
</section>"#,
            subtotal = money(total),
            discount_detail = discount_detail,
            due = money(due),
        )
    } else if payment_method == "cash" {
        let mut presets = Vec::new();
        if due > 0 {
            presets.push((due, money(due), "Exact"));
        }
        if 2000 >= due && due > 0 {
            presets.push((2000, "$20.00".to_string(), "Quick cash"));
        }
        if 5000 >= due && due > 0 {
            presets.push((5000, "$50.00".to_string(), "Notes"));
        }
        if 10000 >= due && due > 0 {
            presets.push((10000, "$100.00".to_string(), "Large note"));
        }
        // Deduplicate: if exact matches a preset, skip it
        let mut unique_presets: Vec<(i64, String, &str)> = Vec::new();
        for p in &presets {
            if !unique_presets.iter().any(|u| u.0 == p.0) {
                unique_presets.push(p.clone());
            }
        }

        let preset_buttons: String = unique_presets
            .iter()
            .map(|(cents, label, note)| {
                format!(
                    r#"<button data-pos-cash-preset="{cents}">{label}<span>{note}</span></button>"#,
                )
            })
            .collect();

        let round_up_class = if donate_change { "round-up-button round-up-button--active" } else { "round-up-button" };
        let round_up_label = if donate_change { "Round Up / Donate change is on" } else { "Round Up / Donate" };

        format!(
            r#"<section class="card">
  <h2 class="section-title">Cash tendered</h2>
  <p class="subtle">Choose a quick amount or type the amount tendered at the counter.</p>
  <div class="cash-grid">{presets}</div>
  <div style="margin-top:14px">
    <label class="field-label" for="custom-tendered">Custom cash amount</label>
    <input id="custom-tendered" value="{custom}" />
  </div>
  <button class="{round_up_class}" style="margin-top:14px" id="pos-toggle-donate">{round_up_label}</button>
  <div class="actions" style="margin-top:14px">
    <button class="pos-button--lg" id="pos-custom-cash">Use custom amount</button>
    <button class="pos-button--lg pos-button--ghost" data-pos-payment-back>Back to methods</button>
  </div>
</section>"#,
            presets = preset_buttons,
            custom = html_escape(&custom_tendered),
            round_up_class = round_up_class,
            round_up_label = round_up_label,
        )
    } else {
        // IOU
        format!(
            r#"<section class="card">
  <h2 class="section-title">Record IOU</h2>
  <p class="subtle">This order will appear in the admin queue until the customer pays.</p>
  <label class="field-label" for="iou-name">Customer name</label>
  <input id="iou-name" value="{iou_name}" />
  <div class="actions" style="margin-top:14px">
    <button class="pos-button--lg pos-button--gold" id="pos-complete-iou">Record IOU</button>
    <button class="pos-button--lg pos-button--ghost" data-pos-payment-back>Back to methods</button>
  </div>
</section>"#,
            iou_name = html_escape(&iou_name),
        )
    };

    format!(
        r#"<main class="pos-shell">
  <div class="pos-wrap">
    {header}
    {total_card}
    {method_section}
    {status}
    <button class="pos-button--lg pos-button--ghost" id="pos-back-to-basket-bottom">Back to basket</button>
  </div>
</main>"#,
        header = header,
        total_card = total_card,
        method_section = method_section,
        status = status_html(),
    )
}

fn render_complete_screen() -> String {
    let sale = read_last_sale().unwrap_or_default();
    let receipt_email = win_get_str(RECEIPT_EMAIL);

    let change_class = if sale.change_due_cents > 0 { "receipt-row receipt-row--big" } else { "receipt-row" };

    format!(
        r#"<main class="pos-shell">
  <div class="pos-wrap complete-screen">
    <div class="complete-mark"><span>✓</span></div>
    <h1 class="complete-title">SALE COMPLETE</h1>
    <section class="receipt-card">
      <div class="receipt-row"><span>Payment outcome</span><strong>{outcome}</strong></div>
      <div class="receipt-row"><span>Order total</span><strong>{total}</strong></div>
      <div class="receipt-row"><span>Discount</span><strong>{discount}</strong></div>
      <div class="{change_class}"><span>Change due</span><strong>{change}</strong></div>
      <div class="receipt-row"><span>Donation</span><strong>{donation}</strong></div>
    </section>
    <section class="receipt-card">
      <label class="field-label" for="receipt-email" style="color:white;text-align:left">Email receipt</label>
      <div class="row">
        <input id="receipt-email" value="{email}" />
        <button class="pos-button--lg" id="pos-send-receipt">Send receipt</button>
      </div>
    </section>
    {status}
    <button class="pos-button--lg pos-button--light" id="pos-next-sale">Start next sale →</button>
  </div>
</main>"#,
        outcome = if sale.status == "iou" { "IOU recorded" } else { "Paid" },
        total = money(sale.total_cents),
        discount = money(sale.discount_cents),
        change_class = change_class,
        change = money(sale.change_due_cents),
        donation = money(sale.donation_cents),
        email = html_escape(&receipt_email),
        status = status_html(),
    )
}

// ---- Render + bind ----

fn render_pos() {
    let screen = win_get_str(SCREEN);
    let html = match screen.as_str() {
        "help" => render_help_screen(),
        "payment" => render_payment_screen(),
        "complete" => render_complete_screen(),
        "main" => render_main_screen(),
        _ => render_login_screen(),
    };

    if let Some(app) = by_id("app") {
        app.set_inner_html(&html);
    }

    bind_pos_controls();
}

fn bind_pos_controls() {
    let doc = document();
    let screen = win_get_str(SCREEN);

    match screen.as_str() {
        "login" | "" => bind_login_controls(&doc),
        "help" => bind_help_controls(&doc),
        "main" => bind_main_controls(&doc),
        "payment" => bind_payment_controls(&doc),
        "complete" => bind_complete_controls(&doc),
        _ => bind_login_controls(&doc),
    }
}

fn bind_login_controls(doc: &Document) {
    // PIN key buttons
    if let Ok(buttons) = doc.query_selector_all("[data-pos-key]") {
        for i in 0..buttons.length() {
            if let Some(node) = buttons.item(i) {
                if let Some(el) = node.dyn_ref::<web_sys::HtmlElement>() {
                    let key = el.get_attribute("data-pos-key").unwrap_or_default();
                    let closure = Closure::wrap(Box::new(move || {
                        if key == "backspace" {
                            let current = win_get_str(PIN);
                            if !current.is_empty() {
                                win_set_str(PIN, &current[..current.len() - 1]);
                                render_pos();
                            }
                        } else {
                            let current = win_get_str(PIN);
                            if current.len() >= 4 {
                                return;
                            }
                            let next = format!("{current}{key}");
                            win_set_str(PIN, &next);
                            if next.len() == 4 {
                                let pin = next.clone();
                                // Delay to show the 4th dot before shifting
                                render_pos();
                                let start_closure = Closure::wrap(Box::new(move || {
                                    let pin = pin.clone();
                                    wasm_bindgen_futures::spawn_local(async move {
                                        start_shift(&pin).await;
                                    });
                                }) as Box<dyn Fn()>);
                                if let Some(w) = web_sys::window() {
                                    let _ = w.set_timeout_with_callback_and_timeout_and_arguments_0(
                                        start_closure.as_ref().unchecked_ref(),
                                        220,
                                    );
                                }
                                start_closure.forget();
                            } else {
                                render_pos();
                            }
                        }
                    }) as Box<dyn Fn()>);
                    el.set_onclick(Some(closure.as_ref().unchecked_ref()));
                    closure.forget();
                }
            }
        }
    }

    // Forgot PIN
    if let Some(el) = doc
        .get_element_by_id("pos-forgot-pin")
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let closure = Closure::wrap(Box::new(|| {
            set_ui_status(
                "warning",
                "PIN help",
                "For local testing, use 1234. In parish use, ask an admin to reset the volunteer PIN before opening the till.",
            );
            win_set_str(SCREEN, "help");
            render_pos();
        }) as Box<dyn Fn()>);
        el.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }
}

fn bind_help_controls(doc: &Document) {
    if let Some(el) = doc
        .get_element_by_id("pos-back-to-keypad")
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let closure = Closure::wrap(Box::new(|| {
            win_set_str(SCREEN, "login");
            render_pos();
        }) as Box<dyn Fn()>);
        el.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }
}

fn bind_main_controls(doc: &Document) {
    // Mode toggle
    if let Some(el) = doc
        .get_element_by_id("pos-mode-scan")
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let closure = Closure::wrap(Box::new(|| {
            win_set_str(MODE, "scan");
            render_pos();
        }) as Box<dyn Fn()>);
        el.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }
    if let Some(el) = doc
        .get_element_by_id("pos-mode-quick")
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let closure = Closure::wrap(Box::new(|| {
            win_set_str(MODE, "quick");
            render_pos();
        }) as Box<dyn Fn()>);
        el.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // Scan button
    if let Some(el) = doc
        .get_element_by_id("pos-scan-item")
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let closure = Closure::wrap(Box::new(|| {
            wasm_bindgen_futures::spawn_local(scan_item());
        }) as Box<dyn Fn()>);
        el.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // Barcode input
    if let Some(el) = doc
        .get_element_by_id("barcode")
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let closure = Closure::wrap(Box::new(|| {
            if let Some(input) = by_id("barcode")
                .and_then(|e| e.dyn_into::<web_sys::HtmlInputElement>().ok())
            {
                win_set_str(BARCODE, &input.value());
            }
        }) as Box<dyn Fn()>);
        el.set_oninput(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // Quick items
    if let Ok(buttons) = doc.query_selector_all("[data-pos-quick]") {
        for i in 0..buttons.length() {
            if let Some(node) = buttons.item(i) {
                if let Some(el) = node.dyn_ref::<web_sys::HtmlElement>() {
                    let item_id = el.get_attribute("data-pos-quick").unwrap_or_default();
                    let config = read_pos_config();
                    let label = config.quick_items
                        .iter()
                        .find(|qi| qi.item_id == item_id)
                        .map(|qi| qi.label.clone())
                        .unwrap_or_default();
                    let closure = Closure::wrap(Box::new(move || {
                        let id = item_id.clone();
                        let lbl = label.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            add_quick_item(&id, &lbl).await;
                        });
                    }) as Box<dyn Fn()>);
                    el.set_onclick(Some(closure.as_ref().unchecked_ref()));
                    closure.forget();
                }
            }
        }
    }

    // Cart quantity controls
    if let Ok(buttons) = doc.query_selector_all("[data-pos-qty-dec]") {
        for i in 0..buttons.length() {
            if let Some(node) = buttons.item(i) {
                if let Some(el) = node.dyn_ref::<web_sys::HtmlElement>() {
                    let item_id = el.get_attribute("data-pos-qty-dec").unwrap_or_default();
                    let title = el.get_attribute("data-pos-qty-title").unwrap_or_default();
                    let qty: i64 = el.get_attribute("data-pos-qty-current").and_then(|v| v.parse().ok()).unwrap_or(0);
                    let closure = Closure::wrap(Box::new(move || {
                        let id = item_id.clone();
                        let t = title.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            change_cart_quantity(&id, &t, qty, -1).await;
                        });
                    }) as Box<dyn Fn()>);
                    el.set_onclick(Some(closure.as_ref().unchecked_ref()));
                    closure.forget();
                }
            }
        }
    }
    if let Ok(buttons) = doc.query_selector_all("[data-pos-qty-inc]") {
        for i in 0..buttons.length() {
            if let Some(node) = buttons.item(i) {
                if let Some(el) = node.dyn_ref::<web_sys::HtmlElement>() {
                    let item_id = el.get_attribute("data-pos-qty-inc").unwrap_or_default();
                    let title = el.get_attribute("data-pos-qty-title").unwrap_or_default();
                    let qty: i64 = el.get_attribute("data-pos-qty-current").and_then(|v| v.parse().ok()).unwrap_or(0);
                    let closure = Closure::wrap(Box::new(move || {
                        let id = item_id.clone();
                        let t = title.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            change_cart_quantity(&id, &t, qty, 1).await;
                        });
                    }) as Box<dyn Fn()>);
                    el.set_onclick(Some(closure.as_ref().unchecked_ref()));
                    closure.forget();
                }
            }
        }
    }

    // Discount chips
    if let Ok(buttons) = doc.query_selector_all("[data-pos-discount]") {
        for i in 0..buttons.length() {
            if let Some(node) = buttons.item(i) {
                if let Some(el) = node.dyn_ref::<web_sys::HtmlElement>() {
                    let code = el.get_attribute("data-pos-discount").unwrap_or_default();
                    let closure = Closure::wrap(Box::new(move || {
                        win_set_str(DISCOUNT_CODE, &code);
                        render_pos();
                    }) as Box<dyn Fn()>);
                    el.set_onclick(Some(closure.as_ref().unchecked_ref()));
                    closure.forget();
                }
            }
        }
    }

    // Checkout button
    if let Some(el) = doc
        .get_element_by_id("pos-checkout")
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let closure = Closure::wrap(Box::new(|| {
            let total = win_get_i64(TOTAL);
            if total == 0 {
                set_ui_status("warning", "Basket empty", "Scan an item or tap a quick tile before opening payment options.");
                render_pos();
                return;
            }
            win_set_str(PAYMENT_METHOD, "");
            win_set_str(SCREEN, "payment");
            render_pos();
        }) as Box<dyn Fn()>);
        el.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }
}

fn bind_payment_controls(doc: &Document) {
    // Back to basket
    for id in &["pos-back-to-basket", "pos-back-to-basket-bottom"] {
        if let Some(el) = doc
            .get_element_by_id(id)
            .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
        {
            let closure = Closure::wrap(Box::new(|| {
                win_set_str(SCREEN, "main");
                render_pos();
            }) as Box<dyn Fn()>);
            el.set_onclick(Some(closure.as_ref().unchecked_ref()));
            closure.forget();
        }
    }

    // Payment method selection
    if let Ok(buttons) = doc.query_selector_all("[data-pos-payment]") {
        for i in 0..buttons.length() {
            if let Some(node) = buttons.item(i) {
                if let Some(el) = node.dyn_ref::<web_sys::HtmlElement>() {
                    let method = el.get_attribute("data-pos-payment").unwrap_or_default();
                    let closure = Closure::wrap(Box::new(move || {
                        win_set_str(PAYMENT_METHOD, &method);
                        render_pos();
                    }) as Box<dyn Fn()>);
                    el.set_onclick(Some(closure.as_ref().unchecked_ref()));
                    closure.forget();
                }
            }
        }
    }

    // Back to methods
    if let Ok(buttons) = doc.query_selector_all("[data-pos-payment-back]") {
        for i in 0..buttons.length() {
            if let Some(node) = buttons.item(i) {
                if let Some(el) = node.dyn_ref::<web_sys::HtmlElement>() {
                    let closure = Closure::wrap(Box::new(|| {
                        win_set_str(PAYMENT_METHOD, "");
                        render_pos();
                    }) as Box<dyn Fn()>);
                    el.set_onclick(Some(closure.as_ref().unchecked_ref()));
                    closure.forget();
                }
            }
        }
    }

    // Card complete
    if let Some(el) = doc
        .get_element_by_id("pos-complete-card")
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let closure = Closure::wrap(Box::new(|| {
            wasm_bindgen_futures::spawn_local(complete_card());
        }) as Box<dyn Fn()>);
        el.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // Cash presets
    if let Ok(buttons) = doc.query_selector_all("[data-pos-cash-preset]") {
        for i in 0..buttons.length() {
            if let Some(node) = buttons.item(i) {
                if let Some(el) = node.dyn_ref::<web_sys::HtmlElement>() {
                    let cents: i64 = el.get_attribute("data-pos-cash-preset").and_then(|v| v.parse().ok()).unwrap_or(0);
                    let closure = Closure::wrap(Box::new(move || {
                        wasm_bindgen_futures::spawn_local(async move {
                            complete_cash(cents).await;
                        });
                    }) as Box<dyn Fn()>);
                    el.set_onclick(Some(closure.as_ref().unchecked_ref()));
                    closure.forget();
                }
            }
        }
    }

    // Custom cash
    if let Some(el) = doc
        .get_element_by_id("pos-custom-cash")
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let closure = Closure::wrap(Box::new(|| {
            // Read input value
            let val = by_id("custom-tendered")
                .and_then(|e| e.dyn_into::<web_sys::HtmlInputElement>().ok())
                .map(|input| input.value())
                .unwrap_or_default();
            let cents = (val.parse::<f64>().unwrap_or(0.0) * 100.0).round() as i64;
            wasm_bindgen_futures::spawn_local(async move {
                complete_cash(cents).await;
            });
        }) as Box<dyn Fn()>);
        el.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // Custom tendered input
    if let Some(el) = doc
        .get_element_by_id("custom-tendered")
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let closure = Closure::wrap(Box::new(|| {
            if let Some(input) = by_id("custom-tendered")
                .and_then(|e| e.dyn_into::<web_sys::HtmlInputElement>().ok())
            {
                win_set_str(CUSTOM_TENDERED, &input.value());
            }
        }) as Box<dyn Fn()>);
        el.set_oninput(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // Toggle donate change
    if let Some(el) = doc
        .get_element_by_id("pos-toggle-donate")
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let closure = Closure::wrap(Box::new(|| {
            let current = win_get_bool(DONATE_CHANGE);
            win_set_bool(DONATE_CHANGE, !current);
            render_pos();
        }) as Box<dyn Fn()>);
        el.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // IOU name input
    if let Some(el) = doc
        .get_element_by_id("iou-name")
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let closure = Closure::wrap(Box::new(|| {
            if let Some(input) = by_id("iou-name")
                .and_then(|e| e.dyn_into::<web_sys::HtmlInputElement>().ok())
            {
                win_set_str(IOU_NAME, &input.value());
            }
        }) as Box<dyn Fn()>);
        el.set_oninput(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // IOU complete
    if let Some(el) = doc
        .get_element_by_id("pos-complete-iou")
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let closure = Closure::wrap(Box::new(|| {
            wasm_bindgen_futures::spawn_local(complete_iou());
        }) as Box<dyn Fn()>);
        el.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }
}

fn bind_complete_controls(doc: &Document) {
    // Receipt email input
    if let Some(el) = doc
        .get_element_by_id("receipt-email")
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let closure = Closure::wrap(Box::new(|| {
            if let Some(input) = by_id("receipt-email")
                .and_then(|e| e.dyn_into::<web_sys::HtmlInputElement>().ok())
            {
                win_set_str(RECEIPT_EMAIL, &input.value());
            }
        }) as Box<dyn Fn()>);
        el.set_oninput(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // Send receipt
    if let Some(el) = doc
        .get_element_by_id("pos-send-receipt")
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let closure = Closure::wrap(Box::new(|| {
            let email = win_get_str(RECEIPT_EMAIL);
            if email.is_empty() {
                set_ui_status("success", "Receipt queued", "Add an email to send a receipt.");
            } else {
                set_ui_status("success", "Receipt queued", &format!("Receipt will be sent to {email}."));
            }
            render_pos();
        }) as Box<dyn Fn()>);
        el.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // Next sale
    if let Some(el) = doc
        .get_element_by_id("pos-next-sale")
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let closure = Closure::wrap(Box::new(|| {
            reset_sale_state();
            set_ui_status("warning", "Ready for next customer", "Scan a title or tap a quick item to build the next basket.");
            render_pos();
        }) as Box<dyn Fn()>);
        el.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }
}

// ---- Entry point ----

pub fn mount_pos_island() {
    // Only mount on POS page (has #app div and no other islands)
    if by_id("app").is_none() {
        return;
    }

    // Initialize state
    win_set_str(SCREEN, "login");
    win_set_str(MODE, "scan");
    win_set_str(PIN, "");
    win_set_str(TOKEN, "");
    win_set_str(BARCODE, "");
    write_cart(&[]);
    win_set_i64(TOTAL, 0);
    set_ui_status("warning", "Shift not started", "Enter the four-digit PIN to open the parish till.");
    win_set_str(PAYMENT_METHOD, "");
    win_set_str(CUSTOM_TENDERED, "20.00");
    win_set_bool(DONATE_CHANGE, true);
    win_set_str(IOU_NAME, "");
    win_set_str(RECEIPT_EMAIL, "");
    win_set_str(DISCOUNT_CODE, "");
    write_last_sale(None);

    // Fetch POS config (quick items, discount codes) from server
    wasm_bindgen_futures::spawn_local(async {
        if let Ok(config) = fetch_pos_config().await {
            write_pos_config(&config);
            render_pos();
        }
    });

    render_pos();

    // Set ready flag
    if let Some(window) = web_sys::window() {
        let _ = js_sys::Reflect::set(
            &window,
            &JsValue::from_str("__SCRIPTORIUM_POS_READY"),
            &JsValue::TRUE,
        );
    }
}
