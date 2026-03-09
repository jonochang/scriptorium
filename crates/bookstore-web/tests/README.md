`bookstore-web` test layout:

- `tests/service/`
  Contains Cucumber-style BDD harnesses that exercise service-layer behavior through HTTP/API calls and server-rendered HTML without launching a browser.
- `tests/features/service/`
  Contains the `.feature` files used by the service BDD harness.
- `tests/browser/`
  Contains real browser-driven tests using `chromiumoxide` against a live Axum test server.

Current commands:

- `cargo test -p bookstore-web --test service_bdd`
- `cargo test -p bookstore-web --test browser_e2e`
- `cargo test -p bookstore-web`

Organizing rule:

- If the assertion can be made through HTTP responses or server-rendered markup alone, put it under `service`.
- If the assertion depends on JavaScript execution, local storage, DOM interaction, or rendered browser state, put it under `browser`.

Service test organization:

- Group `service` features by product surface first, then by concern.
- Preferred folders under `tests/features/service/` are:
  - `foundation/`
  - `domain/`
  - `pos/`
  - `storefront/`
  - `admin/`
  - `hardening/`
- Use `service` when the behavior can be validated with HTTP requests, JSON bodies, status codes, headers, or server-rendered HTML.
- Use `browser` when the behavior requires JavaScript execution, click flows, DOM mutation, local storage, or visual UI state transitions.
