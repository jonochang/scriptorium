use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum MoneyError {
    #[error("currency must be a 3-letter ASCII code")]
    InvalidCurrency,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Money {
    pub currency: String,
    pub minor_units: i64,
}

impl Money {
    pub fn from_minor(currency: &str, minor_units: i64) -> Result<Self, MoneyError> {
        let normalized = currency.trim().to_ascii_uppercase();
        if normalized.len() != 3 || !normalized.chars().all(|ch| ch.is_ascii_alphabetic()) {
            return Err(MoneyError::InvalidCurrency);
        }
        Ok(Self { currency: normalized, minor_units })
    }

    pub fn gst_component_cents(&self) -> i64 {
        // AUD retail pricing is GST-inclusive in MVP. GST component is 1/11 of inclusive total.
        self.minor_units / 11
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OrderChannel {
    Pos,
    Online,
}

impl OrderChannel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pos => "POS",
            Self::Online => "Online",
        }
    }
}

impl fmt::Display for OrderChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PaymentMethod {
    Cash,
    ExternalCard,
    OnlineCard,
    Iou,
    IouSettled,
}

impl PaymentMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Cash => "cash",
            Self::ExternalCard => "external_card",
            Self::OnlineCard => "online_card",
            Self::Iou => "iou",
            Self::IouSettled => "iou_settled",
        }
    }
}

impl fmt::Display for PaymentMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OrderStatus {
    Paid,
    UnpaidIou,
    Refunded,
}

impl OrderStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Paid => "Paid",
            Self::UnpaidIou => "UnpaidIou",
            Self::Refunded => "Refunded",
        }
    }
}

impl fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Book {
    pub id: String,
    pub title: String,
    pub author: String,
    pub category: String,
    pub price_cents: i64,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum InventoryError {
    #[error("book id already exists: {0}")]
    DuplicateId(String),
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Inventory {
    books: Vec<Book>,
}

impl Inventory {
    pub fn new() -> Self {
        Self { books: Vec::new() }
    }

    pub fn add_book(&mut self, book: Book) -> Result<(), InventoryError> {
        if self.books.iter().any(|b| b.id == book.id) {
            return Err(InventoryError::DuplicateId(book.id));
        }
        self.books.push(book);
        Ok(())
    }

    pub fn books(&self) -> &[Book] {
        &self.books
    }

    pub fn by_category(&self, category: &str) -> Vec<Book> {
        self.books.iter().filter(|b| b.category.eq_ignore_ascii_case(category)).cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck_macros::quickcheck;

    #[test]
    fn rejects_duplicate_id() {
        let mut inv = Inventory::new();
        let first = Book {
            id: "bk-1".to_string(),
            title: "A".to_string(),
            author: "B".to_string(),
            category: "C".to_string(),
            price_cents: 100,
        };
        inv.add_book(first.clone()).expect("add should succeed");
        let err = inv.add_book(first).expect_err("duplicate must fail");
        assert_eq!(err, InventoryError::DuplicateId("bk-1".to_string()));
    }

    #[test]
    fn gst_component_is_one_eleventh_of_inclusive_amount() {
        let money = Money::from_minor("AUD", 1650).expect("valid money");
        assert_eq!(money.gst_component_cents(), 150);
    }

    // --- Property-based tests ---

    #[quickcheck]
    fn money_gst_never_exceeds_total(minor_units: i64) -> bool {
        let m = Money { currency: "AUD".to_string(), minor_units };
        let gst = m.gst_component_cents();
        // GST component should never have greater magnitude than the total
        gst.unsigned_abs() <= minor_units.unsigned_abs()
    }

    #[quickcheck]
    fn money_gst_is_one_eleventh(minor_units: i64) -> bool {
        let m = Money { currency: "AUD".to_string(), minor_units };
        m.gst_component_cents() == minor_units / 11
    }

    #[quickcheck]
    fn money_valid_three_letter_currency_accepted(a: u8, b: u8, c: u8) -> bool {
        let a = b'A' + (a % 26);
        let b = b'A' + (b % 26);
        let c = b'A' + (c % 26);
        let code = String::from_utf8(vec![a, b, c]).unwrap();
        Money::from_minor(&code, 0).is_ok()
    }

    #[quickcheck]
    fn money_currency_is_normalized_uppercase(raw: String) -> bool {
        match Money::from_minor(&raw, 0) {
            Ok(m) => m.currency == m.currency.to_ascii_uppercase() && m.currency.len() == 3,
            Err(_) => true, // rejected inputs are fine
        }
    }

    #[quickcheck]
    fn money_normalization_is_idempotent(raw: String) -> bool {
        // If accepted once, passing the normalized value produces the same result
        if let Ok(first) = Money::from_minor(&raw, 42) {
            if let Ok(second) = Money::from_minor(&first.currency, 42) {
                return first.currency == second.currency;
            }
        }
        true
    }

    #[quickcheck]
    fn inventory_unique_ids_all_added(n: u8) -> bool {
        let n = (n % 50) as usize; // cap to keep tests fast
        let mut inv = Inventory::new();
        for i in 0..n {
            let book = Book {
                id: format!("bk-{i}"),
                title: "T".to_string(),
                author: "A".to_string(),
                category: "C".to_string(),
                price_cents: 100,
            };
            inv.add_book(book).unwrap();
        }
        inv.books().len() == n
    }

    #[quickcheck]
    fn inventory_duplicate_always_rejected(id: String) -> bool {
        let mut inv = Inventory::new();
        let book = || Book {
            id: id.clone(),
            title: "T".to_string(),
            author: "A".to_string(),
            category: "C".to_string(),
            price_cents: 100,
        };
        let _ = inv.add_book(book());
        inv.add_book(book()).is_err()
    }

    #[quickcheck]
    fn by_category_is_case_insensitive(category: String) -> bool {
        let mut inv = Inventory::new();
        let _ = inv.add_book(Book {
            id: "bk-1".to_string(),
            title: "T".to_string(),
            author: "A".to_string(),
            category: category.clone(),
            price_cents: 100,
        });
        let upper = inv.by_category(&category.to_ascii_uppercase());
        let lower = inv.by_category(&category.to_ascii_lowercase());
        // If the category is ASCII, case shouldn't matter
        if category.is_ascii() {
            upper.len() == lower.len()
        } else {
            true // non-ASCII is out of scope for eq_ignore_ascii_case
        }
    }
}
