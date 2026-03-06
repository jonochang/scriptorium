Feature: Admin dashboard shell

  Scenario: Admin dashboard page exposes live admin widgets
    Given the bookstore api is running
    When I open the admin dashboard page
    Then the status code is 200
    And the response contains "Dashboard, stock, and reporting"
    And the response contains "/api/admin/reports/summary"
    And the response contains "/api/admin/products"
    And the response contains "report-from"
    And the response contains "default username"
    And the response contains "admin-payment-breakdown"
    And the response contains "/api/admin/inventory/journal"
    And the response contains "admin-export"
