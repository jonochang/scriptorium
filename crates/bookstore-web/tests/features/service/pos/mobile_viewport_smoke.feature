Feature: POS mobile viewport smoke

  Scenario: POS shell is mobile-first and touch friendly
    Given the bookstore api is running
    When I open the POS shell page
    Then the status code is 200
    And the response contains "width=device-width"
    And the response contains "pos-button--lg"
    And the response contains "pos-header"
    And the response contains "Forgot PIN?"
    And the response contains "discount-chip"
    And the response contains "Charcoal"
    And the response contains "SALE COMPLETE"
    And the response contains "changeCartQuantity"
    And the response does not contain "Screen 1 · PIN Login"
    And the response does not contain "Screen 2 · Main POS"
    And the response does not contain "Screen 3 · Payment"
    And the response does not contain "Screen 4 · Complete"
    And the response does not contain "New volunteer login"
    And the response does not contain "Reload sample ISBN"
    And the response does not contain "Use the round-up flow from the design spec."
