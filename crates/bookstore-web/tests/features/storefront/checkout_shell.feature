Feature: Storefront checkout shell

  Scenario: Checkout page can create a checkout session from the browser
    Given the bookstore api is running
    When I open the storefront checkout page
    Then the status code is 200
    And the response contains "create-checkout-session"
    And the response contains "/api/storefront/checkout/session"
