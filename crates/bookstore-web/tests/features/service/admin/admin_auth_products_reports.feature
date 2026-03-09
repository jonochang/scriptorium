Feature: Admin auth, products, and reports

  Scenario: Admin manages products and views tenant report summary
    Given the bookstore api is running
    When I login as admin with username admin and password admin123
    Then the status code is 200
    And the response contains "admin"
    When I upsert admin product bk-1 for tenant church-a
    Then the status code is 200
    And the response contains "bk-1"
    When I list admin products for tenant church-a
    Then the status code is 200
    And the response contains "bk-1"
    When I receive admin inventory for tenant church-a isbn 9780060652937 quantity 5
    Then the status code is 200
    When I adjust admin inventory for tenant church-a isbn 9780060652937 by -1 for damage
    Then the status code is 200
    When I fetch admin inventory journal for tenant church-a
    Then the status code is 200
    And the response contains "damage"
    When I log into POS with shift pin 1234
    And I scan ISBN 9780060652937
    And I complete external card checkout with reference square-report
    When I fetch admin report summary for tenant church-a
    Then the status code is 200
    And the response contains "sales_cents"
