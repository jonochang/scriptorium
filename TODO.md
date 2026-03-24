# Scriptorium BDD TODO

This TODO is execution-ordered and BDD-first.
Rule for every feature pod: reviewer writes/updates BDD scenarios first, then implementer builds.

## Product Review Status (2026-03-06)

### Review findings confirmed
- [x] Backend/API foundation is strong: BDD, tenancy, authz, webhook idempotency, and reporting scaffolding are all in good shape.
- [x] Frontend parity remains materially behind `docs/specs/design-ux.jsx` and `docs/specs/design-palette.jsx`.
- [x] Section F hardening remains largely unstarted.

### Critical fixes completed in this pass
- [x] Fix catalog HTMX wiring and add plain `/catalog?q=...` fallback so search works with or without HTMX.
- [x] Reject POS checkout on an empty cart.
- [x] Return structured JSON error bodies for POS scan/payment failures instead of empty `400` responses.
- [x] Accept both `isbn` and `barcode` in POS scan requests to remove the UI/API field mismatch.
- [x] Replace raw POS JSON dumps with rendered cart, totals, and outcome feedback.
- [x] Remove hardcoded admin credentials from the intake HTML shell.
- [x] Add a live admin dashboard shell at `/admin` backed by the existing admin auth/report/product/category/vendor APIs.
- [x] Upgrade `/checkout` from a static shell to a page that creates checkout sessions from the browser.
- [x] Add storefront product detail and cart pages so catalog browsing now connects into cart and checkout.
- [x] Add deployment notes and a SQLite backup/restore runbook.
- [x] Add basic checkout observability logs with latency and outcome fields on payment/session paths.
- [x] Add admin order and IOU management APIs plus dashboard wiring for recent orders and IOU settlement.
- [x] Add a first CSRF protection slice for state-changing admin requests with cross-origin rejection tests.
- [x] Fix the POS shell template syntax regression so `/pos` renders again.
- [x] Add a friendly storefront 404 page for missing product ids.
- [x] Improve admin login UX by defaulting the seeded username explicitly.
- [x] Filter cart recommendations so titles already in the basket are excluded client-side.
- [x] Fill in the remaining shared palette tokens used by the spec (`--wine-muted`, `--blue`) and tighten responsive catalog/intake layout handling.
- [x] Add direct add-to-cart actions on catalog cards.
- [x] Add product-detail quantity selection and related-title cross-sell content.
- [x] Add checkout confirmation state and optional parish-support amount on the storefront checkout shell.
- [x] Expand the admin dashboard with payment breakdown, order filtering, stock-movement journal, and snapshot export controls.

### Remaining product gaps
- [ ] Reviewer confirms full parity with `design-ux.jsx` screens 1-10.
- [ ] Reviewer confirms full adherence to `design-palette.jsx` tokens across POS, storefront, and admin.
- [~] Expand storefront from the current detail/cart/checkout flow into fuller category browsing, richer product content, and production-grade cart persistence.
  Status: category browsing, richer catalog cards, direct shelf adds, quantity-aware detail adds, related-title cross-sell, and editable cart persistence are now in place; deeper merchandising polish is still open.
- [~] Expand the admin dashboard further with richer report workflows and denser parity against screens 8-10.
  Status: date-window controls, payment breakdown, order filtering, low-stock spotlight, stock journal, snapshot export, and stronger operations framing are now live; full screen-9/10 parity is still open.
- [~] Complete the remaining Section F hardening: perf baseline work.
  Status: release checklist and a concurrent POS rush smoke test are now in place; deeper perf profiling and hotspot remediation are still open.

## 0. Team Workflow (Applies to Every Section)

- [ ] Assign 2 agents per section: `implementer` and `reviewer/tester`.
- [ ] Use `docs/specs/design-ux.jsx` as UX reference input for implementation/review.
- [ ] Use `docs/specs/design-palette.jsx` as mandatory UI palette/token standard.
- [ ] Create branches:
  - [ ] `section-<letter>/<topic>/implementer`
  - [ ] `section-<letter>/<topic>/reviewer`
- [ ] Require section merge gates: lint + unit + integration + cucumber.
- [ ] Require section handoff note: implementation summary, test evidence, UX parity notes, risks/deferred items.
- [ ] Require section handoff note to include palette/token adherence notes.

## 1. Section A: Platform Foundations

### Reviewer/Tester first
- [x] Add `features/foundation/health.feature`.
- [x] Add `features/foundation/tenant_context.feature`.
- [x] Add `features/foundation/locale_context.feature`.

### Implementer
- [x] Confirm workspace crate boundaries (`domain`, `app`, `data`, `web`, `cli`).
- [x] Add/confirm architecture decision record for HTMX + POS island split.
- [x] Implement app boot + middleware chain for tenant and locale context.
- [x] Add base migration framework + DB bootstrap wiring.
- [x] Add CI gates: `fmt`, `clippy -D warnings`, tests, cucumber, deny/audit checks.

### Section A gates
- [x] Foundation BDD scenarios pass.
- [x] App boots and migrations run cleanly.

## 2. Section B: Domain + Data Core

### Reviewer/Tester first
- [x] Add `features/domain/money_gst.feature`.
- [x] Add `features/domain/profit_reporting.feature` (Scenario F baseline).
- [x] Add `features/domain/tenant_isolation.feature` (Scenario G baseline).

### Implementer
- [x] Implement core value objects (`Money`, order/payment/status enums, validations).
- [x] Define repository traits for products, inventory, orders, payments, shifts, tenants.
- [x] Add SQLite migration set `v1` (tenant-scoped schema).
- [x] Add migration parity checklist for future Postgres migration track.
- [x] Implement SQLite repositories and transaction helper.
- [x] Add cost snapshot support at order-line level for COGS/profit reporting.

### Section B gates
- [x] Domain + repository tests pass.
- [x] Profit calculations verified in tests.
- [x] Cross-tenant isolation tests pass.

## 3. Section C: POS Experience (Preact + HTM Island)

### Reviewer/Tester first
- [x] Add `features/pos/scenario_a_sunday_rush.feature`.
- [x] Add `features/pos/scenario_b_quick_items.feature`.
- [x] Add `features/pos/scenario_c_cash_roundup.feature`.
- [x] Add `features/pos/scenario_d_iou.feature`.

### Implementer
- [x] Implement `POST /api/pos/login` with shift PIN auth.
- [x] Implement POS cart/session flow and scan endpoint.
- [x] Implement quick-item grid APIs.
- [x] Implement cash payment flow (exact/custom/change/donation split).
- [x] Implement external card handoff recording (`external_ref` capture).
- [x] Implement IOU checkout and unpaid order status.
- [x] Build `/pos` Preact+HTM island UI with large-button mobile layout.

### Section C gates
- [x] Scenarios A-D pass.
- [x] POS checkout transaction atomicity verified.
- [x] POS mobile viewport smoke test passes.
- [~] Reviewer confirms parity with `design-ux.jsx` screens 1-4.
  Status: `/pos` now has a 4-step PIN login → basket → payment → completion flow on top of the live endpoints, and the v0.2.0 shell render regression has been fixed; final reviewer sign-off is still open.
- [ ] Reviewer confirms UI adheres to `design-palette.jsx` tokens.

## 4. Section D: Storefront Checkout (HTMX)

### Reviewer/Tester first
- [x] Add `features/storefront/catalog_browse.feature`.
- [x] Add `features/storefront/search.feature`.
- [x] Add `features/storefront/checkout.feature`.

### Implementer
- [x] Implement server-rendered storefront pages with HTMX interactions.
- [x] Implement cart and online checkout session creation.
- [x] Implement payment webhook finalize flow (idempotent).
- [x] Add email receipt/invoice dispatch flow.

### Section D gates
- [x] Storefront BDD scenarios pass.
- [x] Webhook idempotency tests pass.
- [ ] Reviewer confirms parity with `design-ux.jsx` screens 5-7.
- [ ] Reviewer confirms UI adheres to `design-palette.jsx` tokens.

## 5. Section E: Admin + Reporting + Mobile ISBN Intake

### Reviewer/Tester first
- [x] Add `features/admin/scenario_e_inventory_add.feature`.
- [x] Add `features/admin/scenario_f_profit_visibility.feature`.
- [x] Add `features/admin/scenario_g_multi_bookshop_isolation.feature`.

### Implementer
- [x] Implement admin auth (accounts + roles).
- [x] Implement product CRUD and tenant-scoped category/vendor management.
- [x] Implement receive stock + adjust stock workflows and stock movement journal.
- [x] Implement mobile camera ISBN intake + metadata auto-lookup.
- [x] Implement reports: sales by date/payment, donations, COGS, gross profit.
- [x] Add i18n plumbing for admin/storefront/POS text resources.

### Section E gates
- [x] Scenarios E, F, and G pass.
- [x] Treasurer report fixtures match expected revenue/COGS/profit values.
- [x] Admin role and tenant isolation tests pass.
- [ ] Reviewer confirms parity with `design-ux.jsx` screens 8-10.
- [ ] Reviewer confirms UI adheres to `design-palette.jsx` tokens.

## 6. Section F: Cross-Cutting Hardening

### Reviewer/Tester first
- [x] Prepare full regression suite list and release checklist.
- [x] Add concurrency/perf test scenarios for Sunday rush load.

### Implementer
 - [x] Add security tests for authz/authn boundaries and CSRF handling.
- [x] Add observability metrics/log fields for checkout latency and failures.
- [x] Add backup/restore runbook for SQLite MVP.
- [x] Add deployment docs for single-binary service + static assets.
- [ ] Address perf hotspots found in load tests.

### Section F gates
- [ ] Full BDD regression (A-G) passes.
- [~] Concurrency/performance baselines pass.
  Status: a concurrent POS rush smoke test now passes; full perf baselines are still open.
 - [x] Release readiness checklist completed.
- [ ] End-to-end UX parity review completed against `docs/specs/design-ux.jsx`.
- [ ] End-to-end palette/token adherence review completed against `docs/specs/design-palette.jsx`.

## 7. Parallel Run Plan

- [ ] Start Section A immediately.
- [ ] Start Sections B and C in parallel after A core interfaces merge.
- [ ] Start Section D once B contracts stabilize (can overlap with late C).
- [ ] Start Section E once B and D are stable.
- [ ] Run Section F after A-E are merge-complete.

## 8. Section G: Leptos Migration (WASM Islands)

Migrate every `bookstore-cart-wasm` module from manual `web-sys` DOM manipulation to Leptos 0.7 CSR components. The dependency is already in `Cargo.toml` but nothing uses it yet. Each island is self-contained, so they can be migrated independently.

Current state: 5 islands, ~5,900 lines of raw `wasm-bindgen`/`web-sys` code with manual `get_element_by_id`, `set_text_content`, `set_inner_html`, closure-based event handlers, and window-global state.

### Phase 0: Foundation and patterns

#### BDD: Write first, before any code
- [x] **G-0-BDD: Add `features/migration/leptos_mount.feature`.**
  - Scenario: WASM module loads and sets ready flag — Given the bookstore api is running, When I open the storefront catalog page, Then the response contains `/static/wasm/bookstore-cart-wasm` and the WASM `__SCRIPTORIUM_*_READY` flags are set.
  - Scenario: Cart localStorage round-trips through Leptos signals — Given I add a book to the cart, When I reload the page, Then the cart badge still shows the correct count.

#### Implementation
- [x] **G-0a: Leptos app shell and mount strategy.** Replace the `#[wasm_bindgen(start)]` entry point in `lib.rs` with a Leptos `mount_to` approach. Each island mounts to its existing DOM container. Decide whether islands share a single Leptos app or mount independently.
- [x] **G-0b: Shared API client module.** Extract the duplicated `fetch_post` / `fetch_json_get` / `json_headers` helpers (repeated across `intake.rs`, `admin.rs`, `pos.rs`, `checkout.rs`) into a shared `api.rs` module that returns typed results. All islands currently copy-paste these ~40 lines each.
- [ ] **G-0c: Shared state patterns.** Replace window-global state (`win_get_f64`/`win_set_f64`/`win_get_str`/`win_set_str` scattered across modules) with Leptos signals and context. Define `provide_context` / `use_context` patterns for auth tokens, tenant ID, and cart state.
- [ ] **G-0d: Cart state as Leptos signals.** Migrate `cart.rs` (236 lines) from `gloo-storage` read/write with manual DOM updates to a Leptos `RwSignal<Vec<CartItem>>` with `Effect` for localStorage sync. This is the shared dependency for `components.rs` and `checkout.rs`.

#### Phase 0 gate
- [ ] Existing tests pass: `browser_cart_wasm_module_loads`, `browser_catalog_add_updates_cart_badge`.
- [ ] New BDD scenarios in `leptos_mount.feature` pass.

### Phase 1: Small islands (low risk, build confidence)

#### BDD: Write first, before any code
- [ ] **G-1-BDD-a: Add `features/migration/leptos_storefront_cart.feature`.**
  - Scenario: Add to cart from catalog card — Given the catalog page is open, When I click "Add to Cart" on a book card, Then the cart badge increments and the item appears on the cart page.
  - Scenario: Cart page shows line items with quantities and totals — Given 2 different books are in the cart, When I open the cart page, Then I see 2 line items with correct titles, prices, and a cart total.
  - Scenario: Remove item from cart — Given a book is in the cart, When I click "Remove" on the cart page, Then the item disappears and the total updates.
  - Scenario: Cart recommendations exclude items already in basket — Given "Beginning to Pray" is in my cart, When I view recommendations on the cart page, Then "Beginning to Pray" is not shown.
  - Scenario: Quantity selector on product detail page — Given I am on a product detail page, When I set quantity to 3 and click "Add to Cart", Then the cart badge shows 3.
- [ ] **G-1-BDD-b: Add `features/migration/leptos_storefront_checkout.feature`.**
  - Scenario: Checkout summary reflects cart contents — Given 1 book at $18.99 is in the cart, When I open the checkout page, Then the order summary shows $18.99.
  - Scenario: Donation slider adjusts total — Given I am on the checkout page, When I set the parish support donation to $5.00, Then the total updates to include the donation.
  - Scenario: Checkout advances through steps — Given I am on the checkout page, When I fill contact details and click "Continue to payment", Then the payment step is visible and the contact step is marked done.
  - Scenario: Checkout session creation calls the API — Given I am on the payment step, When I submit the payment form, Then a POST to `/api/storefront/checkout/session` is made with correct totals.

#### Implementation
- [ ] **G-1a: Storefront cart components** (`components.rs`, 243 lines). Migrate `mount_cart_island` — add-to-cart buttons, cart count badge, cart page render. Currently uses `querySelectorAll("[data-add-book]")` to bind click handlers and `set_inner_html` to render the cart page. Convert to Leptos `#[component]` functions with reactive cart signal, `<For>` loops, and event handlers.
- [ ] **G-1b: Storefront checkout** (`checkout.rs`, 579 lines). Migrate `mount_checkout_island` — order summary, donation slider, Stripe-style payment form, and success state. Currently a 3-step flow managed by `set_step()` toggling CSS classes. Convert to a Leptos component with a `step: RwSignal<u8>` and conditional `view!` rendering per step.

#### Phase 1 gate
- [ ] Existing tests pass: `browser_catalog_add_updates_cart_badge`, `browser_cart_hides_titles_already_in_basket`, `browser_catalog_card_link_opens_product_detail`, `browser_checkout_updates_summary_and_advances_to_payment`.
- [ ] Existing BDD pass: `detail_and_cart.feature`, `checkout_shell.feature`, `checkout.feature`, `catalog_browse.feature`.
- [ ] New BDD scenarios in `leptos_storefront_cart.feature` and `leptos_storefront_checkout.feature` pass.

### Phase 2: Admin dashboard

#### BDD: Write first, before any code
- [ ] **G-2-BDD: Add `features/migration/leptos_admin_dashboard.feature`.**
  - Scenario: Admin login and dashboard load — Given I am on the admin login page, When I sign in with valid credentials, Then the dashboard shows "Today's Sales" and product count.
  - Scenario: Product list shows all admin products — Given I am signed in as admin, When the dashboard loads, Then I see all seeded products with title, ISBN, category, and retail price.
  - Scenario: Product delete removes from list — Given I am signed in as admin and a product exists, When I delete the product, Then it disappears from the product list.
  - Scenario: Order list shows recent orders — Given a POS cash sale has been recorded, When I view the admin orders tab, Then I see the order with status "Paid" and the correct total.
  - Scenario: Report summary shows sales and profit — Given sales events have been recorded, When I view the report summary, Then I see total sales, COGS, and gross profit figures.
  - Scenario: Low-stock alert panel — Given a product has 2 units on hand (below reorder point), When the dashboard loads, Then the low-stock panel shows that product.
  - Scenario: Stock movement journal — Given inventory has been received and adjusted, When I view the stock journal, Then I see entries with deltas and reasons.

#### Implementation
- [ ] **G-2a: Admin island** (`admin.rs`, 1,190 lines). Migrate `mount_admin_island` — dashboard stats, product list, order list, report summary, stock journal, low-stock alerts. This is the most data-heavy island with multiple tabs/views and polling refresh. Convert to Leptos components with `Resource` for async data fetching and `Signal`-driven tab switching. Key sub-components:
  - Product list with inline edit/delete
  - Order list with view detail / mark-paid actions
  - Report summary with date range filtering
  - Stock movement journal
  - Low-stock alert panel

#### Phase 2 gate
- [ ] Existing tests pass: `browser_admin_login_loads_dashboard_data`, `browser_admin_orders_page_shows_row_actions`, `browser_admin_dashboard_renders_payment_breakdown_and_low_stock`.
- [ ] Existing BDD pass: `admin_auth_products_reports.feature`, `admin_category_vendor_lists.feature`, `admin_orders_shell.feature`, `admin_dashboard_shell.feature`, `admin_orders_iou.feature`, `admin_report_date_range.feature`.
- [ ] New BDD scenarios in `leptos_admin_dashboard.feature` pass.

### Phase 3: Intake (medium complexity, camera integration)

#### BDD: Write first, before any code
- [ ] **G-3-BDD: Add `features/migration/leptos_intake.feature`.**
  - Scenario: Intake page renders the 3-step wizard — Given I am signed in as admin, When I open the intake page, Then I see step indicators for scan, review, and save, and the scan step is active.
  - Scenario: ISBN fetch populates the review form — Given I am on the intake page, When I enter ISBN 9781802063271 and click Fetch, Then the title, author, and description fields are auto-filled from the lookup API.
  - Scenario: ISBN fetch with unknown ISBN shows manual entry — Given I am on the intake page, When I enter an ISBN with no metadata and click Fetch, Then a warning status says "No metadata found" and the form is still editable.
  - Scenario: Save product persists to admin store — Given the review form is filled with title "Test Book" and ISBN 9781234567890, When I click "Save product", Then a POST to `/api/admin/products` succeeds and the success step is shown.
  - Scenario: Save product with initial stock calls receive inventory — Given I save a product with initial stock 5, Then a POST to `/api/admin/inventory/receive` is made with quantity 5.
  - Scenario: Cover upload stores an image — Given I select a cover image file, When I click "Attach file", Then a POST to `/api/admin/products/cover-upload` is made and the cover preview updates.
  - Scenario: Category and vendor dropdowns load from API — Given I am on the intake page, When the review step loads, Then the category and vendor dropdowns are populated from `/api/admin/categories` and `/api/admin/vendors`.
  - Scenario: Edit existing product loads its data — Given product "prd-9781802063271" exists, When I open intake with `?product_id=prd-9781802063271`, Then the form is pre-filled with that product's title, ISBN, cost, and retail price.

#### Implementation
- [ ] **G-3a: Intake form and review** (`intake.rs`, 1,297 lines). Migrate `mount_intake_island` — ISBN scanner, metadata lookup, product review form, cover upload, save flow. The 3-step wizard (scan → review → save) with form state, async lookup, and cover image preview. Convert to Leptos components. Key challenge: integrate the `scanner.rs` camera/barcode detection module (573 lines) which uses raw `web-sys` canvas/video APIs — this may need to remain as a non-Leptos utility called from within Leptos components via `create_effect` or `on_mount`.
- [ ] **G-3b: Scanner integration.** Decide whether `scanner.rs` stays as a raw `web-sys` utility (called imperatively from Leptos effects) or gets a thin Leptos wrapper component. The barcode detection loop uses `requestAnimationFrame` and canvas pixel manipulation which don't map naturally to reactive UI.

#### Phase 3 gate
- [ ] Existing tests pass: `browser_admin_intake_save_receives_initial_stock`.
- [ ] Existing BDD pass: `admin_mobile_intake_shell.feature`, `scenario_e_inventory_add.feature`, `admin_pos_shared_inventory.feature`, `admin_product_validation.feature`.
- [ ] New BDD scenarios in `leptos_intake.feature` pass.

### Phase 4: POS (largest, most complex)

#### BDD: Write first, before any code
- [ ] **G-4-BDD: Add `features/migration/leptos_pos.feature`.**
  - Scenario: PIN login authenticates and shows the basket — Given I am on the POS page, When I enter shift PIN 1234, Then the PIN screen hides and the basket view is shown.
  - Scenario: Invalid PIN shows error — Given I am on the POS page, When I enter an incorrect PIN, Then an error message is shown and I remain on the PIN screen.
  - Scenario: Scan barcode adds item to basket — Given I am logged into POS, When I scan barcode 9780060652937, Then the basket shows "Celebration of Discipline" with quantity 1 and the correct price.
  - Scenario: Scan same barcode twice increments quantity — Given I am logged into POS with 1x "Celebration of Discipline", When I scan 9780060652937 again, Then the quantity updates to 2 and the total doubles.
  - Scenario: Quick-item button adds to basket — Given I am logged into POS, When I click the "Prayer Card" quick-item button, Then the basket shows "Prayer Card" with quantity 1.
  - Scenario: Cart quantity adjustment — Given I am logged into POS with 2x of an item, When I set the quantity to 1, Then the total updates to reflect 1 unit.
  - Scenario: Remove item from basket — Given I am logged into POS with an item in the basket, When I set its quantity to 0, Then the item is removed from the basket.
  - Scenario: Cash payment with exact change — Given the basket total is $16.99, When I tender $16.99 cash, Then the sale completes with $0.00 change due.
  - Scenario: Cash payment with change and round-up donation — Given the basket total is $16.99, When I tender $20.00 with donate-change enabled, Then the sale completes with $0.00 change due and $3.01 donated.
  - Scenario: External card payment — Given the basket total is $16.99, When I complete an external card payment, Then the sale completes with status "sale_complete".
  - Scenario: IOU checkout records customer name — Given items are in the basket, When I check out as IOU with customer name "John Doe", Then the sale completes with status "iou".
  - Scenario: Discount code reduces total — Given the basket total is $16.99, When I apply a 10% clergy discount, Then the displayed total is reduced by 10%.
  - Scenario: Unknown barcode shows error — Given I am logged into POS, When I scan an unregistered barcode, Then an error message says "Book not found".
  - Scenario: Stock depletion prevents over-selling — Given only 1 unit is in stock, When I try to add 2 to the basket, Then an error prevents the second add.

#### Implementation
- [ ] **G-4a: POS island** (`pos.rs`, 1,768 lines). Migrate `mount_pos_island` — PIN login, barcode scanning, cart management, quick-item grid, payment flows (cash/card/IOU), receipt display. This is the largest module with the most interactive state: session tokens, cart items, payment modals, scanner integration, and discount code application. Convert to Leptos components. Key sub-components:
  - PIN login screen
  - Scanner + manual barcode entry
  - Cart line items with quantity controls
  - Quick-item grid (loaded from `/api/pos/config`)
  - Payment modal (cash tendered / card / IOU tabs)
  - Receipt / completion screen
  - Discount code application

#### Phase 4 gate
- [ ] Existing tests pass: `browser_pos_flow_reaches_completion_screen`, `browser_pos_payment_screen_shows_total_and_round_up_action`, `browser_pos_forgot_pin_opens_help_state`, `browser_pos_discount_changes_amount_due`.
- [ ] Existing BDD pass: `scenario_a_sunday_rush.feature`, `scenario_b_quick_items.feature`, `scenario_c_cash_roundup.feature`, `scenario_d_iou.feature`, `checkout_atomicity.feature`, `error_handling.feature`, `discounts.feature`, `mobile_viewport_smoke.feature`.
- [ ] New BDD scenarios in `leptos_pos.feature` pass.

### Phase 5: Cleanup and server-side templates

- [ ] **G-5a: Remove inline HTML from Rust controllers.** The admin intake page (`admin_pages.rs`) embeds ~800 lines of HTML+CSS as raw string literals in the Rust controller. Once the Leptos island is self-rendering, strip these down to minimal mount-point containers.
- [ ] **G-5b: Remove dead DOM IDs and CSS classes.** After each island migration, audit and remove CSS classes and DOM IDs that were only used by the old manual wiring (e.g., `intake-form-stack`, `intake-meta-grid`, step indicator data attributes).
- [ ] **G-5c: Remove `web-sys` feature bloat.** Audit `Cargo.toml` web-sys features list — many features (e.g., `HtmlCanvasElement`, `MediaStream`, `NodeList`) may no longer be needed directly once Leptos handles DOM interaction. Keep only what `scanner.rs` and Leptos require.
- [ ] **G-5d: Build and size audit.** Compare WASM binary size before and after migration. The current `.wasm` is ~1.7MB. Leptos CSR adds framework overhead but should eliminate a lot of boilerplate. Target: no more than 20% size increase.

### Phase 6: Full regression and component tests

#### BDD: Full regression suite
- [ ] **G-6-BDD: Add `features/migration/leptos_full_regression.feature`.**
  - Scenario: End-to-end storefront flow — Given I browse the catalog, add a book to the cart, proceed to checkout, and submit payment, Then a checkout session is created and the order total is correct.
  - Scenario: End-to-end POS flow — Given I log into POS, scan a book, and pay cash, Then the sale completes, stock is deducted, and the admin report reflects the sale.
  - Scenario: End-to-end intake flow — Given I log into admin, open intake, enter an ISBN, fetch metadata, and save the product, Then the product appears in the admin product list and is scannable at POS.
  - Scenario: Cross-system inventory consistency — Given I add a product via intake with stock 10, When I sell 2 via POS, Then admin inventory shows 8 on hand.

#### Implementation
- [ ] **G-6a: Component-level tests.** Add Leptos component tests (using `leptos::mount_to` in `wasm-bindgen-test`) for critical flows: cart add/remove, POS scan→cart→checkout, intake scan→review→save.
- [ ] **G-6b: Browser e2e regression.** Verify all existing browser e2e tests (`tests/browser/e2e.rs`) still pass after migration. The tests interact via DOM selectors, so any ID/class changes need corresponding test updates.

#### Phase 6 gate
- [ ] All 36 existing `.feature` files pass.
- [ ] All 13 existing `browser_*` e2e tests pass.
- [ ] All new `features/migration/leptos_*.feature` scenarios pass.
- [ ] WASM binary size within 20% of pre-migration baseline.

### Migration order rationale

`cart.rs` → `components.rs` → `checkout.rs` → `admin.rs` → `intake.rs` → `pos.rs`

Start with the shared cart state (smallest, foundational), then the two small storefront islands to validate patterns, then admin (data-heavy but no hardware), then intake (camera), then POS (largest, most stateful). Each phase produces a working, deployable build.

## 9. Final MVP Definition of Done

- [ ] All brief scenarios A-G pass in cucumber.
- [ ] POS works end-to-end on iOS Safari and Android Chrome as PWA.
- [ ] Inventory and financial records are transactionally consistent for all checkout methods.
- [ ] Gross profit reporting is available for treasurer review.
- [ ] i18n and multi-tenant foundations are active from MVP.
- [ ] SQLite-to-Postgres migration/versioning strategy is documented.
