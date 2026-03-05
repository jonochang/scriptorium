# Scriptorium BDD TODO

This TODO is execution-ordered and BDD-first.
Rule for every feature pod: reviewer writes/updates BDD scenarios first, then implementer builds.

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
- [ ] Add `features/domain/money_gst.feature`.
- [ ] Add `features/domain/profit_reporting.feature` (Scenario F baseline).
- [ ] Add `features/domain/tenant_isolation.feature` (Scenario G baseline).

### Implementer
- [ ] Implement core value objects (`Money`, order/payment/status enums, validations).
- [ ] Define repository traits for products, inventory, orders, payments, shifts, tenants.
- [ ] Add SQLite migration set `v1` (tenant-scoped schema).
- [ ] Add migration parity checklist for future Postgres migration track.
- [ ] Implement SQLite repositories and transaction helper.
- [ ] Add cost snapshot support at order-line level for COGS/profit reporting.

### Section B gates
- [ ] Domain + repository tests pass.
- [ ] Profit calculations verified in tests.
- [ ] Cross-tenant isolation tests pass.

## 3. Section C: POS Experience (Preact + HTM Island)

### Reviewer/Tester first
- [ ] Add `features/pos/scenario_a_sunday_rush.feature`.
- [ ] Add `features/pos/scenario_b_quick_items.feature`.
- [ ] Add `features/pos/scenario_c_cash_roundup.feature`.
- [ ] Add `features/pos/scenario_d_iou.feature`.

### Implementer
- [ ] Implement `POST /api/pos/login` with shift PIN auth.
- [ ] Implement POS cart/session flow and scan endpoint.
- [ ] Implement quick-item grid APIs.
- [ ] Implement cash payment flow (exact/custom/change/donation split).
- [ ] Implement external card handoff recording (`external_ref` capture).
- [ ] Implement IOU checkout and unpaid order status.
- [ ] Build `/pos` Preact+HTM island UI with large-button mobile layout.

### Section C gates
- [ ] Scenarios A-D pass.
- [ ] POS checkout transaction atomicity verified.
- [ ] POS mobile viewport smoke test passes.
- [ ] Reviewer confirms parity with `design-ux.jsx` screens 1-4.
- [ ] Reviewer confirms UI adheres to `design-palette.jsx` tokens.

## 4. Section D: Storefront Checkout (HTMX)

### Reviewer/Tester first
- [ ] Add `features/storefront/catalog_browse.feature`.
- [ ] Add `features/storefront/search.feature`.
- [ ] Add `features/storefront/checkout.feature`.

### Implementer
- [ ] Implement server-rendered storefront pages with HTMX interactions.
- [ ] Implement cart and online checkout session creation.
- [ ] Implement payment webhook finalize flow (idempotent).
- [ ] Add email receipt/invoice dispatch flow.

### Section D gates
- [ ] Storefront BDD scenarios pass.
- [ ] Webhook idempotency tests pass.
- [ ] Reviewer confirms parity with `design-ux.jsx` screens 5-7.
- [ ] Reviewer confirms UI adheres to `design-palette.jsx` tokens.

## 5. Section E: Admin + Reporting + Mobile ISBN Intake

### Reviewer/Tester first
- [ ] Add `features/admin/scenario_e_inventory_add.feature`.
- [ ] Add `features/admin/scenario_f_profit_visibility.feature`.
- [ ] Add `features/admin/scenario_g_multi_bookshop_isolation.feature`.

### Implementer
- [ ] Implement admin auth (accounts + roles).
- [ ] Implement product CRUD and tenant-scoped category/vendor management.
- [ ] Implement receive stock + adjust stock workflows and stock movement journal.
- [ ] Implement mobile camera ISBN intake + metadata auto-lookup.
- [ ] Implement reports: sales by date/payment, donations, COGS, gross profit.
- [ ] Add i18n plumbing for admin/storefront/POS text resources.

### Section E gates
- [ ] Scenarios E, F, and G pass.
- [ ] Treasurer report fixtures match expected revenue/COGS/profit values.
- [ ] Admin role and tenant isolation tests pass.
- [ ] Reviewer confirms parity with `design-ux.jsx` screens 8-10.
- [ ] Reviewer confirms UI adheres to `design-palette.jsx` tokens.

## 6. Section F: Cross-Cutting Hardening

### Reviewer/Tester first
- [ ] Prepare full regression suite list and release checklist.
- [ ] Add concurrency/perf test scenarios for Sunday rush load.

### Implementer
- [ ] Add security tests for authz/authn boundaries and CSRF handling.
- [ ] Add observability metrics/log fields for checkout latency and failures.
- [ ] Add backup/restore runbook for SQLite MVP.
- [ ] Add deployment docs for single-binary service + static assets.
- [ ] Address perf hotspots found in load tests.

### Section F gates
- [ ] Full BDD regression (A-G) passes.
- [ ] Concurrency/performance baselines pass.
- [ ] Release readiness checklist completed.
- [ ] End-to-end UX parity review completed against `docs/specs/design-ux.jsx`.
- [ ] End-to-end palette/token adherence review completed against `docs/specs/design-palette.jsx`.

## 7. Parallel Run Plan

- [ ] Start Section A immediately.
- [ ] Start Sections B and C in parallel after A core interfaces merge.
- [ ] Start Section D once B contracts stabilize (can overlap with late C).
- [ ] Start Section E once B and D are stable.
- [ ] Run Section F after A-E are merge-complete.

## 8. Final MVP Definition of Done

- [ ] All brief scenarios A-G pass in cucumber.
- [ ] POS works end-to-end on iOS Safari and Android Chrome as PWA.
- [ ] Inventory and financial records are transactionally consistent for all checkout methods.
- [ ] Gross profit reporting is available for treasurer review.
- [ ] i18n and multi-tenant foundations are active from MVP.
- [ ] SQLite-to-Postgres migration/versioning strategy is documented.
