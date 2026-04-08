Feature: Admin inventory uses server-side search and pagination

  Scenario: Inventory search and paging reflect saved database updates
    Given the bookstore api is running
    When I login as admin with username admin and password admin123
    Then the status code is 200
    When I upsert admin product inv-alpha with isbn 9781802063271 for tenant church-a
    Then the status code is 200
    When I upsert admin product inv-beta with isbn 9781802063272 title "Shared Inventory Test Beta" for tenant church-a
    Then the status code is 200
    When I upsert admin product inv-gamma with isbn 9781802063273 title "Shared Inventory Test Gamma" for tenant church-a
    Then the status code is 200
    When I fetch admin inventory products for tenant church-a page 1 per page 2 search Shared category All stock All
    Then the status code is 200
    And the response contains "\"total_matches\":3"
    And the response contains "\"total_pages\":2"
    And the response contains "Shared Inventory Test Beta"
    And the response contains "Shared Inventory Test Gamma"
    When I fetch admin inventory products for tenant church-a page 2 per page 2 search Shared category All stock All
    Then the status code is 200
    And the response contains "\"page\":2"
    And the response contains "Shared Inventory Test Title"
    When I update admin product inv-gamma title to "Parish Supply Gamma"
    Then the status code is 200
    When I fetch admin inventory products for tenant church-a page 1 per page 25 search Parish category All stock All
    Then the status code is 200
    And the response contains "Parish Supply Gamma"
    And the response does not contain "Shared Inventory Test Gamma"
