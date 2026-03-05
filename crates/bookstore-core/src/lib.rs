use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Book {
    pub id: String,
    pub title: String,
    pub author: String,
    pub category: String,
    pub price_cents: u32,
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

pub fn seed_church_bookstore() -> Inventory {
    let mut inventory = Inventory::new();
    let seed = [
        Book {
            id: "bk-100".to_string(),
            title: "The Purpose Driven Life".to_string(),
            author: "Rick Warren".to_string(),
            category: "Discipleship".to_string(),
            price_cents: 1899,
        },
        Book {
            id: "bk-101".to_string(),
            title: "Knowing God".to_string(),
            author: "J.I. Packer".to_string(),
            category: "Theology".to_string(),
            price_cents: 2099,
        },
    ];

    for book in seed {
        let _ = inventory.add_book(book);
    }

    inventory
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
