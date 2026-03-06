Feature: Storefront catalog browse

  Scenario: Catalog page shows available books
    Given the bookstore api is running
    When I open the storefront catalog page
    Then the status code is 200
    And the response contains "Celebration of Discipline"
    And the response contains "category-chip"
    And the response contains "catalog-feedback"
    And the response contains "data-add-book-id"
