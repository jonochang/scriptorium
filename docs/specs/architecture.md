# Scriptorium Technical Architecture

## 1. Purpose

This document defines the technology architecture, system design, and implementation approach for Scriptorium.
It is aligned to the functional brief and prioritises:

- ultra-fast mobile POS for volunteers,
- simple and maintainable web/admin delivery,
- multi-tenant support from day one for sister bookshops,
- internationalisation from day one,
- SQLite-first persistence with a clean upgrade path to Postgres.

## 2. High-Level System Shape

Scriptorium is a **modular monolith** written in Rust, deployed as a single service in MVP.

- Backend/API/Web rendering: one Rust process.
- Storefront/Admin UI: server-rendered HTML with HTMX.
- POS UI: a focused Preact + HTM island served at `/pos`.
- Database: SQLite for MVP, with repository abstraction and migration parity for future Postgres.

## 3. Technology Stack

### 3.1 Backend

- Rust (edition 2024)
- `axum` (HTTP server + routing)
- `tower` (middleware)
- `tokio` (async runtime)
- `serde` / `serde_json` (serialization)
- `tracing` / `tracing-subscriber` (observability)

### 3.2 Frontend

- Storefront/Admin: server-side HTML templates (`askama`) + `htmx` progressive enhancement.
- POS Island: Preact + HTM module scripts for scanner/cart/payment flow with minimal JS complexity.

### 3.3 Data

- SQL access: `sqlx`
- Migrations: versioned SQL migrations
- Initial database: SQLite
- Planned database target: Postgres (without domain/API contract changes)

### 3.4 Quality and Testing

- Unit/integration tests via `cargo test`
- BDD via `cucumber`
- Linting/formatting via `clippy` and `rustfmt`
- Supply-chain/license checks via `cargo-deny` / `cargo-audit`

## 4. Architectural Principles

1. **Domain-first boundaries:** business logic is independent of transport and database driver.
2. **Tenant isolation by default:** every business record is tenant-scoped.
3. **i18n by default:** user-facing strings are locale-driven, not hardcoded.
4. **Simple UI where possible:** HTMX first; richer JS only where needed (POS camera workflow).
5. **Transactional correctness:** order/payment/inventory updates are atomic.
6. **Postgres-ready design:** no SQLite-only assumptions in domain contracts.

## 5. Module Design

Planned crate/module responsibilities:

1. `bookstore-domain`
   - entities, value objects, invariants
   - money, tax, discount, profit calculations
2. `bookstore-app`
   - use-case services and orchestration
   - checkout, inventory, reporting, auth workflows
3. `bookstore-data`
   - repository traits and SQL adapters
   - migration runner and transaction helpers
4. `bookstore-web`
   - HTML handlers, JSON API handlers, middleware, assets
5. `bookstore-cli`
   - operational and admin command-line flows via shared services
6. `bookstore-mobile-core` (future-facing)
   - shared interfaces for later native wrappers if needed

## 6. Request/Response Architecture

### 6.1 Web Surface Split

- **HTML/HTMX routes**
  - storefront pages
  - admin pages
  - server-driven partial updates
- **POS island route**
  - `/pos` shell page + static module assets
  - JSON endpoints for scan/cart/checkout
- **JSON API routes**
  - POS auth and checkout
  - inventory operations
  - reporting endpoints
  - payment webhooks

### 6.2 Core API Shape (Initial)

- `POST /api/pos/login`
- `POST /api/pos/scan`
- `POST /api/pos/cart/items`
- `POST /api/pos/checkout`
- `POST /api/pos/payments/cash`
- `POST /api/pos/payments/external-card`
- `POST /api/pos/payments/iou`
- `POST /api/admin/inventory/receive`
- `POST /api/admin/inventory/adjust`
- `POST /api/admin/products/isbn-lookup`
- `GET /api/admin/reports/*`
- `POST /api/payments/webhook`

## 7. Data Architecture

### 7.1 Core Entities

- tenants/accounts
- users/roles/sessions
- products/categories/vendors
- inventory levels and stock movement journal
- orders/order lines
- payments/transactions
- POS shifts and quick items
- localisation resources
- later-phase library loans

### 7.2 Multi-Tenant Model

- Each business table includes `tenant_id`.
- Every query is tenant-scoped by default (middleware injects current tenant context).
- Unique constraints are tenant-aware (example: `(tenant_id, sku)`).
- Cross-tenant access is denied at service/repository boundaries.

### 7.3 Financial and Profit Tracking

At order line and order aggregate levels, persist enough financial detail to report:

- revenue,
- COGS (from captured cost at time of sale),
- gross profit.

Cost snapshots are stored with order lines so historical reports remain stable if product cost changes later.

### 7.4 SQLite to Postgres Path

- Keep SQL repositories behind traits.
- Maintain migration parity between SQLite and Postgres tracks.
- Avoid SQLite-specific SQL in business-critical paths.
- Treat Postgres introduction as an infrastructure migration, not a domain rewrite.

## 8. Internationalisation Architecture

From MVP:

- Store user/account locale and fallback locale.
- Keep UI copy in translation resources keyed by message IDs.
- Render server-side templates using locale dictionaries.
- Ensure emails/receipts use locale-aware templates.
- Keep product metadata capable of localised variants where available.

## 9. Security and Access Design

- Volunteer POS auth via shift PIN (short-lived scoped token).
- Admin auth via account credentials and secure session cookies.
- Password/PIN hashing via Argon2id.
- CSRF protection for admin form flows.
- Role-based authorization (`volunteer`, `admin`, `tenant_owner`, `platform_admin`).
- Audit log records for financial adjustments and stock changes.

## 10. POS Experience Architecture

The POS flow is optimized for the 30-minute Sunday rush:

- large targets and low typing,
- scanner-first flow with camera,
- quick-item tap grid,
- fast payment options (cash, external card handoff, IOU),
- optional receipt capture after sale completion.

Barcode scanning approach:

- Use browser camera APIs in the POS island.
- Attempt native/modern detection path first.
- Fallback to JS decoding strategy if needed.

## 11. Operational Architecture

### 11.1 Deployment (MVP)

- One Rust binary
- Static assets served by the same process
- SQLite file storage with backup strategy

### 11.2 Observability

- Structured logs with request IDs
- Metrics for:
  - checkout latency
  - payment failures
  - inventory conflict/retry rates
  - webhook processing outcomes

## 12. Testing and BDD Approach

BDD is the primary delivery driver:

1. Write scenario in cucumber feature.
2. Add/adjust step definitions.
3. Implement minimum code to pass.
4. Refactor while keeping scenario green.

Mandatory scenario coverage:

- Sunday rush checkout
- quick items
- cash + donation round-up
- IOU flow
- mobile admin ISBN intake with auto lookup
- gross profit reporting
- tenant isolation behavior

## 13. Phased Delivery Alignment

### Phase 1 (MVP)

- Core catalog/inventory
- POS flows
- storefront checkout
- admin inventory basics
- gross profit baseline reporting
- i18n and multi-tenant foundations

### Phase 2

- richer admin workflows
- advanced reporting
- expanded localisation management
- stronger tenant/account administration

### Phase 3

- customer loyalty/accounts
- multi-location inventory
- accounting integrations
- library borrowing module

