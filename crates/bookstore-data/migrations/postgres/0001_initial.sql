CREATE TABLE IF NOT EXISTS tenants (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    default_locale TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO tenants (id, name, default_locale)
VALUES ('default', 'Default Bookshop', 'en-AU')
ON CONFLICT (id) DO NOTHING;
