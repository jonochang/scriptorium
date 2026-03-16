use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};

const CART_KEY: &str = "scriptorium-storefront-cart";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CartItem {
    pub id: String,
    pub title: String,
    pub author: String,
    pub price_cents: i64,
    pub quantity: i64,
}

pub fn read_cart() -> Vec<CartItem> {
    LocalStorage::get::<Vec<CartItem>>(CART_KEY).unwrap_or_default()
}

pub fn write_cart(cart: &[CartItem]) {
    let _ = LocalStorage::set(CART_KEY, cart);
}

pub fn add_to_cart(cart: &mut Vec<CartItem>, item: CartItem) {
    if let Some(existing) = cart.iter_mut().find(|i| i.id == item.id) {
        existing.quantity += item.quantity;
    } else {
        cart.push(item);
    }
}

pub fn mutate_cart(cart: &mut Vec<CartItem>, id: &str, operation: &str) {
    if let Some(entry) = cart.iter_mut().find(|i| i.id == id) {
        match operation {
            "increment" => entry.quantity += 1,
            "decrement" => entry.quantity = (entry.quantity - 1).max(0),
            _ => {}
        }
    }
    if operation == "remove" {
        cart.retain(|i| i.id != id);
    } else {
        cart.retain(|i| i.quantity > 0);
    }
}

pub fn cart_total_count(cart: &[CartItem]) -> i64 {
    cart.iter().map(|i| i.quantity).sum()
}

pub fn cart_total_cents(cart: &[CartItem]) -> i64 {
    cart.iter().map(|i| i.price_cents * i.quantity).sum()
}

pub fn format_money(cents: i64) -> String {
    format!("${:.2}", cents as f64 / 100.0)
}

pub fn shipping_cents(subtotal: i64, delivery: &str) -> i64 {
    if subtotal <= 0 {
        return 0;
    }
    if delivery == "shipping" { 599 } else { 0 }
}

pub fn tax_cents(subtotal: i64) -> i64 {
    ((subtotal as f64) * 0.07).round() as i64
}

pub struct CheckoutState {
    pub cart: Vec<CartItem>,
    pub subtotal: i64,
    pub shipping: i64,
    pub tax: i64,
    pub support: i64,
    pub total: i64,
}

pub fn checkout_state(cart: Vec<CartItem>, delivery: &str, support: i64) -> CheckoutState {
    let subtotal = cart_total_cents(&cart);
    let shipping = shipping_cents(subtotal, delivery);
    let tax = tax_cents(subtotal);
    let total = subtotal + shipping + tax + support;
    CheckoutState { cart, subtotal, shipping, tax, support, total }
}

pub fn format_card(value: &str) -> String {
    let digits: String = value.chars().filter(|c| c.is_ascii_digit()).take(16).collect();
    digits
        .as_bytes()
        .chunks(4)
        .map(|chunk| std::str::from_utf8(chunk).unwrap_or(""))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn format_expiry(value: &str) -> String {
    let digits: String = value.chars().filter(|c| c.is_ascii_digit()).take(4).collect();
    if digits.len() > 2 {
        format!("{} / {}", &digits[..2], &digits[2..])
    } else {
        digits
    }
}

pub fn strip_non_digits(value: &str, max_len: usize) -> String {
    value.chars().filter(|c| c.is_ascii_digit()).take(max_len).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_item(id: &str) -> CartItem {
        CartItem {
            id: id.to_string(),
            title: format!("Book {id}"),
            author: "Author".to_string(),
            price_cents: 1000,
            quantity: 1,
        }
    }

    #[test]
    fn add_to_empty_cart() {
        let mut cart = Vec::new();
        add_to_cart(&mut cart, sample_item("bk-1"));
        assert_eq!(cart.len(), 1);
        assert_eq!(cart[0].quantity, 1);
    }

    #[test]
    fn add_existing_increments() {
        let mut cart = vec![sample_item("bk-1")];
        add_to_cart(&mut cart, sample_item("bk-1"));
        assert_eq!(cart.len(), 1);
        assert_eq!(cart[0].quantity, 2);
    }

    #[test]
    fn decrement_to_zero_removes() {
        let mut cart = vec![sample_item("bk-1")];
        mutate_cart(&mut cart, "bk-1", "decrement");
        assert!(cart.is_empty());
    }

    #[test]
    fn remove_deletes() {
        let mut cart = vec![sample_item("bk-1"), sample_item("bk-2")];
        mutate_cart(&mut cart, "bk-1", "remove");
        assert_eq!(cart.len(), 1);
        assert_eq!(cart[0].id, "bk-2");
    }

    #[test]
    fn format_money_formats_correctly() {
        assert_eq!(format_money(1699), "$16.99");
        assert_eq!(format_money(0), "$0.00");
        assert_eq!(format_money(100), "$1.00");
        assert_eq!(format_money(5), "$0.05");
    }

    #[test]
    fn total_cents() {
        let cart = vec![
            CartItem { quantity: 2, ..sample_item("bk-1") },
            CartItem { price_cents: 500, ..sample_item("bk-2") },
        ];
        assert_eq!(cart_total_cents(&cart), 2500);
    }

    #[test]
    fn total_count() {
        let cart = vec![
            CartItem { quantity: 3, ..sample_item("bk-1") },
            CartItem { quantity: 2, ..sample_item("bk-2") },
        ];
        assert_eq!(cart_total_count(&cart), 5);
    }

    #[test]
    fn increment_increases_quantity() {
        let mut cart = vec![sample_item("bk-1")];
        mutate_cart(&mut cart, "bk-1", "increment");
        assert_eq!(cart[0].quantity, 2);
    }

    #[test]
    fn shipping_free_for_pickup() {
        assert_eq!(shipping_cents(5000, "pickup"), 0);
    }

    #[test]
    fn shipping_599_for_shipping() {
        assert_eq!(shipping_cents(5000, "shipping"), 599);
    }

    #[test]
    fn shipping_zero_when_empty_cart() {
        assert_eq!(shipping_cents(0, "shipping"), 0);
    }

    #[test]
    fn tax_seven_percent() {
        assert_eq!(tax_cents(5697), 399);
    }

    #[test]
    fn checkout_state_computes_total() {
        let cart = vec![
            CartItem { price_cents: 1899, quantity: 1, ..sample_item("bk-1") },
            CartItem { price_cents: 2099, quantity: 1, ..sample_item("bk-2") },
            CartItem { price_cents: 1699, quantity: 1, ..sample_item("bk-3") },
        ];
        let state = checkout_state(cart, "pickup", 0);
        assert_eq!(state.subtotal, 5697);
        assert_eq!(state.shipping, 0);
        assert_eq!(state.tax, 399);
        assert_eq!(state.total, 6096);
    }

    #[test]
    fn format_card_groups_digits() {
        assert_eq!(format_card("4242424242424242"), "4242 4242 4242 4242");
        assert_eq!(format_card("42424"), "4242 4");
        assert_eq!(format_card("abc123def456"), "1234 56");
    }

    #[test]
    fn format_expiry_inserts_slash() {
        assert_eq!(format_expiry("1234"), "12 / 34");
        assert_eq!(format_expiry("12"), "12");
        assert_eq!(format_expiry("1"), "1");
    }

    #[test]
    fn strip_non_digits_works() {
        assert_eq!(strip_non_digits("abc123def456", 4), "1234");
        assert_eq!(strip_non_digits("987", 10), "987");
    }
}
