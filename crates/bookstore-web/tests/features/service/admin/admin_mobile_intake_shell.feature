Feature: Admin mobile intake shell

  Scenario: Admin opens mobile camera ISBN intake page
    Given the bookstore api is running
    When I open the admin intake page
    Then the status code is 200
    And the response contains "getUserMedia"
    And the response contains "BarcodeDetector"
    And the response contains "/api/admin/products/isbn-lookup"
    And the response contains "intake-grid"
    And the response contains "field-label"
    And the response contains "scanner-status"
    And the response contains "camera-start"
    And the response contains "Publisher"
    And the response contains "Upload Cover"
    And the response contains "Save Product"
    And the response contains "Cancel"
    And the response contains "Cost price"
    And the response contains "Retail price"
    And the response contains "Initial stock"
    And the response contains "Reorder point"
    And the response contains "Icons"
    And the response contains "St. Herman Press"
    And the response contains "Holy Trinity"
    And the response contains "page-header"
    And the response does not contain "Admin Token"
    And the response does not contain "hero-card"
