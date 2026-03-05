# Scriptorium Implementation Plan

## 1. Summary

Scriptorium will be implemented as a modular monolith in Rust:

- `axum` + `tower` + `tokio` for HTTP/API and runtime.
- Server-rendered HTML + `htmx` for storefront and admin.
- A focused Preact + HTM POS island at `/pos` for fast volunteer workflows on phones.
- SQLite for MVP with a strict repository boundary and migration parity strategy so Postgres can be introduced without changing domain APIs.

Default product decisions:

- Scope detail: MVP + Phase 2 extension hooks.
- Runtime: single Rust binary.
- Auth: volunteer shift PIN + admin accounts.
- Payments: POS manual app-switch flow first; online hosted checkout flow.
- Currency/tax: AUD with GST-inclusive pricing.
- Mobile: PWA-first POS (installable on iOS/Android home screen).

## 2. Architecture

### 2.0 UX Design Source of Truth

- UX reference artifact: `docs/specs/design-ux.jsx` (interactive 10-screen prototype).
- UI palette/token reference artifact: `docs/specs/design-palette.jsx`.
- Planning assumption: this file is the active UX spec input until/unless replaced by a markdown version.
- Delivery rule: implementation must map to the prototype flows and adhere to `design-palette.jsx` tokens while keeping BDD scenarios as final behavior authority.

### 2.1 Backend

- **Framework:** `axum`.
- **Templating:** `askama`.
- **Service style:** modular monolith with clear crate boundaries.
- **Observability:** structured logs + request IDs + basic operational metrics.

### 2.2 Frontend

- **Storefront/Admin:** HTML templates rendered by Rust, progressively enhanced via `htmx`.
- **POS:** single-page island at `/pos` using Preact + HTM modules, no heavy frontend build stack in MVP.

### 2.3 Data and DB Portability

- **Data access:** repository traits in app/domain boundaries.
- **Persistence implementation:** SQLx-backed adapter, SQLite first.
- **Migration strategy:** keep semantic migration parity between SQLite and Postgres tracks.
- **Rule:** avoid SQLite-only modeling shortcuts for core relations (normalize entities rather than JSON blobs for business-critical links).

## 3. Workspace/Crate Targets

Planned crate responsibilities:

1. `bookstore-domain`:
   - entities, value objects, invariants, pricing/tax logic.
2. `bookstore-app`:
   - use-cases (catalog, checkout, POS, inventory, reporting).
3. `bookstore-data`:
   - repository interfaces + SQLite implementation now; Postgres adapter next.
4. `bookstore-web`:
   - routes, handlers, templates, static assets, auth middleware.
5. `bookstore-cli`:
   - operational/admin commands via shared app services.
6. `bookstore-mobile-core`:
   - shared contracts/hooks for later native packaging work.

## 4. Domain and Data Model

### 4.1 Core Domain Types

- `Money` (minor units/cents + currency code)
- `OrderChannel` (`Pos`, `Online`)
- `PaymentMethod` (`Cash`, `ExternalCard`, `OnlineCard`, `Iou`)
- `OrderStatus` (`Paid`, `UnpaidIou`, `Refunded`)

### 4.2 Core Entities

- Product
- Inventory level
- Order + order lines
- Payment transaction
- Shift
- Vendor / consignment metadata
- Stock movement journal

### 4.3 MVP Schema Shape

- `products`
- `categories`
- `product_categories`
- `inventory_levels`
- `stock_movements`
- `orders`
- `order_lines`
- `payments`
- `shifts`
- `users`
- `roles`
- `sessions`
- `quick_items`
- `vendors`

## 5. API and UI Contracts

### 5.1 HTML Routes

- `/` storefront
- `/catalog`
- `/product/:id`
- `/cart`
- `/checkout`
- `/admin/*`
- `/pos` (POS island shell)

### 5.2 JSON API (Initial)

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

### 5.3 POS UX Guarantees

- 4-digit shift PIN entry with large touch targets.
- Scan-first item add workflow.
- Quick-item grid for non-barcoded goods.
- Fast cash workflow with exact/custom/change + donation round-up.
- External card handoff capture.
- IOU capture with customer name.
- Optional email receipt at sale completion.

## 6. Security, Reliability, and Operations

- Argon2id for PIN/password hashing.
- Role-based auth checks on admin surfaces.
- Transactional checkout: order + payment + stock changes committed atomically.
- Idempotent webhook processing for online payment confirmation.
- Audit trails for stock and payment adjustments.

## 7. Testing Strategy

- Unit tests for pricing/tax, donation/change, state transitions.
- Repository tests for data invariants and transaction behavior.
- API integration tests for POS, storefront checkout, admin actions.
- Cucumber BDD features covering brief scenarios A-G plus failure/regression paths.
- Mobile viewport smoke tests for `/pos`.
- UX conformance checks against `docs/specs/design-ux.jsx` for the core 10 screens and interaction flows.
- Visual conformance checks against `docs/specs/design-palette.jsx` (color, typography, spacing, component token usage).

## 8. Milestones

1. **Foundation**
   - Domain model, service interfaces, DB schema v1, migrations, basic auth primitives.
2. **POS MVP**
   - PIN login, scanner flow, quick items, cash/external card/IOU checkout.
3. **Storefront MVP**
   - browse/search/product/cart/online checkout + webhook finalize.
4. **Admin MVP**
   - inventory receive/adjust, product management, low stock, IOU queue, baseline reports.
5. **Hardening**
   - full BDD pass, performance tuning for rush-hour use, migration parity checks, runbooks.
6. **Phase 2 Hooks**
   - ISBN metadata ingestion, consignment reporting depth, custom barcode workflows.

## 9. Explicit Assumptions

- MVP is online-first (no offline queueing yet).
- PWA distribution is sufficient for initial iOS/Android volunteer usage.
- Payment provider details remain adapter-driven (domain remains gateway-agnostic).
- Multi-location support is out of MVP scope but schema/service design will not block it.

## 10. Multi-Agent Delivery Model

Work is divided into **key sections (pods)**. Each section is staffed by:

- **1 Implementer Agent**: builds production code and docs for the section.
- **1 Reviewer/Tester Agent**: owns BDD scenarios, test depth, code review, and acceptance sign-off.

Rules for every section:

1. Reviewer creates/updates cucumber scenarios first.
2. Implementer delivers code to satisfy scenarios.
3. Reviewer runs full section test pack and performs review.
4. Section can merge only after all section gates are green.

## 11. Key Sections and Ownership Split

### Section A: Platform Foundations

Scope:

- crate boundaries and module skeletons
- base config, tenant context plumbing, locale context plumbing
- base migration framework and transaction helper

Dependencies: none (starts first).

Implementer responsibilities:

- establish crate interfaces and dependency rules
- implement base app boot and middleware chain
- add migration runner and DB bootstrap wiring

Reviewer/Tester responsibilities:

- write foundation BDD scenarios (health, tenant context, locale context)
- verify layering boundaries and portability rules
- run smoke checks on bootstrapping and migrations

Completion gates:

- foundation BDD scenarios pass
- app boots and migrations run cleanly
- architecture boundaries documented in code comments/README

### Section B: Domain + Data Core

Scope:

- core entities/value objects
- SQLite schema v1
- repository traits + SQLite implementations
- financial fields for COGS/profit

Dependencies: Section A.

Implementer responsibilities:

- implement domain invariants and repository contracts
- build schema and persistence mappings
- ensure tenant-scoped query behavior

Reviewer/Tester responsibilities:

- create BDD scenarios for profit calculation and tenant isolation
- add repository/integration tests for transaction correctness
- review schema for Postgres portability risks

Completion gates:

- all domain/repository tests pass
- COGS + gross profit reports are test-verified
- cross-tenant data isolation tests pass

### Section C: POS Experience (Preact + HTM Island)

Scope:

- `/pos` app shell and JS modules
- PIN login, scanning, quick items, cash/card/IOU flows
- POS checkout APIs
- UX parity with prototype screens 1-4 in `docs/specs/design-ux.jsx`

Dependencies: Sections A and B.

Implementer responsibilities:

- build POS UI and state flow
- implement POS endpoints and checkout orchestration
- integrate barcode scan path with fallback strategy

Reviewer/Tester responsibilities:

- author BDD scenarios A-D (Sunday rush, quick items, cash/donation, IOU)
- run mobile viewport and workflow smoke tests
- verify usability constraints (large targets, low typing)
- validate implemented flows against UX screens 1-4

Completion gates:

- scenarios A-D passing
- checkout transaction atomicity verified
- mobile browser smoke tests passing
- UX parity sign-off for screens 1-4
- palette/token adherence sign-off against `docs/specs/design-palette.jsx`

### Section D: Storefront Checkout (HTMX)

Scope:

- catalog/search/product/cart/checkout pages
- hosted online checkout integration
- payment webhook finalize flow
- UX parity with prototype screens 5-7 in `docs/specs/design-ux.jsx`

Dependencies: Sections A and B.

Implementer responsibilities:

- implement HTML/HTMX storefront flows
- implement online checkout endpoint + webhook handler
- connect receipts/invoice dispatch pipeline

Reviewer/Tester responsibilities:

- author storefront BDD scenarios (browse/search/checkout)
- validate webhook idempotency and failure recovery tests
- review customer UX for minimal friction and correctness
- validate implemented flows against UX screens 5-7

Completion gates:

- storefront BDD scenarios pass
- webhook idempotency tests pass
- online orders reconcile to payments and inventory updates
- UX parity sign-off for screens 5-7
- palette/token adherence sign-off against `docs/specs/design-palette.jsx`

### Section E: Admin + Reporting + Mobile ISBN Intake

Scope:

- admin auth and role checks
- product/inventory management
- mobile ISBN scan intake + metadata auto-lookup
- reporting for sales/payment/donations/profit
- UX parity with prototype screens 8-10 in `docs/specs/design-ux.jsx`

Dependencies: Sections A, B, and D.

Implementer responsibilities:

- implement admin pages and APIs
- add mobile camera ISBN intake flow in admin
- implement reporting queries including gross profit

Reviewer/Tester responsibilities:

- author BDD scenario E and F
- test metadata lookup and manual override behavior
- verify treasurer-facing outputs against fixture data
- validate implemented flows against UX screens 8-10

Completion gates:

- scenarios E and F passing
- reports match fixture expectations
- admin role/auth tests passing
- UX parity sign-off for screens 8-10
- palette/token adherence sign-off against `docs/specs/design-palette.jsx`

### Section F: Cross-Cutting Hardening

Scope:

- security, observability, performance, backups/runbooks
- regression suite and release readiness

Dependencies: Sections A-E.

Implementer responsibilities:

- add metrics/logging coverage and security protections
- improve hotspots found in rush-hour tests
- finalize deploy and operations docs

Reviewer/Tester responsibilities:

- execute full BDD regression suite A-G
- run concurrency/performance tests
- perform release sign-off checklist

Completion gates:

- full test suite green
- performance baseline met for rush-hour scenarios
- release checklist completed

## 12. Parallel Execution Plan

Recommended order and parallelism:

1. Start Section A immediately (single pod).
2. Start Sections B and C in parallel once A base interfaces are merged.
3. Start Section D in parallel with late C once B contracts stabilize.
4. Start Section E once B and D are stable; it can run alongside final D polish.
5. Run Section F after feature sections reach merge-complete status.

## 13. Branching and Handoffs Between Agent Pairs

- Branch naming: `section-<letter>/<topic>/<agent-role>`
  - example: `section-c/pos-checkout/implementer`
  - example: `section-c/pos-checkout/reviewer`
- Implementer opens PR first.
- Reviewer pushes test commits either to reviewer branch or directly to PR branch (team preference).
- Reviewer signs off only when section gates pass.

Handoff artifact required per section:

- short implementation note
- test evidence summary (what passed, what failed/fixed)
- known risks/deferred items
- UX parity note referencing matched prototype screens in `docs/specs/design-ux.jsx`
- palette adherence note referencing `docs/specs/design-palette.jsx`

## 14. Section-Level Definition of Done

A section is done only when:

1. Implementer scope is complete.
2. Reviewer BDD scenarios and tests are complete.
3. No open high-severity review findings remain.
4. CI passes for lint, unit, integration, and cucumber tests.
5. Section docs are updated (API/schema/ops notes as applicable).
