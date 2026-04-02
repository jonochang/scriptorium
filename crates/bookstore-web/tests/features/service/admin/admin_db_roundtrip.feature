Feature: Admin, catalog, and POS share persisted product data

  Scenario: Product and stock survive a server restart and appear in admin, catalog, and POS
    Given the bookstore api is running
    When I login as admin with username admin and password admin123
    Then the status code is 200
    When I upsert admin product bk-db-roundtrip with isbn 9781802063271 for tenant church-a
    Then the status code is 200
    When I receive admin inventory for tenant church-a isbn 9781802063271 quantity 4
    Then the status code is 200
    When I restart the bookstore api using the same database
    When I login as admin with username admin and password admin123
    Then the status code is 200
    When I list admin products for tenant church-a
    Then the status code is 200
    And the response contains "Shared Inventory Test Title"
    And admin product bk-db-roundtrip has quantity on hand 4
    When I search the storefront catalog for Shared
    Then the status code is 200
    And the response contains "Shared Inventory Test Title"
    When I log into POS with shift pin 1234
    Then the status code is 200
    When I scan ISBN 9781802063271
    Then the status code is 200
    And the response contains "Shared Inventory Test Title"
