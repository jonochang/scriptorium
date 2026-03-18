Feature: Storefront catalog browse

  Scenario: Catalog page shows available books
    Given the bookstore api is running
    When I open the storefront catalog page
    Then the status code is 200
    And the response contains "admin-topbar"
    And the response contains "admin-topnav"
    And the response contains "admin-header"
    And the response contains ">SCRIPTORIUM<"
    And the response contains "Feed your soul."
    And the response contains "Celebration of Discipline"
    And the response contains "Books"
    And the response contains "Icons"
    And the response contains "Liturgical"
    And the response contains "Gifts"
    And the response contains "category-chip"
    And the response contains "catalog-feedback"
    And the response contains "data-add-book-id"
    And the response contains "site-cart-count"
    And the response contains "/static/wasm/bookstore-cart-wasm"
    And the response contains "Only 2 left"
    And the response contains "pagination-link"
    And the response contains "catalog-card__link"
    And the response does not contain "hero-card"
    And the response does not contain "function readCart"

  Scenario: WASM cart module is served as a static asset
    Given the bookstore api is running
    When I fetch static file "/static/wasm/bookstore-cart-wasm.js"
    Then the status code is 200
    And the response contains "wasm"
