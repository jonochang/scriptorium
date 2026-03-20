use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct SeedData {
    pub defaults: SeedDefaults,
    pub catalog: SeedCatalog,
    pub pos: SeedPos,
    pub admin: SeedAdmin,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SeedDefaults {
    pub currency: String,
    pub locale: String,
    pub pos_pin: String,
    #[serde(default = "default_tenant_id")]
    pub tenant_id: String,
    #[serde(default = "default_admin_username")]
    pub admin_username: String,
    #[serde(default = "default_admin_password")]
    pub admin_password: String,
}

fn default_tenant_id() -> String { "church-a".to_string() }
fn default_admin_username() -> String { "admin".to_string() }
fn default_admin_password() -> String { "admin123".to_string() }

#[derive(Debug, Clone, Deserialize)]
pub struct SeedCatalog {
    pub books: Vec<SeedBook>,
}

impl SeedCatalog {
    pub fn find_book(&self, id: &str) -> Option<&SeedBook> {
        self.books.iter().find(|b| b.id == id)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SeedBook {
    pub id: String,
    pub title: String,
    pub author: String,
    pub category: String,
    pub price_cents: i64,
    #[serde(default)]
    pub blurb: String,
    #[serde(default)]
    pub publisher: String,
    #[serde(default)]
    pub binding: String,
    #[serde(default)]
    pub pages: String,
    #[serde(default)]
    pub isbn: String,
    #[serde(default)]
    pub cover_symbol: String,
    #[serde(default)]
    pub stock_hint: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SeedPos {
    pub barcode_items: Vec<SeedBarcodeItem>,
    pub quick_items: Vec<SeedQuickItem>,
    pub discount_codes: Vec<SeedDiscountCode>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SeedBarcodeItem {
    pub barcode: String,
    pub item_id: String,
    pub title: String,
    pub price_cents: i64,
    pub stock_on_hand: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedQuickItem {
    pub item_id: String,
    pub title: String,
    pub emoji: String,
    pub price_label: String,
    pub price_cents: i64,
    pub stock_on_hand: i64,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedDiscountCode {
    pub code: String,
    pub label: String,
    pub rate: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SeedAdmin {
    pub products: Vec<SeedAdminProduct>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SeedAdminProduct {
    pub isbn: String,
    pub title: String,
    pub category: String,
    pub vendor: String,
    pub cost_cents: i64,
    pub retail_cents: i64,
}

impl SeedData {
    pub fn from_toml_str(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        Ok(Self::from_toml_str(&contents)?)
    }
}

impl Default for SeedData {
    fn default() -> Self {
        Self::from_toml_str(include_str!("../../../db/seed.toml"))
            .expect("compiled-in seed.toml must be valid")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_seed_parses() {
        let seed = SeedData::default();
        assert!(!seed.catalog.books.is_empty());
        assert!(!seed.pos.quick_items.is_empty());
        assert!(!seed.pos.discount_codes.is_empty());
        assert!(!seed.admin.products.is_empty());
    }

    #[test]
    fn seed_has_expected_counts() {
        let seed = SeedData::default();
        assert_eq!(seed.catalog.books.len(), 12);
        assert_eq!(seed.pos.quick_items.len(), 8);
        assert_eq!(seed.pos.barcode_items.len(), 1);
        assert_eq!(seed.pos.discount_codes.len(), 3);
        assert_eq!(seed.admin.products.len(), 4);
    }
}
