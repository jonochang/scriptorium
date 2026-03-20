# Changelog

## [0.4.12] - 2026-03-20

### Added
- Centralized `db/seed.toml` seed data system (Rails-style `db/seed.rb` equivalent) replacing hardcoded data scattered across crates.
- `SeedData` module with TOML deserialization, `include_str!` compiled-in defaults, and runtime override via `SEED_FILE` env var.
- `/api/pos/config` endpoint serving POS quick items and discount codes dynamically to the WASM client.
- Admin Orders page View button now shows an inline detail panel with order fields.

### Fixed
- ISBN lookup on the admin intake page no longer returns "unknown book" for known products when Open Library is unavailable.
- Dashboard "Done" buttons now dismiss follow-up items.
- Dashboard "Save note" button clears the textarea with visual confirmation.
- "Resend" receipt button opens the order in a new tab instead of only showing a status message.
- "Reorder" button navigates to the intake page instead of only showing a status message.

### Changed
- Removed `seed_church_bookstore()` from bookstore-core; all catalog/POS/admin fixture data now lives in `db/seed.toml`.
- Catalog product details (blurb, publisher, ISBN, binding, pages, cover symbol, stock hint) moved from hardcoded match arms to seed data.
- POS payment controllers use `RequestContext` tenant instead of hardcoded "church-a".
- Admin bootstrap credentials and tenant ID now read from seed defaults.
- WASM POS client fetches config dynamically instead of using hardcoded constants.

## [0.4.11] - 2026-03-19

### Fixed
- Increase all font sizes for elderly readability: no body text below 13px. Uppercase eyebrow/metric labels at 12px minimum.
- Verified POS payment screen (card/cash/IOU selection, cash tender grid, round-up donation toggle).
- Verified POS transaction-complete screen (receipt card, email receipt, start-next-sale flow).

### Changed
- Updated design-ux-review.md: spec parity now ~96% (up from ~92%).

## [0.4.10] - 2026-03-19

### Fixed
- Hide "Add" button on catalog cards for out-of-stock items (bk-105).
- Remove dev-facing "for tenant church-a." text from the intake page subtitle.
- Add +/− quantity stepper buttons on the product detail page.

### Changed
- Updated design-ux-review.md: spec parity now ~92% (up from ~88%).

## [0.4.9] - 2026-03-19

### Changed
- Redesigned the entire UI to the new design spec: Source Serif 4 / Source Sans 3 / JetBrains Mono font stack replacing Crimson Pro and DM Sans.
- Implemented zone-aware navigation: StorefrontNav (Catalog, Cart, Checkout + dim Admin), AdminNav (Dashboard, Orders, Intake + dim Store/POS/Sign out with ADMIN badge), and GatewayNav (logo + Back to store) for the sign-in page.
- Centered catalog hero header with "Feed your soul." headline and no eyebrow text.
- Rebuilt admin sign-in page with a dedicated gateway layout, login card with privacy footer, and POS terminal launch card.
- Updated brand mark to ✝ cross and all-caps SCRIPTORIUM across every zone.
- Styled dashboard filter pills (All/POS/Online/IOU) as proper pill buttons matching the FilterPills design component.
- Added "Back to catalog" breadcrumb link with chevron icon on the product detail page and the product-not-found 404 page.
- Updated admin footer to show zone-specific links (Dashboard, Orders, Intake) instead of all six links.

### Fixed
- Intake page no longer shows storefront navigation; it now uses the admin-zone nav pattern with ADMIN badge.
- Removed duplicate footer on the intake page caused by both inline HTML footer and `site_footer()` call.
- Date inputs on the dashboard and orders pages now render inline instead of expanding to full width.
- Intake nav separator no longer renders as a visible styled circle.
- Updated Google Fonts import on POS page to load the new font families.

## [0.4.8] - 2026-03-17

### Changed
- Polished CSS spacing across all pages for a more professional appearance.
- Tightened page header padding, catalog search margins, and results header alignment.
- Improved checkout layout with equal-width columns and better trust section spacing.
- Balanced admin dashboard tab spacing, orders footnote separator, and intake nav link styling.
- Matched topbar height (56px) consistently across admin and intake pages.
- Increased POS PIN screen vertical spacing for better visual rhythm.

### Fixed
- Cart page total and action buttons now render inside the cart items panel instead of the recommendations panel.
- Removed redundant shelf label and notice text from catalog page.
- Eliminated duplicate author line on product detail page.
- Replaced developer-facing placeholder text on POS screen with clean user-facing copy.

## [0.4.7] - 2026-03-17

### Changed
- Removed ghost-link action buttons from all storefront page headers (catalog, product detail, cart, checkout) for a cleaner layout.
- Demoted "Clear basket" from accent button to ghost link so it no longer competes visually with the checkout call-to-action.
- Removed duplicate "Pilgrim note" panel from product detail and "Gift-table guidance" panel from cart.
- Removed duplicate heading and lead paragraph from checkout, keeping only the step indicator.
- Removed "ADMIN FOCUS" eyebrow text from admin dashboard and orders pages.
- Removed duplicate "Export" button from orders page header (toolbar copy remains).
- Removed duplicate Dashboard/Add product links from orders page header actions.
- Footer now shows all six navigation links (Catalog, Cart, Checkout, Dashboard, Orders, Intake) on every page including admin intake and template-based admin pages.

### Fixed
- Checkout card panel now hides correctly when the `[hidden]` attribute is set; a CSS specificity issue where `.checkout-card { display: grid }` overrode `display: none` was resolved with `!important`.
- Checkout form fields no longer ship with pre-filled placeholder values ("Jane Parishioner" / "jane@example.com"), letting real validation run on first submit.
- Browser E2E checkout test now fills name and email before advancing to payment step, matching the new empty-field behavior.

## [0.4.6] - 2026-03-17

### Changed
- Unified the top navigation bar so every page shows the same consistent set of links: Catalog, Cart, Checkout, Dashboard, Orders, Intake. Previously the links changed depending on which page you were on.
- Removed decorative header badges (buzzword pills like "Parish bookshop", "Protected access", "Reader favorite") from all page headers across admin and storefront views.
- Removed duplicate stock status badge on the product detail page, keeping only the one in the price row.

## [0.4.5] - 2026-03-17

### Changed
- Migrated all remaining inline JavaScript to typed Rust/WASM using wasm-bindgen and web-sys, completing the five-phase migration that eliminates Preact, HTM, and vanilla JS dependencies.
- POS point-of-sale system (Phase 5) now renders entirely from WASM instead of Preact+HTM, covering PIN login, barcode scan, quick items, basket management, three payment methods (card/cash/IOU), discount codes, and receipt display.
- Storefront cart, checkout, admin dashboard, and admin intake modules (Phases 1–4) were migrated in earlier commits within this release cycle.
- Browser E2E tests updated to wait for WASM initialization before interacting with POS screens.
- BDD smoke test for POS updated to assert WASM loader presence instead of inline JS content.

### Removed
- All inline JavaScript from server-rendered HTML across storefront, admin, and POS surfaces (~1,500 lines total across five phases).
- Preact 10.25.4 and HTM 3.1.1 CDN imports from POS shell.

## [0.4.4] - 2026-03-13

### Added
- Multi-step storefront checkout coverage in browser E2E, including summary recalculation, delivery/support adjustments, and formatted card entry.
- Checkout-shell unit coverage that pins the release-specific CSS selectors used by the redesigned flow.

### Changed
- Storefront checkout now matches the latest redesign with explicit details and payment steps, stronger order-summary treatment, and parish-support quick picks.
- Checkout client logic is split into smaller render and validation helpers while preserving the existing `/api/storefront/checkout/session` contract.

## [0.4.3] - 2026-03-11

### Added
- S3-compatible cover upload support for admin intake, with uploaded assets served back through `/media/...`.
- MinIO-backed storage integration coverage plus a mock-backed ISBN provider integration test for `bookstore-web`.
- External ISBN metadata lookup support using Open Library in the live app, with automatic cover preview when metadata includes artwork.

### Changed
- Admin web shells now require a real sign-in flow before exposing dashboard, orders, or intake surfaces.
- POS login now includes a working PIN-help state instead of a dead "Forgot PIN?" link.
- The Nix dev shell now includes MinIO and avoids the Linux-only Chromium dependency on Darwin hosts.

### Fixed
- Admin intake no longer depends on inline auth fields inside the protected intake page.
- ISBN lookup now works against a free online provider in live environments while retaining deterministic fallback behavior for tests and offline use.
- Cover upload is no longer a placeholder-only UI element.

## [0.4.2] - 2026-03-10

### Added
- Service BDD coverage for POS discount/reporting behavior and explicit cash-underpayment rejection.
- Browser E2E coverage for POS discounted payment totals, admin intake opening-stock receipt, and admin dashboard payment-breakdown/low-stock rendering.
- App-level unit coverage for discounted POS card checkout.

### Changed
- Storefront checkout session creation is now server-authoritative: the browser submits line items and donation, while the backend computes sales, tax, shipping, and final totals.
- POS discounts now affect the live payment amount, order total, admin reporting, and completion receipt instead of remaining cosmetic UI-only state.
- Admin intake save now performs the opening stock receive step after product upsert so the operator-facing on-hand message reflects real state.

### Fixed
- Payment webhooks now create online orders and sales events with the correct tenant and non-zero totals.
- POS underpayment is rejected without clearing the basket or mutating stock.
- POS stock enforcement now blocks oversell for scans, quick items, quantity changes, paid sales, and IOU checkout.
- Admin product/report payloads now match the dashboard’s expected shapes for `quantity_on_hand` and `sales_by_payment`.

## [0.4.1] - 2026-03-09

### Added
- `chromiumoxide` browser E2E coverage for the highest-risk UX flows: catalog add-to-cart, cart recommendation hydration, admin login/data loading, and POS sale completion.
- Explicit test-suite separation for `bookstore-web` between `service` BDD coverage and `browser` E2E coverage, including in-repo test layout documentation.
- Additional live POS quick-item coverage and quantity-adjustment support for the volunteer basket flow.

### Changed
- `bookstore-web` test layout now uses `tests/service`, `tests/browser`, and `tests/features/service` so service-level and browser-level checks are organized by intent instead of mixed together.
- Service feature files are grouped by product surface: foundation, domain, POS, storefront, admin, and hardening.
- Storefront, admin, and POS copy/treatment were tightened further to close the latest UX-review gaps.

### Fixed
- POS basket controls now support real quantity changes instead of static display.
- Product-detail shelf copy now uses title-specific reader blurbs instead of developer-facing placeholder text.
- Shared storefront navigation now reflects live cart count state during browser flows.

## [0.4.0] - 2026-03-09

### Added
- Expanded seeded storefront data with a broader catalog, more categories, and a stable feature-detail route for richer browser demos.
- Storefront pagination, stock-status badges, richer product metadata, and a more complete checkout form with shipping/contact capture and cost breakdowns.
- Admin intake save flow with pricing, stock, category, and vendor controls, plus clearer lookup and scanner status feedback.
- POS completion receipt capture, discount-selection UI, and stronger screen polish for the volunteer flow.

### Changed
- Storefront, POS, and admin shells now align more closely with the latest UX review, including user-facing copy, shared navigation/footer treatment, and denser dashboard presentation.
- Admin reporting and order views now render with more structured tables and metric treatments instead of simple card lists.
- Barcode scanning on intake now degrades more safely with explicit capability detection, start/stop controls, and manual-entry fallback.

### Fixed
- Catalog add-to-cart actions now work directly from `/catalog`.
- Admin dashboard JavaScript no longer crashes on load from the raw-string escaping bug.
- Cart recommendations are filtered against the hydrated browser cart.
- POS payment options render with readable label/description spacing.

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
