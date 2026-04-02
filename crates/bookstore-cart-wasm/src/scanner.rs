use wasm_bindgen::prelude::*;

#[derive(Clone, Copy)]
pub(crate) struct ScannerBindings {
    pub video_id: &'static str,
    pub overlay_id: &'static str,
    pub empty_id: &'static str,
    pub start_button_id: &'static str,
    pub stop_button_id: &'static str,
    pub status_id: &'static str,
    pub idle_start_label: &'static str,
    pub active_start_label: &'static str,
    pub scan_timer_key: &'static str,
    pub last_scan_key: &'static str,
    pub last_scan_at_key: &'static str,
    pub camera_stream_key: &'static str,
    pub detector_key: &'static str,
    pub status_message_key: Option<&'static str>,
    pub status_tone_key: Option<&'static str>,
    pub status_class: fn(&str) -> String,
    pub debug_toggle_id: Option<&'static str>,
    pub debug_panel_id: Option<&'static str>,
    pub debug_canvas_id: Option<&'static str>,
    pub debug_meta_id: Option<&'static str>,
    pub debug_enabled_key: Option<&'static str>,
}

fn by_id(id: &str) -> Option<web_sys::Element> {
    web_sys::window()?.document()?.get_element_by_id(id)
}

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

pub(crate) fn stored_status(bindings: ScannerBindings) -> String {
    bindings.status_message_key.map(win_get_str).unwrap_or_default()
}

pub(crate) fn debug_enabled(bindings: ScannerBindings) -> bool {
    bindings
        .debug_enabled_key
        .map(|key| {
            web_sys::window()
                .and_then(|w| js_sys::Reflect::get(&w, &JsValue::from_str(key)).ok())
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
        })
        .unwrap_or(false)
}

pub(crate) fn set_debug_enabled(bindings: ScannerBindings, enabled: bool) {
    if let Some(key) = bindings.debug_enabled_key {
        if let Some(w) = web_sys::window() {
            let _ = js_sys::Reflect::set(&w, &JsValue::from_str(key), &JsValue::from(enabled));
        }
    }
    sync_debug_panel(bindings);
}

pub(crate) fn sync_debug_panel(bindings: ScannerBindings) {
    let enabled = debug_enabled(bindings);
    if let Some(id) = bindings.debug_toggle_id {
        if let Some(input) = by_id(id).and_then(|e| e.dyn_into::<web_sys::HtmlInputElement>().ok())
        {
            input.set_checked(enabled);
        }
    }
    if let Some(id) = bindings.debug_panel_id {
        if let Some(panel) = by_id(id).and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok()) {
            panel.set_hidden(!enabled);
        }
    }
}

pub(crate) fn status_class(bindings: ScannerBindings) -> String {
    let tone = bindings.status_tone_key.map(win_get_str).unwrap_or_default();
    (bindings.status_class)(&tone)
}

pub(crate) fn set_scanner_status(bindings: ScannerBindings, message: &str, tone: &str) {
    if let Some(key) = bindings.status_message_key {
        win_set_str(key, message);
    }
    if let Some(key) = bindings.status_tone_key {
        win_set_str(key, tone);
    }

    if let Some(panel) = by_id(bindings.status_id) {
        panel.set_text_content(Some(message));
        panel.set_class_name(&(bindings.status_class)(tone));
    }
}

pub(crate) fn camera_stream_present(bindings: ScannerBindings) -> bool {
    let stream = win_get(bindings.camera_stream_key);
    !stream.is_null() && !stream.is_undefined()
}

pub(crate) fn set_camera_state(bindings: ScannerBindings, active: bool) {
    if let Some(el) =
        by_id(bindings.overlay_id).and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        el.set_hidden(!active);
    }
    if let Some(el) =
        by_id(bindings.empty_id).and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        el.set_hidden(active);
    }
    if let Some(el) =
        by_id(bindings.stop_button_id).and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        el.set_hidden(!active);
    }
    if let Some(btn) =
        by_id(bindings.start_button_id).and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let _ = js_sys::Reflect::set(&btn, &JsValue::from_str("disabled"), &JsValue::FALSE);
        btn.set_hidden(false);
        btn.set_text_content(Some(if active {
            bindings.active_start_label
        } else {
            bindings.idle_start_label
        }));
    }
    sync_debug_panel(bindings);
}

pub(crate) fn sync_camera_view(bindings: ScannerBindings) {
    let stream = win_get(bindings.camera_stream_key);
    let active = !stream.is_null() && !stream.is_undefined();
    if active {
        if let Some(video) = by_id(bindings.video_id) {
            let _ = js_sys::Reflect::set(&video, &JsValue::from_str("srcObject"), &stream);
            if let Ok(media_el) = video.dyn_into::<web_sys::HtmlMediaElement>() {
                let _ = media_el.play();
            }
        }
    }
    set_camera_state(bindings, active);
}

fn debug_meta(bindings: ScannerBindings, message: &str) {
    if !debug_enabled(bindings) {
        return;
    }
    if let Some(id) = bindings.debug_meta_id {
        if let Some(meta) = by_id(id) {
            meta.set_text_content(Some(message));
        }
    }
}

fn draw_debug_frame(
    bindings: ScannerBindings,
    video: &web_sys::Element,
    detections: Option<&js_sys::Array>,
    note: &str,
) {
    if !debug_enabled(bindings) {
        return;
    }
    let canvas_id = match bindings.debug_canvas_id {
        Some(id) => id,
        None => return,
    };
    let canvas =
        match by_id(canvas_id).and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok()) {
            Some(canvas) => canvas,
            None => return,
        };
    let width = js_sys::Reflect::get(video, &JsValue::from_str("videoWidth"))
        .ok()
        .and_then(|v| v.as_f64())
        .filter(|v| *v > 0.0)
        .unwrap_or(640.0) as u32;
    let height = js_sys::Reflect::get(video, &JsValue::from_str("videoHeight"))
        .ok()
        .and_then(|v| v.as_f64())
        .filter(|v| *v > 0.0)
        .unwrap_or(360.0) as u32;
    canvas.set_width(width);
    canvas.set_height(height);
    let ctx = match canvas
        .get_context("2d")
        .ok()
        .flatten()
        .and_then(|v| v.dyn_into::<web_sys::CanvasRenderingContext2d>().ok())
    {
        Some(ctx) => ctx,
        None => return,
    };
    if let Ok(video) = video.clone().dyn_into::<web_sys::HtmlVideoElement>() {
        let _ = ctx.draw_image_with_html_video_element_and_dw_and_dh(
            &video,
            0.0,
            0.0,
            width as f64,
            height as f64,
        );
    }
    let mut lines = vec![note.to_string(), format!("frame: {}x{}", width, height)];
    if let Some(detections) = detections {
        ctx.set_stroke_style_str("#22c55e");
        ctx.set_line_width(3.0);
        for i in 0..detections.length() {
            let detection = detections.get(i);
            let raw = js_sys::Reflect::get(&detection, &JsValue::from_str("rawValue"))
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_default();
            let format = js_sys::Reflect::get(&detection, &JsValue::from_str("format"))
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_default();
            if let Ok(box_value) =
                js_sys::Reflect::get(&detection, &JsValue::from_str("boundingBox"))
            {
                let x = js_sys::Reflect::get(&box_value, &JsValue::from_str("x"))
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let y = js_sys::Reflect::get(&box_value, &JsValue::from_str("y"))
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let w = js_sys::Reflect::get(&box_value, &JsValue::from_str("width"))
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let h = js_sys::Reflect::get(&box_value, &JsValue::from_str("height"))
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                if w > 0.0 && h > 0.0 {
                    ctx.stroke_rect(x, y, w, h);
                    ctx.set_fill_style_str("rgba(34,197,94,.18)");
                    ctx.fill_rect(x, y, w, h);
                }
                lines.push(format!(
                    "detected[{i}]: raw={raw} format={format} box=({:.0},{:.0},{:.0},{:.0})",
                    x, y, w, h
                ));
            } else {
                lines.push(format!("detected[{i}]: raw={raw} format={format} box=n/a"));
            }
        }
    } else {
        lines.push("detections: none".to_string());
    }
    debug_meta(bindings, &lines.join("\n"));
}

pub(crate) fn teardown_camera(bindings: ScannerBindings) {
    let timer_id = win_get_f64(bindings.scan_timer_key) as i32;
    if timer_id != 0 {
        if let Some(w) = web_sys::window() {
            w.clear_interval_with_handle(timer_id);
        }
        win_set_f64(bindings.scan_timer_key, 0.0);
    }

    let stream = win_get(bindings.camera_stream_key);
    if !stream.is_undefined() && !stream.is_null() {
        if let Ok(media_stream) = stream.dyn_into::<web_sys::MediaStream>() {
            let tracks = media_stream.get_tracks();
            for i in 0..tracks.length() {
                if let Ok(track) = tracks.get(i).dyn_into::<web_sys::MediaStreamTrack>() {
                    track.stop();
                }
            }
        }
        win_set(bindings.camera_stream_key, &JsValue::NULL);
    }

    if let Some(video) = by_id(bindings.video_id) {
        let _ = js_sys::Reflect::set(&video, &JsValue::from_str("srcObject"), &JsValue::NULL);
    }

    set_camera_state(bindings, false);
}

async fn ensure_detector(bindings: ScannerBindings) -> JsValue {
    let existing = win_get(bindings.detector_key);
    if !existing.is_undefined() && !existing.is_null() {
        return existing;
    }

    let window = match web_sys::window() {
        Some(w) => w,
        None => return JsValue::NULL,
    };

    let barcode_detector =
        match js_sys::Reflect::get(&window, &JsValue::from_str("BarcodeDetector")).ok() {
            Some(value) if !value.is_undefined() && !value.is_null() => value,
            _ => return JsValue::NULL,
        };

    let preferred =
        ["ean_13", "ean_8", "upc_a", "upc_e", "code_128", "code_39", "codabar", "itf", "pdf417"];
    let mut active_formats: Vec<&str> = Vec::new();

    if let Ok(get_fn) =
        js_sys::Reflect::get(&barcode_detector, &JsValue::from_str("getSupportedFormats"))
    {
        if let Ok(func) = get_fn.dyn_into::<js_sys::Function>() {
            if let Ok(promise) = func.call0(&barcode_detector) {
                if let Ok(result) =
                    wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(promise)).await
                {
                    let arr = js_sys::Array::from(&result);
                    let supported: Vec<String> =
                        (0..arr.length()).filter_map(|i| arr.get(i).as_string()).collect();
                    if !supported.is_empty() {
                        active_formats = preferred
                            .iter()
                            .filter(|format| {
                                supported.iter().any(|supported| supported == **format)
                            })
                            .copied()
                            .collect();
                    }
                }
            }
        }
    }

    if let Ok(constructor) = barcode_detector.dyn_into::<js_sys::Function>() {
        let detector = if active_formats.is_empty() {
            js_sys::Reflect::construct(&constructor, &js_sys::Array::new())
        } else {
            let formats_arr = js_sys::Array::new();
            for format in &active_formats {
                formats_arr.push(&JsValue::from_str(format));
            }
            let options = js_sys::Object::new();
            let _ =
                js_sys::Reflect::set(&options, &JsValue::from_str("formats"), &formats_arr.into());
            let args = js_sys::Array::new();
            args.push(&options.into());
            js_sys::Reflect::construct(&constructor, &args)
        };

        if let Ok(detector) = detector {
            win_set(bindings.detector_key, &detector);
            return detector;
        }
    }

    JsValue::NULL
}

pub(crate) async fn boot_camera(bindings: ScannerBindings, rerender: fn(), on_detect: fn(String)) {
    let window = match web_sys::window() {
        Some(window) => window,
        None => return,
    };

    let media_devices = match window.navigator().media_devices().ok() {
        Some(media_devices) => media_devices,
        None => {
            set_scanner_status(
                bindings,
                "Camera access is not available in this browser. Enter the ISBN manually.",
                "warning",
            );
            rerender();
            return;
        }
    };

    let constraints = web_sys::MediaStreamConstraints::new();
    let video_obj = js_sys::Object::new();
    let facing_obj = js_sys::Object::new();
    let _ = js_sys::Reflect::set(
        &facing_obj,
        &JsValue::from_str("ideal"),
        &JsValue::from_str("environment"),
    );
    let _ = js_sys::Reflect::set(&video_obj, &JsValue::from_str("facingMode"), &facing_obj.into());
    constraints.set_video(&video_obj.into());

    let stream_promise = match media_devices.get_user_media_with_constraints(&constraints) {
        Ok(promise) => promise,
        Err(_) => {
            set_scanner_status(
                bindings,
                "Camera permission was denied or unavailable. Enter the ISBN manually instead.",
                "danger",
            );
            rerender();
            return;
        }
    };

    let stream_js = match wasm_bindgen_futures::JsFuture::from(stream_promise).await {
        Ok(stream) => stream,
        Err(_) => {
            set_scanner_status(
                bindings,
                "Camera permission was denied or unavailable. Enter the ISBN manually instead.",
                "danger",
            );
            rerender();
            return;
        }
    };

    win_set(bindings.camera_stream_key, &stream_js);
    sync_camera_view(bindings);
    rerender();

    let detector = ensure_detector(bindings).await;
    if detector.is_null() || detector.is_undefined() {
        set_scanner_status(
            bindings,
            "Camera started. Barcode detection is unavailable here, so type the ISBN manually.",
            "warning",
        );
        sync_camera_view(bindings);
        rerender();
        return;
    }

    set_scanner_status(bindings, "Scanner live. Hold the ISBN barcode steady in frame.", "");
    sync_camera_view(bindings);
    rerender();

    let closure = Closure::wrap(Box::new(move || {
        wasm_bindgen_futures::spawn_local(scan_frame(bindings, on_detect));
    }) as Box<dyn Fn()>);

    if let Some(window) = web_sys::window() {
        if let Ok(id) = window.set_interval_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            700,
        ) {
            win_set_f64(bindings.scan_timer_key, id as f64);
        }
    }
    closure.forget();
}

pub(crate) async fn scan_frame(bindings: ScannerBindings, on_detect: fn(String)) {
    let detector = win_get(bindings.detector_key);
    if detector.is_null() || detector.is_undefined() {
        return;
    }

    let video = match by_id(bindings.video_id) {
        Some(video) => video,
        None => return,
    };
    draw_debug_frame(bindings, &video, None, "scan tick");
    let ready_state = js_sys::Reflect::get(&video, &JsValue::from_str("readyState"))
        .ok()
        .and_then(|value| value.as_f64())
        .unwrap_or(0.0);
    if ready_state < 2.0 {
        debug_meta(bindings, "scan tick\nvideo not ready");
        return;
    }

    let detect_fn = match js_sys::Reflect::get(&detector, &JsValue::from_str("detect"))
        .ok()
        .and_then(|f| f.dyn_into::<js_sys::Function>().ok())
    {
        Some(function) => function,
        None => return,
    };

    let promise = match detect_fn.call1(&detector, &video) {
        Ok(promise) => promise,
        Err(_) => {
            set_scanner_status(
                bindings,
                "Camera is live, but barcode detection needs a steadier frame or better light.",
                "warning",
            );
            return;
        }
    };

    let result = match wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(promise)).await {
        Ok(result) => result,
        Err(_) => {
            set_scanner_status(
                bindings,
                "Camera is live, but barcode detection needs a steadier frame or better light.",
                "warning",
            );
            return;
        }
    };

    let detections = js_sys::Array::from(&result);
    draw_debug_frame(bindings, &video, Some(&detections), "scan result");
    let mut raw_value: Option<String> = None;
    for i in 0..detections.length() {
        let detection = detections.get(i);
        if let Some(value) = js_sys::Reflect::get(&detection, &JsValue::from_str("rawValue"))
            .ok()
            .and_then(|value| value.as_string())
        {
            let normalized = value.trim().replace(' ', "");
            if !normalized.is_empty() {
                raw_value = Some(normalized);
                break;
            }
        }
    }

    let raw = match raw_value {
        Some(raw) => raw,
        None => return,
    };

    let now = js_sys::Date::now();
    let last_scan = win_get_str(bindings.last_scan_key);
    let last_scan_at = win_get_f64(bindings.last_scan_at_key);
    if raw == last_scan && now - last_scan_at < 2000.0 {
        return;
    }

    win_set_str(bindings.last_scan_key, &raw);
    win_set_f64(bindings.last_scan_at_key, now);
    on_detect(raw);
}

pub(crate) fn install_beforeunload_stop(bindings: ScannerBindings) {
    if let Some(window) = web_sys::window() {
        let closure = Closure::wrap(Box::new(move || teardown_camera(bindings)) as Box<dyn Fn()>);
        let _ = window
            .add_event_listener_with_callback("beforeunload", closure.as_ref().unchecked_ref());
        closure.forget();
    }
}
