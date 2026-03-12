Feature: Storefront checkout and webhook

  Scenario: Checkout session creates order directly and webhook is idempotent
    Given the bookstore api is running
    When I create a storefront checkout session for item bk-102 quantity 1 and email jane@example.com
    Then the status code is 200
    And the response contains "session_id"
    And the response contains "order_id"
    And the response contains "2417"
    When I login as admin with username admin and password admin123
    Then the status code is 200
    When I fetch admin orders for tenant church-a
    Then the status code is 200
    And the response contains "Online"
    And the response contains "jane@example.com"
    And the response contains "2417"
    When I fetch admin report summary for tenant church-a
    Then the status code is 200
    And the response contains "online_card"
    And the response contains "2417"
    When I finalize payment webhook with reference pay-001 for created session
    Then the status code is 200
    And the response contains "duplicate"
    When I finalize payment webhook with reference pay-002 for created session
    Then the status code is 200
    And the response contains "duplicate"
