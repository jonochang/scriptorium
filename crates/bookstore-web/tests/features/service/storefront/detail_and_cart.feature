Feature: Storefront detail and cart

  Scenario: Product detail page links the catalog into the cart flow
    Given the bookstore api is running
    When I open the storefront product page for bk-900
    Then the status code is 200
    And the response contains "Add to Cart"
    And the response contains "/static/wasm/bookstore-cart-wasm"
    And the response contains "detail-quantity"
    And the response contains "Related titles"
    And the response contains "A compact prayer companion"
    And the response contains "Description"
    And the response contains "Details"
    And the response contains "Publisher"
    And the response contains "Binding"
    And the response contains "Pages"
    And the response contains "In stock"

  Scenario: Missing product detail page returns a friendly 404 shell
    Given the bookstore api is running
    When I open the storefront product page for bk-missing
    Then the status code is 404
    And the response contains "We could not find that product"
    And the response contains "Back to catalog"

  Scenario: Cart page renders the cart shell
    Given the bookstore api is running
    When I open the storefront cart page
    Then the status code is 200
    And the response contains "admin-topbar"
    And the response contains "admin-header"
    And the response contains "Review your basket"
    And the response contains "Cart total"
    And the response contains "data-recommendation-book-id"
    And the response contains "data-recommendation-title"
