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

fn js_str(obj: &JsValue, key: &str) -> String {
    js_sys::Reflect::get(obj, &JsValue::from_str(key))
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_default()
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

fn win_get(key: &str) -> JsValue {
    web_sys::window()
        .and_then(|w| js_sys::Reflect::get(&w, &JsValue::from_str(key)).ok())
        .unwrap_or(JsValue::UNDEFINED)
}

fn win_set(key: &str, value: &JsValue) {
    if let Some(w) = web_sys::window() {
        let _ = js_sys::Reflect::set(&w, &JsValue::from_str(key), value);
    }
}

const INTAKE_STEP: &str = "__intakeStep";
const SCAN_TIMER: &str = "__intakeScanTimer";
const RESET_TIMER: &str = "__intakeResetTimer";
const LAST_SCAN: &str = "__intakeLastScan";
const LAST_SCAN_AT: &str = "__intakeLastScanAt";
const CAMERA_STREAM: &str = "__intakeCameraStream";
const DETECTOR: &str = "__intakeDetector";

// ---- UI functions ----

fn set_scanner_status(message: &str, tone: &str) {
    if let Some(panel) = by_id("scanner-status") {
        panel.set_text_content(Some(message));
        let class = if tone.is_empty() {
            "intake-status-copy".to_string()
        } else {
            format!("intake-status-copy is-{tone}")
        };
        panel.set_class_name(&class);
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
                    let current: i32 = el
                        .get_attribute("data-step")
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(0);
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
    if let Some(el) = by_id("intake-reset")
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        el.set_hidden(step == 0);
    }

    // Toggle hint
    if let Some(el) =
        by_id("intake-hint").and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        el.set_hidden(step != 0);
    }
}

fn set_camera_state(active: bool) {
    if let Some(el) = by_id("camera-overlay")
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        el.set_hidden(!active);
    }
    if let Some(el) =
        by_id("camera-empty").and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        el.set_hidden(active);
    }
    if let Some(el) =
        by_id("camera-stop").and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        el.set_hidden(!active);
    }
    if let Some(btn) = by_id("camera-start")
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let _ = js_sys::Reflect::set(
            &btn,
            &JsValue::from_str("disabled"),
            &JsValue::from(active),
        );
        btn.set_text_content(Some(if active {
            "Scanning..."
        } else {
            "Start scanner"
        }));
    }
}

fn set_cover_preview(url: &str, has_stored_asset: bool) {
    let preview = by_id("cover-preview");
    let frame = by_id("cover-frame");
    let placeholder =
        by_id("cover-placeholder").and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok());
    let loaded =
        by_id("cover-loaded").and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok());

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

fn reset_intake_form() {
    let timer_id = win_get_f64(RESET_TIMER) as i32;
    if timer_id != 0 {
        if let Some(w) = web_sys::window() {
            w.clear_timeout_with_handle(timer_id);
        }
        win_set_f64(RESET_TIMER, 0.0);
    }

    set_value("isbn", "");
    set_value("title", "");
    set_value("author", "");
    set_value("publisher", "");
    set_value("description", "");
    set_value("cost-cents", "900");
    set_value("retail-cents", "1699");
    set_value("initial-stock", "5");
    set_value("reorder-point", "3");
    set_value("category", "Books");
    set_value("vendor", "Church Supplier");
    set_value("cover-image-key", "");
    set_value("cover-file", "");

    set_cover_preview("", false);
    set_lookup_status("Lookup and save status will appear here.", "");
    set_scanner_status("Scan a barcode or type an ISBN to begin.", "");
    set_step(0);
}

// ---- Camera / Barcode scanning ----

async fn ensure_detector() -> JsValue {
    let existing = win_get(DETECTOR);
    if !existing.is_undefined() && !existing.is_null() {
        return existing;
    }

    let window = match web_sys::window() {
        Some(w) => w,
        None => return JsValue::NULL,
    };

    let bd_class =
        match js_sys::Reflect::get(&window, &JsValue::from_str("BarcodeDetector")).ok() {
            Some(v) if !v.is_undefined() && !v.is_null() => v,
            _ => return JsValue::NULL,
        };

    let preferred = ["ean_13", "ean_8", "upc_a", "upc_e"];
    let mut active_formats: Vec<&str> = Vec::new();

    // Try getSupportedFormats (static method on BarcodeDetector)
    if let Ok(get_fn) =
        js_sys::Reflect::get(&bd_class, &JsValue::from_str("getSupportedFormats"))
    {
        if let Ok(func) = get_fn.dyn_into::<js_sys::Function>() {
            if let Ok(promise) = func.call0(&bd_class) {
                if let Ok(result) =
                    wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(promise)).await
                {
                    let arr = js_sys::Array::from(&result);
                    let supported: Vec<String> = (0..arr.length())
                        .filter_map(|i| arr.get(i).as_string())
                        .collect();
                    if !supported.is_empty() {
                        active_formats = preferred
                            .iter()
                            .filter(|f| supported.iter().any(|s| s == **f))
                            .copied()
                            .collect();
                    }
                }
            }
        }
    }

    if active_formats.is_empty() {
        active_formats = preferred.to_vec();
    }

    // Construct new BarcodeDetector({ formats: [...] })
    let formats_arr = js_sys::Array::new();
    for f in &active_formats {
        formats_arr.push(&JsValue::from_str(f));
    }
    let opts = js_sys::Object::new();
    let _ = js_sys::Reflect::set(&opts, &JsValue::from_str("formats"), &formats_arr.into());
    let args = js_sys::Array::new();
    args.push(&opts.into());

    if let Ok(bd_func) = bd_class.dyn_into::<js_sys::Function>() {
        if let Ok(detector) = js_sys::Reflect::construct(&bd_func, &args) {
            win_set(DETECTOR, &detector);
            return detector;
        }
    }

    JsValue::NULL
}

fn stop_camera() {
    let timer_id = win_get_f64(SCAN_TIMER) as i32;
    if timer_id != 0 {
        if let Some(w) = web_sys::window() {
            w.clear_interval_with_handle(timer_id);
        }
        win_set_f64(SCAN_TIMER, 0.0);
    }

    let stream = win_get(CAMERA_STREAM);
    if !stream.is_undefined() && !stream.is_null() {
        if let Ok(ms) = stream.dyn_into::<web_sys::MediaStream>() {
            let tracks = ms.get_tracks();
            for i in 0..tracks.length() {
                if let Ok(track) = tracks.get(i).dyn_into::<web_sys::MediaStreamTrack>() {
                    track.stop();
                }
            }
        }
        win_set(CAMERA_STREAM, &JsValue::NULL);
    }

    if let Some(video) = by_id("camera") {
        let _ = js_sys::Reflect::set(&video, &JsValue::from_str("srcObject"), &JsValue::NULL);
    }

    set_camera_state(false);
    set_scanner_status(
        "Scanner stopped. Manual ISBN entry is still available.",
        "",
    );
}

async fn boot_camera() {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };

    let navigator = window.navigator();
    let media_devices = match navigator.media_devices().ok() {
        Some(md) => md,
        None => {
            set_scanner_status(
                "Camera access is not available in this browser. Enter the ISBN manually.",
                "warning",
            );
            return;
        }
    };

    // { video: { facingMode: { ideal: "environment" } } }
    let constraints = web_sys::MediaStreamConstraints::new();
    let video_obj = js_sys::Object::new();
    let facing_obj = js_sys::Object::new();
    let _ = js_sys::Reflect::set(
        &facing_obj,
        &JsValue::from_str("ideal"),
        &JsValue::from_str("environment"),
    );
    let _ = js_sys::Reflect::set(
        &video_obj,
        &JsValue::from_str("facingMode"),
        &facing_obj.into(),
    );
    constraints.set_video(&video_obj.into());

    let stream_promise = match media_devices.get_user_media_with_constraints(&constraints) {
        Ok(p) => p,
        Err(_) => {
            set_camera_state(false);
            set_scanner_status(
                "Camera permission was denied or unavailable. Enter the ISBN manually instead.",
                "danger",
            );
            return;
        }
    };

    let stream_js = match wasm_bindgen_futures::JsFuture::from(stream_promise).await {
        Ok(s) => s,
        Err(_) => {
            set_camera_state(false);
            set_scanner_status(
                "Camera permission was denied or unavailable. Enter the ISBN manually instead.",
                "danger",
            );
            return;
        }
    };

    win_set(CAMERA_STREAM, &stream_js);

    if let Some(video) = by_id("camera") {
        let _ = js_sys::Reflect::set(&video, &JsValue::from_str("srcObject"), &stream_js);
        if let Ok(media_el) = video.dyn_into::<web_sys::HtmlMediaElement>() {
            if let Ok(promise) = media_el.play() {
                let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
            }
        }
    }

    set_camera_state(true);

    let detector = ensure_detector().await;
    if detector.is_null() || detector.is_undefined() {
        set_scanner_status(
            "Camera started. Barcode detection is unavailable here, so type the ISBN manually.",
            "warning",
        );
        return;
    }

    set_scanner_status("Scanner live. Hold the ISBN barcode steady in frame.", "");

    let closure = Closure::wrap(Box::new(move || {
        wasm_bindgen_futures::spawn_local(scan_frame());
    }) as Box<dyn Fn()>);

    if let Some(w) = web_sys::window() {
        if let Ok(id) = w.set_interval_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            700,
        ) {
            win_set_f64(SCAN_TIMER, id as f64);
        }
    }
    closure.forget();
}

async fn scan_frame() {
    let detector = win_get(DETECTOR);
    if detector.is_null() || detector.is_undefined() {
        return;
    }

    let video = match by_id("camera") {
        Some(v) => v,
        None => return,
    };

    let detect_fn = match js_sys::Reflect::get(&detector, &JsValue::from_str("detect"))
        .ok()
        .and_then(|f| f.dyn_into::<js_sys::Function>().ok())
    {
        Some(f) => f,
        None => return,
    };

    let promise = match detect_fn.call1(&detector, &video) {
        Ok(p) => p,
        Err(_) => {
            set_scanner_status(
                "Camera is live, but barcode detection needs a steadier frame or better light.",
                "warning",
            );
            return;
        }
    };

    let result = match wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(promise)).await {
        Ok(r) => r,
        Err(_) => {
            set_scanner_status(
                "Camera is live, but barcode detection needs a steadier frame or better light.",
                "warning",
            );
            return;
        }
    };

    let barcodes = js_sys::Array::from(&result);
    let mut raw_value: Option<String> = None;
    for i in 0..barcodes.length() {
        let barcode = barcodes.get(i);
        if let Some(s) = js_sys::Reflect::get(&barcode, &JsValue::from_str("rawValue"))
            .ok()
            .and_then(|v| v.as_string())
        {
            if !s.is_empty() {
                raw_value = Some(s);
                break;
            }
        }
    }

    let raw = match raw_value {
        Some(v) => v,
        None => return,
    };

    // Debounce: same barcode within 2 seconds
    let now = js_sys::Date::now();
    let last_scan = win_get_str(LAST_SCAN);
    let last_scan_at = win_get_f64(LAST_SCAN_AT);
    if raw == last_scan && now - last_scan_at < 2000.0 {
        return;
    }

    win_set_str(LAST_SCAN, &raw);
    win_set_f64(LAST_SCAN_AT, now);

    set_value("isbn", &raw);
    let step = win_get_f64(INTAKE_STEP) as i32;
    set_step(step.max(0));
    set_scanner_status(
        &format!("Detected ISBN {raw}. Review and run lookup when ready."),
        "success",
    );
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
        Ok(p) => wasm_bindgen_futures::JsFuture::from(p)
            .await
            .unwrap_or(JsValue::NULL),
        Err(_) => JsValue::NULL,
    };
    Ok((ok, json))
}

fn json_headers() -> Result<web_sys::Headers, String> {
    let headers = web_sys::Headers::new().map_err(|e| format!("{e:?}"))?;
    headers
        .set("content-type", "application/json")
        .map_err(|e| format!("{e:?}"))?;
    Ok(headers)
}

fn json_headers_with_origin() -> Result<web_sys::Headers, String> {
    let headers = json_headers()?;
    let origin = web_sys::window()
        .and_then(|w| w.location().origin().ok())
        .unwrap_or_default();
    headers
        .set("Origin", &origin)
        .map_err(|e| format!("{e:?}"))?;
    Ok(headers)
}

async fn lookup_impl() {
    let isbn = get_value("isbn").trim().to_string();
    let token = get_value("token");

    if token.is_empty() {
        set_lookup_status("Admin session missing. Sign in again.", "danger");
        return;
    }
    if isbn.is_empty() {
        set_lookup_status(
            "Enter or scan an ISBN before fetching metadata.",
            "warning",
        );
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
    let result = fetch_post(
        "/api/admin/products/isbn-lookup",
        &JsValue::from_str(&body),
        &headers,
    )
    .await;

    match result {
        Err(e) => {
            set_lookup_status(&format!("Metadata lookup failed: {e}"), "danger");
            set_scanner_status("Lookup failed. Check the ISBN and try again.", "warning");
        }
        Ok((false, json)) => {
            let msg = js_str(&json, "message");
            set_lookup_status(
                if msg.is_empty() {
                    "Metadata lookup failed."
                } else {
                    &msg
                },
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

            set_value("title", &title);
            set_value("author", &author);
            set_value("publisher", &publisher);
            set_value("description", &description);

            if !cover_url.is_empty() && get_value("cover-image-key").is_empty() {
                set_cover_preview(&cover_url, false);
            }

            set_step(1);

            if !title.is_empty() {
                set_lookup_status(
                    "Found metadata and auto-filled the product form.",
                    "success",
                );
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
        set_lookup_status(
            "Admin session missing. Sign in again before uploading.",
            "danger",
        );
        return;
    }

    let file_input = match by_id("cover-file")
        .and_then(|e| e.dyn_into::<web_sys::HtmlInputElement>().ok())
    {
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
            Ok(p) => wasm_bindgen_futures::JsFuture::from(p)
                .await
                .unwrap_or(JsValue::NULL),
            Err(_) => JsValue::NULL,
        };
        Ok::<(bool, JsValue), String>((ok, json))
    }
    .await;

    match result {
        Err(e) => set_lookup_status(&format!("Cover upload failed: {e}"), "danger"),
        Ok((false, json)) => {
            let msg = js_str(&json, "message");
            set_lookup_status(
                if msg.is_empty() {
                    "Cover upload failed."
                } else {
                    &msg
                },
                "danger",
            );
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
        if v.is_empty() {
            "Books".to_string()
        } else {
            v
        }
    };
    let vendor = {
        let v = get_value("vendor").trim().to_string();
        if v.is_empty() {
            "Church Supplier".to_string()
        } else {
            v
        }
    };
    let initial_stock: i64 = get_value("initial-stock").parse().unwrap_or(0);
    let cost_cents: i64 = get_value("cost-cents").parse().unwrap_or(0);
    let retail_cents: i64 = get_value("retail-cents").parse().unwrap_or(0);
    let cover_image_key = get_value("cover-image-key");

    set_lookup_status("Saving product...", "warning");

    let product_id = if isbn.is_empty() {
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

    let result = fetch_post(
        "/api/admin/products",
        &JsValue::from_str(&body.to_string()),
        &headers,
    )
    .await;

    match result {
        Err(e) => set_lookup_status(&format!("Save failed: {e}"), "danger"),
        Ok((false, json)) => {
            let msg = js_str(&json, "message");
            set_lookup_status(
                if msg.is_empty() { "Save failed." } else { &msg },
                "danger",
            );
        }
        Ok((true, json)) => {
            let saved_title = js_str(&json, "title");
            let display_title = if saved_title.is_empty() {
                &title
            } else {
                &saved_title
            };

            let mut success_message = format!("Saved {display_title} for {category}.");

            if initial_stock <= 0 {
                success_message.push_str(" No opening stock was received.");
            } else {
                let receive_body = serde_json::json!({
                    "token": token,
                    "tenant_id": tenant_id,
                    "isbn": isbn,
                    "quantity": initial_stock,
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
                        let on_hand = js_sys::Reflect::get(
                            &rjson,
                            &JsValue::from_str("on_hand"),
                        )
                        .ok()
                        .and_then(|v| v.as_f64())
                        .map(|v| v as i64)
                        .unwrap_or(initial_stock);
                        success_message.push_str(&format!(
                            " Received opening stock, now on hand {on_hand}."
                        ));
                    }
                    Ok((false, rjson)) => {
                        let msg = js_str(&rjson, "message");
                        let err = if msg.is_empty() { "unknown error" } else { &msg };
                        success_message = format!(
                            "Saved {display_title}, but stock receive failed: {err}."
                        );
                    }
                    Err(e) => {
                        success_message = format!(
                            "Saved {display_title}, but stock receive failed: {e}."
                        );
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
    if let Some(el) = doc
        .get_element_by_id("lookup")
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
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
            wasm_bindgen_futures::spawn_local(boot_camera());
        }) as Box<dyn Fn()>);
        el.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // Camera stop
    if let Some(el) = doc
        .get_element_by_id("camera-stop")
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let closure = Closure::wrap(Box::new(|| stop_camera()) as Box<dyn Fn()>);
        el.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
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
    if let Some(el) = doc
        .get_element_by_id("isbn")
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let closure = Closure::wrap(Box::new(|| {
            let value = get_value("isbn").trim().to_string();
            if value.is_empty() {
                set_scanner_status("Scan a barcode or type an ISBN to begin.", "");
            } else if value.len() >= 10 {
                set_scanner_status(
                    &format!(
                        "\u{2713} ISBN {value} detected \u{2014} click Fetch to pull metadata."
                    ),
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
    if let Some(el) = doc
        .get_element_by_id("cover-file")
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let closure = Closure::wrap(Box::new(|| {
            let input = by_id("cover-file")
                .and_then(|e| e.dyn_into::<web_sys::HtmlInputElement>().ok());
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

    // beforeunload - stop camera
    if let Some(window) = web_sys::window() {
        let closure = Closure::wrap(Box::new(|| stop_camera()) as Box<dyn Fn()>);
        let _ = window
            .add_event_listener_with_callback("beforeunload", closure.as_ref().unchecked_ref());
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
            el.set_text_content(Some(
                "Signed in. You can fetch metadata and save a product.",
            ));
            el.set_class_name("notice-panel notice-panel--success");
        }
    }

    set_step(0);
    set_camera_state(false);
    bind_intake_controls();

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
