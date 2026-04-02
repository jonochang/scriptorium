Feature: Leptos mount readiness
  The storefront shell should continue to expose the WASM mount points needed by the Leptos migration.

  Scenario: Storefront catalog includes the WASM bundle
    Given the bookstore api is running
    When I open the storefront catalog page
    Then the status code is 200
    And the response contains "/static/wasm/bookstore-cart-wasm.js"
    And the response contains "site-cart-count"

  Scenario: Storefront cart shell includes the cart island mount points
    Given the bookstore api is running
    When I open the storefront cart page
    Then the status code is 200
    And the response contains "/static/wasm/bookstore-cart-wasm.js"
    And the response contains "cart-items"
    And the response contains "site-cart-count"
