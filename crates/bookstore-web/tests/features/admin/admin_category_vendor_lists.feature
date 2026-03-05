Feature: Admin category and vendor management

  Scenario: Admin lists tenant categories and vendors from products
    Given the bookstore api is running
    When I login as admin with username admin and password admin123
    Then the status code is 200
    When I upsert admin product bk-1 for tenant church-a
    Then the status code is 200
    When I fetch admin categories for tenant church-a
    Then the status code is 200
    And the response contains "Spiritual Formation"
    When I fetch admin vendors for tenant church-a
    Then the status code is 200
    And the response contains "Church Supplier"
    When I delete admin product bk-1 for tenant church-a
    Then the status code is 200
    And the response contains "deleted"
    When I list admin products for tenant church-a
    Then the status code is 200
    And the response does not contain "bk-1"
