Feature: Admin orders shell

  Scenario: Signed-out admin orders request returns to the login gate
    Given the bookstore api is running
    When I open the admin orders page
    Then the status code is 200
    And the response contains "Admin Sign-In"
    And the response contains "admin-login-form"
    And the response does not contain "Order ID"
