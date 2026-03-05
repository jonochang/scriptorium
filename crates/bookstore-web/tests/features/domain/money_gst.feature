Feature: Domain money and GST

  Scenario: GST component is derived from GST-inclusive total
    Given a gst-inclusive amount of 1650 cents in AUD
    When I calculate the GST component
    Then the GST component is 150 cents
