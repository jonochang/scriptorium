# Changelog

## [0.2.0] - 2026-03-06

### Added
- Live admin dashboard shell with report summary, product/category/vendor loading, recent orders, open IOUs, low-stock spotlight, and date-window controls.
- Storefront product detail, cart, checkout, and category browsing shells tied into browser cart persistence and live checkout-session creation.
- POS shell rebuilt into a 4-step flow: PIN login, basket/scan screen, payment selection, and sale completion.
- Deployment documentation and SQLite backup/restore runbook.
- Basic observability fields for checkout/session/webhook paths.
- CSRF rejection coverage for cross-origin admin writes.

### Changed
- Catalog search now works with HTMX enhancement and plain `/catalog?q=...` fallback.
- Storefront and admin pages now use the shared palette/fonts/tokens from the design spec more consistently.
- POS responses are rendered as user-facing status, cart, and payment outcomes rather than raw JSON.
- Admin order and IOU workflows are exposed through the dashboard and supporting APIs.

### Fixed
- Empty-cart POS checkout is rejected instead of completing a zero-dollar sale.
- POS failures return structured JSON errors instead of empty `400` responses.
- POS scan requests accept both `isbn` and `barcode`.
- Hardcoded admin credentials were removed from the intake HTML shell.
