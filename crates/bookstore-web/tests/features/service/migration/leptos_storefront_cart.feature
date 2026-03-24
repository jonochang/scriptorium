Feature: Leptos storefront cart migration
  The storefront cart shell should preserve the current cart-facing affordances during the Leptos migration.

  Scenario: Catalog and cart pages expose the cart island shell
    Given the bookstore api is running
    When I open the storefront catalog page
    Then the status code is 200
    And the response contains "/static/wasm/bookstore-cart-wasm.js"
    And the response contains "site-cart-count"
    When I open the storefront cart page
    Then the status code is 200
    And the response contains "cart-items"
    And the response contains "Cart total"

  Scenario: Product detail page exposes quantity-aware add-to-cart controls
    Given the bookstore api is running
    When I open the storefront product page for bk-100
    Then the status code is 200
    And the response contains "detail-quantity"
    And the response contains "Add to Cart"
