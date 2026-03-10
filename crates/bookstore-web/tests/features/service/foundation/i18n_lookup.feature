Feature: i18n lookup

  Scenario: Greek translation is returned for checkout completion
    Given the bookstore api is running
    When I lookup i18n key "checkout.complete" for locale "el-GR"
    Then the status code is 200
    And the response contains "Η πώληση ολοκληρώθηκε"

  Scenario: English fallback is returned for supported keys outside configured locales
    Given the bookstore api is running
    When I lookup i18n key "admin.intake.title" for locale "fr-FR"
    Then the status code is 200
    And the response contains "Admin Inventory Intake"
