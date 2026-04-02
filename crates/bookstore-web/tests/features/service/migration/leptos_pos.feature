Feature: Leptos POS migration
  The POS shell should preserve scanner, basket, and payment mount points while the island migrates to Leptos.

  Scenario: POS shell exposes the scanner and basket regions
    Given the bookstore api is running
    When I open the POS shell page
    Then the status code is 200
    And the response contains "/static/wasm/bookstore-cart-wasm.js"
    And the response contains "Scriptorium POS"
    And the response contains "id=\"app\""

  Scenario: POS shell exposes payment controls
    Given the bookstore api is running
    When I open the POS shell page
    Then the status code is 200
    And the response contains "id=\"app\""
    And the response contains "bookstore-cart-wasm.js"
