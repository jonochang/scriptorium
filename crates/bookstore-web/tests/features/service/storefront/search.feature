Feature: Storefront search

  Scenario: Search returns matching catalog entries
    Given the bookstore api is running
    When I search the storefront catalog for discipline
    Then the status code is 200
    And the response contains "Celebration of Discipline"

  Scenario: Catalog page supports plain form-submit search fallback
    Given the bookstore api is running
    When I open the storefront catalog page filtered for discipline
    Then the status code is 200
    And the response contains "Celebration of Discipline"
    And the response contains "htmx.org"
