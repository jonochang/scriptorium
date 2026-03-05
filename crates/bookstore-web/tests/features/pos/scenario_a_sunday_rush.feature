Feature: POS scenario A sunday rush

  Scenario: Volunteer scans a book and completes card checkout
    Given the bookstore api is running
    When I log into POS with shift pin 1234
    And I scan ISBN 9780060652937
    And I complete external card checkout with reference square-123
    Then the status code is 200
    And the response contains "sale_complete"
