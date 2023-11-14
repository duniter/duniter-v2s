Feature: Account creation

  Scenario: Create a new account with enough funds
    When alice sends 5 ĞD to dave
    Then dave should have 5 ĞD
    When 1 block later
    """
    The blockchain should automatically withdraw account creation tax (3 ĞD)
    """
    Then dave should have 2 ĞD

  Scenario: Create a new account without enough funds then retry with enough funds
    When alice sends 2 ĞD to eve
    Then eve should have 2 ĞD
    When 1 block later
    """
    The blockchain should automatically destroy Eve account
    because Eve does not have enough funds to pay the new account tax
    """
    Then eve should have 0 ĞD
    When alice send 5 ĞD to eve
    Then eve should have 5 ĞD
    When 1 block later
    """
    The blockchain should automatically withdraw account creation tax (3 ĞD)
    """
    Then eve should have 2 ĞD

  @ignoreErrors
  Scenario: Create a new account without any funds
    Then eve should have 0 ĞD
    # Alice is treasury funder for 1 ĞD
    Then alice should have 9 ĞD
    When eve send 0 ĞD to alice
    Then alice should have 9 ĞD
    When alice send 5 ĞD to eve
    Then eve should have 5 ĞD
    When 1 block later
    """
    The blockchain should automatically withdraw account creation tax (3 ĞD)
    """
    Then eve should have 2 ĞD
