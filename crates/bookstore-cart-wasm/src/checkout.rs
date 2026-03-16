use crate::cart::{
    cart_total_count, checkout_state, format_card, format_expiry, format_money, read_cart,
    strip_non_digits, write_cart,
};
use wasm_bindgen::prelude::*;
use web_sys::{Document, HtmlElement, HtmlInputElement, HtmlTextAreaElement};

fn document() -> Document {
    web_sys::window().expect("window").document().expect("document")
}

fn get_input_value(id: &str) -> String {
    document()
        .get_element_by_id(id)
        .and_then(|el| el.dyn_into::<HtmlInputElement>().ok())
        .map(|input| input.value())
        .unwrap_or_default()
}

fn get_textarea_value(id: &str) -> String {
    document()
        .get_element_by_id(id)
        .and_then(|el| el.dyn_into::<HtmlTextAreaElement>().ok())
        .map(|ta| ta.value())
        .unwrap_or_default()
}

fn selected_delivery() -> String {
    let doc = document();
    if let Ok(nodes) = doc.query_selector_all("[data-delivery-option].is-selected") {
        if let Some(node) = nodes.item(0) {
            if let Some(el) = node.dyn_ref::<HtmlElement>() {
                return el
                    .get_attribute("data-delivery-option")
                    .unwrap_or_else(|| "pickup".to_string());
            }
        }
    }
    "pickup".to_string()
}

fn selected_support() -> i64 {
    let doc = document();
    if let Ok(nodes) = doc.query_selector_all("[data-support-amount].is-selected") {
        if let Some(node) = nodes.item(0) {
            if let Some(el) = node.dyn_ref::<HtmlElement>() {
                return el
                    .get_attribute("data-support-amount")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0);
            }
        }
    }
    0
}

fn card_digits() -> String {
    get_input_value("checkout-card-number")
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect()
}

fn can_continue() -> bool {
    let cart = read_cart();
    !cart.is_empty()
        && !get_input_value("checkout-name").trim().is_empty()
        && !get_input_value("checkout-email").trim().is_empty()
}

fn can_submit() -> bool {
    can_continue() && card_digits().len() >= 15
}

fn get_checkout_step() -> i32 {
    web_sys::window()
        .and_then(|w| js_sys::Reflect::get(&w, &JsValue::from_str("__checkoutStep")).ok())
        .and_then(|v| v.as_f64())
        .map(|v| v as i32)
        .unwrap_or(0)
}

fn set_checkout_step(step: i32) {
    if let Some(w) = web_sys::window() {
        let _ =
            js_sys::Reflect::set(&w, &JsValue::from_str("__checkoutStep"), &JsValue::from(step));
    }
}

fn is_placing_order() -> bool {
    web_sys::window()
        .and_then(|w| js_sys::Reflect::get(&w, &JsValue::from_str("__placingOrder")).ok())
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

fn set_placing_order(v: bool) {
    if let Some(w) = web_sys::window() {
        let _ =
            js_sys::Reflect::set(&w, &JsValue::from_str("__placingOrder"), &JsValue::from(v));
    }
}

fn set_status(message: &str, variant: &str) {
    if let Some(panel) = document().get_element_by_id("checkout-status") {
        panel.set_text_content(Some(message));
        let class = if variant.is_empty() {
            "notice-panel".to_string()
        } else {
            format!("notice-panel notice-panel--{variant}")
        };
        panel.set_class_name(&class);
    }
}

fn set_delivery(delivery: &str) {
    let doc = document();
    if let Ok(nodes) = doc.query_selector_all("[data-delivery-option]") {
        for i in 0..nodes.length() {
            if let Some(node) = nodes.item(i) {
                if let Some(el) = node.dyn_ref::<HtmlElement>() {
                    let active = el
                        .get_attribute("data-delivery-option")
                        .map(|v| v == delivery)
                        .unwrap_or(false);
                    let _ = el
                        .class_list()
                        .toggle_with_force("is-selected", active);
                    el.set_attribute("aria-pressed", if active { "true" } else { "false" })
                        .ok();
                }
            }
        }
    }
    if let Some(addr) = doc
        .get_element_by_id("checkout-address-section")
        .and_then(|el| el.dyn_into::<HtmlElement>().ok())
    {
        addr.set_hidden(delivery != "shipping");
    }
}

fn set_support(amount: i64) {
    let doc = document();
    if let Ok(nodes) = doc.query_selector_all("[data-support-amount]") {
        for i in 0..nodes.length() {
            if let Some(node) = nodes.item(i) {
                if let Some(el) = node.dyn_ref::<HtmlElement>() {
                    let btn_amount: i64 = el
                        .get_attribute("data-support-amount")
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(0);
                    let active = btn_amount == amount;
                    let _ = el
                        .class_list()
                        .toggle_with_force("is-selected", active);
                    el.set_attribute("aria-pressed", if active { "true" } else { "false" })
                        .ok();
                }
            }
        }
    }
}

fn sync_step_ui() {
    let step = get_checkout_step();
    let doc = document();

    if let Some(details) = doc.get_element_by_id("checkout-step-details") {
        let _ = details.class_list().toggle_with_force("is-active", step == 0);
        let _ = details.class_list().toggle_with_force("is-done", step > 0);
    }
    if let Some(payment) = doc.get_element_by_id("checkout-step-payment") {
        let _ = payment.class_list().toggle_with_force("is-active", step == 1);
    }
    if let Some(confirm) = doc.get_element_by_id("checkout-step-confirmation") {
        let _ = confirm.class_list().toggle_with_force("is-active", false);
    }

    if let Some(details_panel) = doc
        .get_element_by_id("checkout-panel-details")
        .and_then(|el| el.dyn_into::<HtmlElement>().ok())
    {
        details_panel.set_hidden(step != 0);
    }
    if let Some(payment_panel) = doc
        .get_element_by_id("checkout-panel-payment")
        .and_then(|el| el.dyn_into::<HtmlElement>().ok())
    {
        payment_panel.set_hidden(step != 1);
    }

    if let Some(title) = doc.get_element_by_id("checkout-lead") {
        title.set_text_content(Some(if step == 0 {
            "Confirm your details and delivery preference."
        } else {
            "Enter payment to complete your order."
        }));
    }
}

fn render_trust_signals(email: &str, delivery: &str) {
    let doc = document();
    if let Some(receipt) = doc.get_element_by_id("checkout-trust-receipt") {
        let label = if email.is_empty() {
            "Receipt sent to your email".to_string()
        } else {
            format!("Receipt sent to {email}")
        };
        receipt.set_text_content(Some(&label));
    }
    if let Some(del_node) = doc.get_element_by_id("checkout-trust-delivery") {
        del_node.set_text_content(Some(if delivery == "shipping" {
            "Shipped to your address"
        } else {
            "Pick up at church after liturgy"
        }));
    }
}

fn render_checkout() {
    let cart = read_cart();
    let delivery = selected_delivery();
    let support = selected_support();
    let state = checkout_state(cart, &delivery, support);

    // Update cart badge
    let count = cart_total_count(&state.cart);
    if let Some(badge) = document().get_element_by_id("site-cart-count") {
        badge.set_text_content(Some(&count.to_string()));
    }

    let doc = document();

    // Order summary totals
    if let Some(el) = doc.get_element_by_id("checkout-subtotal") {
        el.set_text_content(Some(&format_money(state.subtotal)));
    }
    if let Some(el) = doc.get_element_by_id("checkout-shipping") {
        let shipping_text = if state.shipping > 0 {
            format_money(state.shipping)
        } else {
            "Free".to_string()
        };
        el.set_text_content(Some(&shipping_text));
    }
    if let Some(el) = doc.get_element_by_id("checkout-tax") {
        el.set_text_content(Some(&format_money(state.tax)));
    }
    if let Some(el) = doc.get_element_by_id("checkout-donation") {
        el.set_text_content(Some(&format_money(state.support)));
    }
    if let Some(el) = doc.get_element_by_id("checkout-total") {
        el.set_text_content(Some(&format_money(state.total)));
    }

    // Button states
    if let Some(el) = doc
        .get_element_by_id("checkout-continue")
        .and_then(|e| e.dyn_into::<HtmlElement>().ok())
    {
        let _ = js_sys::Reflect::set(
            &el,
            &JsValue::from_str("disabled"),
            &JsValue::from(!can_continue()),
        );
    }
    let placing = is_placing_order();
    if let Some(el) = doc
        .get_element_by_id("create-checkout-session")
        .and_then(|e| e.dyn_into::<HtmlElement>().ok())
    {
        let _ = js_sys::Reflect::set(
            &el,
            &JsValue::from_str("disabled"),
            &JsValue::from(!can_submit() || placing),
        );
    }
    if let Some(el) = doc.get_element_by_id("checkout-submit-label") {
        let label = if placing {
            "Placing order...".to_string()
        } else {
            format!("Place Order \u{2014} {}", format_money(state.total))
        };
        el.set_text_content(Some(&label));
    }

    // Trust signals
    let email = get_input_value("checkout-email").trim().to_string();
    render_trust_signals(&email, &delivery);

    // Line items
    if let Some(lines) = doc.get_element_by_id("checkout-lines") {
        if state.cart.is_empty() {
            lines.set_inner_html(r#"<div class="empty-inline">Your cart is empty.</div>"#);
        } else {
            let html: String = state
                .cart
                .iter()
                .map(|item| {
                    format!(
                        r#"<div class="checkout-summary-line"><div><div class="list-title">{title}</div><div class="list-meta">{author} · Qty {qty}</div></div><strong>{line_total}</strong></div>"#,
                        title = item.title,
                        author = item.author,
                        qty = item.quantity,
                        line_total = format_money(item.price_cents * item.quantity),
                    )
                })
                .collect();
            lines.set_inner_html(&html);
        }
    }
}

fn go_to_step(step: i32) {
    if step == 1 && !can_continue() {
        render_checkout();
        return;
    }
    set_checkout_step(step);
    sync_step_ui();
    render_checkout();
}

async fn create_checkout_session() {
    let delivery = selected_delivery();
    let support = selected_support();
    let cart = read_cart();
    let state = checkout_state(cart, &delivery, support);

    let email = get_input_value("checkout-email").trim().to_string();
    let name = get_input_value("checkout-name").trim().to_string();

    let line_items: Vec<_> = state
        .cart
        .iter()
        .filter(|item| item.quantity > 0)
        .map(|item| {
            serde_json::json!({
                "item_id": item.id,
                "quantity": item.quantity,
            })
        })
        .collect();

    if state.total == 0 || line_items.is_empty() {
        set_status(
            "Add at least one title before placing the order.",
            "danger",
        );
        return;
    }
    if name.is_empty() {
        set_status("Please enter your name.", "danger");
        return;
    }
    if email.is_empty() {
        set_status("Please enter a receipt email.", "danger");
        return;
    }
    if delivery == "shipping" {
        let addr = get_textarea_value("checkout-address").trim().to_string();
        if addr.is_empty() {
            set_status("Please enter a shipping address.", "danger");
            return;
        }
    }
    if card_digits().len() < 15 {
        set_status("Enter a valid card number to continue.", "danger");
        return;
    }

    set_placing_order(true);
    render_checkout();
    set_status("Placing your order...", "");

    let body = serde_json::json!({
        "email": email,
        "customer_name": name,
        "delivery_method": delivery,
        "donation_cents": support,
        "line_items": line_items,
    });

    let result = async {
        let opts = web_sys::RequestInit::new();
        opts.set_method("POST");
        let headers = web_sys::Headers::new().map_err(|e| format!("{e:?}"))?;
        headers
            .set("content-type", "application/json")
            .map_err(|e| format!("{e:?}"))?;
        opts.set_headers(&headers);
        opts.set_body(&JsValue::from_str(&body.to_string()));

        let request =
            web_sys::Request::new_with_str_and_init("/api/storefront/checkout/session", &opts)
                .map_err(|e| format!("{e:?}"))?;

        let window = web_sys::window().ok_or("no window")?;
        let resp_value =
            wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
                .await
                .map_err(|e| format!("{e:?}"))?;
        let resp: web_sys::Response = resp_value.dyn_into().map_err(|e| format!("{e:?}"))?;
        let ok = resp.ok();
        let json_promise = resp.json().map_err(|e| format!("{e:?}"))?;
        let json = wasm_bindgen_futures::JsFuture::from(json_promise)
            .await
            .unwrap_or(JsValue::NULL);
        Ok::<(bool, JsValue), String>((ok, json))
    }
    .await;

    set_placing_order(false);
    render_checkout();

    match result {
        Ok((true, json)) => {
            write_cart(&[]);
            let order_id = js_sys::Reflect::get(&json, &JsValue::from_str("order_id"))
                .ok()
                .and_then(|v| v.as_string())
                .or_else(|| {
                    js_sys::Reflect::get(&json, &JsValue::from_str("session_id"))
                        .ok()
                        .and_then(|v| v.as_string())
                })
                .unwrap_or_default();
            if let Some(window) = web_sys::window() {
                let url = format!(
                    "/orders?placed={}",
                    js_sys::encode_uri_component(&order_id)
                );
                let _ = window.location().set_href(&url);
            }
        }
        Ok((false, json)) => {
            let msg = js_sys::Reflect::get(&json, &JsValue::from_str("message"))
                .ok()
                .and_then(|v| v.as_string())
                .or_else(|| {
                    js_sys::Reflect::get(&json, &JsValue::from_str("error"))
                        .ok()
                        .and_then(|v| v.as_string())
                })
                .unwrap_or_else(|| "Order failed. Please try again.".to_string());
            set_status(&msg, "danger");
        }
        Err(e) => {
            set_status(&format!("Order failed: {e}"), "danger");
        }
    }
}

fn bind_checkout_controls() {
    let doc = document();

    // Continue to payment
    if let Some(el) = doc
        .get_element_by_id("checkout-continue")
        .and_then(|e| e.dyn_into::<HtmlElement>().ok())
    {
        let closure = Closure::wrap(Box::new(|| go_to_step(1)) as Box<dyn Fn()>);
        el.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // Back to details
    if let Some(el) = doc
        .get_element_by_id("checkout-back-to-details")
        .and_then(|e| e.dyn_into::<HtmlElement>().ok())
    {
        let closure = Closure::wrap(Box::new(|| go_to_step(0)) as Box<dyn Fn()>);
        el.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // Delivery options
    if let Ok(nodes) = doc.query_selector_all("[data-delivery-option]") {
        for i in 0..nodes.length() {
            if let Some(node) = nodes.item(i) {
                if let Some(el) = node.dyn_ref::<HtmlElement>() {
                    let delivery = el
                        .get_attribute("data-delivery-option")
                        .unwrap_or_else(|| "pickup".to_string());
                    let closure = Closure::wrap(Box::new(move || {
                        set_delivery(&delivery);
                        render_checkout();
                    }) as Box<dyn Fn()>);
                    el.set_onclick(Some(closure.as_ref().unchecked_ref()));
                    closure.forget();
                }
            }
        }
    }

    // Support amount buttons
    if let Ok(nodes) = doc.query_selector_all("[data-support-amount]") {
        for i in 0..nodes.length() {
            if let Some(node) = nodes.item(i) {
                if let Some(el) = node.dyn_ref::<HtmlElement>() {
                    let amount: i64 = el
                        .get_attribute("data-support-amount")
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(0);
                    let closure = Closure::wrap(Box::new(move || {
                        set_support(amount);
                        render_checkout();
                    }) as Box<dyn Fn()>);
                    el.set_onclick(Some(closure.as_ref().unchecked_ref()));
                    closure.forget();
                }
            }
        }
    }

    // Input listeners for name, email, address
    for id in &["checkout-name", "checkout-email", "checkout-address"] {
        if let Some(el) = doc
            .get_element_by_id(id)
            .and_then(|e| e.dyn_into::<HtmlElement>().ok())
        {
            let closure = Closure::wrap(Box::new(|| render_checkout()) as Box<dyn Fn()>);
            el.set_oninput(Some(closure.as_ref().unchecked_ref()));
            closure.forget();
        }
    }

    // Card number formatting
    if let Some(el) = doc
        .get_element_by_id("checkout-card-number")
        .and_then(|e| e.dyn_into::<HtmlInputElement>().ok())
    {
        let el_clone = el.clone();
        let closure = Closure::wrap(Box::new(move || {
            el_clone.set_value(&format_card(&el_clone.value()));
            render_checkout();
        }) as Box<dyn Fn()>);
        el.set_oninput(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // Expiry formatting
    if let Some(el) = doc
        .get_element_by_id("checkout-card-expiry")
        .and_then(|e| e.dyn_into::<HtmlInputElement>().ok())
    {
        let el_clone = el.clone();
        let closure = Closure::wrap(Box::new(move || {
            el_clone.set_value(&format_expiry(&el_clone.value()));
            render_checkout();
        }) as Box<dyn Fn()>);
        el.set_oninput(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // CVC formatting
    if let Some(el) = doc
        .get_element_by_id("checkout-card-cvc")
        .and_then(|e| e.dyn_into::<HtmlInputElement>().ok())
    {
        let el_clone = el.clone();
        let closure = Closure::wrap(Box::new(move || {
            el_clone.set_value(&strip_non_digits(&el_clone.value(), 4));
            render_checkout();
        }) as Box<dyn Fn()>);
        el.set_oninput(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // Place order button
    if let Some(el) = doc
        .get_element_by_id("create-checkout-session")
        .and_then(|e| e.dyn_into::<HtmlElement>().ok())
    {
        let closure = Closure::wrap(Box::new(|| {
            wasm_bindgen_futures::spawn_local(create_checkout_session());
        }) as Box<dyn Fn()>);
        el.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }
}

pub fn mount_checkout_island() {
    // Only mount if checkout elements exist on this page
    if document().get_element_by_id("checkout-step-details").is_none() {
        return;
    }

    set_checkout_step(0);
    set_placing_order(false);
    set_delivery("pickup");
    set_support(0);
    sync_step_ui();
    set_status("Ready to place your order.", "");
    bind_checkout_controls();
    render_checkout();

    // Set ready flag for browser tests
    if let Some(window) = web_sys::window() {
        let _ = js_sys::Reflect::set(
            &window,
            &JsValue::from_str("__SCRIPTORIUM_CHECKOUT_READY"),
            &JsValue::TRUE,
        );
    }
}
