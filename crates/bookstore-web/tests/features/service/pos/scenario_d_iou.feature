Feature: POS scenario D iou

  Scenario: Customer purchase is placed on IOU
    Given the bookstore api is running
    When I log into POS with shift pin 1234
    And I scan ISBN 9780060652937
    And I complete IOU checkout for John Doe
    Then the status code is 200
    And the response contains "status"
    And the response contains "iou"
