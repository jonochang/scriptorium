Feature: Admin dashboard shell

  Scenario: Admin dashboard page exposes live admin widgets
    Given the bookstore api is running
    When I open the admin dashboard page
    Then the status code is 200
    And the response contains "Good morning, Father Michael"
    And the response contains "page-header"
    And the response contains "Today's Sales"
    And the response contains "POS Revenue"
    And the response contains "Online Revenue"
    And the response contains "Open IOUs"
    And the response contains "/api/admin/reports/summary"
    And the response contains "/api/admin/products"
    And the response contains "report-from"
    And the response contains "admin-payment-breakdown"
    And the response contains "/api/admin/inventory/journal"
    And the response contains "/admin/orders"
    And the response does not contain "hero-card"
