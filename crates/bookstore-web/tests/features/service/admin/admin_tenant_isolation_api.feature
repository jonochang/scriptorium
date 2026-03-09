Feature: Admin tenant isolation on APIs

  Scenario: Admin cannot read another tenant summary
    Given the bookstore api is running
    When I login as admin with username admin and password admin123
    Then the status code is 200
    When I fetch admin report summary for tenant church-b
    Then the status code is 403
