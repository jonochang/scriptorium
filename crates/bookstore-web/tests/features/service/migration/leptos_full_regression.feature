Feature: Leptos migration full regression shell
  The key app shells should continue to load their WASM islands while the migration proceeds incrementally.

  Scenario: Storefront pages still load the shared WASM bundle
    Given the bookstore api is running
    When I open the storefront catalog page
    Then the status code is 200
    And the response contains "/static/wasm/bookstore-cart-wasm.js"
    When I open the storefront checkout page
    Then the status code is 200
    And the response contains "/static/wasm/bookstore-cart-wasm.js"
    When I open the storefront cart page
    Then the status code is 200
    And the response contains "/static/wasm/bookstore-cart-wasm.js"

  Scenario: Admin and POS shells still load the shared WASM bundle
    Given the bookstore api is running
    When I open the admin dashboard page
    Then the status code is 200
    And the response contains "/static/wasm/bookstore-cart-wasm.js"
    When I open the admin intake page
    Then the status code is 200
    And the response contains "/static/wasm/bookstore-cart-wasm.js"
    When I open the POS shell page
    Then the status code is 200
    And the response contains "/static/wasm/bookstore-cart-wasm.js"
