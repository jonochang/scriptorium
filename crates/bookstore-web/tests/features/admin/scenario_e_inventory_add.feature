Feature: Admin scenario E inventory add

  Scenario: Admin scans ISBN and records initial inventory
    Given I scan ISBN 9780060652937 for admin intake
    When I lookup isbn metadata for intake
    Then the intake metadata title is "Celebration of Discipline"
    And the intake metadata author is "Richard Foster"
    When I record intake with cost 900 cents retail 1699 cents and quantity 5
    Then the intake quantity on hand is 5
