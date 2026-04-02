use crate::api::{get_json, json_headers, json_headers_with_origin, post_json};
use crate::scanner::{self, ScannerBindings};
use leptos::{mount::mount_to, prelude::*};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

const SCAN_TIMER: &str = "__intakeScanTimer";
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

thread_local! {
    static SCAN_CALLBACK: RefCell<Option<Box<dyn Fn(String)>>> = RefCell::new(None);
    static RERENDER_CALLBACK: RefCell<Option<Box<dyn Fn()>>> = RefCell::new(None);
}

#[derive(Clone, Default, PartialEq)]
struct FormState {
    product_id: String,
    current_on_hand: i64,
    isbn: String,
    title: String,
    author: String,
    publisher: String,
    description: String,
    public_title: String,
    public_author: String,
    public_publisher: String,
    public_description: String,
    public_cover_image_url: Option<String>,
    category: String,
    vendor: String,
    cost_input: String,
    retail_input: String,
    initial_stock_input: String,
    reorder_point_input: String,
    cover_image_key: Option<String>,
}

impl FormState {
    fn fresh() -> Self {
        Self {
            category: "Books".to_string(),
            vendor: "Church Supplier".to_string(),
            initial_stock_input: "5".to_string(),
            reorder_point_input: "3".to_string(),
            ..Self::default()
        }
    }

    fn save_button_label(&self) -> &'static str {
        if self.product_id.is_empty() {
            "Save product"
        } else {
            "Update Product"
        }
    }
}

#[derive(Clone, Default, PartialEq)]
struct StatusMessage {
    message: String,
    tone: String,
}

impl StatusMessage {
    fn new(message: impl Into<String>, tone: impl Into<String>) -> Self {
        Self { message: message.into(), tone: tone.into() }
    }

    fn class_name(&self) -> String {
        if self.tone.is_empty() {
            "notice-panel".to_string()
        } else {
            format!("notice-panel notice-panel--{}", self.tone)
        }
    }
}

#[derive(Clone)]
struct UploadOutcome {
    object_key: String,
    asset_url: String,
}

#[derive(Clone)]
struct LookupOutcome {
    form: FormState,
    lookup_status: StatusMessage,
    scanner_message: String,
    scanner_tone: String,
    step: i32,
    cover_preview_url: Option<String>,
    cover_loaded: bool,
}

#[derive(Clone)]
struct SaveOutcome {
    form: FormState,
    message: String,
}

#[derive(Clone, Default)]
struct RootConfig {
    token: String,
    tenant_id: String,
    product_id: String,
}

fn intake_scanner_status_class(tone: &str) -> String {
    if tone.is_empty() {
        "intake-status-copy".to_string()
    } else {
        format!("intake-status-copy is-{tone}")
    }
}

fn window() -> Option<web_sys::Window> {
    web_sys::window()
}

fn document() -> Option<web_sys::Document> {
    window()?.document()
}

fn by_id(id: &str) -> Option<web_sys::Element> {
    document()?.get_element_by_id(id)
}

fn read_root_config() -> Option<RootConfig> {
    let root = by_id("intake-root")?;
    Some(RootConfig {
        token: root.get_attribute("data-token").unwrap_or_default(),
        tenant_id: root.get_attribute("data-tenant-id").unwrap_or_default(),
        product_id: query_param("product_id"),
    })
}

fn query_param(name: &str) -> String {
    window()
        .and_then(|w| w.location().search().ok())
        .and_then(|search| web_sys::UrlSearchParams::new_with_str(&search).ok())
        .and_then(|params| params.get(name))
        .unwrap_or_default()
}

fn normalize_isbn(raw: &str) -> String {
    raw.chars().filter(|ch| ch.is_ascii_digit()).collect()
}

fn parse_money_cents(raw: &str, label: &str) -> Result<i64, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(0);
    }

    let cleaned = raw
        .chars()
        .filter(|ch| ch.is_ascii_digit() || *ch == '.' || *ch == '-')
        .collect::<String>();
    if cleaned.is_empty() || cleaned == "-" || cleaned == "." || cleaned == "-." {
        return Err(format!("{label} must be a valid amount."));
    }

    let value = cleaned.parse::<f64>().map_err(|_| format!("{label} must be a valid amount."))?;
    if value < 0.0 {
        return Err(format!("{label} cannot be negative."));
    }

    Ok((value * 100.0).round() as i64)
}

fn parse_non_negative_i64(raw: &str, label: &str) -> Result<i64, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(format!("{label} is required."));
    }
    let value = trimmed.parse::<i64>().map_err(|_| format!("{label} must be a whole number."))?;
    if value < 0 {
        return Err(format!("{label} cannot be negative."));
    }
    Ok(value)
}

fn merge_options(mut values: Vec<String>, fallback: &str, selected: &str) -> Vec<String> {
    values.retain(|value| !value.trim().is_empty());
    if !values.iter().any(|value| value == fallback) {
        values.push(fallback.to_string());
    }
    if !selected.trim().is_empty() && !values.iter().any(|value| value == selected) {
        values.push(selected.to_string());
    }
    values.sort();
    values.dedup();
    values
}

fn js_str(obj: &JsValue, key: &str) -> String {
    js_sys::Reflect::get(obj, &JsValue::from_str(key))
        .ok()
        .and_then(|value| value.as_string())
        .unwrap_or_default()
}

fn js_f64(obj: &JsValue, key: &str) -> f64 {
    js_sys::Reflect::get(obj, &JsValue::from_str(key))
        .ok()
        .and_then(|value| value.as_f64())
        .unwrap_or(0.0)
}

fn set_scanner_status(message: &str, tone: &str) {
    scanner::set_scanner_status(SCANNER, message, tone);
}

fn stop_camera() {
    scanner::teardown_camera(SCANNER);
    set_scanner_status("Scanner stopped. Manual ISBN entry is still available.", "");
}

fn scanner_rerender_bridge() {
    RERENDER_CALLBACK.with(|callback| {
        if let Some(callback) = callback.borrow().as_ref() {
            callback();
        }
    });
}

fn scanner_detect_bridge(raw: String) {
    SCAN_CALLBACK.with(|callback| {
        if let Some(callback) = callback.borrow().as_ref() {
            callback(raw);
        }
    });
}

fn current_cover_file() -> Option<web_sys::File> {
    by_id("cover-file")
        .and_then(|element| element.dyn_into::<web_sys::HtmlInputElement>().ok())
        .and_then(|input| input.files())
        .and_then(|files| files.get(0))
}

async fn load_taxonomies(
    token: String,
    tenant_id: String,
    current_category: String,
    current_vendor: String,
) -> (Vec<String>, Vec<String>) {
    let categories = get_json(&format!("/api/admin/categories?tenant_id={tenant_id}"), Some(&token)).await;
    let vendors = get_json(&format!("/api/admin/vendors?tenant_id={tenant_id}"), Some(&token)).await;

    let category_values = match categories {
        Ok(json) => js_sys::Reflect::get(&json, &JsValue::from_str("values"))
            .ok()
            .and_then(|value| value.dyn_into::<js_sys::Array>().ok())
            .map(|array| array.iter().filter_map(|value| value.as_string()).collect::<Vec<_>>())
            .unwrap_or_default(),
        Err(_) => Vec::new(),
    };
    let vendor_values = match vendors {
        Ok(json) => js_sys::Reflect::get(&json, &JsValue::from_str("values"))
            .ok()
            .and_then(|value| value.dyn_into::<js_sys::Array>().ok())
            .map(|array| array.iter().filter_map(|value| value.as_string()).collect::<Vec<_>>())
            .unwrap_or_default(),
        Err(_) => Vec::new(),
    };

    (
        merge_options(category_values, "Books", &current_category),
        merge_options(vendor_values, "Church Supplier", &current_vendor),
    )
}

async fn load_existing_product(
    token: String,
    tenant_id: String,
    product_id: String,
) -> Result<(FormState, Option<String>, bool), String> {
    let json = get_json(&format!("/api/admin/products?tenant_id={tenant_id}"), Some(&token)).await?;
    let products = js_sys::Array::from(&json);
    let product = products
        .iter()
        .find(|product| js_str(product, "product_id") == product_id)
        .ok_or_else(|| "That product could not be found.".to_string())?;

    let cover_key = js_str(&product, "cover_image_key");
    let cover_url = js_str(&product, "cover_image_url");
    let form = FormState {
        product_id,
        current_on_hand: js_f64(&product, "quantity_on_hand") as i64,
        isbn: js_str(&product, "isbn"),
        title: js_str(&product, "title"),
        author: js_str(&product, "author"),
        publisher: js_str(&product, "publisher"),
        description: js_str(&product, "description"),
        public_title: js_str(&product, "public_title"),
        public_author: js_str(&product, "public_author"),
        public_publisher: js_str(&product, "public_publisher"),
        public_description: js_str(&product, "public_description"),
        public_cover_image_url: {
            let value = js_str(&product, "public_cover_image_url");
            if value.is_empty() { None } else { Some(value) }
        },
        category: {
            let value = js_str(&product, "category");
            if value.is_empty() { "Books".to_string() } else { value }
        },
        vendor: {
            let value = js_str(&product, "vendor");
            if value.is_empty() { "Church Supplier".to_string() } else { value }
        },
        cost_input: format!("{:.2}", js_f64(&product, "cost_cents") / 100.0),
        retail_input: format!("{:.2}", js_f64(&product, "retail_cents") / 100.0),
        initial_stock_input: format!("{}", js_f64(&product, "quantity_on_hand") as i64),
        reorder_point_input: "3".to_string(),
        cover_image_key: if cover_key.is_empty() { None } else { Some(cover_key.clone()) },
    };

    Ok((form, if cover_url.is_empty() { None } else { Some(cover_url) }, !cover_key.is_empty()))
}

async fn lookup_isbn_request(token: String, isbn: String) -> Result<LookupOutcome, String> {
    let headers = json_headers()?;
    let body = serde_json::json!({ "token": token, "isbn": isbn }).to_string();
    let (ok, json) =
        post_json("/api/admin/products/isbn-lookup", &JsValue::from_str(&body), &headers).await?;
    if !ok {
        let message = js_str(&json, "message");
        return Err(if message.is_empty() {
            "Metadata lookup failed.".to_string()
        } else {
            message
        });
    }

    let title = js_str(&json, "title");
    let cover_url = js_str(&json, "cover_image_url");
    let cover_key = js_str(&json, "cover_image_key");
    let quantity_on_hand = js_f64(&json, "quantity_on_hand") as i64;

    let form = FormState {
        product_id: js_str(&json, "product_id"),
        current_on_hand: quantity_on_hand,
        isbn: js_str(&json, "isbn"),
        title: title.clone(),
        author: js_str(&json, "author"),
        publisher: js_str(&json, "publisher"),
        description: js_str(&json, "description"),
        public_title: js_str(&json, "public_title"),
        public_author: js_str(&json, "public_author"),
        public_publisher: js_str(&json, "public_publisher"),
        public_description: js_str(&json, "public_description"),
        public_cover_image_url: {
            let value = js_str(&json, "public_cover_image_url");
            if value.is_empty() { None } else { Some(value) }
        },
        category: {
            let value = js_str(&json, "category");
            if value.is_empty() { "Books".to_string() } else { value }
        },
        vendor: {
            let value = js_str(&json, "vendor");
            if value.is_empty() { "Church Supplier".to_string() } else { value }
        },
        cost_input: {
            let value = js_f64(&json, "cost_cents") as i64;
            if value > 0 { format!("{:.2}", value as f64 / 100.0) } else { String::new() }
        },
        retail_input: {
            let value = js_f64(&json, "retail_cents") as i64;
            if value > 0 { format!("{:.2}", value as f64 / 100.0) } else { String::new() }
        },
        initial_stock_input: quantity_on_hand.to_string(),
        reorder_point_input: "3".to_string(),
        cover_image_key: if cover_key.is_empty() { None } else { Some(cover_key.clone()) },
    };

    let found = !title.is_empty();
    Ok(LookupOutcome {
        form,
        lookup_status: if found {
            StatusMessage::new("Found metadata and auto-filled the product form.", "success")
        } else {
            StatusMessage::new(
                "No metadata found for that ISBN. You can still fill the form manually.",
                "warning",
            )
        },
        scanner_message: if found {
            format!("\u{2713} ISBN {isbn} detected. Review the details below.")
        } else {
            format!("ISBN {isbn} detected. Complete the form manually.")
        },
        scanner_tone: "success".to_string(),
        step: 1,
        cover_preview_url: if cover_url.is_empty() { None } else { Some(cover_url) },
        cover_loaded: !cover_key.is_empty(),
    })
}

fn validate_form(form: &FormState) -> Result<(), String> {
    if form.title.trim().is_empty() {
        return Err("Enter a title before saving the product.".to_string());
    }
    if !form.isbn.is_empty() && form.isbn.len() != 10 && form.isbn.len() != 13 {
        return Err("ISBN must be 10 or 13 digits.".to_string());
    }
    if form.category.trim().is_empty() {
        return Err("Choose a category before saving the product.".to_string());
    }
    if form.vendor.trim().is_empty() {
        return Err("Choose a vendor before saving the product.".to_string());
    }
    parse_non_negative_i64(&form.initial_stock_input, "Stock")?;
    parse_non_negative_i64(&form.reorder_point_input, "Reorder point")?;
    parse_non_negative_i64(&form.current_on_hand.to_string(), "Current stock")?;
    parse_money_cents(&form.cost_input, "Cost")?;
    parse_money_cents(&form.retail_input, "Retail price")?;
    Ok(())
}

async fn save_product_request(
    token: String,
    tenant_id: String,
    form: FormState,
) -> Result<SaveOutcome, String> {
    validate_form(&form)?;

    let initial_stock = parse_non_negative_i64(&form.initial_stock_input, "Stock")?;
    let cost_cents = parse_money_cents(&form.cost_input, "Cost")?;
    let retail_cents = parse_money_cents(&form.retail_input, "Retail price")?;

    let product_id = if !form.product_id.is_empty() {
        form.product_id.clone()
    } else if form.isbn.is_empty() {
        format!("prd-{}", js_sys::Date::now() as u64)
    } else {
        format!("prd-{}", form.isbn)
    };

    let body = serde_json::json!({
        "token": token,
        "tenant_id": tenant_id,
        "product_id": product_id,
        "title": form.title.trim(),
        "isbn": form.isbn,
        "author": form.author.trim(),
        "publisher": form.publisher.trim(),
        "description": form.description.trim(),
        "public_title": form.public_title.trim(),
        "public_author": form.public_author.trim(),
        "public_publisher": form.public_publisher.trim(),
        "public_description": form.public_description.trim(),
        "public_cover_image_url": form.public_cover_image_url,
        "category": form.category.trim(),
        "vendor": form.vendor.trim(),
        "cost_cents": cost_cents,
        "retail_cents": retail_cents,
        "cover_image_key": form.cover_image_key,
    });

    let headers = json_headers_with_origin()?;
    let (ok, json) =
        post_json("/api/admin/products", &JsValue::from_str(&body.to_string()), &headers).await?;
    if !ok {
        let message = js_str(&json, "message");
        return Err(if message.is_empty() {
            "Save failed. The product payload was rejected. Check ISBN, title, and price fields."
                .to_string()
        } else {
            message
        });
    }

    let saved_title = js_str(&json, "title");
    let display_title = if saved_title.is_empty() { form.title.clone() } else { saved_title };
    let mut success_message = if form.product_id.is_empty() {
        format!("Saved {display_title} for {}.", form.category)
    } else {
        format!("Updated {display_title}.")
    };

    let desired_stock = initial_stock.max(0);
    let stock_delta = desired_stock - form.current_on_hand;
    let mut updated_form = form.clone();
    updated_form.product_id = product_id;
    updated_form.current_on_hand = desired_stock;
    updated_form.cost_input = format!("{:.2}", cost_cents as f64 / 100.0);
    updated_form.retail_input = format!("{:.2}", retail_cents as f64 / 100.0);

    if stock_delta == 0 {
        if form.product_id.is_empty() {
            success_message.push_str(" Stock level unchanged.");
        }
        return Ok(SaveOutcome { form: updated_form, message: success_message });
    }

    let stock_headers = json_headers_with_origin()?;
    let stock_result = if stock_delta > 0 {
        let receive_body = serde_json::json!({
            "token": token,
            "tenant_id": tenant_id,
            "isbn": updated_form.isbn,
            "quantity": stock_delta,
        });
        post_json(
            "/api/admin/inventory/receive",
            &JsValue::from_str(&receive_body.to_string()),
            &stock_headers,
        )
        .await
    } else {
        let adjust_body = serde_json::json!({
            "token": token,
            "tenant_id": tenant_id,
            "isbn": updated_form.isbn,
            "delta": stock_delta,
            "reason": "intake_update",
        });
        post_json(
            "/api/admin/inventory/adjust",
            &JsValue::from_str(&adjust_body.to_string()),
            &stock_headers,
        )
        .await
    };

    match stock_result {
        Ok((true, json)) => {
            let on_hand = js_f64(&json, "on_hand") as i64;
            updated_form.current_on_hand = on_hand;
            updated_form.initial_stock_input = on_hand.to_string();
            success_message.push_str(&format!(" Stock updated to {on_hand}."));
        }
        Ok((false, json)) => {
            let message = js_str(&json, "message");
            let error = if message.is_empty() { "unknown error" } else { &message };
            success_message.push_str(&format!(" Stock update failed: {error}."));
        }
        Err(error) => {
            success_message.push_str(&format!(" Stock update failed: {error}."));
        }
    }

    Ok(SaveOutcome { form: updated_form, message: success_message })
}

async fn upload_cover_request(token: String, tenant_id: String, file: web_sys::File) -> Result<UploadOutcome, String> {
    let form_data = web_sys::FormData::new().map_err(|_| "Failed to prepare upload.".to_string())?;
    let _ = form_data.append_with_str("token", &token);
    let _ = form_data.append_with_str("tenant_id", &tenant_id);
    let _ = form_data.append_with_blob("file", &file);

    let opts = web_sys::RequestInit::new();
    opts.set_method("POST");
    opts.set_body(&form_data.into());

    let request = web_sys::Request::new_with_str_and_init("/api/admin/products/cover-upload", &opts)
        .map_err(|e| format!("{e:?}"))?;
    let resp_value = wasm_bindgen_futures::JsFuture::from(
        window()
            .ok_or_else(|| "no window".to_string())?
            .fetch_with_request(&request),
    )
    .await
    .map_err(|e| format!("{e:?}"))?;
    let resp: web_sys::Response = resp_value.dyn_into().map_err(|e| format!("{e:?}"))?;
    let json = match resp.json() {
        Ok(promise) => wasm_bindgen_futures::JsFuture::from(promise).await.unwrap_or(JsValue::NULL),
        Err(_) => JsValue::NULL,
    };
    if !resp.ok() {
        let message = js_str(&json, "message");
        return Err(if message.is_empty() { "Cover upload failed.".to_string() } else { message });
    }

    Ok(UploadOutcome {
        object_key: js_str(&json, "object_key"),
        asset_url: js_str(&json, "asset_url"),
    })
}

fn schedule_reset(form: RwSignal<FormState>, step: RwSignal<i32>, lookup_status: RwSignal<StatusMessage>, cover_preview_url: RwSignal<Option<String>>, cover_loaded: RwSignal<bool>) {
    let closure = Closure::wrap(Box::new(move || {
        form.set(FormState::fresh());
        step.set(0);
        lookup_status.set(StatusMessage::new("Lookup and save status will appear here.", ""));
        cover_preview_url.set(None);
        cover_loaded.set(false);
        set_scanner_status("Scan a barcode or type an ISBN to begin.", "");
    }) as Box<dyn Fn()>);
    if let Some(window) = window() {
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            2500,
        );
    }
    closure.forget();
}

#[component]
fn IntakeApp(config: RootConfig) -> impl IntoView {
    let form = RwSignal::new(FormState::fresh());
    let step = RwSignal::new(0);
    let lookup_status =
        RwSignal::new(StatusMessage::new("Lookup and save status will appear here.", ""));
    let success_message = RwSignal::new("Resetting for next item...".to_string());
    let categories = RwSignal::new(vec!["Books".to_string()]);
    let vendors = RwSignal::new(vec!["Church Supplier".to_string()]);
    let cover_preview_url = RwSignal::new(None::<String>);
    let cover_loaded = RwSignal::new(false);
    let lookup_action = Action::new_local({
        let token = config.token.clone();
        move |isbn: &String| {
            let token = token.clone();
            let isbn = isbn.clone();
            async move { lookup_isbn_request(token, isbn).await }
        }
    });
    let save_action = Action::new_local({
        let token = config.token.clone();
        let tenant_id = config.tenant_id.clone();
        move |state: &FormState| {
            let token = token.clone();
            let tenant_id = tenant_id.clone();
            let state = state.clone();
            async move { save_product_request(token, tenant_id, state).await }
        }
    });
    let lookup_token = config.token.clone();
    let upload_token = config.token.clone();
    let upload_tenant = config.tenant_id.clone();

    {
        let form = form;
        let step = step;
        let lookup_action = lookup_action.clone();
        let lookup_status = lookup_status;
        SCAN_CALLBACK.with(|slot| {
            *slot.borrow_mut() = Some(Box::new(move |raw| {
                let isbn = normalize_isbn(&raw);
                form.update(|state| state.isbn = isbn.clone());
                step.set(step.get().max(0));
                lookup_status
                    .set(StatusMessage::new("Fetching metadata...", "warning"));
                set_scanner_status(&format!("Detected ISBN {isbn}. Fetching metadata..."), "success");
                lookup_action.dispatch(isbn);
            }));
        });
    }

    {
        RERENDER_CALLBACK.with(|slot| {
            *slot.borrow_mut() = Some(Box::new(move || {
                scanner::sync_camera_view(SCANNER);
            }));
        });
    }

    {
        let form = form;
        let step = step;
        let lookup_status = lookup_status;
        let cover_preview_url = cover_preview_url;
        let cover_loaded = cover_loaded;
        let categories = categories;
        let vendors = vendors;
        let token = config.token.clone();
        let tenant_id = config.tenant_id.clone();
        let product_id = config.product_id.clone();
        leptos::task::spawn_local(async move {
            if !product_id.is_empty() {
                lookup_status.set(StatusMessage::new("Loading product details...", "warning"));
                match load_existing_product(token.clone(), tenant_id.clone(), product_id).await {
                    Ok((loaded_form, cover_url, loaded_flag)) => {
                        let category = loaded_form.category.clone();
                        let vendor = loaded_form.vendor.clone();
                        form.set(loaded_form);
                        cover_preview_url.set(cover_url);
                        cover_loaded.set(loaded_flag);
                        step.set(1);
                        lookup_status.set(StatusMessage::new(
                            "Editing existing product. Save updates details only; adjust stock in Inventory.",
                            "success",
                        ));
                        set_scanner_status("Product loaded for editing.", "success");
                        let (category_values, vendor_values) =
                            load_taxonomies(token, tenant_id, category, vendor).await;
                        categories.set(category_values);
                        vendors.set(vendor_values);
                    }
                    Err(message) => {
                        lookup_status.set(StatusMessage::new(message, "danger"));
                    }
                }
            } else {
                let current = form.get();
                let (category_values, vendor_values) =
                    load_taxonomies(token, tenant_id, current.category, current.vendor).await;
                categories.set(category_values);
                vendors.set(vendor_values);
            }
        });
    }

    Effect::new({
        let form = form;
        let step = step;
        let lookup_status = lookup_status;
        let cover_preview_url = cover_preview_url;
        let cover_loaded = cover_loaded;
        move |_| {
            if let Some(result) = lookup_action.value().get() {
                match result {
                    Ok(outcome) => {
                        let current = form.get();
                        let merged = FormState {
                            reorder_point_input: current.reorder_point_input,
                            cost_input: if outcome.form.cost_input.is_empty() {
                                current.cost_input
                            } else {
                                outcome.form.cost_input.clone()
                            },
                            retail_input: if outcome.form.retail_input.is_empty() {
                                current.retail_input
                            } else {
                                outcome.form.retail_input.clone()
                            },
                            initial_stock_input: outcome.form.initial_stock_input.clone(),
                            ..outcome.form.clone()
                        };
                        form.set(merged);
                        step.set(outcome.step);
                        lookup_status.set(outcome.lookup_status);
                        cover_preview_url.set(outcome.cover_preview_url);
                        cover_loaded.set(outcome.cover_loaded);
                        set_scanner_status(&outcome.scanner_message, &outcome.scanner_tone);
                    }
                    Err(message) => {
                        lookup_status.set(StatusMessage::new(message, "danger"));
                        set_scanner_status("Lookup failed. Check the ISBN and try again.", "warning");
                    }
                }
            }
        }
    });

    Effect::new({
        let form = form;
        let step = step;
        let lookup_status = lookup_status;
        let success_message = success_message;
        let cover_preview_url = cover_preview_url;
        let cover_loaded = cover_loaded;
        move |_| {
            if let Some(result) = save_action.value().get() {
                match result {
                    Ok(outcome) => {
                        form.set(outcome.form);
                        success_message.set(outcome.message.clone());
                        lookup_status.set(StatusMessage::new(outcome.message.clone(), "success"));
                        step.set(2);
                        schedule_reset(form, step, lookup_status, cover_preview_url, cover_loaded);
                    }
                    Err(message) => {
                        lookup_status.set(StatusMessage::new(message, "danger"));
                    }
                }
            }
        }
    });

    Effect::new(move |_| {
        if let Some(window) = window() {
            let _ = js_sys::Reflect::set(
                &window,
                &JsValue::from_str("__SCRIPTORIUM_INTAKE_READY"),
                &JsValue::TRUE,
            );
        }
    });

    let install_debug_shortcut = Closure::wrap(Box::new(move |event: web_sys::Event| {
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
    if let Some(window) = window() {
        let _ = window.add_event_listener_with_callback(
            "keydown",
            install_debug_shortcut.as_ref().unchecked_ref(),
        );
    }
    install_debug_shortcut.forget();

    scanner::install_beforeunload_stop(SCANNER);

    view! {
        <main class="intake-main">
            <div class="intake-header">
                <div>
                    <h1>Add New Product</h1>
                    <p>Scan or type an ISBN, review the metadata, then save a shelf-ready product record.</p>
                </div>
                <div class="intake-steps" aria-label="Intake steps">
                    <div
                        class=move || {
                            let current = step.get();
                            format!(
                                "intake-step{}{}",
                                if current == 0 { " is-active" } else { "" },
                                if current > 0 { " is-done" } else { "" }
                            )
                        }
                        data-step="0"
                    >
                        <span class="intake-step-badge">{move || if step.get() > 0 { "\u{2713}".to_string() } else { "1".to_string() }}</span>
                        <span>scan</span>
                    </div>
                    <div
                        class=move || {
                            format!(
                                "intake-step-connector{}",
                                if step.get() > 0 { " is-done" } else { "" }
                            )
                        }
                        data-step-connector="0"
                    ></div>
                    <div
                        class=move || {
                            let current = step.get();
                            format!(
                                "intake-step{}{}",
                                if current == 1 { " is-active" } else { "" },
                                if current > 1 { " is-done" } else { "" }
                            )
                        }
                        data-step="1"
                    >
                        <span class="intake-step-badge">{move || if step.get() > 1 { "\u{2713}".to_string() } else { "2".to_string() }}</span>
                        <span>review</span>
                    </div>
                    <div
                        class=move || {
                            format!(
                                "intake-step-connector{}",
                                if step.get() > 1 { " is-done" } else { "" }
                            )
                        }
                        data-step-connector="1"
                    ></div>
                    <div
                        class=move || {
                            format!(
                                "intake-step{}",
                                if step.get() == 2 { " is-active" } else { "" }
                            )
                        }
                        data-step="2"
                    >
                        <span class="intake-step-badge">3</span>
                        <span>save</span>
                    </div>
                </div>
            </div>
            <section class="intake-card">
                <div class="intake-card-head">
                    <h2>ISBN & Cover</h2>
                    <button
                        type="button"
                        class="intake-reset"
                        id="intake-reset"
                        hidden=move || step.get() == 0
                        on:click=move |_| {
                            form.set(FormState::fresh());
                            step.set(0);
                            cover_preview_url.set(None);
                            cover_loaded.set(false);
                            lookup_status.set(StatusMessage::new("Lookup and save status will appear here.", ""));
                            set_scanner_status("Scan a barcode or type an ISBN to begin.", "");
                        }
                    >
                        Start over
                    </button>
                </div>
                <div class="intake-scanner-layout">
                    <div class="intake-camera-panel">
                        <video id="camera" autoplay=true playsinline=true></video>
                        <div id="camera-overlay" class="intake-camera-overlay" hidden=true>
                            <div class="intake-scan-frame"><div class="intake-scan-line"></div></div>
                            <span style="font-size:13px;color:#fff;opacity:0.72;">Hold barcode steady</span>
                        </div>
                        <div id="camera-empty" class="intake-camera-empty">
                            <svg width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="#666" stroke-width="1.5">
                                <rect x="2" y="4" width="20" height="16" rx="2"></rect>
                                <circle cx="12" cy="12" r="3"></circle>
                                <path d="M17 4h2a2 2 0 0 1 2 2v2M7 4H5a2 2 0 0 0-2 2v2M17 20h2a2 2 0 0 0 2-2v-2M7 20H5a2 2 0 0 1-2-2v-2"></path>
                            </svg>
                            <div style="font-size:13px;margin-top:8px;">Camera off</div>
                        </div>
                    </div>
                    <div class="intake-lookup-panel">
                        <div>
                            <label class="field-label" for="isbn">ISBN</label>
                            <div class="intake-lookup-row">
                                <input
                                    class="intake-isbn"
                                    id="isbn"
                                    name="isbn"
                                    placeholder="978..."
                                    inputmode="numeric"
                                    prop:value=move || form.get().isbn
                                    on:input=move |ev| {
                                        let value = normalize_isbn(&event_target_value(&ev));
                                        form.update(|state| state.isbn = value.clone());
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
                                    }
                                />
                                <button
                                    class="accent-button"
                                    type="button"
                                    id="lookup"
                                    on:click=move |_| {
                                        let isbn = form.get().isbn;
                                        if lookup_token.is_empty() {
                                            lookup_status.set(StatusMessage::new("Admin session missing. Sign in again.", "danger"));
                                            return;
                                        }
                                        if isbn.is_empty() {
                                            lookup_status.set(StatusMessage::new("Enter or scan an ISBN before fetching metadata.", "warning"));
                                            return;
                                        }
                                        lookup_status.set(StatusMessage::new("Fetching metadata...", "warning"));
                                        set_scanner_status("Retrieving metadata...", "busy");
                                        lookup_action.dispatch(isbn);
                                    }
                                >
                                    Fetch
                                </button>
                            </div>
                        </div>
                        <div class="intake-inline-actions">
                            <button
                                class="primary-button"
                                type="button"
                                id="camera-start"
                                on:click=move |_| {
                                    if scanner::camera_stream_present(SCANNER) {
                                        stop_camera();
                                    } else {
                                        leptos::task::spawn_local(async move {
                                            scanner::boot_camera(SCANNER, scanner_rerender_bridge, scanner_detect_bridge).await;
                                        });
                                    }
                                }
                            >
                                Start scanner
                            </button>
                            <button class="primary-button" type="button" id="camera-stop" hidden=true>
                                Stop scanner
                            </button>
                        </div>
                        <div id="scanner-status" class="intake-status-copy" aria-live="polite">
                            Scan a barcode or type an ISBN to begin.
                        </div>
                        <div id="scanner-debug-panel" class="intake-debug-panel" hidden=true>
                            <canvas id="scanner-debug-canvas" width="640" height="360"></canvas>
                            <div id="scanner-debug-meta" class="intake-debug-meta">Debug mode is off.</div>
                        </div>
                        <div id="intake-auth-status" class="notice-panel notice-panel--success" aria-live="polite">
                            Signed in. Metadata lookup and product save are ready.
                        </div>
                        <div id="intake-lookup-status" class=move || lookup_status.get().class_name() aria-live="polite">
                            {move || lookup_status.get().message}
                        </div>
                    </div>
                </div>
            </section>
            <section
                id="intake-review"
                class=move || {
                    format!(
                        "intake-card intake-review{}",
                        if step.get() >= 1 { " is-visible" } else { "" }
                    )
                }
            >
                <div class="intake-card-head">
                    <h2>Product details</h2>
                    <div style="display:flex;align-items:center;gap:0.6rem;">
                        <span class="intake-category-badge" id="intake-category-badge">
                            {move || {
                                let category = form.get().category;
                                if category.is_empty() { "BOOKS".to_string() } else { category.to_uppercase() }
                            }}
                        </span>
                        <button type="button" class="intake-menu-btn" aria-label="More options">&middot;&middot;&middot;</button>
                    </div>
                </div>
                <div class="intake-review-layout">
                    <div class="intake-cover-column">
                        <div
                            id="cover-frame"
                            class=move || {
                                format!(
                                    "intake-cover-frame{}",
                                    if cover_preview_url.get().is_some() { " has-image" } else { "" }
                                )
                            }
                        >
                            <Show when=move || cover_preview_url.get().is_some() fallback=move || view! {
                                <div id="cover-placeholder" class="intake-cover-placeholder">
                                    <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="#c4b9a8" stroke-width="1.5">
                                        <rect x="3" y="3" width="18" height="18" rx="2"></rect>
                                        <circle cx="8.5" cy="8.5" r="1.5"></circle>
                                        <path d="M21 15l-5-5L5 21"></path>
                                    </svg>
                                    <div style="margin-top:6px;">No cover</div>
                                </div>
                            }>
                                <img id="cover-preview" alt="Uploaded cover preview" src=move || cover_preview_url.get().unwrap_or_default() />
                            </Show>
                            <div id="cover-loaded" class="intake-cover-loaded" hidden=move || !cover_loaded.get()>
                                <div style="font-size:12px;text-transform:uppercase;letter-spacing:1.5px;opacity:0.72;">Cover loaded</div>
                                <strong>Cover asset</strong>
                                <span>Stored for the product record.</span>
                            </div>
                        </div>
                        <label class="intake-cover-upload">
                            Replace cover
                            <input
                                id="cover-file"
                                name="cover-file"
                                type="file"
                                accept="image/*,.svg"
                                on:change=move |_| {
                                    if let Some(file) = current_cover_file() {
                                        if let Ok(url) = web_sys::Url::create_object_url_with_blob(&file) {
                                            cover_preview_url.set(Some(url));
                                            cover_loaded.set(false);
                                        }
                                        lookup_status.set(StatusMessage::new(
                                            "Cover selected. Upload it to store with the product.",
                                            "warning",
                                        ));
                                    }
                                }
                            />
                        </label>
                        <button
                            class="ghost-link ghost-link--ink"
                            type="button"
                            id="upload-cover"
                            on:click=move |_| {
                                if upload_token.is_empty() || upload_tenant.is_empty() {
                                    lookup_status.set(StatusMessage::new(
                                        "Admin session missing. Sign in again before uploading.",
                                        "danger",
                                    ));
                                    return;
                                }
                                let Some(file) = current_cover_file() else {
                                    lookup_status.set(StatusMessage::new(
                                        "Choose an image file before uploading.",
                                        "warning",
                                    ));
                                    return;
                                };
                                lookup_status.set(StatusMessage::new("Uploading cover...", "warning"));
                                let token = upload_token.clone();
                                let tenant_id = upload_tenant.clone();
                                let form = form;
                                let lookup_status = lookup_status;
                                let cover_preview_url = cover_preview_url;
                                let cover_loaded = cover_loaded;
                                leptos::task::spawn_local(async move {
                                    match upload_cover_request(token, tenant_id, file).await {
                                        Ok(outcome) => {
                                            form.update(|state| state.cover_image_key = Some(outcome.object_key));
                                            if !outcome.asset_url.is_empty() {
                                                cover_preview_url.set(Some(outcome.asset_url));
                                            }
                                            cover_loaded.set(true);
                                            lookup_status.set(StatusMessage::new(
                                                "Cover uploaded and ready to save with the product record.",
                                                "success",
                                            ));
                                        }
                                        Err(message) => {
                                            lookup_status.set(StatusMessage::new(message, "danger"));
                                        }
                                    }
                                });
                            }
                        >
                            Attach file
                        </button>
                    </div>
                    <form class="intake-form-stack" on:submit=move |ev| ev.prevent_default()>
                        <div class="intake-meta-stack">
                            <p class="intake-section-label">Bibliographic</p>
                            <div class="intake-field">
                                <label class="field-label" for="title">Title</label>
                                <input id="title" name="title" placeholder="Book title" prop:value=move || form.get().title on:input=move |ev| form.update(|state| state.title = event_target_value(&ev)) />
                            </div>
                            <div class="intake-meta-grid">
                                <div class="intake-field">
                                    <label class="field-label" for="author">Author</label>
                                    <input id="author" name="author" placeholder="Author name" prop:value=move || form.get().author on:input=move |ev| form.update(|state| state.author = event_target_value(&ev)) />
                                </div>
                                <div class="intake-field">
                                    <label class="field-label" for="publisher">Publisher</label>
                                    <input id="publisher" name="publisher" placeholder="Publisher" prop:value=move || form.get().publisher on:input=move |ev| form.update(|state| state.publisher = event_target_value(&ev)) />
                                </div>
                            </div>
                            <div class="intake-meta-grid">
                                <div class="intake-field">
                                    <label class="field-label" for="isbn-review">ISBN</label>
                                    <input id="isbn-review" placeholder="978-0-00-000000-0" readonly=true prop:value=move || form.get().isbn />
                                </div>
                                <div class="intake-field">
                                    <label class="field-label" for="category">Category</label>
                                    <select id="category" name="category" on:change=move |ev| form.update(|state| state.category = event_target_value(&ev))>
                                        {move || {
                                            let selected = form.get().category;
                                            categories
                                                .get()
                                                .into_iter()
                                                .map(|value| {
                                                    let selected_value = selected.clone();
                                                    view! {
                                                        <option value=value.clone() selected=value == selected_value>{value.clone()}</option>
                                                    }
                                                })
                                                .collect_view()
                                        }}
                                    </select>
                                </div>
                            </div>
                            <div class="intake-field">
                                <label class="field-label" for="description">Description</label>
                                <textarea id="description" name="description" placeholder="Description" prop:value=move || form.get().description on:input=move |ev| form.update(|state| state.description = event_target_value(&ev))></textarea>
                            </div>
                        </div>
                        <div class="intake-pricing-card">
                            <p class="intake-section-label">Pricing & Inventory</p>
                            <div class="intake-pricing-grid">
                                <div class="intake-field">
                                    <label class="field-label" for="cost-cents">Cost</label>
                                    <div class="intake-price-wrap">
                                        <input id="cost-cents" name="cost-cents" placeholder="0.00" inputmode="decimal" prop:value=move || form.get().cost_input on:input=move |ev| form.update(|state| state.cost_input = event_target_value(&ev)) />
                                    </div>
                                </div>
                                <div class="intake-field">
                                    <label class="field-label" for="retail-cents">Retail</label>
                                    <div class="intake-price-wrap">
                                        <input id="retail-cents" name="retail-cents" placeholder="0.00" inputmode="decimal" prop:value=move || form.get().retail_input on:input=move |ev| form.update(|state| state.retail_input = event_target_value(&ev)) />
                                    </div>
                                </div>
                                <div class="intake-field">
                                    <label class="field-label" for="initial-stock">Stock</label>
                                    <input id="initial-stock" name="initial-stock" inputmode="numeric" prop:value=move || form.get().initial_stock_input on:input=move |ev| form.update(|state| state.initial_stock_input = event_target_value(&ev)) />
                                </div>
                                <div class="intake-field">
                                    <label class="field-label" for="reorder-point">Reorder at</label>
                                    <input id="reorder-point" name="reorder-point" inputmode="numeric" prop:value=move || form.get().reorder_point_input on:input=move |ev| form.update(|state| state.reorder_point_input = event_target_value(&ev)) />
                                </div>
                            </div>
                            <div class="intake-field">
                                <label class="field-label" for="vendor">Vendor</label>
                                <select id="vendor" name="vendor" on:change=move |ev| form.update(|state| state.vendor = event_target_value(&ev))>
                                    {move || {
                                        let selected = form.get().vendor;
                                        vendors
                                            .get()
                                            .into_iter()
                                            .map(|value| {
                                                let selected_value = selected.clone();
                                                view! {
                                                    <option value=value.clone() selected=value == selected_value>{value.clone()}</option>
                                                }
                                            })
                                            .collect_view()
                                    }}
                                </select>
                            </div>
                        </div>
                        <div class="intake-actions">
                            <span class="intake-stock-status" id="intake-stock-status">
                                <span class="intake-stock-dot"></span>
                                <span id="intake-stock-label">
                                    {move || {
                                        let state = form.get();
                                        let stock = if state.initial_stock_input.is_empty() { "0".to_string() } else { state.initial_stock_input };
                                        let reorder = if state.reorder_point_input.is_empty() { "0".to_string() } else { state.reorder_point_input };
                                        format!("{stock} in stock \u{00B7} reorders at {reorder}")
                                    }}
                                </span>
                            </span>
                            <a class="ghost-link ghost-link--ink" href="/admin">Cancel</a>
                            <button
                                class="accent-button"
                                type="button"
                                id="save-product"
                                on:click=move |_| {
                                    let mut state = form.get();
                                    state.isbn = normalize_isbn(&state.isbn);
                                    if let Err(message) = validate_form(&state) {
                                        lookup_status.set(StatusMessage::new(message, "warning"));
                                        return;
                                    }
                                    form.set(state.clone());
                                    lookup_status.set(StatusMessage::new("Saving product...", "warning"));
                                    save_action.dispatch(state);
                                }
                            >
                                {move || form.get().save_button_label()}
                            </button>
                        </div>
                    </form>
                </div>
            </section>
            <section
                id="intake-success"
                class=move || {
                    format!(
                        "intake-success{}",
                        if step.get() == 2 { " is-visible" } else { "" }
                    )
                }
                aria-live="polite"
            >
                <div class="intake-success-mark">{"\u{2713}"}</div>
                <h2 style="margin:0 0 0.35rem;font-family:'Source Serif 4',Georgia,serif;font-size:1.45rem;">Product saved</h2>
                <p id="intake-success-copy" style="margin:0;opacity:0.84;">{move || success_message.get()}</p>
            </section>
            <section class="intake-hint" id="intake-hint" hidden=move || step.get() != 0>
                <div style="font-size:14px;font-weight:700;color:#8b2635;margin-bottom:4px;">Volunteer flow</div>
                <p style="margin:0;font-size:14px;line-height:1.5;">
                    Start the scanner and hold the book barcode in frame. The ISBN will auto-fill and metadata will fetch automatically.
                    Use <strong>Fetch</strong> for manual ISBN entry, then confirm the details, optionally upload a cover, and hit <strong>Save Product</strong>.
                </p>
            </section>
        </main>
    }
}

pub fn mount_intake_island() {
    let Some(config) = read_root_config() else {
        return;
    };
    let Some(root) = by_id("intake-root").and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok()) else {
        return;
    };

    mount_to(root, move || view! { <IntakeApp config=config.clone() /> }).forget();
    scanner::set_debug_enabled(SCANNER, false);
    set_scanner_status("Scan a barcode or type an ISBN to begin.", "");
    leptos::task::spawn_local(async move {
        scanner::boot_camera(SCANNER, scanner_rerender_bridge, scanner_detect_bridge).await;
    });
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    fn test_div() -> web_sys::HtmlElement {
        let document = web_sys::window().unwrap().document().unwrap();
        let container = document.create_element("div").unwrap();
        document.body().unwrap().append_child(&container).unwrap();
        container.dyn_into::<web_sys::HtmlElement>().unwrap()
    }

    #[wasm_bindgen_test]
    fn intake_component_mounts_required_controls() {
        let root = test_div();
        mount_to(root.clone(), move || view! { <IntakeApp config=RootConfig::default() /> }).forget();
        assert!(root.query_selector("#isbn").unwrap().is_some());
        assert!(root.query_selector("#lookup").unwrap().is_some());
        assert!(root.query_selector("#save-product").unwrap().is_some());
    }

    #[wasm_bindgen_test]
    fn validation_rejects_non_digit_isbn() {
        let mut form = FormState::fresh();
        form.title = "Test Title".to_string();
        form.isbn = "abc".to_string();
        let message = validate_form(&form).unwrap_err();
        assert_eq!(message, "ISBN must be 10 or 13 digits.");
    }
}
