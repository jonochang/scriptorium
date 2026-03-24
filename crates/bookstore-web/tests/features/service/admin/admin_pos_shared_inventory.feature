Feature: Admin and POS share inventory data

  Scenario: Product saved in admin becomes scannable in POS and disappears after delete
    Given the bookstore api is running
    When I login as admin with username admin and password admin123
    Then the status code is 200
    When I upsert admin product bk-sync with isbn 9781802063271 for tenant church-a
    Then the status code is 200
    When I receive admin inventory for tenant church-a isbn 9781802063271 quantity 3
    Then the status code is 200
    When I log into POS with shift pin 1234
    And I scan ISBN 9781802063271
    Then the status code is 200
    And the response contains "Item added to cart"
    When I delete admin product bk-sync for tenant church-a
    Then the status code is 200
    When I log into POS with shift pin 1234
    And I scan ISBN 9781802063271
    Then the status code is 400
    And the response contains "Book not found"
