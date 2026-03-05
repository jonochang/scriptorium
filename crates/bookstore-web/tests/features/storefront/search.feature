Feature: Storefront search

  Scenario: Search returns matching catalog entries
    Given the bookstore api is running
    When I search the storefront catalog for discipline
    Then the status code is 200
    And the response contains "Celebration of Discipline"
