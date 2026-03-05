Feature: POS checkout atomicity

  Scenario: Failed IOU validation does not partially finalize checkout
    Given the bookstore api is running
    When I log into POS with shift pin 1234
    And I scan ISBN 9780060652937
    And I attempt IOU checkout with blank customer name
    Then the status code is 400
    When I complete external card checkout with reference square-atomic
    Then the status code is 200
    And the response contains "sale_complete"
    And the response contains "1699"
