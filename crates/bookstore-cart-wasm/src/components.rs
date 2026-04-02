use crate::cart::{
    CartItem, add_to_cart, cart_total_cents, cart_total_count, format_money, mutate_cart,
    read_cart, write_cart,
};
use leptos::{mount::mount_to, prelude::*};
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, HtmlElement, HtmlInputElement};

fn document() -> Document {
    web_sys::window().expect("window").document().expect("document")
}

fn by_id(id: &str) -> Option<web_sys::Element> {
    document().get_element_by_id(id)
}

fn as_html(element: web_sys::Element) -> Option<HtmlElement> {
    element.dyn_into::<HtmlElement>().ok()
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

fn set_feedback_message(target_id: &str, message: &str) {
    if let Some(feedback) = by_id(target_id) {
        feedback.set_text_content(Some(message));
        feedback.set_class_name("notice-panel notice-panel--success");
    }
}

fn update_cart_count_dom(cart: &[CartItem]) {
    if let Some(badge) = by_id("site-cart-count") {
        badge.set_text_content(Some(&cart_total_count(cart).to_string()));
    }
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

fn set_cart_ready_flag() {
    if let Some(window) = web_sys::window() {
        let _ = js_sys::Reflect::set(
            &window,
            &JsValue::from_str("__SCRIPTORIUM_CART_READY"),
            &JsValue::TRUE,
        );
    }
}

fn bind_add_buttons(cart: RwSignal<Vec<CartItem>>) {
    let doc = document();
    if let Ok(buttons) = doc.query_selector_all("[data-add-book-id]") {
        for i in 0..buttons.length() {
            if let Some(node) = buttons.item(i) {
                if let Some(el) = node.dyn_ref::<HtmlElement>() {
                    let el_clone: Element = el.clone().into();
                    let cart_signal = cart;
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

                        cart_signal.update(|cart_items| {
                            add_to_cart(
                                cart_items,
                                CartItem { id, title, author, price_cents, quantity },
                            );
                        });

                        let total_count = cart_total_count(&cart_signal.get_untracked());
                        let feedback_target = el_clone
                            .get_attribute("data-feedback-target")
                            .unwrap_or_else(|| "cart-feedback".to_string());
                        set_feedback_message(
                            &feedback_target,
                            &format!("Added {quantity} to cart. Cart now has {total_count} item(s)."),
                        );
                    }) as Box<dyn Fn()>);
                    el.set_onclick(Some(closure.as_ref().unchecked_ref()));
                    closure.forget();
                }
            }
        }
    }
}

fn bind_clear_cart(cart: RwSignal<Vec<CartItem>>) {
    if let Some(clear) = by_id("clear-cart").and_then(as_html) {
        let closure = Closure::wrap(Box::new(move || {
            cart.set(Vec::new());
        }) as Box<dyn Fn()>);
        clear.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }
}

#[component]
fn CartBadge(cart: RwSignal<Vec<CartItem>>) -> impl IntoView {
    view! { {move || cart_total_count(&cart.get()).to_string()} }
}

#[component]
fn CartSummary(cart: RwSignal<Vec<CartItem>>) -> impl IntoView {
    view! { {move || format!("Cart total: {}", format_money(cart_total_cents(&cart.get())))} }
}

#[component]
fn CartItemsList(cart: RwSignal<Vec<CartItem>>) -> impl IntoView {
    view! {
        <Show
            when=move || !cart.get().is_empty()
            fallback=|| view! { <div class="empty-inline">"Your cart is empty."</div> }
        >
            <For
                each=move || cart.get()
                key=|item| item.id.clone()
                children=move |item: CartItem| {
                    let increment = {
                        let cart = cart;
                        let id = item.id.clone();
                        move |_| {
                            cart.update(|cart_items| mutate_cart(cart_items, &id, "increment"));
                        }
                    };
                    let decrement = {
                        let cart = cart;
                        let id = item.id.clone();
                        move |_| {
                            cart.update(|cart_items| mutate_cart(cart_items, &id, "decrement"));
                        }
                    };
                    let remove = {
                        let cart = cart;
                        let id = item.id.clone();
                        move |_| {
                            cart.update(|cart_items| mutate_cart(cart_items, &id, "remove"));
                        }
                    };

                    view! {
                        <div class="list-row">
                            <div>
                                <div class="list-title">{item.title.clone()}</div>
                                <div class="list-meta">
                                    {format!("{} · Qty {}", item.author, item.quantity)}
                                </div>
                            </div>
                            <div class="button-row button-row--compact">
                                <button
                                    class="ghost-link ghost-link--ink ghost-link--mini"
                                    type="button"
                                    attr:data-cart-decrement=item.id.clone()
                                    on:click=decrement
                                >
                                    "−"
                                </button>
                                <button
                                    class="ghost-link ghost-link--ink ghost-link--mini"
                                    type="button"
                                    attr:data-cart-increment=item.id.clone()
                                    on:click=increment
                                >
                                    "+"
                                </button>
                                <button
                                    class="ghost-link ghost-link--ink ghost-link--mini"
                                    type="button"
                                    attr:data-cart-remove=item.id.clone()
                                    on:click=remove
                                >
                                    "Remove"
                                </button>
                                <strong>{format_money(item.price_cents * item.quantity)}</strong>
                            </div>
                        </div>
                    }
                }
            />
        </Show>
    }
}

fn mount_into(id: &str, view: impl FnOnce() -> AnyView + 'static) {
    let Some(root) = by_id(id).and_then(as_html) else {
        return;
    };
    mount_to(root, view).forget();
}

pub fn mount_cart_island() {
    let cart = RwSignal::new(read_cart());

    Effect::new(move |_| {
        let current = cart.get();
        write_cart(&current);
        update_cart_count_dom(&current);
        sync_recommendations(&current);
    });

    bind_add_buttons(cart);
    bind_clear_cart(cart);

    mount_into("site-cart-count", move || view! { <CartBadge cart=cart /> }.into_any());
    mount_into("cart-items", move || view! { <CartItemsList cart=cart /> }.into_any());
    mount_into("cart-summary", move || view! { <CartSummary cart=cart /> }.into_any());
    set_cart_ready_flag();
}
