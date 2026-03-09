Feature: Admin report date range

  Scenario: Admin filters summary by date range
    Given the bookstore api is running
    When I login as admin with username admin and password admin123
    Then the status code is 200
    When I record sales event for tenant church-a on 2026-02-15 with payment cash sales 500 donations 0 cogs 250
    Then the status code is 200
    When I record sales event for tenant church-a on 2026-03-02 with payment external_card sales 200 donations 10 cogs 120
    Then the status code is 200
    When I fetch admin report summary for tenant church-a from 2026-03-01 to 2026-03-31
    Then the status code is 200
    And the json field sales_cents equals 200
    And the json field donations_cents equals 10
