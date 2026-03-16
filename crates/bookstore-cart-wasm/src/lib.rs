mod admin;
pub mod cart;
mod checkout;
mod components;
mod intake;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    components::mount_cart_island();
    checkout::mount_checkout_island();
    admin::mount_admin_island();
    intake::mount_intake_island();
}
