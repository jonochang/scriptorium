Feature: Storefront detail and cart

  Scenario: Product detail page links the catalog into the cart flow
    Given the bookstore api is running
    When I open the storefront product page for bk-900
    Then the status code is 200
    And the response contains "Add to cart"
    And the response contains "scriptorium-storefront-cart"

  Scenario: Cart page renders the cart shell
    Given the bookstore api is running
    When I open the storefront cart page
    Then the status code is 200
    And the response contains "Review your basket"
    And the response contains "Cart total"
