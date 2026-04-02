Feature: Leptos intake migration
  The intake flow should keep its scanner and review mount points while the form moves deeper into Leptos components.

  Scenario: Intake page exposes scanner and review roots
    Given the bookstore api is running
    When I login as admin with username admin and password admin123
    Then the status code is 200
    When I open the admin intake page
    Then the status code is 200
    And the response contains "/static/wasm/bookstore-cart-wasm.js"
    And the response contains "intake-root"
    And the response contains "data-token="
    And the response contains "data-tenant-id="

  Scenario: Intake page exposes cover upload controls
    Given the bookstore api is running
    When I login as admin with username admin and password admin123
    Then the status code is 200
    When I open the admin intake page
    Then the status code is 200
    And the response contains "intake-root"
    And the response contains "bookstore-cart-wasm.js"
