Feature: POS scenario B quick items

  Scenario: Volunteer adds two 50 cent prayer cards
    Given the bookstore api is running
    When I log into POS with shift pin 1234
    And I add quick item prayer-card-50c with quantity 2
    Then the status code is 200
    And the response contains "total_cents"
    And the response contains "100"
