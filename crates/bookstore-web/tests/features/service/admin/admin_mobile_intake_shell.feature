Feature: Admin mobile intake shell

  Scenario: Signed-out admin intake request returns to the login gate
    Given the bookstore api is running
    When I open the admin intake page
    Then the status code is 200
    And the response contains "Admin Sign-In"
    And the response contains "Continue to dashboard"
    And the response does not contain "BarcodeDetector"
