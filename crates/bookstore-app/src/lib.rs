use std::sync::Arc;

use bookstore_domain::{Book, Inventory, InventoryError, seed_church_bookstore};
use tokio::sync::RwLock;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestContext {
    pub tenant_id: String,
    pub locale: String,
}

impl Default for RequestContext {
    fn default() -> Self {
        Self { tenant_id: "default".to_string(), locale: "en-AU".to_string() }
    }
}

#[derive(Clone, Default)]
pub struct CatalogService {
    inventory: Arc<RwLock<Inventory>>,
}

impl CatalogService {
    pub fn with_seed() -> Self {
        Self { inventory: Arc::new(RwLock::new(seed_church_bookstore())) }
    }

    pub fn from_inventory(inventory: Inventory) -> Self {
        Self { inventory: Arc::new(RwLock::new(inventory)) }
    }

    pub async fn list_books(&self) -> Vec<Book> {
        let inventory = self.inventory.read().await;
        inventory.books().to_vec()
    }

    pub async fn add_book(&self, book: Book) -> Result<(), InventoryError> {
        let mut inventory = self.inventory.write().await;
        inventory.add_book(book)
    }
}
