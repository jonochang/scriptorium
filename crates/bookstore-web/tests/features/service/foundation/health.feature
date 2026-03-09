Feature: Foundation health

  Scenario: Health endpoint is alive
    Given the bookstore api is running
    When I request the health endpoint
    Then the status code is 200
    And the response contains "ok"
