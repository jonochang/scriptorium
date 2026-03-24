use crate::scanner::{self, ScannerBindings};
use wasm_bindgen::prelude::*;
use web_sys::Document;

// ---- Helpers ----

fn document() -> Document {
    web_sys::window().expect("window").document().expect("document")
}

fn by_id(id: &str) -> Option<web_sys::Element> {
    document().get_element_by_id(id)
}

/// Read `.value` from any element (input, select, textarea) via JS reflection.
fn get_value(id: &str) -> String {
    by_id(id)
        .and_then(|el| js_sys::Reflect::get(&el, &JsValue::from_str("value")).ok())
        .and_then(|v| v.as_string())
        .unwrap_or_default()
}

fn set_value(id: &str, value: &str) {
    if let Some(el) = by_id(id) {
        let _ = js_sys::Reflect::set(&el, &JsValue::from_str("value"), &JsValue::from_str(value));
    }
}

fn set_isbn_value(value: &str) {
    set_value("isbn", value);
    set_value("isbn-review", value);
}

fn set_save_button_label(label: &str) {
    if let Some(el) = by_id("save-product") {
        el.set_text_content(Some(label));
    }
}

fn js_str(obj: &JsValue, key: &str) -> String {
    js_sys::Reflect::get(obj, &JsValue::from_str(key))
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_default()
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn js_f64(obj: &JsValue, key: &str) -> f64 {
    js_sys::Reflect::get(obj, &JsValue::from_str(key)).ok().and_then(|v| v.as_f64()).unwrap_or(0.0)
}

// ---- Window-global state ----

fn win_get_f64(key: &str) -> f64 {
    web_sys::window()
        .and_then(|w| js_sys::Reflect::get(&w, &JsValue::from_str(key)).ok())
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0)
}

fn win_set_f64(key: &str, value: f64) {
    if let Some(w) = web_sys::window() {
        let _ = js_sys::Reflect::set(&w, &JsValue::from_str(key), &JsValue::from(value));
    }
}

const INTAKE_STEP: &str = "__intakeStep";
const SCAN_TIMER: &str = "__intakeScanTimer";
const RESET_TIMER: &str = "__intakeResetTimer";
const LAST_SCAN: &str = "__intakeLastScan";
const LAST_SCAN_AT: &str = "__intakeLastScanAt";
const CAMERA_STREAM: &str = "__intakeCameraStream";
const DETECTOR: &str = "__intakeDetector";
const SCANNER_DEBUG: &str = "__intakeScannerDebug";

const SCANNER: ScannerBindings = ScannerBindings {
    video_id: "camera",
    overlay_id: "camera-overlay",
    empty_id: "camera-empty",
    start_button_id: "camera-start",
    stop_button_id: "camera-stop",
    status_id: "scanner-status",
    idle_start_label: "Start scanner",
    active_start_label: "Stop scanner",
    scan_timer_key: SCAN_TIMER,
    last_scan_key: LAST_SCAN,
    last_scan_at_key: LAST_SCAN_AT,
    camera_stream_key: CAMERA_STREAM,
    detector_key: DETECTOR,
    status_message_key: None,
    status_tone_key: None,
    status_class: intake_scanner_status_class,
    debug_toggle_id: None,
    debug_panel_id: Some("scanner-debug-panel"),
    debug_canvas_id: Some("scanner-debug-canvas"),
    debug_meta_id: Some("scanner-debug-meta"),
    debug_enabled_key: Some(SCANNER_DEBUG),
};

// ---- UI functions ----

fn set_scanner_status(message: &str, tone: &str) {
    scanner::set_scanner_status(SCANNER, message, tone);
}

fn intake_scanner_status_class(tone: &str) -> String {
    if tone.is_empty() {
        "intake-status-copy".to_string()
    } else {
        format!("intake-status-copy is-{tone}")
    }
}

fn set_lookup_status(message: &str, tone: &str) {
    if let Some(panel) = by_id("intake-lookup-status") {
        panel.set_text_content(Some(message));
        let class = if tone.is_empty() {
            "notice-panel".to_string()
        } else {
            format!("notice-panel notice-panel--{tone}")
        };
        panel.set_class_name(&class);
    }
}

fn set_step(step: i32) {
    win_set_f64(INTAKE_STEP, step as f64);
    let doc = document();

    // Update step indicators
    if let Ok(nodes) = doc.query_selector_all("[data-step]") {
        for i in 0..nodes.length() {
            if let Some(node) = nodes.item(i) {
                if let Some(el) = node.dyn_ref::<web_sys::HtmlElement>() {
                    let current: i32 =
                        el.get_attribute("data-step").and_then(|v| v.parse().ok()).unwrap_or(0);
                    let _ = el.class_list().toggle_with_force("is-active", current == step);
                    let _ = el.class_list().toggle_with_force("is-done", current < step);
                    if let Ok(Some(badge)) = el.query_selector(".intake-step-badge") {
                        let text = if current < step {
                            "\u{2713}".to_string()
                        } else {
                            (current + 1).to_string()
                        };
                        badge.set_text_content(Some(&text));
                    }
                }
            }
        }
    }

    // Update step connectors
    if let Ok(nodes) = doc.query_selector_all("[data-step-connector]") {
        for i in 0..nodes.length() {
            if let Some(node) = nodes.item(i) {
                if let Some(el) = node.dyn_ref::<web_sys::HtmlElement>() {
                    let current: i32 = el
                        .get_attribute("data-step-connector")
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(0);
                    let _ = el.class_list().toggle_with_force("is-done", current < step);
                }
            }
        }
    }

    // Toggle review visibility
    if let Some(el) = by_id("intake-review") {
        let _ = el
            .dyn_ref::<web_sys::HtmlElement>()
            .map(|e| e.class_list().toggle_with_force("is-visible", step >= 1));
    }

    // Toggle success visibility
    if let Some(el) = by_id("intake-success") {
        let _ = el
            .dyn_ref::<web_sys::HtmlElement>()
            .map(|e| e.class_list().toggle_with_force("is-visible", step == 2));
    }

    // Toggle reset button
    if let Some(el) = by_id("intake-reset").and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        el.set_hidden(step == 0);
    }

    // Toggle hint
    if let Some(el) = by_id("intake-hint").and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok()) {
        el.set_hidden(step != 0);
    }
}

fn set_camera_state(active: bool) {
    scanner::set_camera_state(SCANNER, active);
}

fn set_cover_preview(url: &str, has_stored_asset: bool) {
    let preview = by_id("cover-preview");
    let frame = by_id("cover-frame");
    let placeholder =
        by_id("cover-placeholder").and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok());
    let loaded = by_id("cover-loaded").and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok());

    if !url.is_empty() {
        if let Some(ref img) = preview {
            let _ = img.set_attribute("src", url);
            if let Some(he) = img.dyn_ref::<web_sys::HtmlElement>() {
                he.set_hidden(false);
            }
        }
        if let Some(ref f) = frame {
            let _ = f.class_list().add_1("has-image");
        }
        if let Some(ref p) = placeholder {
            p.set_hidden(true);
        }
        if let Some(ref l) = loaded {
            l.set_hidden(!has_stored_asset);
        }
    } else {
        if let Some(ref img) = preview {
            let _ = img.remove_attribute("src");
            if let Some(he) = img.dyn_ref::<web_sys::HtmlElement>() {
                he.set_hidden(true);
            }
        }
        if let Some(ref f) = frame {
            let _ = f.class_list().remove_1("has-image");
        }
        if let Some(ref p) = placeholder {
            p.set_hidden(false);
        }
        if let Some(ref l) = loaded {
            l.set_hidden(true);
        }
    }
}

fn query_param(name: &str) -> String {
    web_sys::window()
        .and_then(|w| w.location().search().ok())
        .and_then(|search| web_sys::UrlSearchParams::new_with_str(&search).ok())
        .and_then(|params| params.get(name))
        .unwrap_or_default()
}

fn reset_intake_form() {
    let timer_id = win_get_f64(RESET_TIMER) as i32;
    if timer_id != 0 {
        if let Some(w) = web_sys::window() {
            w.clear_timeout_with_handle(timer_id);
        }
        win_set_f64(RESET_TIMER, 0.0);
    }

    set_isbn_value("");
    set_value("title", "");
    set_value("author", "");
    set_value("publisher", "");
    set_value("description", "");
    set_value("product-id", "");
    set_value("current-on-hand", "0");
    set_value("cost-cents", "");
    set_value("retail-cents", "");
    set_value("initial-stock", "5");
    set_value("reorder-point", "3");
    set_value("category", "Books");
    set_value("vendor", "Church Supplier");
    set_value("cover-image-key", "");
    set_value("cover-file", "");

    set_cover_preview("", false);
    set_save_button_label("Save Product");
    set_lookup_status("Lookup and save status will appear here.", "");
    set_scanner_status("Scan a barcode or type an ISBN to begin.", "");
    update_stock_status();
    update_category_badge();
    set_step(0);
}

fn update_stock_status() {
    let stock = get_value("initial-stock");
    let reorder = get_value("reorder-point");
    if let Some(el) = by_id("intake-stock-label") {
        let stock_display = if stock.is_empty() { "0" } else { &stock };
        let reorder_display = if reorder.is_empty() { "0" } else { &reorder };
        el.set_text_content(Some(&format!(
            "{stock_display} in stock \u{00B7} reorders at {reorder_display}"
        )));
    }
}

fn update_category_badge() {
    let cat = get_value("category");
    if let Some(el) = by_id("intake-category-badge") {
        el.set_text_content(Some(if cat.is_empty() { "BOOKS" } else { &cat }));
    }
}

fn stop_camera() {
    scanner::teardown_camera(SCANNER);
    set_scanner_status("Scanner stopped. Manual ISBN entry is still available.", "");
}

async fn boot_camera() {
    scanner::boot_camera(SCANNER, intake_render_noop, intake_handle_scan).await;
}

fn intake_render_noop() {}

fn intake_handle_scan(raw: String) {
    set_isbn_value(&raw);
    let step = win_get_f64(INTAKE_STEP) as i32;
    set_step(step.max(0));
    set_scanner_status(&format!("Detected ISBN {raw}. Fetching metadata..."), "success");
    wasm_bindgen_futures::spawn_local(lookup_impl());
}

// ---- API calls ----

async fn fetch_post(
    url: &str,
    body: &JsValue,
    headers: &web_sys::Headers,
) -> Result<(bool, JsValue), String> {
    let opts = web_sys::RequestInit::new();
    opts.set_method("POST");
    opts.set_headers(headers);
    opts.set_body(body);

    let request =
        web_sys::Request::new_with_str_and_init(url, &opts).map_err(|e| format!("{e:?}"))?;
    let window = web_sys::window().ok_or("no window")?;
    let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("{e:?}"))?;
    let resp: web_sys::Response = resp_value.dyn_into().map_err(|e| format!("{e:?}"))?;
    let ok = resp.ok();
    let json = match resp.json() {
        Ok(p) => wasm_bindgen_futures::JsFuture::from(p).await.unwrap_or(JsValue::NULL),
        Err(_) => JsValue::NULL,
    };
    Ok((ok, json))
}

async fn fetch_json_get(url: &str) -> Result<JsValue, String> {
    let token = get_value("token");
    let opts = web_sys::RequestInit::new();
    opts.set_method("GET");
    let headers = web_sys::Headers::new().map_err(|e| format!("{e:?}"))?;
    headers.set("Authorization", &format!("Bearer {token}")).map_err(|e| format!("{e:?}"))?;
    opts.set_headers(&headers);

    let request =
        web_sys::Request::new_with_str_and_init(url, &opts).map_err(|e| format!("{e:?}"))?;
    let window = web_sys::window().ok_or("no window")?;
    let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("{e:?}"))?;
    let resp: web_sys::Response = resp_value.dyn_into().map_err(|e| format!("{e:?}"))?;
    let json = match resp.json() {
        Ok(p) => wasm_bindgen_futures::JsFuture::from(p).await.unwrap_or(JsValue::NULL),
        Err(_) => JsValue::NULL,
    };
    if !resp.ok() {
        let message = js_str(&json, "message");
        if message.is_empty() {
            return Err(format!("Request failed for {url}"));
        }
        return Err(message);
    }
    Ok(json)
}

fn json_headers() -> Result<web_sys::Headers, String> {
    let headers = web_sys::Headers::new().map_err(|e| format!("{e:?}"))?;
    headers.set("content-type", "application/json").map_err(|e| format!("{e:?}"))?;
    Ok(headers)
}

fn json_headers_with_origin() -> Result<web_sys::Headers, String> {
    let headers = json_headers()?;
    let origin = web_sys::window().and_then(|w| w.location().origin().ok()).unwrap_or_default();
    headers.set("Origin", &origin).map_err(|e| format!("{e:?}"))?;
    Ok(headers)
}

fn populate_select(id: &str, values: &[String], fallback: &str, selected: &str) {
    let Some(el) = by_id(id) else {
        return;
    };

    let mut options = values
        .iter()
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .collect::<Vec<_>>();

    if options.is_empty() {
        options.push(fallback.to_string());
    } else if !options.iter().any(|value| value == fallback) {
        options.push(fallback.to_string());
    }

    if !selected.is_empty() && !options.iter().any(|value| value == selected) {
        options.push(selected.to_string());
    }

    options.sort();
    options.dedup();

    let selected_value = if selected.is_empty() { fallback } else { selected };
    let html = options
        .into_iter()
        .map(|value| {
            let escaped = escape_html(&value);
            let selected_attr = if value == selected_value { " selected" } else { "" };
            format!(r#"<option value="{escaped}"{selected_attr}>{escaped}</option>"#)
        })
        .collect::<String>();

    el.set_inner_html(&html);
    set_value(id, selected_value);
}

async fn load_taxonomies() {
    let tenant_id = get_value("tenant-id").trim().to_string();
    if tenant_id.is_empty() || get_value("token").is_empty() {
        return;
    }

    let current_category = get_value("category");
    let current_vendor = get_value("vendor");

    let categories = fetch_json_get(&format!("/api/admin/categories?tenant_id={tenant_id}")).await;
    let vendors = fetch_json_get(&format!("/api/admin/vendors?tenant_id={tenant_id}")).await;

    match categories {
        Ok(json) => {
            let values = js_sys::Reflect::get(&json, &JsValue::from_str("values"))
                .ok()
                .and_then(|v| v.dyn_into::<js_sys::Array>().ok())
                .map(|array| {
                    array
                        .iter()
                        .filter_map(|value| value.as_string())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            populate_select("category", &values, "Books", &current_category);
        }
        Err(_) => populate_select("category", &[], "Books", &current_category),
    }

    match vendors {
        Ok(json) => {
            let values = js_sys::Reflect::get(&json, &JsValue::from_str("values"))
                .ok()
                .and_then(|v| v.dyn_into::<js_sys::Array>().ok())
                .map(|array| {
                    array
                        .iter()
                        .filter_map(|value| value.as_string())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            populate_select("vendor", &values, "Church Supplier", &current_vendor);
        }
        Err(_) => populate_select("vendor", &[], "Church Supplier", &current_vendor),
    }
}

async fn load_existing_product() {
    let product_id = query_param("product_id");
    if product_id.is_empty() {
        return;
    }

    let tenant_id = get_value("tenant-id").trim().to_string();
    if tenant_id.is_empty() || get_value("token").is_empty() {
        return;
    }

    set_lookup_status("Loading product details...", "warning");

    let response =
        fetch_json_get(&format!("/api/admin/products?tenant_id={tenant_id}")).await;
    let Ok(json) = response else {
        set_lookup_status("Could not load product details for editing.", "danger");
        return;
    };

    let products = js_sys::Array::from(&json);
    let matched = products.iter().find(|product| js_str(product, "product_id") == product_id);
    let Some(product) = matched else {
        set_lookup_status("That product could not be found.", "danger");
        return;
    };

    let isbn = js_str(&product, "isbn");
    let title = js_str(&product, "title");
    let category = js_str(&product, "category");
    let vendor = js_str(&product, "vendor");
    let cover_key = js_str(&product, "cover_image_key");
    let cover_url = js_str(&product, "cover_image_url");

    set_value("product-id", &product_id);
    set_value("current-on-hand", &format!("{}", js_f64(&product, "quantity_on_hand") as i64));
    set_isbn_value(&isbn);
    set_value("title", &title);
    set_value("author", &js_str(&product, "author"));
    set_value("publisher", &js_str(&product, "publisher"));
    set_value("description", &js_str(&product, "description"));
    set_value("category", &category);
    set_value("vendor", &vendor);
    set_value("cost-cents", &format!("{:.2}", js_f64(&product, "cost_cents") / 100.0));
    set_value("retail-cents", &format!("{:.2}", js_f64(&product, "retail_cents") / 100.0));
    set_value("initial-stock", &format!("{}", js_f64(&product, "quantity_on_hand") as i64));
    set_value("cover-image-key", &cover_key);

    if !cover_url.is_empty() {
        set_cover_preview(&cover_url, !cover_key.is_empty());
    }

    load_taxonomies().await;
    set_save_button_label("Update Product");
    update_stock_status();
    update_category_badge();
    set_step(1);
    set_lookup_status(
        "Editing existing product. Save updates details only; adjust stock in Inventory.",
        "success",
    );
    set_scanner_status("Product loaded for editing.", "success");
}

async fn lookup_impl() {
    let isbn = get_value("isbn").trim().to_string();
    let token = get_value("token");

    if token.is_empty() {
        set_lookup_status("Admin session missing. Sign in again.", "danger");
        return;
    }
    if isbn.is_empty() {
        set_lookup_status("Enter or scan an ISBN before fetching metadata.", "warning");
        return;
    }

    set_scanner_status("Retrieving metadata from Open Library...", "busy");
    set_lookup_status("Fetching metadata...", "warning");

    let body = serde_json::json!({ "token": token, "isbn": isbn }).to_string();
    let headers = match json_headers() {
        Ok(h) => h,
        Err(e) => {
            set_lookup_status(&format!("Request failed: {e}"), "danger");
            return;
        }
    };
    let result =
        fetch_post("/api/admin/products/isbn-lookup", &JsValue::from_str(&body), &headers).await;

    match result {
        Err(e) => {
            set_lookup_status(&format!("Metadata lookup failed: {e}"), "danger");
            set_scanner_status("Lookup failed. Check the ISBN and try again.", "warning");
        }
        Ok((false, json)) => {
            let msg = js_str(&json, "message");
            set_lookup_status(
                if msg.is_empty() { "Metadata lookup failed." } else { &msg },
                "danger",
            );
            set_scanner_status("Lookup failed. Check the ISBN and try again.", "warning");
        }
        Ok((true, json)) => {
            let title = js_str(&json, "title");
            let author = js_str(&json, "author");
            let publisher = js_str(&json, "publisher");
            let description = js_str(&json, "description");
            let cover_url = js_str(&json, "cover_image_url");
            let category = js_str(&json, "category");
            let vendor = js_str(&json, "vendor");
            let product_id = js_str(&json, "product_id");
            let quantity_on_hand = js_f64(&json, "quantity_on_hand") as i64;
            let cost_cents = js_f64(&json, "cost_cents") as i64;
            let retail_cents = js_f64(&json, "retail_cents") as i64;
            let cover_key = js_str(&json, "cover_image_key");

            set_value("title", &title);
            set_value("author", &author);
            set_value("publisher", &publisher);
            set_value("description", &description);
            set_value("product-id", &product_id);
            set_value("current-on-hand", &quantity_on_hand.to_string());
            if !category.is_empty() {
                set_value("category", &category);
            }
            if !vendor.is_empty() {
                set_value("vendor", &vendor);
            }
            if cost_cents > 0 {
                set_value("cost-cents", &format!("{:.2}", cost_cents as f64 / 100.0));
            }
            if retail_cents > 0 {
                set_value("retail-cents", &format!("{:.2}", retail_cents as f64 / 100.0));
            }
            if quantity_on_hand >= 0 {
                set_value("initial-stock", &quantity_on_hand.to_string());
            }
            if !cover_key.is_empty() {
                set_value("cover-image-key", &cover_key);
            }

            if !cover_url.is_empty() && get_value("cover-image-key").is_empty() {
                set_cover_preview(&cover_url, false);
            } else if !cover_url.is_empty() {
                set_cover_preview(&cover_url, true);
            }

            update_stock_status();
            update_category_badge();
            set_step(1);

            if !title.is_empty() {
                set_lookup_status("Found metadata and auto-filled the product form.", "success");
                set_scanner_status(
                    &format!("\u{2713} ISBN {isbn} detected. Review the details below."),
                    "success",
                );
            } else {
                set_lookup_status(
                    "No metadata found for that ISBN. You can still fill the form manually.",
                    "warning",
                );
                set_scanner_status(
                    &format!("ISBN {isbn} detected. Complete the form manually."),
                    "success",
                );
            }
        }
    }
}

async fn upload_cover_impl() {
    let token = get_value("token");
    let tenant_id = get_value("tenant-id").trim().to_string();

    if token.is_empty() || tenant_id.is_empty() {
        set_lookup_status("Admin session missing. Sign in again before uploading.", "danger");
        return;
    }

    let file_input =
        match by_id("cover-file").and_then(|e| e.dyn_into::<web_sys::HtmlInputElement>().ok()) {
            Some(i) => i,
            None => return,
        };
    let file = match file_input.files().and_then(|fl| fl.get(0)) {
        Some(f) => f,
        None => {
            set_lookup_status("Choose an image file before uploading.", "warning");
            return;
        }
    };

    if let Ok(url) = web_sys::Url::create_object_url_with_blob(&file) {
        set_cover_preview(&url, false);
    }
    set_lookup_status("Uploading cover...", "warning");

    let form_data = match web_sys::FormData::new() {
        Ok(fd) => fd,
        Err(_) => {
            set_lookup_status("Failed to prepare upload.", "danger");
            return;
        }
    };
    let _ = form_data.append_with_str("token", &token);
    let _ = form_data.append_with_str("tenant_id", &tenant_id);
    let _ = form_data.append_with_blob("file", &file);

    let opts = web_sys::RequestInit::new();
    opts.set_method("POST");
    opts.set_body(&form_data.into());

    let result = async {
        let request =
            web_sys::Request::new_with_str_and_init("/api/admin/products/cover-upload", &opts)
                .map_err(|e| format!("{e:?}"))?;
        let window = web_sys::window().ok_or("no window")?;
        let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
            .await
            .map_err(|e| format!("{e:?}"))?;
        let resp: web_sys::Response = resp_value.dyn_into().map_err(|e| format!("{e:?}"))?;
        let ok = resp.ok();
        let json = match resp.json() {
            Ok(p) => wasm_bindgen_futures::JsFuture::from(p).await.unwrap_or(JsValue::NULL),
            Err(_) => JsValue::NULL,
        };
        Ok::<(bool, JsValue), String>((ok, json))
    }
    .await;

    match result {
        Err(e) => set_lookup_status(&format!("Cover upload failed: {e}"), "danger"),
        Ok((false, json)) => {
            let msg = js_str(&json, "message");
            set_lookup_status(if msg.is_empty() { "Cover upload failed." } else { &msg }, "danger");
        }
        Ok((true, json)) => {
            let object_key = js_str(&json, "object_key");
            set_value("cover-image-key", &object_key);

            let asset_url = js_str(&json, "asset_url");
            if !asset_url.is_empty() {
                set_cover_preview(&asset_url, true);
            } else if let Some(img) = by_id("cover-preview") {
                let src = img.get_attribute("src").unwrap_or_default();
                set_cover_preview(&src, true);
            }
            set_lookup_status(
                "Cover uploaded and ready to save with the product record.",
                "success",
            );
        }
    }
}

async fn save_product_impl() {
    let token = get_value("token");
    let tenant_id = get_value("tenant-id").trim().to_string();

    if tenant_id.is_empty() {
        set_lookup_status(
            "Admin session missing. Sign in again to load the tenant before saving inventory.",
            "danger",
        );
        return;
    }

    let isbn = get_value("isbn").trim().to_string();
    let title = get_value("title").trim().to_string();
    if title.is_empty() {
        set_lookup_status("Enter a title before saving the product.", "warning");
        return;
    }

    let category = {
        let v = get_value("category").trim().to_string();
        if v.is_empty() { "Books".to_string() } else { v }
    };
    let vendor = {
        let v = get_value("vendor").trim().to_string();
        if v.is_empty() { "Church Supplier".to_string() } else { v }
    };
    let initial_stock: i64 = get_value("initial-stock").parse().unwrap_or(0);
    let cost_cents: i64 = (get_value("cost-cents").parse::<f64>().unwrap_or(0.0) * 100.0).round() as i64;
    let retail_cents: i64 = (get_value("retail-cents").parse::<f64>().unwrap_or(0.0) * 100.0).round() as i64;
    let cover_image_key = get_value("cover-image-key");
    let existing_product_id = get_value("product-id").trim().to_string();
    let is_edit = !existing_product_id.is_empty();
    let current_on_hand: i64 = get_value("current-on-hand").parse().unwrap_or(0);

    set_lookup_status("Saving product...", "warning");

    let product_id = if is_edit {
        existing_product_id.clone()
    } else if isbn.is_empty() {
        format!("prd-{}", js_sys::Date::now() as u64)
    } else {
        format!("prd-{isbn}")
    };

    let body = serde_json::json!({
        "token": token,
        "tenant_id": tenant_id,
        "product_id": product_id,
        "title": title,
        "isbn": isbn,
        "author": get_value("author").trim(),
        "publisher": get_value("publisher").trim(),
        "description": get_value("description").trim(),
        "category": category,
        "vendor": vendor,
        "cost_cents": cost_cents,
        "retail_cents": retail_cents,
        "cover_image_key": if cover_image_key.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(cover_image_key) },
    });

    let headers = match json_headers_with_origin() {
        Ok(h) => h,
        Err(e) => {
            set_lookup_status(&format!("Request failed: {e}"), "danger");
            return;
        }
    };

    let result =
        fetch_post("/api/admin/products", &JsValue::from_str(&body.to_string()), &headers).await;

    match result {
        Err(e) => set_lookup_status(&format!("Save failed: {e}"), "danger"),
        Ok((false, json)) => {
            let msg = js_str(&json, "message");
            set_lookup_status(if msg.is_empty() { "Save failed." } else { &msg }, "danger");
        }
        Ok((true, json)) => {
            let saved_title = js_str(&json, "title");
            let display_title = if saved_title.is_empty() { &title } else { &saved_title };

            let mut success_message = if is_edit {
                format!("Updated {display_title}.")
            } else {
                format!("Saved {display_title} for {category}.")
            };

            let desired_stock = initial_stock.max(0);
            let stock_delta = desired_stock - current_on_hand;

            if stock_delta == 0 {
                if !is_edit {
                    success_message.push_str(" Stock level unchanged.");
                }
            } else if stock_delta > 0 {
                let receive_body = serde_json::json!({
                    "token": token,
                    "tenant_id": tenant_id,
                    "isbn": isbn,
                    "quantity": stock_delta,
                });

                let receive_headers = match json_headers_with_origin() {
                    Ok(h) => h,
                    Err(_) => {
                        success_message.push_str(
                            ", but stock receive failed: could not build request headers.",
                        );
                        finish_save(&success_message);
                        return;
                    }
                };

                let receive_result = fetch_post(
                    "/api/admin/inventory/receive",
                    &JsValue::from_str(&receive_body.to_string()),
                    &receive_headers,
                )
                .await;

                match receive_result {
                    Ok((true, rjson)) => {
                        let on_hand = js_sys::Reflect::get(&rjson, &JsValue::from_str("on_hand"))
                            .ok()
                            .and_then(|v| v.as_f64())
                            .map(|v| v as i64)
                            .unwrap_or(desired_stock);
                        if is_edit {
                            success_message.push_str(&format!(" Stock updated to {on_hand}."));
                        } else {
                            success_message
                                .push_str(&format!(" Received opening stock, now on hand {on_hand}."));
                        }
                        set_value("current-on-hand", &on_hand.to_string());
                    }
                    Ok((false, rjson)) => {
                        let msg = js_str(&rjson, "message");
                        let err = if msg.is_empty() { "unknown error" } else { &msg };
                        success_message =
                            format!("{success_message} Stock update failed: {err}.");
                    }
                    Err(e) => {
                        success_message =
                            format!("{success_message} Stock update failed: {e}.");
                    }
                }
            } else {
                let adjust_body = serde_json::json!({
                    "token": token,
                    "tenant_id": tenant_id,
                    "isbn": isbn,
                    "delta": stock_delta,
                    "reason": "intake_update",
                });

                let adjust_headers = match json_headers_with_origin() {
                    Ok(h) => h,
                    Err(_) => {
                        success_message.push_str(
                            ", but stock adjustment failed: could not build request headers.",
                        );
                        finish_save(&success_message);
                        return;
                    }
                };

                let adjust_result = fetch_post(
                    "/api/admin/inventory/adjust",
                    &JsValue::from_str(&adjust_body.to_string()),
                    &adjust_headers,
                )
                .await;

                match adjust_result {
                    Ok((true, rjson)) => {
                        let on_hand = js_sys::Reflect::get(&rjson, &JsValue::from_str("on_hand"))
                            .ok()
                            .and_then(|v| v.as_f64())
                            .map(|v| v as i64)
                            .unwrap_or(desired_stock);
                        success_message.push_str(&format!(" Stock updated to {on_hand}."));
                        set_value("current-on-hand", &on_hand.to_string());
                    }
                    Ok((false, rjson)) => {
                        let msg = js_str(&rjson, "message");
                        let err = if msg.is_empty() { "unknown error" } else { &msg };
                        success_message =
                            format!("{success_message} Stock update failed: {err}.");
                    }
                    Err(e) => {
                        success_message =
                            format!("{success_message} Stock update failed: {e}.");
                    }
                }
            }

            finish_save(&success_message);
        }
    }
}

fn finish_save(message: &str) {
    set_lookup_status(message, "success");
    if let Some(el) = by_id("intake-success-copy") {
        el.set_text_content(Some(message));
    }
    set_step(2);

    // Auto-reset after 2500ms
    let msg = message.to_string();
    let _ = msg; // suppress unused warning; reset_intake_form doesn't need it
    let closure = Closure::wrap(Box::new(|| {
        reset_intake_form();
    }) as Box<dyn Fn()>);
    if let Some(w) = web_sys::window() {
        if let Ok(id) = w.set_timeout_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            2500,
        ) {
            win_set_f64(RESET_TIMER, id as f64);
        }
    }
    closure.forget();
}

// ---- Event binding ----

fn bind_intake_controls() {
    let doc = document();

    // Lookup button
    if let Some(el) =
        doc.get_element_by_id("lookup").and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let closure = Closure::wrap(Box::new(|| {
            wasm_bindgen_futures::spawn_local(lookup_impl());
        }) as Box<dyn Fn()>);
        el.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // Upload cover button
    if let Some(el) = doc
        .get_element_by_id("upload-cover")
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let closure = Closure::wrap(Box::new(|| {
            wasm_bindgen_futures::spawn_local(upload_cover_impl());
        }) as Box<dyn Fn()>);
        el.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // Save product button
    if let Some(el) = doc
        .get_element_by_id("save-product")
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let closure = Closure::wrap(Box::new(|| {
            wasm_bindgen_futures::spawn_local(save_product_impl());
        }) as Box<dyn Fn()>);
        el.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // Camera start
    if let Some(el) = doc
        .get_element_by_id("camera-start")
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let closure = Closure::wrap(Box::new(|| {
            if scanner::camera_stream_present(SCANNER) {
                stop_camera();
            } else {
                wasm_bindgen_futures::spawn_local(boot_camera());
            }
        }) as Box<dyn Fn()>);
        el.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // Camera stop
    if let Some(el) =
        doc.get_element_by_id("camera-stop").and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        el.set_hidden(true);
    }

    // Reset button
    if let Some(el) = doc
        .get_element_by_id("intake-reset")
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let closure = Closure::wrap(Box::new(|| reset_intake_form()) as Box<dyn Fn()>);
        el.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // ISBN input listener
    if let Some(el) =
        doc.get_element_by_id("isbn").and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let closure = Closure::wrap(Box::new(|| {
            let value = get_value("isbn").trim().to_string();
            set_value("isbn-review", &value);
            if value.is_empty() {
                set_scanner_status("Scan a barcode or type an ISBN to begin.", "");
            } else if value.len() >= 10 {
                set_scanner_status(
                    &format!("\u{2713} ISBN {value} detected. Click Fetch to pull metadata."),
                    "success",
                );
            } else {
                set_scanner_status("Keep typing the ISBN or start the scanner.", "busy");
            }
        }) as Box<dyn Fn()>);
        el.set_oninput(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // Cover file change listener
    if let Some(el) =
        doc.get_element_by_id("cover-file").and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let closure = Closure::wrap(Box::new(|| {
            let input =
                by_id("cover-file").and_then(|e| e.dyn_into::<web_sys::HtmlInputElement>().ok());
            if let Some(file) = input.and_then(|i| i.files()).and_then(|fl| fl.get(0)) {
                if let Ok(url) = web_sys::Url::create_object_url_with_blob(&file) {
                    set_cover_preview(&url, false);
                }
                set_lookup_status(
                    "Cover selected. Upload it to store with the product.",
                    "warning",
                );
            }
        }) as Box<dyn Fn()>);
        let _ = el.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref());
        closure.forget();
    }

    // Stock/reorder input listeners → update status bar
    for field_id in &["initial-stock", "reorder-point"] {
        if let Some(el) =
            doc.get_element_by_id(field_id).and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
        {
            let closure = Closure::wrap(Box::new(|| update_stock_status()) as Box<dyn Fn()>);
            el.set_oninput(Some(closure.as_ref().unchecked_ref()));
            closure.forget();
        }
    }

    // Category change → update badge
    if let Some(el) =
        doc.get_element_by_id("category").and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let closure = Closure::wrap(Box::new(|| update_category_badge()) as Box<dyn Fn()>);
        let _ = el.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref());
        closure.forget();
    }

    // beforeunload - stop camera
    scanner::install_beforeunload_stop(SCANNER);

    if let Some(window) = web_sys::window() {
        let closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
            let Some(keyboard) = event.dyn_ref::<web_sys::KeyboardEvent>() else {
                return;
            };
            if !(keyboard.ctrl_key() || keyboard.meta_key()) || !keyboard.shift_key() {
                return;
            }
            if keyboard.key().to_ascii_lowercase() != "d" {
                return;
            }
            keyboard.prevent_default();
            let next = !scanner::debug_enabled(SCANNER);
            scanner::set_debug_enabled(SCANNER, next);
            set_scanner_status(
                if next {
                    "Scanner debug enabled. Press Cmd/Ctrl+Shift+D to hide it."
                } else {
                    "Scanner debug hidden. Press Cmd/Ctrl+Shift+D to show it again."
                },
                "busy",
            );
        }) as Box<dyn FnMut(web_sys::Event)>);
        let _ =
            window.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref());
        closure.forget();
    }
}

// ---- Entry point ----

pub fn mount_intake_island() {
    // Auto-detect: only mount on intake page
    if by_id("scanner-status").is_none() {
        return;
    }

    // Set auth status if token present
    if !get_value("token").is_empty() {
        if let Some(el) = by_id("intake-auth-status") {
            el.set_text_content(Some("Signed in. You can fetch metadata and save a product."));
            el.set_class_name("notice-panel notice-panel--success");
        }
    }

    set_step(0);
    set_camera_state(false);
    scanner::set_debug_enabled(SCANNER, false);
    bind_intake_controls();
    wasm_bindgen_futures::spawn_local(async {
        load_existing_product().await;
        if get_value("product-id").is_empty() {
            load_taxonomies().await;
        }
    });

    // Boot camera asynchronously
    wasm_bindgen_futures::spawn_local(boot_camera());

    // Set ready flag for browser tests
    if let Some(window) = web_sys::window() {
        let _ = js_sys::Reflect::set(
            &window,
            &JsValue::from_str("__SCRIPTORIUM_INTAKE_READY"),
            &JsValue::TRUE,
        );
    }
}
