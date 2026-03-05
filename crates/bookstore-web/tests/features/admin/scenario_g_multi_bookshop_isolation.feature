Feature: Admin scenario G multi-bookshop isolation

  Scenario: Admin is restricted to own tenant records
    Given tenant church-a has a sold line with revenue 2000 cents and cost 1200 cents
    And tenant church-b has a sold line with revenue 5000 cents and cost 3500 cents
    When I build a profit report for tenant church-a
    Then reported revenue is 2000 cents
    And reported cogs is 1200 cents
    And reported gross profit is 800 cents
