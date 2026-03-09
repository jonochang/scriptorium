Feature: Book catalog

  Scenario: Church bookstore catalog endpoint returns seeded title
    Given the bookstore api is running
    When I request the books catalog
    Then the response contains Celebration of Discipline
