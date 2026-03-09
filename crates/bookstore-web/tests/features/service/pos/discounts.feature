Feature: POS discounts and payment invariants

  Scenario: Cash checkout rejects underpayment for a scanned title
    Given the bookstore api is running
    When I log into POS with shift pin 1234
    And I scan ISBN 9780060652937
    And I attempt cash checkout with tendered 1000 cents
    Then the status code is 400
    And the response contains "tendered amount is less than cart total"

  Scenario: POS discount changes the charged amount and reporting totals
    Given the bookstore api is running
    When I log into POS with shift pin 1234
    And I scan ISBN 9780060652937
    And I complete external card checkout with reference square-discount and discount 170 cents
    Then the status code is 200
    And the response contains "sale_complete"
    And the response contains "1529"
    And the response contains "170"
    When I login as admin with username admin and password admin123
    Then the status code is 200
    When I fetch admin orders for tenant church-a
    Then the status code is 200
    And the response contains "1529"
    When I fetch admin report summary for tenant church-a
    Then the status code is 200
    And the response contains "external_card"
    And the response contains "1529"
