CREATE TABLE IF NOT EXISTS products (
    tenant_id TEXT NOT NULL,
    product_id TEXT NOT NULL,
    isbn TEXT NOT NULL DEFAULT '',
    title TEXT NOT NULL,
    author TEXT NOT NULL DEFAULT '',
    publisher TEXT NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    public_title TEXT NOT NULL DEFAULT '',
    public_author TEXT NOT NULL DEFAULT '',
    public_publisher TEXT NOT NULL DEFAULT '',
    public_description TEXT NOT NULL DEFAULT '',
    public_cover_image_url TEXT,
    category TEXT NOT NULL DEFAULT '',
    vendor TEXT NOT NULL DEFAULT '',
    cost_cents INTEGER NOT NULL DEFAULT 0,
    retail_cents INTEGER NOT NULL DEFAULT 0,
    quantity_on_hand INTEGER NOT NULL DEFAULT 0,
    cover_image_key TEXT,
    binding TEXT NOT NULL DEFAULT '',
    pages TEXT NOT NULL DEFAULT '',
    is_catalog_visible INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (tenant_id, product_id)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_products_tenant_isbn
    ON products (tenant_id, isbn)
    WHERE isbn <> '';

CREATE INDEX IF NOT EXISTS idx_products_tenant_visible
    ON products (tenant_id, is_catalog_visible, category, title);

CREATE TABLE IF NOT EXISTS stock_movements (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id TEXT NOT NULL,
    isbn TEXT NOT NULL,
    delta INTEGER NOT NULL,
    reason TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_stock_movements_tenant_created
    ON stock_movements (tenant_id, created_at DESC);

CREATE TABLE IF NOT EXISTS orders (
    tenant_id TEXT NOT NULL,
    order_id TEXT NOT NULL,
    customer_name TEXT NOT NULL,
    channel TEXT NOT NULL,
    status TEXT NOT NULL,
    payment_method TEXT NOT NULL,
    total_cents INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY (tenant_id, order_id)
);

CREATE INDEX IF NOT EXISTS idx_orders_tenant_created
    ON orders (tenant_id, created_at DESC);

CREATE TABLE IF NOT EXISTS sales_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id TEXT NOT NULL,
    payment_method TEXT NOT NULL,
    sales_cents INTEGER NOT NULL,
    donations_cents INTEGER NOT NULL,
    cogs_cents INTEGER NOT NULL,
    occurred_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_sales_events_tenant_occurred
    ON sales_events (tenant_id, occurred_at DESC);

CREATE TABLE IF NOT EXISTS pos_quick_items (
    item_id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    emoji TEXT NOT NULL DEFAULT '',
    price_label TEXT NOT NULL DEFAULT '',
    price_cents INTEGER NOT NULL,
    stock_on_hand INTEGER NOT NULL DEFAULT 0,
    note TEXT NOT NULL DEFAULT '',
    sort_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS pos_discount_codes (
    code TEXT PRIMARY KEY,
    label TEXT NOT NULL,
    rate REAL NOT NULL
);
