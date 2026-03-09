Feature: Admin scenario F profit visibility

  Scenario: Admin can see revenue cogs and gross profit
    Given tenant church-a has a sold line with revenue 2000 cents and cost 1200 cents
    And tenant church-a has a sold line with revenue 1000 cents and cost 700 cents
    When I build a profit report for tenant church-a
    Then reported revenue is 3000 cents
    And reported cogs is 1900 cents
    And reported gross profit is 1100 cents
