Feature: Admin CSRF protection

  Scenario: Admin write APIs reject cross-origin requests
    Given the bookstore api is running
    When I login as admin with username admin and password admin123
    Then the status code is 200
    When I attempt cross-origin admin product upsert for tenant church-a
    Then the status code is 403
