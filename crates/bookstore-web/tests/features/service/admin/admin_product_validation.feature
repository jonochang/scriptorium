Feature: Admin product validation

  Scenario: Admin gets a clear validation message for an invalid ISBN
    Given the bookstore api is running
    When I login as admin with username admin and password admin123
    Then the status code is 200
    When I attempt admin upsert with invalid isbn abc for tenant church-a
    Then the status code is 400
    And the response contains "ISBN must contain digits."
