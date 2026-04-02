Feature: Leptos admin dashboard migration
  The admin dashboard shells should preserve the current data mount points while the island migrates to Leptos.

  Scenario: Admin dashboard shell exposes reporting and inventory regions
    Given the bookstore api is running
    When I login as admin with username admin and password admin123
    Then the status code is 200
    When I open the admin dashboard page
    Then the status code is 200
    And the response contains "/static/wasm/bookstore-cart-wasm.js"
    And the response contains "metric-today-sales"
    And the response contains "admin-products"
    And the response contains "admin-orders"
    And the response contains "admin-journal"

  Scenario: Admin orders shell exposes order actions region
    Given the bookstore api is running
    When I login as admin with username admin and password admin123
    Then the status code is 200
    When I open the admin orders page
    Then the status code is 200
    And the response contains "/static/wasm/bookstore-cart-wasm.js"
    And the response contains "admin-orders"
    And the response contains "order-summary-count"
