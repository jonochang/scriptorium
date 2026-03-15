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
}
