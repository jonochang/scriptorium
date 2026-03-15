use crate::cart::{
    CartItem, add_to_cart, cart_total_cents, cart_total_count, format_money, mutate_cart,
    read_cart, write_cart,
};
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, HtmlElement, HtmlInputElement};

fn document() -> Document {
    web_sys::window().expect("window").document().expect("document")
}

fn update_cart_count(cart: &[CartItem]) {
    let count = cart_total_count(cart);
    if let Some(badge) = document().get_element_by_id("site-cart-count") {
        badge.set_text_content(Some(&count.to_string()));
    }
}

fn read_add_quantity(button: &Element) -> i64 {
    let target_id = button.get_attribute("data-add-book-quantity-target").unwrap_or_default();
    if target_id.is_empty() {
        return 1;
    }
    document()
        .get_element_by_id(&target_id)
        .and_then(|el| el.dyn_into::<HtmlInputElement>().ok())
        .and_then(|input| input.value().parse::<i64>().ok())
        .map(|v| v.max(1))
        .unwrap_or(1)
}

fn sync_recommendations(cart: &[CartItem]) {
    let cart_ids: std::collections::HashSet<&str> = cart.iter().map(|i| i.id.as_str()).collect();
    let cart_titles: std::collections::HashSet<String> =
        cart.iter().map(|i| i.title.trim().to_lowercase()).collect();

    let doc = document();
    let rows = doc.query_selector_all("[data-recommendation-book-id]").ok();
    let mut visible = 0u32;

    if let Some(rows) = rows {
        for i in 0..rows.length() {
            if let Some(node) = rows.item(i) {
                if let Some(el) = node.dyn_ref::<HtmlElement>() {
                    let rec_id =
                        el.get_attribute("data-recommendation-book-id").unwrap_or_default();
                    let rec_title = el
                        .get_attribute("data-recommendation-title")
                        .unwrap_or_default()
                        .trim()
                        .to_lowercase();

                    let hidden = cart_ids.contains(rec_id.as_str())
                        || (!rec_title.is_empty() && cart_titles.contains(&rec_title));

                    el.set_hidden(hidden);
                    let _ = el.style().set_property("display", if hidden { "none" } else { "" });
                    if !hidden {
                        visible += 1;
                    }
                }
            }
        }
    }

    if let Some(empty) = doc
        .get_element_by_id("cart-recommendations-empty")
        .and_then(|e| e.dyn_into::<HtmlElement>().ok())
    {
        empty.set_hidden(visible != 0);
    }
}

fn render_cart_page() {
    let cart = read_cart();
    update_cart_count(&cart);

    let doc = document();

    if let Some(cart_items) = doc.get_element_by_id("cart-items") {
        if cart.is_empty() {
            cart_items.set_inner_html(r#"<div class="empty-inline">Your cart is empty.</div>"#);
        } else {
            let html: String = cart
                .iter()
                .map(|item| {
                    format!(
                        r#"<div class="list-row"><div><div class="list-title">{title}</div><div class="list-meta">{author} · Qty {qty}</div></div><div class="button-row button-row--compact"><button class="ghost-link ghost-link--ink ghost-link--mini" type="button" data-cart-decrement="{id}">−</button><button class="ghost-link ghost-link--ink ghost-link--mini" type="button" data-cart-increment="{id}">+</button><button class="ghost-link ghost-link--ink ghost-link--mini" type="button" data-cart-remove="{id}">Remove</button><strong>{line_total}</strong></div></div>"#,
                        title = item.title,
                        author = item.author,
                        qty = item.quantity,
                        id = item.id,
                        line_total = format_money(item.price_cents * item.quantity),
                    )
                })
                .collect();
            cart_items.set_inner_html(&html);
        }
    }

    if let Some(cart_summary) = doc.get_element_by_id("cart-summary") {
        let total = cart_total_cents(&cart);
        cart_summary.set_text_content(Some(&format!("Cart total: {}", format_money(total))));
    }

    sync_recommendations(&cart);
    bind_cart_controls();
}

fn bind_cart_controls() {
    let doc = document();

    if let Ok(buttons) = doc.query_selector_all("[data-cart-increment]") {
        for i in 0..buttons.length() {
            if let Some(node) = buttons.item(i) {
                if let Some(el) = node.dyn_ref::<HtmlElement>() {
                    let id = el.get_attribute("data-cart-increment").unwrap_or_default();
                    let closure = Closure::wrap(Box::new(move || {
                        let mut cart = read_cart();
                        mutate_cart(&mut cart, &id, "increment");
                        write_cart(&cart);
                        render_cart_page();
                    }) as Box<dyn Fn()>);
                    el.set_onclick(Some(closure.as_ref().unchecked_ref()));
                    closure.forget();
                }
            }
        }
    }

    if let Ok(buttons) = doc.query_selector_all("[data-cart-decrement]") {
        for i in 0..buttons.length() {
            if let Some(node) = buttons.item(i) {
                if let Some(el) = node.dyn_ref::<HtmlElement>() {
                    let id = el.get_attribute("data-cart-decrement").unwrap_or_default();
                    let closure = Closure::wrap(Box::new(move || {
                        let mut cart = read_cart();
                        mutate_cart(&mut cart, &id, "decrement");
                        write_cart(&cart);
                        render_cart_page();
                    }) as Box<dyn Fn()>);
                    el.set_onclick(Some(closure.as_ref().unchecked_ref()));
                    closure.forget();
                }
            }
        }
    }

    if let Ok(buttons) = doc.query_selector_all("[data-cart-remove]") {
        for i in 0..buttons.length() {
            if let Some(node) = buttons.item(i) {
                if let Some(el) = node.dyn_ref::<HtmlElement>() {
                    let id = el.get_attribute("data-cart-remove").unwrap_or_default();
                    let closure = Closure::wrap(Box::new(move || {
                        let mut cart = read_cart();
                        mutate_cart(&mut cart, &id, "remove");
                        write_cart(&cart);
                        render_cart_page();
                    }) as Box<dyn Fn()>);
                    el.set_onclick(Some(closure.as_ref().unchecked_ref()));
                    closure.forget();
                }
            }
        }
    }
}

fn bind_add_buttons() {
    let doc = document();
    if let Ok(buttons) = doc.query_selector_all("[data-add-book-id]") {
        for i in 0..buttons.length() {
            if let Some(node) = buttons.item(i) {
                if let Some(el) = node.dyn_ref::<HtmlElement>() {
                    let el_clone: Element = el.clone().into();
                    let closure = Closure::wrap(Box::new(move || {
                        let id = el_clone.get_attribute("data-add-book-id").unwrap_or_default();
                        let title =
                            el_clone.get_attribute("data-add-book-title").unwrap_or_default();
                        let author =
                            el_clone.get_attribute("data-add-book-author").unwrap_or_default();
                        let price_cents: i64 = el_clone
                            .get_attribute("data-add-book-price-cents")
                            .and_then(|v| v.parse().ok())
                            .unwrap_or(0);
                        let quantity = read_add_quantity(&el_clone);

                        let mut cart = read_cart();
                        add_to_cart(
                            &mut cart,
                            CartItem { id, title, author, price_cents, quantity },
                        );
                        write_cart(&cart);

                        let total_count = cart_total_count(&cart);
                        let feedback_target = el_clone
                            .get_attribute("data-feedback-target")
                            .unwrap_or_else(|| "cart-feedback".to_string());
                        if let Some(feedback) = document().get_element_by_id(&feedback_target) {
                            feedback.set_text_content(Some(&format!(
                                "Added {quantity} to cart. Cart now has {total_count} item(s)."
                            )));
                            feedback.set_class_name("notice-panel notice-panel--success");
                        }

                        render_cart_page();
                    }) as Box<dyn Fn()>);
                    el.set_onclick(Some(closure.as_ref().unchecked_ref()));
                    closure.forget();
                }
            }
        }
    }
}

fn bind_clear_cart() {
    if let Some(clear) =
        document().get_element_by_id("clear-cart").and_then(|e| e.dyn_into::<HtmlElement>().ok())
    {
        let closure = Closure::wrap(Box::new(move || {
            write_cart(&[]);
            render_cart_page();
        }) as Box<dyn Fn()>);
        clear.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }
}

fn set_cart_ready_flag() {
    if let Some(window) = web_sys::window() {
        let _ = js_sys::Reflect::set(
            &window,
            &JsValue::from_str("__SCRIPTORIUM_CART_READY"),
            &JsValue::TRUE,
        );
    }
}

pub fn mount_cart_island() {
    bind_add_buttons();
    bind_clear_cart();
    render_cart_page();
    set_cart_ready_flag();
}
