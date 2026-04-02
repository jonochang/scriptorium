# Database Audit

Date: 2026-03-26

## Summary

The application is not using the SQL database as the source of truth for most product, inventory, catalog, POS, and order flows.

Current reality:

- The SQL database is initialized at startup and stored in `AppState.db_pool`.
- The SQL database is currently used for health/readiness checks and profit-report snapshot storage.
- The public catalog is loaded from seed data into an in-memory service.
- Admin inventory/products/orders are stored in an in-memory `HashMap` service.
- POS catalog/session state is stored in an in-memory `HashMap` service, with partial sync from admin.
- Several screens appear to share data, but they are doing so through in-memory bridging, not through a shared database table.

This means the product/inventory data model is currently split across:

- seeded data
- in-memory admin state
- in-memory POS state
- a small amount of SQL persistence unrelated to catalog/inventory

## What The Database Actually Backs Today

### Database-backed

- `/health` readiness dependency
  - `crates/bookstore-web/src/controllers/health.rs`
  - `ready(...)` checks `state.db_pool`
- `bookstore_data::bootstrap_database(...)`
  - `crates/bookstore-data/src/lib.rs`
  - runs SQL migrations for SQLite/Postgres
- profit report snapshot repository
  - `crates/bookstore-data/src/lib.rs`
  - persists `order_line_snapshots`

### Not database-backed

- public catalog
- product detail pages
- cart pricing lookup
- admin products
- admin inventory
- admin taxonomy lists
- admin orders
- admin reporting summary
- POS barcode catalog
- POS quick items
- POS sessions/carts

## Source Of Truth By Service

### `CatalogService`

Code:

- [crates/bookstore-app/src/lib.rs](/Users/jonochang/projects/lib/jc/scriptorium/crates/bookstore-app/src/lib.rs)
- `CatalogService::from_seed(...)`

Behavior:

- Builds inventory from `seed.catalog.books`
- Stores it in `Arc<RwLock<Inventory>>`
- No SQL reads or writes

Conclusion:

- Catalog is seeded and memory-backed, not database-backed.

### `AdminService`

Code:

- [crates/bookstore-app/src/lib.rs](/Users/jonochang/projects/lib/jc/scriptorium/crates/bookstore-app/src/lib.rs)
- `AdminStore`
- `AdminService::with_bootstrap_and_seed(...)`

Behavior:

- Stores users, auth sessions, products, inventory, stock movements, sales events, and orders in in-memory collections
- Product records live in `HashMap<(tenant_id, product_id), AdminProduct>`
- Inventory levels live in `HashMap<(tenant_id, isbn), i64>`
- Orders live in `Vec<AdminOrder>`
- No SQL reads or writes

Conclusion:

- Admin products, inventory, and orders are not database-backed.

### `PosService`

Code:

- [crates/bookstore-app/src/lib.rs](/Users/jonochang/projects/lib/jc/scriptorium/crates/bookstore-app/src/lib.rs)
- `PosService::from_seed(...)`

Behavior:

- Loads barcode items and quick items from seed
- Stores barcode catalog, quick items, and active sessions in memory
- Later receives sync updates from admin APIs and POS fallback paths
- No SQL reads or writes

Conclusion:

- POS is memory-backed, seeded at startup, and only partially synchronized with admin.

### `StorefrontService`

Code:

- [crates/bookstore-app/src/lib.rs](/Users/jonochang/projects/lib/jc/scriptorium/crates/bookstore-app/src/lib.rs)

Behavior:

- Uses in-memory session state for checkout sessions
- No SQL reads or writes

Conclusion:

- Storefront checkout session state is not database-backed.

## View Audit

## Public Storefront Views

| Route | Backing source | Uses DB? | Notes |
| --- | --- | --- | --- |
| `/catalog` | `state.catalog.list_books()` + `state.seed` | No | Catalog items come from seed-loaded `CatalogService`; metadata rendering still reads directly from `SeedData`. |
| `/catalog/search` | `state.catalog.list_books()` + `state.seed` | No | Same as catalog page. |
| `/catalog/items/{book_id}` | `state.catalog.list_books()` + `state.seed` | No | Detail page metadata such as blurb/publisher/isbn/binding/pages comes from `SeedData`. |
| `/cart` | HTML shell + `state.catalog.list_books()` for recommendations | No | Recommendation list comes from seeded catalog. Cart itself is client-side state. |
| `/checkout` | static shell | No | View itself does not read DB. |
| `/orders` | `state.admin.list_orders(...)` | No | Storefront order history reads admin in-memory orders. |

### Evidence

- [crates/bookstore-web/src/controllers/storefront.rs](/Users/jonochang/projects/lib/jc/scriptorium/crates/bookstore-web/src/controllers/storefront.rs)
- [crates/bookstore-web/src/catalog_ui.rs](/Users/jonochang/projects/lib/jc/scriptorium/crates/bookstore-web/src/catalog_ui.rs)
- [crates/bookstore-web/src/main.rs](/Users/jonochang/projects/lib/jc/scriptorium/crates/bookstore-web/src/main.rs)

## Admin Views

| Route | Backing source | Uses DB? | Notes |
| --- | --- | --- | --- |
| `/admin` | shell only, data later fetched from admin APIs | No | Dashboard shell is server-rendered; live data comes from in-memory admin APIs. |
| `/admin/orders` | shell only, data later fetched from admin APIs | No | Orders page is not SQL-backed. |
| `/admin/inventory` | shell only, data later fetched from admin APIs | No | Inventory page uses admin product/inventory APIs backed by memory. |
| `/admin/intake` | shell only, ISBN lookup + save via admin APIs | No | Product lookup/save uses in-memory admin store plus external ISBN API. |

### Evidence

- [crates/bookstore-web/src/controllers/admin_pages.rs](/Users/jonochang/projects/lib/jc/scriptorium/crates/bookstore-web/src/controllers/admin_pages.rs)
- [crates/bookstore-web/src/controllers/admin_api.rs](/Users/jonochang/projects/lib/jc/scriptorium/crates/bookstore-web/src/controllers/admin_api.rs)
- [crates/bookstore-app/src/lib.rs](/Users/jonochang/projects/lib/jc/scriptorium/crates/bookstore-app/src/lib.rs)

## POS Views

| Route | Backing source | Uses DB? | Notes |
| --- | --- | --- | --- |
| `/pos` | static shell + POS APIs | No | Page shell is static HTML. |
| POS scan/cart/payment flows | `state.pos` with fallback to `state.admin` | No | POS catalog is memory-backed; fallback bridges to in-memory admin product store. |

### Evidence

- [crates/bookstore-web/src/controllers/pos.rs](/Users/jonochang/projects/lib/jc/scriptorium/crates/bookstore-web/src/controllers/pos.rs)
- [crates/bookstore-app/src/lib.rs](/Users/jonochang/projects/lib/jc/scriptorium/crates/bookstore-app/src/lib.rs)

## API Audit

These are not page views, but they determine whether the views use SQL in practice.

| API | Backing source | Uses DB? | Notes |
| --- | --- | --- | --- |
| `GET /books` | `state.catalog.list_books()` | No | Seeded in-memory catalog. |
| `POST /api/admin/products/isbn-lookup` | `state.admin.lookup_isbn(...)` + remote ISBN API | No | Saved data from in-memory admin store. |
| `POST /api/admin/products` | `state.admin.upsert_product(...)` | No | Writes only to in-memory admin store. |
| `GET /api/admin/products` | `state.admin.list_products(...)` | No | Reads only from in-memory admin store. |
| `POST /api/admin/inventory/receive` | `state.admin.receive_inventory(...)` | No | Updates in-memory inventory, then syncs POS memory. |
| `POST /api/admin/inventory/adjust` | `state.admin.adjust_inventory(...)` | No | Updates in-memory inventory, then syncs POS memory. |
| `GET /api/admin/inventory/journal` | `state.admin.movement_journal(...)` | No | Reads in-memory stock movement list. |
| `GET /api/admin/categories` | `state.admin.list_categories(...)` | No | Derived from in-memory products. |
| `GET /api/admin/vendors` | `state.admin.list_vendors(...)` | No | Derived from in-memory products. |
| `GET /api/admin/orders` | `state.admin.list_orders(...)` | No | Reads in-memory orders. |
| `POST /api/admin/orders/{order_id}/mark-paid` | `state.admin.mark_order_paid(...)` | No | Mutates in-memory orders. |
| `GET /api/admin/reports/summary` | `state.admin.report_summary_range(...)` | No | Reads in-memory sales events. |
| `POST /api/pos/scan` | `state.pos.scan_item(...)`, fallback `state.admin.product_by_isbn(...)` | No | Memory-backed POS with memory-backed admin fallback. |
| `GET /api/pos/config` | `state.seed.pos.quick_items`, `state.seed.pos.discount_codes` | No | Seed-backed POS config. |
| `POST /api/storefront/checkout/session` | `state.catalog`, `state.storefront`, `state.admin` | No | Prices from seeded catalog; session/order/sales event all memory-backed. |

## Specific Split-Brain Problems

### 1. Public catalog does not come from admin inventory

Proof:

- `AppState` is initialized with `CatalogService::from_seed(&seed)`
  - [crates/bookstore-web/src/main.rs](/Users/jonochang/projects/lib/jc/scriptorium/crates/bookstore-web/src/main.rs)
- `/catalog` reads from `state.catalog.list_books().await`
  - [crates/bookstore-web/src/controllers/storefront.rs](/Users/jonochang/projects/lib/jc/scriptorium/crates/bookstore-web/src/controllers/storefront.rs)
- catalog metadata helpers read from `state.seed`
  - [crates/bookstore-web/src/catalog_ui.rs](/Users/jonochang/projects/lib/jc/scriptorium/crates/bookstore-web/src/catalog_ui.rs)

Impact:

- Products saved in Admin Intake do not automatically become storefront catalog items.
- Storefront blurbs/publisher/isbn/stock badges can diverge from admin-edited product data.

### 2. Admin inventory is not persisted

Proof:

- `AdminStore` uses in-memory collections
  - [crates/bookstore-app/src/lib.rs](/Users/jonochang/projects/lib/jc/scriptorium/crates/bookstore-app/src/lib.rs)
- `AdminService` methods mutate `HashMap`/`Vec` state only

Impact:

- Product/inventory/order changes are process-local unless separately projected elsewhere.
- Restart safety depends on seed/bootstrap behavior rather than persistence.

### 3. POS is not using a shared product table

Proof:

- `PosService::from_seed(...)` loads barcode and quick items from seed
  - [crates/bookstore-app/src/lib.rs](/Users/jonochang/projects/lib/jc/scriptorium/crates/bookstore-app/src/lib.rs)
- `pos_scan(...)` first tries POS memory, then falls back to admin memory
  - [crates/bookstore-web/src/controllers/pos.rs](/Users/jonochang/projects/lib/jc/scriptorium/crates/bookstore-web/src/controllers/pos.rs)

Impact:

- POS and admin can appear synchronized while still not sharing one persisted table.
- POS quick items and discount codes are still seed-backed.

### 4. Storefront order history is reading admin memory, not a persisted order table

Proof:

- `/orders` calls `state.admin.list_orders(...)`
  - [crates/bookstore-web/src/controllers/storefront.rs](/Users/jonochang/projects/lib/jc/scriptorium/crates/bookstore-web/src/controllers/storefront.rs)
- `AdminService` stores orders in `Vec<AdminOrder>`
  - [crates/bookstore-app/src/lib.rs](/Users/jonochang/projects/lib/jc/scriptorium/crates/bookstore-app/src/lib.rs)

Impact:

- Customer-visible order history is not durable.

## Views That Do Use The Database Indirectly

Only these paths visibly depend on SQL today:

- `/ready`
  - readiness fails if DB ping fails
- any flow using profit-report repositories outside the web controllers in future

Even here, the main product/catalog/admin/POS views are not reading their content from SQL.

## Recommended Remediation

### Priority 1

- Create real SQL tables for:
  - products
  - inventory levels
  - stock movements
  - vendors
  - categories or taxonomy tables
  - orders
  - order lines
  - POS quick items
  - POS discount codes if they must be editable

### Priority 2

- Replace `AdminService` in-memory `HashMap` state with repository-backed persistence.
- Replace `CatalogService::from_seed(...)` as the runtime catalog source.
- Replace POS runtime catalog lookup with DB-backed product/inventory lookup.

### Priority 3

- Restrict seed data to bootstrap/dev fixtures only.
- Add BDD coverage that proves:
  - saving in Admin Intake updates Inventory
  - Inventory changes appear in Catalog
  - POS scans the same persisted product row
  - restarting the server preserves those changes

## Bottom Line

The product-facing views are currently not using a shared database table.

The main non-database-backed views are:

- `/catalog`
- `/catalog/search`
- `/catalog/items/{book_id}`
- `/cart`
- `/orders`
- `/admin`
- `/admin/orders`
- `/admin/inventory`
- `/admin/intake`
- `/pos`

And the APIs behind those screens are also mostly not database-backed.
