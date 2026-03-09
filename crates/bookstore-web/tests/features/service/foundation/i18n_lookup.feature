Feature: i18n lookup

  Scenario: Greek translation is returned for checkout completion
    Given the bookstore api is running
    When I lookup i18n key "checkout.complete" for locale "el-GR"
    Then the status code is 200
    And the response contains "Η πώληση ολοκληρώθηκε"
