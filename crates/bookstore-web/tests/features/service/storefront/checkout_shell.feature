Feature: Storefront checkout shell

  Scenario: Checkout page can create a checkout session from the browser
    Given the bookstore api is running
    When I open the storefront checkout page
    Then the status code is 200
    And the response contains "create-checkout-session"
    And the response contains "/api/storefront/checkout/session"
    And the response contains "checkout-confirmation"
    And the response contains "checkout-donation-select"
    And the response contains "Contact and delivery"
    And the response contains "Payment"
    And the response contains "4242 4242 4242 4242"
    And the response contains "Place Order"
    And the response contains "checkout-submit-label"
    And the response contains "page-header"
    And the response does not contain "hero-card"
