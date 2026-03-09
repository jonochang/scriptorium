Feature: Admin auth boundaries

  Scenario: Admin APIs reject requests without auth token
    Given the bookstore api is running
    When I list admin products for tenant church-a without auth
    Then the status code is 401
    When I fetch admin report summary for tenant church-a without auth
    Then the status code is 401
