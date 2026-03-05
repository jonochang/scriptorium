use bookstore_core::seed_church_bookstore;

pub fn catalog_json() -> String {
    serde_json::to_string(&seed_church_bookstore().books()).unwrap_or_else(|_| "[]".to_string())
}
