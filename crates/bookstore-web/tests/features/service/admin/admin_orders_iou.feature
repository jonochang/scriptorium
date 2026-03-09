Feature: Admin orders and IOU management

  Scenario: Admin lists orders and marks an IOU as paid
    Given the bookstore api is running
    When I login as admin with username admin and password admin123
    Then the status code is 200
    When I log into POS with shift pin 1234
    And I scan ISBN 9780060652937
    And I complete IOU checkout for John Doe
    Then the status code is 200
    When I fetch admin orders for tenant church-a
    Then the status code is 200
    And the response contains "John Doe"
    And the response contains "UnpaidIou"
    When I mark the first admin IOU order paid for tenant church-a
    Then the status code is 200
    And the response contains "Paid"
