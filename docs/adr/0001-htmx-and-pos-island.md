# ADR 0001: HTMX Pages with POS Island Split

## Status
Accepted - March 5, 2026

## Context
Scriptorium needs a fast volunteer POS UX on mobile while keeping the storefront/admin stack simple and maintainable for MVP delivery.

## Decision
Use server-rendered HTML with HTMX for storefront and admin surfaces, and keep the POS as a focused Preact + HTM island mounted at `/pos`.

## Consequences
- Storefront/admin can iterate quickly with low JavaScript complexity.
- POS can optimize for scanner, touch targets, and payment workflows without forcing SPA complexity across the whole app.
- Rust backend contracts remain shared across HTMX and POS JSON endpoints.
- Future POS enhancements are isolated from storefront/admin rendering concerns.
