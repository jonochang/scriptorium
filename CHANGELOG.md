# Changelog

## [0.3.0] - 2026-03-07

### Added
- Direct add-to-cart actions on catalog cards, quantity-aware product detail adds, related-title cross-sell, and richer browser-cart feedback across the storefront.
- Storefront checkout confirmation state with optional parish-support amount before session creation.
- Admin dashboard payment-breakdown reporting, order filtering, inventory-journal visibility, and dashboard snapshot export controls.

### Changed
- Storefront shells now land closer to screens 5-7 in `design-ux.jsx`, including stronger merchandising and confirmation flow continuity.
- Admin shells now land closer to screens 8-10 in `design-ux.jsx`, with denser reporting and operations tooling on top of live APIs.
- Shared palette-backed UI tokens and layout patterns are applied more consistently across POS, storefront, and admin shells.

### Fixed
- POS shell render regression caused by inline template-literal backticks on `/pos`.
- Storefront product-detail 404 responses now return a friendly shell instead of an empty body.
- Admin login no longer relies on a placeholder-looking username field.
- Cart recommendations no longer repeat titles already present in the basket.

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
