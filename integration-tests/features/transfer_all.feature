Feature: Balance transfer all

  Scenario: If alice sends all her ĞDs to Dave, Dave will get 10 ĞD
    Given alice have 10 ĞD
    When alice sends all her ĞDs to dave
    Then dave should have 10 ĞD
