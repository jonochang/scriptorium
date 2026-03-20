use bookstore_app::seed::SeedData;
use bookstore_domain::Book;

pub fn catalog_json() -> String {
    let seed = SeedData::default();
    let books: Vec<Book> = seed
        .catalog
        .books
        .iter()
        .map(|b| Book {
            id: b.id.clone(),
            title: b.title.clone(),
            author: b.author.clone(),
            category: b.category.clone(),
            price_cents: b.price_cents,
        })
        .collect();
    serde_json::to_string(&books).unwrap_or_else(|_| "[]".to_string())
}
