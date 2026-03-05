Feature: Storefront checkout and webhook

  Scenario: Checkout session is finalized idempotently
    Given the bookstore api is running
    When I create a storefront checkout session for 1699 cents and email jane@example.com
    Then the status code is 200
    And the response contains "session_id"
    When I finalize payment webhook with reference pay-001 for created session
    Then the status code is 200
    And the response contains "processed"
    And the response contains "receipt_sent"
    When I finalize payment webhook with reference pay-001 for created session
    Then the status code is 200
    And the response contains "duplicate"
