Feature: POS error handling

  Scenario: Cash checkout fails when the cart is empty
    Given the bookstore api is running
    When I log into POS with shift pin 1234
    And I attempt cash checkout on an empty cart with tendered 2000 cents
    Then the status code is 400
    And the response contains "cart is empty"

  Scenario: POS scan returns a structured error for an unknown barcode
    Given the bookstore api is running
    When I log into POS with shift pin 1234
    And I scan ISBN 0000000000000
    Then the status code is 400
    And the response contains "bad_request"
    And the response contains "unknown barcode"

  Scenario: POS scan returns a structured error for a blank session token
    Given the bookstore api is running
    When I scan ISBN 9780060652937 with a blank POS session token
    Then the status code is 400
    And the response contains "bad_request"
    And the response contains "invalid session token"
