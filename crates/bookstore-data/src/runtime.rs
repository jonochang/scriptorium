use anyhow::Context;
use bookstore_app::seed::{SeedData, SeedDiscountCode, SeedQuickItem};
use bookstore_domain::{Book, OrderChannel, OrderStatus, PaymentMethod};
use sqlx::postgres::PgRow;
use sqlx::sqlite::SqliteRow;
use sqlx::{PgPool, Row, SqlitePool};

use crate::DatabasePool;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeProduct {
    pub tenant_id: String,
    pub product_id: String,
    pub isbn: String,
    pub title: String,
    pub author: String,
    pub publisher: String,
    pub description: String,
    pub public_title: String,
    pub public_author: String,
    pub public_publisher: String,
    pub public_description: String,
    pub public_cover_image_url: Option<String>,
    pub category: String,
    pub vendor: String,
    pub cost_cents: i64,
    pub retail_cents: i64,
    pub quantity_on_hand: i64,
    pub cover_image_key: Option<String>,
    pub binding: String,
    pub pages: String,
    pub is_catalog_visible: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeStockMovement {
    pub tenant_id: String,
    pub isbn: String,
    pub delta: i64,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOrder {
    pub order_id: String,
    pub tenant_id: String,
    pub customer_name: String,
    pub channel: String,
    pub status: String,
    pub payment_method: String,
    pub total_cents: i64,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeReportSummary {
    pub tenant_id: String,
    pub sales_cents: i64,
    pub donations_cents: i64,
    pub cogs_cents: i64,
    pub gross_profit_cents: i64,
    pub sales_by_payment: Vec<(String, i64)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeProductPage {
    pub products: Vec<RuntimeProduct>,
    pub page: u32,
    pub per_page: u32,
    pub total_matches: u32,
    pub total_pages: u32,
    pub total_products: u32,
    pub retail_value_cents: i64,
    pub low_stock_count: u32,
    pub out_of_stock_count: u32,
}

fn normalize_isbn(isbn: &str) -> String {
    isbn.chars().filter(|ch| ch.is_ascii_digit()).collect()
}

fn timestamp_now() -> String {
    chrono::Utc::now().naive_utc().format("%Y-%m-%d %H:%M:%S").to_string()
}

fn stock_hint_quantity(hint: &str) -> i64 {
    match hint {
        "out_of_stock" => 0,
        "low_2" => 2,
        "low_3" => 3,
        _ => 10,
    }
}

fn inventory_status(product: &RuntimeProduct) -> &'static str {
    if product.quantity_on_hand <= 0 {
        "out"
    } else if product.quantity_on_hand <= 3 {
        "low"
    } else {
        "ok"
    }
}

fn normalize_opt(value: Option<String>) -> Option<String> {
    value.map(|value| value.trim().to_string()).filter(|value| !value.is_empty())
}

fn trim_owned(value: String) -> String {
    value.trim().to_string()
}

fn choose_richer_str<'a>(preferred: &'a str, fallback: &'a str) -> &'a str {
    if preferred.trim().is_empty() { fallback } else { preferred }
}

fn runtime_seed_product_from_admin_seed(
    tenant_id: &str,
    isbn: &str,
    product_id: &str,
    existing: Option<&RuntimeProduct>,
    product: &bookstore_app::seed::SeedAdminProduct,
) -> RuntimeProduct {
    RuntimeProduct {
        tenant_id: tenant_id.to_string(),
        product_id: product_id.to_string(),
        isbn: isbn.to_string(),
        title: existing
            .map(|item| choose_richer_str(&item.title, &product.title).to_string())
            .unwrap_or_else(|| product.title.clone()),
        author: existing.map(|item| item.author.clone()).unwrap_or_default(),
        publisher: existing.map(|item| item.publisher.clone()).unwrap_or_default(),
        description: existing.map(|item| item.description.clone()).unwrap_or_default(),
        public_title: existing
            .map(|item| choose_richer_str(&item.public_title, &product.title).to_string())
            .unwrap_or_else(|| product.title.clone()),
        public_author: existing.map(|item| item.public_author.clone()).unwrap_or_default(),
        public_publisher: existing.map(|item| item.public_publisher.clone()).unwrap_or_default(),
        public_description: existing
            .map(|item| item.public_description.clone())
            .unwrap_or_default(),
        public_cover_image_url: existing.and_then(|item| item.public_cover_image_url.clone()),
        category: product.category.clone(),
        vendor: product.vendor.clone(),
        cost_cents: product.cost_cents,
        retail_cents: product.retail_cents,
        quantity_on_hand: existing.map_or(10_i64, |item| item.quantity_on_hand),
        cover_image_key: existing.and_then(|item| item.cover_image_key.clone()),
        binding: existing.map(|item| item.binding.clone()).unwrap_or_default(),
        pages: existing.map(|item| item.pages.clone()).unwrap_or_default(),
        is_catalog_visible: existing.map(|item| item.is_catalog_visible).unwrap_or(true),
    }
}

fn validate_product(pool_product: &RuntimeProduct) -> anyhow::Result<()> {
    if pool_product.tenant_id.trim().is_empty() {
        anyhow::bail!("Tenant is required.");
    }
    if pool_product.product_id.trim().is_empty() {
        anyhow::bail!("Product id is required.");
    }
    if pool_product.title.trim().is_empty() {
        anyhow::bail!("Title is required.");
    }
    if pool_product.category.trim().is_empty() {
        anyhow::bail!("Category is required.");
    }
    if pool_product.vendor.trim().is_empty() {
        anyhow::bail!("Vendor is required.");
    }
    if pool_product.cost_cents < 0 {
        anyhow::bail!("Cost cannot be negative.");
    }
    if pool_product.retail_cents < 0 {
        anyhow::bail!("Retail price cannot be negative.");
    }
    if !pool_product.isbn.is_empty() && pool_product.isbn.len() != 10 && pool_product.isbn.len() != 13 {
        anyhow::bail!("ISBN must be 10 or 13 digits.");
    }
    Ok(())
}

async fn sqlite_count(pool: &SqlitePool, sql: &str) -> anyhow::Result<i64> {
    sqlx::query_scalar::<_, i64>(sql).fetch_one(pool).await.map_err(Into::into)
}

async fn postgres_count(pool: &PgPool, sql: &str) -> anyhow::Result<i64> {
    sqlx::query_scalar::<_, i64>(sql).fetch_one(pool).await.map_err(Into::into)
}

pub async fn seed_runtime_data(
    pool: &DatabasePool,
    tenant_id: &str,
    seed: &SeedData,
) -> anyhow::Result<()> {
    match pool {
        DatabasePool::Sqlite(pool) => seed_runtime_data_sqlite(pool, tenant_id, seed).await,
        DatabasePool::Postgres(pool) => seed_runtime_data_postgres(pool, tenant_id, seed).await,
    }
}

async fn seed_runtime_data_sqlite(
    pool: &SqlitePool,
    tenant_id: &str,
    seed: &SeedData,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT OR IGNORE INTO tenants (id, name, default_locale) VALUES (?1, ?2, ?3)",
    )
    .bind(tenant_id)
    .bind("Default Bookshop")
    .bind(&seed.defaults.locale)
    .execute(pool)
    .await?;

    if sqlite_count(pool, "SELECT COUNT(*) FROM products").await? == 0 {
        for book in &seed.catalog.books {
            let now = timestamp_now();
            sqlx::query(
                "INSERT INTO products (
                    tenant_id, product_id, isbn, title, author, publisher, description,
                    public_title, public_author, public_publisher, public_description,
                    public_cover_image_url, category, vendor, cost_cents, retail_cents,
                    quantity_on_hand, cover_image_key, binding, pages, is_catalog_visible,
                    created_at, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23)",
            )
            .bind(tenant_id)
            .bind(&book.id)
            .bind(normalize_isbn(&book.isbn))
            .bind(&book.title)
            .bind(&book.author)
            .bind(&book.publisher)
            .bind(&book.blurb)
            .bind(&book.title)
            .bind(&book.author)
            .bind(&book.publisher)
            .bind(&book.blurb)
            .bind(Option::<String>::None)
            .bind(&book.category)
            .bind("Church Supplier")
            .bind(0_i64)
            .bind(book.price_cents)
            .bind(stock_hint_quantity(&book.stock_hint))
            .bind(Option::<String>::None)
            .bind(&book.binding)
            .bind(&book.pages)
            .bind(1_i64)
            .bind(&now)
            .bind(&now)
            .execute(pool)
            .await?;
        }
    }

    if sqlite_count(pool, "SELECT COUNT(*) FROM pos_quick_items").await? == 0 {
        for (idx, item) in seed.pos.quick_items.iter().enumerate() {
            sqlx::query(
                "INSERT INTO pos_quick_items (item_id, title, emoji, price_label, price_cents, stock_on_hand, note, sort_order)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            )
            .bind(&item.item_id)
            .bind(&item.title)
            .bind(&item.emoji)
            .bind(&item.price_label)
            .bind(item.price_cents)
            .bind(item.stock_on_hand)
            .bind(&item.note)
            .bind(idx as i64)
            .execute(pool)
            .await?;
        }
    }

    if sqlite_count(pool, "SELECT COUNT(*) FROM pos_discount_codes").await? == 0 {
        for item in &seed.pos.discount_codes {
            sqlx::query(
                "INSERT INTO pos_discount_codes (code, label, rate) VALUES (?1, ?2, ?3)",
            )
            .bind(&item.code)
            .bind(&item.label)
            .bind(item.rate)
            .execute(pool)
            .await?;
        }
    }

    for product in &seed.admin.products {
        let isbn = normalize_isbn(&product.isbn);
        let existing_product_id: Option<String> =
            sqlx::query_scalar("SELECT product_id FROM products WHERE tenant_id = ?1 AND isbn = ?2")
                .bind(tenant_id)
                .bind(&isbn)
                .fetch_optional(pool)
                .await?;
        let product_id = existing_product_id.unwrap_or_else(|| format!("prd-{}", isbn));
        let existing = get_product_by_id_sqlite(pool, tenant_id, &product_id).await?;
        let merged = runtime_seed_product_from_admin_seed(
            tenant_id,
            &isbn,
            &product_id,
            existing.as_ref(),
            product,
        );
        let now = timestamp_now();
        sqlx::query(
            "INSERT INTO products (
                tenant_id, product_id, isbn, title, author, publisher, description,
                public_title, public_author, public_publisher, public_description,
                public_cover_image_url, category, vendor, cost_cents, retail_cents,
                quantity_on_hand, cover_image_key, binding, pages, is_catalog_visible,
                created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23)
             ON CONFLICT(tenant_id, product_id) DO UPDATE SET
                isbn = excluded.isbn,
                title = excluded.title,
                category = excluded.category,
                vendor = excluded.vendor,
                cost_cents = excluded.cost_cents,
                retail_cents = excluded.retail_cents,
                quantity_on_hand = CASE
                    WHEN products.quantity_on_hand = 0 THEN excluded.quantity_on_hand
                    ELSE products.quantity_on_hand
                END,
                updated_at = excluded.updated_at",
        )
        .bind(&merged.tenant_id)
        .bind(&merged.product_id)
        .bind(&merged.isbn)
        .bind(&merged.title)
        .bind(&merged.author)
        .bind(&merged.publisher)
        .bind(&merged.description)
        .bind(&merged.public_title)
        .bind(&merged.public_author)
        .bind(&merged.public_publisher)
        .bind(&merged.public_description)
        .bind(merged.public_cover_image_url.clone())
        .bind(&merged.category)
        .bind(&merged.vendor)
        .bind(merged.cost_cents)
        .bind(merged.retail_cents)
        .bind(merged.quantity_on_hand)
        .bind(merged.cover_image_key.clone())
        .bind(&merged.binding)
        .bind(&merged.pages)
        .bind(if merged.is_catalog_visible { 1_i64 } else { 0_i64 })
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn seed_runtime_data_postgres(
    pool: &PgPool,
    tenant_id: &str,
    seed: &SeedData,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO tenants (id, name, default_locale) VALUES ($1, $2, $3) ON CONFLICT (id) DO NOTHING",
    )
    .bind(tenant_id)
    .bind("Default Bookshop")
    .bind(&seed.defaults.locale)
    .execute(pool)
    .await?;

    if postgres_count(pool, "SELECT COUNT(*) FROM products").await? == 0 {
        for book in &seed.catalog.books {
            let now = timestamp_now();
            sqlx::query(
                "INSERT INTO products (
                    tenant_id, product_id, isbn, title, author, publisher, description,
                    public_title, public_author, public_publisher, public_description,
                    public_cover_image_url, category, vendor, cost_cents, retail_cents,
                    quantity_on_hand, cover_image_key, binding, pages, is_catalog_visible,
                    created_at, updated_at
                 ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23)",
            )
            .bind(tenant_id)
            .bind(&book.id)
            .bind(normalize_isbn(&book.isbn))
            .bind(&book.title)
            .bind(&book.author)
            .bind(&book.publisher)
            .bind(&book.blurb)
            .bind(&book.title)
            .bind(&book.author)
            .bind(&book.publisher)
            .bind(&book.blurb)
            .bind(Option::<String>::None)
            .bind(&book.category)
            .bind("Church Supplier")
            .bind(0_i64)
            .bind(book.price_cents)
            .bind(stock_hint_quantity(&book.stock_hint))
            .bind(Option::<String>::None)
            .bind(&book.binding)
            .bind(&book.pages)
            .bind(true)
            .bind(&now)
            .bind(&now)
            .execute(pool)
            .await?;
        }
    }

    if postgres_count(pool, "SELECT COUNT(*) FROM pos_quick_items").await? == 0 {
        for (idx, item) in seed.pos.quick_items.iter().enumerate() {
            sqlx::query(
                "INSERT INTO pos_quick_items (item_id, title, emoji, price_label, price_cents, stock_on_hand, note, sort_order)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            )
            .bind(&item.item_id)
            .bind(&item.title)
            .bind(&item.emoji)
            .bind(&item.price_label)
            .bind(item.price_cents)
            .bind(item.stock_on_hand)
            .bind(&item.note)
            .bind(idx as i64)
            .execute(pool)
            .await?;
        }
    }

    if postgres_count(pool, "SELECT COUNT(*) FROM pos_discount_codes").await? == 0 {
        for item in &seed.pos.discount_codes {
            sqlx::query(
                "INSERT INTO pos_discount_codes (code, label, rate) VALUES ($1, $2, $3)",
            )
            .bind(&item.code)
            .bind(&item.label)
            .bind(item.rate)
            .execute(pool)
            .await?;
        }
    }

    for product in &seed.admin.products {
        let isbn = normalize_isbn(&product.isbn);
        let existing_product_id: Option<String> =
            sqlx::query_scalar("SELECT product_id FROM products WHERE tenant_id = $1 AND isbn = $2")
                .bind(tenant_id)
                .bind(&isbn)
                .fetch_optional(pool)
                .await?;
        let product_id = existing_product_id.unwrap_or_else(|| format!("prd-{}", isbn));
        let existing = get_product_by_id_postgres(pool, tenant_id, &product_id).await?;
        let merged = runtime_seed_product_from_admin_seed(
            tenant_id,
            &isbn,
            &product_id,
            existing.as_ref(),
            product,
        );
        let now = timestamp_now();
        sqlx::query(
            "INSERT INTO products (
                tenant_id, product_id, isbn, title, author, publisher, description,
                public_title, public_author, public_publisher, public_description,
                public_cover_image_url, category, vendor, cost_cents, retail_cents,
                quantity_on_hand, cover_image_key, binding, pages, is_catalog_visible,
                created_at, updated_at
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23)
             ON CONFLICT(tenant_id, product_id) DO UPDATE SET
                isbn = excluded.isbn,
                title = excluded.title,
                category = excluded.category,
                vendor = excluded.vendor,
                cost_cents = excluded.cost_cents,
                retail_cents = excluded.retail_cents,
                quantity_on_hand = CASE
                    WHEN products.quantity_on_hand = 0 THEN excluded.quantity_on_hand
                    ELSE products.quantity_on_hand
                END,
                updated_at = excluded.updated_at",
        )
        .bind(&merged.tenant_id)
        .bind(&merged.product_id)
        .bind(&merged.isbn)
        .bind(&merged.title)
        .bind(&merged.author)
        .bind(&merged.publisher)
        .bind(&merged.description)
        .bind(&merged.public_title)
        .bind(&merged.public_author)
        .bind(&merged.public_publisher)
        .bind(&merged.public_description)
        .bind(merged.public_cover_image_url.clone())
        .bind(&merged.category)
        .bind(&merged.vendor)
        .bind(merged.cost_cents)
        .bind(merged.retail_cents)
        .bind(merged.quantity_on_hand)
        .bind(merged.cover_image_key.clone())
        .bind(&merged.binding)
        .bind(&merged.pages)
        .bind(merged.is_catalog_visible)
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await?;
    }

    Ok(())
}

fn runtime_product_from_sqlite_row(row: &SqliteRow) -> RuntimeProduct {
    RuntimeProduct {
        tenant_id: row.get("tenant_id"),
        product_id: row.get("product_id"),
        isbn: row.get("isbn"),
        title: row.get("title"),
        author: row.get("author"),
        publisher: row.get("publisher"),
        description: row.get("description"),
        public_title: row.get("public_title"),
        public_author: row.get("public_author"),
        public_publisher: row.get("public_publisher"),
        public_description: row.get("public_description"),
        public_cover_image_url: row.get("public_cover_image_url"),
        category: row.get("category"),
        vendor: row.get("vendor"),
        cost_cents: row.get("cost_cents"),
        retail_cents: row.get("retail_cents"),
        quantity_on_hand: row.get("quantity_on_hand"),
        cover_image_key: row.get("cover_image_key"),
        binding: row.get("binding"),
        pages: row.get("pages"),
        is_catalog_visible: row.get::<i64, _>("is_catalog_visible") != 0,
    }
}

fn runtime_product_from_postgres_row(row: &PgRow) -> RuntimeProduct {
    RuntimeProduct {
        tenant_id: row.get("tenant_id"),
        product_id: row.get("product_id"),
        isbn: row.get("isbn"),
        title: row.get("title"),
        author: row.get("author"),
        publisher: row.get("publisher"),
        description: row.get("description"),
        public_title: row.get("public_title"),
        public_author: row.get("public_author"),
        public_publisher: row.get("public_publisher"),
        public_description: row.get("public_description"),
        public_cover_image_url: row.get("public_cover_image_url"),
        category: row.get("category"),
        vendor: row.get("vendor"),
        cost_cents: row.get("cost_cents"),
        retail_cents: row.get("retail_cents"),
        quantity_on_hand: row.get("quantity_on_hand"),
        cover_image_key: row.get("cover_image_key"),
        binding: row.get("binding"),
        pages: row.get("pages"),
        is_catalog_visible: row.get("is_catalog_visible"),
    }
}

async fn get_product_by_id_sqlite(
    pool: &SqlitePool,
    tenant_id: &str,
    product_id: &str,
) -> anyhow::Result<Option<RuntimeProduct>> {
    let row = sqlx::query(
        "SELECT tenant_id, product_id, isbn, title, author, publisher, description,
                public_title, public_author, public_publisher, public_description,
                public_cover_image_url, category, vendor, cost_cents, retail_cents,
                quantity_on_hand, cover_image_key, binding, pages, is_catalog_visible
         FROM products WHERE tenant_id = ?1 AND product_id = ?2",
    )
    .bind(tenant_id)
    .bind(product_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|row| runtime_product_from_sqlite_row(&row)))
}

async fn get_product_by_id_postgres(
    pool: &PgPool,
    tenant_id: &str,
    product_id: &str,
) -> anyhow::Result<Option<RuntimeProduct>> {
    let row = sqlx::query(
        "SELECT tenant_id, product_id, isbn, title, author, publisher, description,
                public_title, public_author, public_publisher, public_description,
                public_cover_image_url, category, vendor, cost_cents, retail_cents,
                quantity_on_hand, cover_image_key, binding, pages, is_catalog_visible
         FROM products WHERE tenant_id = $1 AND product_id = $2",
    )
    .bind(tenant_id)
    .bind(product_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|row| runtime_product_from_postgres_row(&row)))
}

pub async fn get_product_by_id(
    pool: &DatabasePool,
    tenant_id: &str,
    product_id: &str,
) -> anyhow::Result<Option<RuntimeProduct>> {
    match pool {
        DatabasePool::Sqlite(pool) => get_product_by_id_sqlite(pool, tenant_id, product_id).await,
        DatabasePool::Postgres(pool) => get_product_by_id_postgres(pool, tenant_id, product_id).await,
    }
}

pub async fn get_product_by_isbn(
    pool: &DatabasePool,
    tenant_id: &str,
    isbn: &str,
) -> anyhow::Result<Option<RuntimeProduct>> {
    let isbn = normalize_isbn(isbn);
    match pool {
        DatabasePool::Sqlite(pool) => {
            let product_id: Option<String> =
                sqlx::query_scalar("SELECT product_id FROM products WHERE tenant_id = ?1 AND isbn = ?2")
                    .bind(tenant_id)
                    .bind(&isbn)
                    .fetch_optional(pool)
                    .await?;
            match product_id {
                Some(product_id) => get_product_by_id_sqlite(pool, tenant_id, &product_id).await,
                None => Ok(None),
            }
        }
        DatabasePool::Postgres(pool) => {
            let product_id: Option<String> =
                sqlx::query_scalar("SELECT product_id FROM products WHERE tenant_id = $1 AND isbn = $2")
                    .bind(tenant_id)
                    .bind(&isbn)
                    .fetch_optional(pool)
                    .await?;
            match product_id {
                Some(product_id) => get_product_by_id_postgres(pool, tenant_id, &product_id).await,
                None => Ok(None),
            }
        }
    }
}

pub async fn list_products(pool: &DatabasePool, tenant_id: &str) -> anyhow::Result<Vec<RuntimeProduct>> {
    match pool {
        DatabasePool::Sqlite(pool) => {
            let rows = sqlx::query(
                "SELECT tenant_id, product_id, isbn, title, author, publisher, description,
                        public_title, public_author, public_publisher, public_description,
                        public_cover_image_url, category, vendor, cost_cents, retail_cents,
                        quantity_on_hand, cover_image_key, binding, pages, is_catalog_visible
                 FROM products WHERE tenant_id = ?1 ORDER BY title ASC",
            )
            .bind(tenant_id)
            .fetch_all(pool)
            .await?;
            Ok(rows
                .into_iter()
                .map(|row| runtime_product_from_sqlite_row(&row))
                .collect())
        }
        DatabasePool::Postgres(pool) => {
            let rows = sqlx::query(
                "SELECT tenant_id, product_id, isbn, title, author, publisher, description,
                        public_title, public_author, public_publisher, public_description,
                        public_cover_image_url, category, vendor, cost_cents, retail_cents,
                        quantity_on_hand, cover_image_key, binding, pages, is_catalog_visible
                 FROM products WHERE tenant_id = $1 ORDER BY title ASC",
            )
            .bind(tenant_id)
            .fetch_all(pool)
            .await?;
            Ok(rows.into_iter().map(|row| runtime_product_from_postgres_row(&row)).collect())
        }
    }
}

pub async fn list_products_page(
    pool: &DatabasePool,
    tenant_id: &str,
    search: Option<&str>,
    category: Option<&str>,
    stock: Option<&str>,
    page: u32,
    per_page: u32,
) -> anyhow::Result<RuntimeProductPage> {
    let all_products = list_products(pool, tenant_id).await?;
    let search = search.unwrap_or_default().trim().to_lowercase();
    let category = category.unwrap_or("All").trim();
    let stock = stock.unwrap_or("All").trim();

    let total_products = all_products.len() as u32;

    let filtered: Vec<RuntimeProduct> = all_products
        .into_iter()
        .filter(|product| {
            if !matches!(category, "" | "All") && product.category != category {
                return false;
            }
            let status = inventory_status(product);
            if stock == "Low" && status != "low" {
                return false;
            }
            if stock == "Out" && status != "out" {
                return false;
            }
            if search.is_empty() {
                return true;
            }
            [
                product.product_id.as_str(),
                product.title.as_str(),
                product.category.as_str(),
                product.vendor.as_str(),
                product.isbn.as_str(),
                product.author.as_str(),
                product.publisher.as_str(),
            ]
            .iter()
            .any(|value| value.to_lowercase().contains(&search))
        })
        .collect();

    let total_matches = filtered.len() as u32;
    let retail_value_cents =
        filtered.iter().map(|product| product.retail_cents * product.quantity_on_hand).sum();
    let low_stock_count =
        filtered.iter().filter(|product| inventory_status(product) == "low").count() as u32;
    let out_of_stock_count =
        filtered.iter().filter(|product| inventory_status(product) == "out").count() as u32;

    let per_page = per_page.clamp(1, 100);
    let total_pages = total_matches.max(1).div_ceil(per_page);
    let page = page.clamp(1, total_pages);
    let start = ((page - 1) * per_page) as usize;
    let products = filtered.into_iter().skip(start).take(per_page as usize).collect();

    Ok(RuntimeProductPage {
        products,
        page,
        per_page,
        total_matches,
        total_pages,
        total_products,
        retail_value_cents,
        low_stock_count,
        out_of_stock_count,
    })
}

pub async fn list_catalog_books(pool: &DatabasePool, tenant_id: &str) -> anyhow::Result<Vec<RuntimeProduct>> {
    let mut products = list_products(pool, tenant_id).await?;
    products.retain(|product| product.is_catalog_visible);
    Ok(products)
}

pub async fn list_book_summaries(pool: &DatabasePool, tenant_id: &str) -> anyhow::Result<Vec<Book>> {
    let products = list_catalog_books(pool, tenant_id).await?;
    Ok(products
        .into_iter()
        .map(|product| Book {
            id: product.product_id,
            title: product.public_title.clone().if_empty_then(&product.title),
            author: product.public_author.clone().if_empty_then(&product.author),
            category: product.category,
            price_cents: product.retail_cents,
        })
        .collect())
}

trait StringFallback {
    fn if_empty_then(self, fallback: &str) -> String;
}

impl StringFallback for String {
    fn if_empty_then(self, fallback: &str) -> String {
        if self.trim().is_empty() { fallback.to_string() } else { self }
    }
}

pub async fn upsert_product(pool: &DatabasePool, product: RuntimeProduct) -> anyhow::Result<RuntimeProduct> {
    let mut product = product;
    let original_isbn = trim_owned(product.isbn);
    product.tenant_id = trim_owned(product.tenant_id);
    product.product_id = trim_owned(product.product_id);
    product.title = trim_owned(product.title);
    product.author = trim_owned(product.author);
    product.publisher = trim_owned(product.publisher);
    product.description = trim_owned(product.description);
    product.public_title = trim_owned(product.public_title);
    product.public_author = trim_owned(product.public_author);
    product.public_publisher = trim_owned(product.public_publisher);
    product.public_description = trim_owned(product.public_description);
    product.category = trim_owned(product.category);
    product.vendor = trim_owned(product.vendor);
    product.binding = trim_owned(product.binding);
    product.pages = trim_owned(product.pages);
    product.isbn = normalize_isbn(&original_isbn);
    if !original_isbn.is_empty() && product.isbn.is_empty() {
        anyhow::bail!("ISBN must contain digits.");
    }
    product.public_cover_image_url = normalize_opt(product.public_cover_image_url);
    product.cover_image_key = normalize_opt(product.cover_image_key);
    validate_product(&product)?;
    if !product.isbn.is_empty() {
        if let Some(existing) = get_product_by_isbn(pool, &product.tenant_id, &product.isbn).await? {
            if existing.product_id != product.product_id {
                anyhow::bail!(
                    "That ISBN already belongs to another product. Open the existing product to edit it instead."
                );
            }
        }
    }
    let now = timestamp_now();
    match pool {
        DatabasePool::Sqlite(pool) => {
            sqlx::query(
                "INSERT INTO products (
                    tenant_id, product_id, isbn, title, author, publisher, description,
                    public_title, public_author, public_publisher, public_description,
                    public_cover_image_url, category, vendor, cost_cents, retail_cents,
                    quantity_on_hand, cover_image_key, binding, pages, is_catalog_visible,
                    created_at, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23)
                 ON CONFLICT(tenant_id, product_id) DO UPDATE SET
                    isbn = excluded.isbn,
                    title = excluded.title,
                    author = excluded.author,
                    publisher = excluded.publisher,
                    description = excluded.description,
                    public_title = excluded.public_title,
                    public_author = excluded.public_author,
                    public_publisher = excluded.public_publisher,
                    public_description = excluded.public_description,
                    public_cover_image_url = excluded.public_cover_image_url,
                    category = excluded.category,
                    vendor = excluded.vendor,
                    cost_cents = excluded.cost_cents,
                    retail_cents = excluded.retail_cents,
                    cover_image_key = excluded.cover_image_key,
                    binding = excluded.binding,
                    pages = excluded.pages,
                    is_catalog_visible = excluded.is_catalog_visible,
                    updated_at = excluded.updated_at",
            )
            .bind(&product.tenant_id)
            .bind(&product.product_id)
            .bind(&product.isbn)
            .bind(&product.title)
            .bind(&product.author)
            .bind(&product.publisher)
            .bind(&product.description)
            .bind(&product.public_title)
            .bind(&product.public_author)
            .bind(&product.public_publisher)
            .bind(&product.public_description)
            .bind(product.public_cover_image_url.clone())
            .bind(&product.category)
            .bind(&product.vendor)
            .bind(product.cost_cents)
            .bind(product.retail_cents)
            .bind(product.quantity_on_hand)
            .bind(product.cover_image_key.clone())
            .bind(&product.binding)
            .bind(&product.pages)
            .bind(if product.is_catalog_visible { 1_i64 } else { 0_i64 })
            .bind(&now)
            .bind(&now)
            .execute(pool)
            .await?;
        }
        DatabasePool::Postgres(pool) => {
            sqlx::query(
                "INSERT INTO products (
                    tenant_id, product_id, isbn, title, author, publisher, description,
                    public_title, public_author, public_publisher, public_description,
                    public_cover_image_url, category, vendor, cost_cents, retail_cents,
                    quantity_on_hand, cover_image_key, binding, pages, is_catalog_visible,
                    created_at, updated_at
                 ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23)
                 ON CONFLICT(tenant_id, product_id) DO UPDATE SET
                    isbn = excluded.isbn,
                    title = excluded.title,
                    author = excluded.author,
                    publisher = excluded.publisher,
                    description = excluded.description,
                    public_title = excluded.public_title,
                    public_author = excluded.public_author,
                    public_publisher = excluded.public_publisher,
                    public_description = excluded.public_description,
                    public_cover_image_url = excluded.public_cover_image_url,
                    category = excluded.category,
                    vendor = excluded.vendor,
                    cost_cents = excluded.cost_cents,
                    retail_cents = excluded.retail_cents,
                    cover_image_key = excluded.cover_image_key,
                    binding = excluded.binding,
                    pages = excluded.pages,
                    is_catalog_visible = excluded.is_catalog_visible,
                    updated_at = excluded.updated_at",
            )
            .bind(&product.tenant_id)
            .bind(&product.product_id)
            .bind(&product.isbn)
            .bind(&product.title)
            .bind(&product.author)
            .bind(&product.publisher)
            .bind(&product.description)
            .bind(&product.public_title)
            .bind(&product.public_author)
            .bind(&product.public_publisher)
            .bind(&product.public_description)
            .bind(product.public_cover_image_url.clone())
            .bind(&product.category)
            .bind(&product.vendor)
            .bind(product.cost_cents)
            .bind(product.retail_cents)
            .bind(product.quantity_on_hand)
            .bind(product.cover_image_key.clone())
            .bind(&product.binding)
            .bind(&product.pages)
            .bind(product.is_catalog_visible)
            .bind(&now)
            .bind(&now)
            .execute(pool)
            .await?;
        }
    }
    get_product_by_id(pool, &product.tenant_id, &product.product_id)
        .await?
        .context("product missing after upsert")
}

pub async fn receive_inventory(
    pool: &DatabasePool,
    tenant_id: &str,
    isbn: &str,
    quantity: i64,
) -> anyhow::Result<i64> {
    adjust_inventory(pool, tenant_id, isbn, quantity, "receive").await
}

pub async fn adjust_inventory(
    pool: &DatabasePool,
    tenant_id: &str,
    isbn: &str,
    delta: i64,
    reason: &str,
) -> anyhow::Result<i64> {
    let isbn = normalize_isbn(isbn);
    let now = timestamp_now();
    match pool {
        DatabasePool::Sqlite(pool) => {
            sqlx::query(
                "UPDATE products SET quantity_on_hand = quantity_on_hand + ?1, updated_at = ?2
                 WHERE tenant_id = ?3 AND isbn = ?4",
            )
            .bind(delta)
            .bind(&now)
            .bind(tenant_id)
            .bind(&isbn)
            .execute(pool)
            .await?;
            sqlx::query(
                "INSERT INTO stock_movements (tenant_id, isbn, delta, reason, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            )
            .bind(tenant_id)
            .bind(&isbn)
            .bind(delta)
            .bind(reason)
            .bind(&now)
            .execute(pool)
            .await?;
            let on_hand: Option<i64> =
                sqlx::query_scalar("SELECT quantity_on_hand FROM products WHERE tenant_id = ?1 AND isbn = ?2")
                    .bind(tenant_id)
                    .bind(&isbn)
                    .fetch_optional(pool)
                    .await?;
            on_hand.context("product not found after inventory adjustment")
        }
        DatabasePool::Postgres(pool) => {
            sqlx::query(
                "UPDATE products SET quantity_on_hand = quantity_on_hand + $1, updated_at = $2
                 WHERE tenant_id = $3 AND isbn = $4",
            )
            .bind(delta)
            .bind(&now)
            .bind(tenant_id)
            .bind(&isbn)
            .execute(pool)
            .await?;
            sqlx::query(
                "INSERT INTO stock_movements (tenant_id, isbn, delta, reason, created_at) VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(tenant_id)
            .bind(&isbn)
            .bind(delta)
            .bind(reason)
            .bind(&now)
            .execute(pool)
            .await?;
            let on_hand: Option<i64> =
                sqlx::query_scalar("SELECT quantity_on_hand FROM products WHERE tenant_id = $1 AND isbn = $2")
                    .bind(tenant_id)
                    .bind(&isbn)
                    .fetch_optional(pool)
                    .await?;
            on_hand.context("product not found after inventory adjustment")
        }
    }
}

pub async fn list_stock_movements(
    pool: &DatabasePool,
    tenant_id: &str,
) -> anyhow::Result<Vec<RuntimeStockMovement>> {
    match pool {
        DatabasePool::Sqlite(pool) => {
            let rows = sqlx::query_as::<_, (String, String, i64, String)>(
                "SELECT tenant_id, isbn, delta, reason FROM stock_movements WHERE tenant_id = ?1 ORDER BY id DESC",
            )
            .bind(tenant_id)
            .fetch_all(pool)
            .await?;
            Ok(rows.into_iter().map(|row| RuntimeStockMovement {
                tenant_id: row.0,
                isbn: row.1,
                delta: row.2,
                reason: row.3,
            }).collect())
        }
        DatabasePool::Postgres(pool) => {
            let rows = sqlx::query_as::<_, (String, String, i64, String)>(
                "SELECT tenant_id, isbn, delta, reason FROM stock_movements WHERE tenant_id = $1 ORDER BY id DESC",
            )
            .bind(tenant_id)
            .fetch_all(pool)
            .await?;
            Ok(rows.into_iter().map(|row| RuntimeStockMovement {
                tenant_id: row.0,
                isbn: row.1,
                delta: row.2,
                reason: row.3,
            }).collect())
        }
    }
}

pub async fn delete_product(
    pool: &DatabasePool,
    tenant_id: &str,
    product_id: &str,
) -> anyhow::Result<()> {
    match pool {
        DatabasePool::Sqlite(pool) => {
            sqlx::query("DELETE FROM products WHERE tenant_id = ?1 AND product_id = ?2")
                .bind(tenant_id)
                .bind(product_id)
                .execute(pool)
                .await?;
        }
        DatabasePool::Postgres(pool) => {
            sqlx::query("DELETE FROM products WHERE tenant_id = $1 AND product_id = $2")
                .bind(tenant_id)
                .bind(product_id)
                .execute(pool)
                .await?;
        }
    }
    Ok(())
}

pub async fn list_categories(pool: &DatabasePool, tenant_id: &str) -> anyhow::Result<Vec<String>> {
    let mut values = match pool {
        DatabasePool::Sqlite(pool) => {
            sqlx::query_scalar::<_, String>(
                "SELECT DISTINCT category FROM products WHERE tenant_id = ?1 AND category <> '' ORDER BY category ASC",
            )
            .bind(tenant_id)
            .fetch_all(pool)
            .await?
        }
        DatabasePool::Postgres(pool) => {
            sqlx::query_scalar::<_, String>(
                "SELECT DISTINCT category FROM products WHERE tenant_id = $1 AND category <> '' ORDER BY category ASC",
            )
            .bind(tenant_id)
            .fetch_all(pool)
            .await?
        }
    };
    values.retain(|value| !value.trim().is_empty());
    Ok(values)
}

pub async fn list_vendors(pool: &DatabasePool, tenant_id: &str) -> anyhow::Result<Vec<String>> {
    let mut values = match pool {
        DatabasePool::Sqlite(pool) => {
            sqlx::query_scalar::<_, String>(
                "SELECT DISTINCT vendor FROM products WHERE tenant_id = ?1 AND vendor <> '' ORDER BY vendor ASC",
            )
            .bind(tenant_id)
            .fetch_all(pool)
            .await?
        }
        DatabasePool::Postgres(pool) => {
            sqlx::query_scalar::<_, String>(
                "SELECT DISTINCT vendor FROM products WHERE tenant_id = $1 AND vendor <> '' ORDER BY vendor ASC",
            )
            .bind(tenant_id)
            .fetch_all(pool)
            .await?
        }
    };
    values.retain(|value| !value.trim().is_empty());
    Ok(values)
}

pub async fn create_order(
    pool: &DatabasePool,
    tenant_id: &str,
    customer_name: &str,
    channel: OrderChannel,
    status: OrderStatus,
    payment_method: PaymentMethod,
    total_cents: i64,
    created_at: &str,
) -> anyhow::Result<RuntimeOrder> {
    let next_seq = match pool {
        DatabasePool::Sqlite(pool) => {
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM orders WHERE tenant_id = ?1")
                .bind(tenant_id)
                .fetch_one(pool)
                .await?
                + 1
        }
        DatabasePool::Postgres(pool) => {
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM orders WHERE tenant_id = $1")
                .bind(tenant_id)
                .fetch_one(pool)
                .await?
                + 1
        }
    };
    let order_id = format!("ORD-{}", 1000 + next_seq);
    match pool {
        DatabasePool::Sqlite(pool) => {
            sqlx::query(
                "INSERT INTO orders (tenant_id, order_id, customer_name, channel, status, payment_method, total_cents, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            )
            .bind(tenant_id)
            .bind(&order_id)
            .bind(customer_name)
            .bind(channel.as_str())
            .bind(status.as_str())
            .bind(payment_method.as_str())
            .bind(total_cents)
            .bind(created_at)
            .execute(pool)
            .await?;
        }
        DatabasePool::Postgres(pool) => {
            sqlx::query(
                "INSERT INTO orders (tenant_id, order_id, customer_name, channel, status, payment_method, total_cents, created_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            )
            .bind(tenant_id)
            .bind(&order_id)
            .bind(customer_name)
            .bind(channel.as_str())
            .bind(status.as_str())
            .bind(payment_method.as_str())
            .bind(total_cents)
            .bind(created_at)
            .execute(pool)
            .await?;
        }
    }
    Ok(RuntimeOrder {
        order_id,
        tenant_id: tenant_id.to_string(),
        customer_name: customer_name.to_string(),
        channel: channel.as_str().to_string(),
        status: status.as_str().to_string(),
        payment_method: payment_method.as_str().to_string(),
        total_cents,
        created_at: created_at.to_string(),
    })
}

pub async fn list_orders(pool: &DatabasePool, tenant_id: &str) -> anyhow::Result<Vec<RuntimeOrder>> {
    match pool {
        DatabasePool::Sqlite(pool) => {
            let rows = sqlx::query_as::<_, (String, String, String, String, String, String, i64, String)>(
                "SELECT order_id, tenant_id, customer_name, channel, status, payment_method, total_cents, created_at
                 FROM orders WHERE tenant_id = ?1 ORDER BY created_at DESC, order_id DESC",
            )
            .bind(tenant_id)
            .fetch_all(pool)
            .await?;
            Ok(rows.into_iter().map(|row| RuntimeOrder {
                order_id: row.0, tenant_id: row.1, customer_name: row.2, channel: row.3,
                status: row.4, payment_method: row.5, total_cents: row.6, created_at: row.7,
            }).collect())
        }
        DatabasePool::Postgres(pool) => {
            let rows = sqlx::query_as::<_, (String, String, String, String, String, String, i64, String)>(
                "SELECT order_id, tenant_id, customer_name, channel, status, payment_method, total_cents, created_at
                 FROM orders WHERE tenant_id = $1 ORDER BY created_at DESC, order_id DESC",
            )
            .bind(tenant_id)
            .fetch_all(pool)
            .await?;
            Ok(rows.into_iter().map(|row| RuntimeOrder {
                order_id: row.0, tenant_id: row.1, customer_name: row.2, channel: row.3,
                status: row.4, payment_method: row.5, total_cents: row.6, created_at: row.7,
            }).collect())
        }
    }
}

pub async fn mark_order_paid(
    pool: &DatabasePool,
    tenant_id: &str,
    order_id: &str,
) -> anyhow::Result<RuntimeOrder> {
    match pool {
        DatabasePool::Sqlite(pool) => {
            sqlx::query(
                "UPDATE orders SET status = 'Paid', payment_method = 'iou_settled' WHERE tenant_id = ?1 AND order_id = ?2",
            )
            .bind(tenant_id)
            .bind(order_id)
            .execute(pool)
            .await?;
            let row = sqlx::query_as::<_, (String, String, String, String, String, String, i64, String)>(
                "SELECT order_id, tenant_id, customer_name, channel, status, payment_method, total_cents, created_at
                 FROM orders WHERE tenant_id = ?1 AND order_id = ?2",
            )
            .bind(tenant_id)
            .bind(order_id)
            .fetch_one(pool)
            .await?;
            Ok(RuntimeOrder {
                order_id: row.0, tenant_id: row.1, customer_name: row.2, channel: row.3,
                status: row.4, payment_method: row.5, total_cents: row.6, created_at: row.7,
            })
        }
        DatabasePool::Postgres(pool) => {
            sqlx::query(
                "UPDATE orders SET status = 'Paid', payment_method = 'iou_settled' WHERE tenant_id = $1 AND order_id = $2",
            )
            .bind(tenant_id)
            .bind(order_id)
            .execute(pool)
            .await?;
            let row = sqlx::query_as::<_, (String, String, String, String, String, String, i64, String)>(
                "SELECT order_id, tenant_id, customer_name, channel, status, payment_method, total_cents, created_at
                 FROM orders WHERE tenant_id = $1 AND order_id = $2",
            )
            .bind(tenant_id)
            .bind(order_id)
            .fetch_one(pool)
            .await?;
            Ok(RuntimeOrder {
                order_id: row.0, tenant_id: row.1, customer_name: row.2, channel: row.3,
                status: row.4, payment_method: row.5, total_cents: row.6, created_at: row.7,
            })
        }
    }
}

pub async fn record_sales_event(
    pool: &DatabasePool,
    tenant_id: &str,
    payment_method: PaymentMethod,
    sales_cents: i64,
    donations_cents: i64,
    cogs_cents: i64,
    occurred_at: &str,
) -> anyhow::Result<()> {
    match pool {
        DatabasePool::Sqlite(pool) => {
            sqlx::query(
                "INSERT INTO sales_events (tenant_id, payment_method, sales_cents, donations_cents, cogs_cents, occurred_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )
            .bind(tenant_id)
            .bind(payment_method.as_str())
            .bind(sales_cents)
            .bind(donations_cents)
            .bind(cogs_cents)
            .bind(occurred_at)
            .execute(pool)
            .await?;
        }
        DatabasePool::Postgres(pool) => {
            sqlx::query(
                "INSERT INTO sales_events (tenant_id, payment_method, sales_cents, donations_cents, cogs_cents, occurred_at)
                 VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(tenant_id)
            .bind(payment_method.as_str())
            .bind(sales_cents)
            .bind(donations_cents)
            .bind(cogs_cents)
            .bind(occurred_at)
            .execute(pool)
            .await?;
        }
    }
    Ok(())
}

pub async fn report_summary(
    pool: &DatabasePool,
    tenant_id: &str,
    from: Option<&str>,
    to: Option<&str>,
) -> anyhow::Result<RuntimeReportSummary> {
    let rows = match pool {
        DatabasePool::Sqlite(pool) => {
            let mut query = String::from(
                "SELECT payment_method, COALESCE(SUM(sales_cents), 0), COALESCE(SUM(donations_cents), 0), COALESCE(SUM(cogs_cents), 0)
                 FROM sales_events WHERE tenant_id = ?1",
            );
            if from.is_some() {
                query.push_str(" AND occurred_at >= ?2");
            }
            if to.is_some() {
                query.push_str(if from.is_some() { " AND occurred_at < ?3" } else { " AND occurred_at < ?2" });
            }
            query.push_str(" GROUP BY payment_method ORDER BY payment_method");
            let mut built = sqlx::query_as::<_, (String, i64, i64, i64)>(&query).bind(tenant_id);
            if let Some(from) = from {
                built = built.bind(from);
            }
            if let Some(to) = to {
                built = built.bind(to);
            }
            built.fetch_all(pool).await?
        }
        DatabasePool::Postgres(pool) => {
            let mut query = String::from(
                "SELECT payment_method, COALESCE(SUM(sales_cents), 0)::BIGINT, COALESCE(SUM(donations_cents), 0)::BIGINT, COALESCE(SUM(cogs_cents), 0)::BIGINT
                 FROM sales_events WHERE tenant_id = $1",
            );
            if from.is_some() {
                query.push_str(" AND occurred_at >= $2");
            }
            if to.is_some() {
                query.push_str(if from.is_some() { " AND occurred_at < $3" } else { " AND occurred_at < $2" });
            }
            query.push_str(" GROUP BY payment_method ORDER BY payment_method");
            let mut built = sqlx::query_as::<_, (String, i64, i64, i64)>(&query).bind(tenant_id);
            if let Some(from) = from {
                built = built.bind(from);
            }
            if let Some(to) = to {
                built = built.bind(to);
            }
            built.fetch_all(pool).await?
        }
    };
    let sales_cents: i64 = rows.iter().map(|row| row.1).sum();
    let donations_cents: i64 = rows.iter().map(|row| row.2).sum();
    let cogs_cents: i64 = rows.iter().map(|row| row.3).sum();
    Ok(RuntimeReportSummary {
        tenant_id: tenant_id.to_string(),
        sales_cents,
        donations_cents,
        cogs_cents,
        gross_profit_cents: sales_cents - cogs_cents,
        sales_by_payment: rows.into_iter().map(|row| (row.0, row.1)).collect(),
    })
}

pub async fn list_pos_quick_items(pool: &DatabasePool) -> anyhow::Result<Vec<SeedQuickItem>> {
    match pool {
        DatabasePool::Sqlite(pool) => {
            let rows = sqlx::query_as::<_, (String, String, String, String, i64, i64, String)>(
                "SELECT item_id, title, emoji, price_label, price_cents, stock_on_hand, note
                 FROM pos_quick_items ORDER BY sort_order ASC, item_id ASC",
            )
            .fetch_all(pool)
            .await?;
            Ok(rows.into_iter().map(|row| SeedQuickItem {
                item_id: row.0, title: row.1, emoji: row.2, price_label: row.3,
                price_cents: row.4, stock_on_hand: row.5, note: row.6,
            }).collect())
        }
        DatabasePool::Postgres(pool) => {
            let rows = sqlx::query_as::<_, (String, String, String, String, i64, i64, String)>(
                "SELECT item_id, title, emoji, price_label, price_cents, stock_on_hand, note
                 FROM pos_quick_items ORDER BY sort_order ASC, item_id ASC",
            )
            .fetch_all(pool)
            .await?;
            Ok(rows.into_iter().map(|row| SeedQuickItem {
                item_id: row.0, title: row.1, emoji: row.2, price_label: row.3,
                price_cents: row.4, stock_on_hand: row.5, note: row.6,
            }).collect())
        }
    }
}

pub async fn list_pos_discount_codes(pool: &DatabasePool) -> anyhow::Result<Vec<SeedDiscountCode>> {
    match pool {
        DatabasePool::Sqlite(pool) => {
            let rows = sqlx::query_as::<_, (String, String, f64)>(
                "SELECT code, label, rate FROM pos_discount_codes ORDER BY code ASC",
            )
            .fetch_all(pool)
            .await?;
            Ok(rows.into_iter().map(|row| SeedDiscountCode { code: row.0, label: row.1, rate: row.2 }).collect())
        }
        DatabasePool::Postgres(pool) => {
            let rows = sqlx::query_as::<_, (String, String, f64)>(
                "SELECT code, label, rate FROM pos_discount_codes ORDER BY code ASC",
            )
            .fetch_all(pool)
            .await?;
            Ok(rows.into_iter().map(|row| SeedDiscountCode { code: row.0, label: row.1, rate: row.2 }).collect())
        }
    }
}
