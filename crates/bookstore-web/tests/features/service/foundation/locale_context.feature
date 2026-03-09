Feature: Foundation locale context

  Scenario: Locale context is extracted from request headers
    Given the bookstore api is running
    And I set locale to en-AU
    When I request the request context endpoint
    Then the status code is 200
    And the response contains "en-AU"
