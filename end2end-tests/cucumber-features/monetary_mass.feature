Feature: Balance transfer

  Scenario: After 10 blocks, the monetary mass should be 60 ĞD
    Then Monetary mass should be 30.00 ĞD
    Then Current UD amount should be 10.00 ĞD
    When 10 blocks later
    Then Monetary mass should be 60.00 ĞD
    When 10 blocks later
    Then Monetary mass should be 90.00 ĞD
    Then Current UD amount should be 10.00 ĞD
