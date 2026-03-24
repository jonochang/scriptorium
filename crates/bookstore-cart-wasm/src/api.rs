use wasm_bindgen::prelude::*;

fn window() -> Result<web_sys::Window, String> {
    web_sys::window().ok_or_else(|| "no window".to_string())
}

pub fn json_headers() -> Result<web_sys::Headers, String> {
    let headers = web_sys::Headers::new().map_err(|e| format!("{e:?}"))?;
    headers.set("content-type", "application/json").map_err(|e| format!("{e:?}"))?;
    Ok(headers)
}

pub fn json_headers_with_origin() -> Result<web_sys::Headers, String> {
    let headers = json_headers()?;
    let origin = window()?.location().origin().map_err(|e| format!("{e:?}"))?;
    headers.set("Origin", &origin).map_err(|e| format!("{e:?}"))?;
    Ok(headers)
}

pub async fn get_json(url: &str, bearer_token: Option<&str>) -> Result<JsValue, String> {
    let opts = web_sys::RequestInit::new();
    opts.set_method("GET");
    let headers = web_sys::Headers::new().map_err(|e| format!("{e:?}"))?;
    if let Some(token) = bearer_token {
        headers
            .set("Authorization", &format!("Bearer {token}"))
            .map_err(|e| format!("{e:?}"))?;
    }
    opts.set_headers(&headers);

    let request =
        web_sys::Request::new_with_str_and_init(url, &opts).map_err(|e| format!("{e:?}"))?;
    let resp_value = wasm_bindgen_futures::JsFuture::from(window()?.fetch_with_request(&request))
        .await
        .map_err(|e| format!("{e:?}"))?;
    let resp: web_sys::Response = resp_value.dyn_into().map_err(|e| format!("{e:?}"))?;
    let json = match resp.json() {
        Ok(promise) => wasm_bindgen_futures::JsFuture::from(promise).await.unwrap_or(JsValue::NULL),
        Err(_) => JsValue::NULL,
    };
    if !resp.ok() {
        let message = js_field(&json, "message");
        if message.is_empty() {
            return Err(format!("Request failed for {url}"));
        }
        return Err(message);
    }
    Ok(json)
}

pub async fn post_json(
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
    let resp_value = wasm_bindgen_futures::JsFuture::from(window()?.fetch_with_request(&request))
        .await
        .map_err(|e| format!("{e:?}"))?;
    let resp: web_sys::Response = resp_value.dyn_into().map_err(|e| format!("{e:?}"))?;
    let ok = resp.ok();
    let json = match resp.json() {
        Ok(promise) => wasm_bindgen_futures::JsFuture::from(promise).await.unwrap_or(JsValue::NULL),
        Err(_) => JsValue::NULL,
    };
    Ok((ok, json))
}

pub fn js_field(obj: &JsValue, key: &str) -> String {
    js_sys::Reflect::get(obj, &JsValue::from_str(key))
        .ok()
        .and_then(|value| value.as_string())
        .unwrap_or_default()
}

pub fn js_to_json(value: &JsValue) -> serde_json::Value {
    let json_str = js_sys::JSON::stringify(value)
        .map(|value| value.as_string().unwrap_or_default())
        .unwrap_or_else(|_| "{}".to_string());
    serde_json::from_str(&json_str).unwrap_or(serde_json::Value::Object(Default::default()))
}
