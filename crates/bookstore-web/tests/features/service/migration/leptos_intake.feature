Feature: Leptos intake migration
  The intake flow should keep its scanner and review mount points while the form moves deeper into Leptos components.

  Scenario: Intake page exposes scanner and review roots
    Given the bookstore api is running
    When I open the admin intake page
    Then the status code is 200
    And the response contains "/static/wasm/bookstore-cart-wasm.js"
    And the response contains "intake-root"
    And the response contains "camera"
    And the response contains "scanner-status"
    And the response contains "Save Product"

  Scenario: Intake page exposes cover upload controls
    Given the bookstore api is running
    When I open the admin intake page
    Then the status code is 200
    And the response contains "Upload cover"
    And the response contains "Attach file"
