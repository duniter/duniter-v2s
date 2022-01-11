Feature: Balance transfer

  Scenario: If alice sends 5 ĞD to Dave, Dave will get 5 ĞD
    Given alice have 10 ĞD
    When alice send 5 ĞD to dave
    Then dave have 5 ĞD
