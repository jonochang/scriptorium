Feature: Admin orders shell

  Scenario: Admin orders page exposes the dedicated order-management surface
    Given the bookstore api is running
    When I open the admin orders page
    Then the status code is 200
    And the response contains "Order Management"
    And the response contains "data-order-filter"
    And the response contains "Export"
    And the response contains "orders-table"
    And the response contains "Order ID"
    And the response contains "Actions"
    And the response contains "page-header"
    And the response does not contain "hero-card"
