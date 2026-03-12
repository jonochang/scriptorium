CREATE TABLE IF NOT EXISTS order_line_snapshots (
    id BIGSERIAL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    revenue_cents BIGINT NOT NULL,
    cost_cents BIGINT NOT NULL,
    currency TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_order_line_snapshots_tenant
    ON order_line_snapshots(tenant_id);
