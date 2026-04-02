Feature: Leptos storefront checkout migration
  The checkout shell should preserve the existing multi-step checkout markup and client mount points during the Leptos migration.

  Scenario: Checkout page exposes the checkout island shell
    Given the bookstore api is running
    When I open the storefront checkout page
    Then the status code is 200
    And the response contains "/static/wasm/bookstore-cart-wasm.js"
    And the response contains "checkout-step-details"
    And the response contains "create-checkout-session"
    And the response contains "checkout-total"

  Scenario: Checkout page includes parish support controls
    Given the bookstore api is running
    When I open the storefront checkout page
    Then the status code is 200
    And the response contains "data-support-amount"
    And the response contains "Parish support"
