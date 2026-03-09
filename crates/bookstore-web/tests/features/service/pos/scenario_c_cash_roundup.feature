Feature: POS scenario C cash roundup

  Scenario: Customer donates change from cash payment
    Given the bookstore api is running
    When I log into POS with shift pin 1234
    And I scan ISBN 9780060652937
    And I pay cash with tendered 2000 cents and donate change
    Then the status code is 200
    And the response contains "donation_cents"
    And the response contains "301"
    And the response contains "change_due_cents"
