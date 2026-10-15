Feature: Monetary mass

  Scenario: Each UD creates currency for the six members
    Then Monetary mass should grow by 0.00 ĞD
    Then Current UD amount should be 10.00 ĞD
    When the next UD is created
    Then Monetary mass should grow by 60.00 ĞD
    When the next UD is created
    Then Monetary mass should grow by 120.00 ĞD
    Then Current UD amount should be 10.00 ĞD
