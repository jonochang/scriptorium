Feature: Foundation tenant context

  Scenario: Tenant context is extracted from request headers
    Given the bookstore api is running
    And I set tenant id to church-a
    When I request the request context endpoint
    Then the status code is 200
    And the response contains "church-a"
