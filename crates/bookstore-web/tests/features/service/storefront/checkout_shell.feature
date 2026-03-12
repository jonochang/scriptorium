Feature: Storefront checkout shell

  Scenario: Checkout page can create a checkout session from the browser
    Given the bookstore api is running
    When I open the storefront checkout page
    Then the status code is 200
    And the response contains "checkout-step"
    And the response contains "Contact &amp; delivery"
    And the response contains "Parish support"
    And the response contains "Continue to payment"
    And the response contains "Card details"
    And the response contains "checkout-card-number"
    And the response contains "checkout-card-expiry"
    And the response contains "checkout-card-cvc"
    And the response contains "Order summary"
    And the response contains "checkout-trust-receipt"
    And the response contains "create-checkout-session"
    And the response contains "/api/storefront/checkout/session"
    And the response contains "page-header"
    And the response does not contain "hero-card"
