CREATE TABLE IF NOT EXISTS order_line_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id TEXT NOT NULL,
    revenue_cents INTEGER NOT NULL,
    cost_cents INTEGER NOT NULL,
    currency TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_order_line_snapshots_tenant
    ON order_line_snapshots(tenant_id);
