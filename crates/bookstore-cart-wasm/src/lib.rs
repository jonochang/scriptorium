mod admin;
mod api;
pub mod cart;
mod checkout;
mod components;
mod intake;
mod pos;
mod scanner;

use leptos::{mount::mount_to, prelude::*};
#[cfg(not(test))]
use wasm_bindgen::prelude::*;

#[component]
fn IslandBootstrap(bootstrap: fn()) -> impl IntoView {
    Effect::new(move |_| {
        bootstrap();
    });
    view! { <></> }
}

fn ensure_mount_host(id: &str) -> Option<web_sys::HtmlElement> {
    let document = web_sys::window()?.document()?;
    if let Some(existing) = document.get_element_by_id(id) {
        return existing.dyn_into::<web_sys::HtmlElement>().ok();
    }

    let body = document.body()?;
    let host = document.create_element("div").ok()?.dyn_into::<web_sys::HtmlElement>().ok()?;
    host.set_id(id);
    host.set_hidden(true);
    let _ = body.append_child(&host);
    Some(host)
}

fn mount_bootstrap(id: &str, bootstrap: fn()) {
    let Some(host) = ensure_mount_host(id) else {
        return;
    };
    mount_to(host, move || view! { <IslandBootstrap bootstrap=bootstrap /> }).forget();
}

#[cfg_attr(not(test), wasm_bindgen(start))]
pub fn main() {
    console_error_panic_hook::set_once();
    mount_bootstrap("scriptorium-cart-bootstrap", components::mount_cart_island);
    mount_bootstrap("scriptorium-checkout-bootstrap", checkout::mount_checkout_island);
    mount_bootstrap("scriptorium-admin-bootstrap", admin::mount_admin_island);
    mount_bootstrap("scriptorium-intake-bootstrap", intake::mount_intake_island);
    mount_bootstrap("scriptorium-pos-bootstrap", pos::mount_pos_island);
}
