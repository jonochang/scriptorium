Feature: POS mobile viewport smoke

  Scenario: POS shell is mobile-first and touch friendly
    Given the bookstore api is running
    When I open the POS shell page
    Then the status code is 200
    And the response contains "width=device-width"
    And the response contains "pos-button--lg"
    And the response contains "Forgot PIN?"
    And the response contains "discount-chip"
    And the response contains "Charcoal"
    And the response contains "SALE COMPLETE"
    And the response contains "changeCartQuantity"
