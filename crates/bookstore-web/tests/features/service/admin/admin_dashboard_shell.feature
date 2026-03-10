Feature: Admin dashboard shell

  Scenario: Signed-out admin dashboard request shows the login screen
    Given the bookstore api is running
    When I open the admin dashboard page
    Then the status code is 200
    And the response contains "Admin Sign-In"
    And the response contains "admin-login-form"
    And the response contains "Continue to dashboard"
    And the response does not contain "Today's Sales"
    And the response does not contain "POS Revenue"
