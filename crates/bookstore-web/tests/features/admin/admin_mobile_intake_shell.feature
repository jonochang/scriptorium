Feature: Admin mobile intake shell

  Scenario: Admin opens mobile camera ISBN intake page
    Given the bookstore api is running
    When I open the admin intake page
    Then the status code is 200
    And the response contains "getUserMedia"
    And the response contains "BarcodeDetector"
    And the response contains "/api/admin/products/isbn-lookup"
