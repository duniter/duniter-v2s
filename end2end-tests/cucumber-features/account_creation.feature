Feature: Account creation

  Scenario: Create a new account with enough funds
    When alice sends 5 ĞD to dave
    Then dave should have 5 ĞD
    When 1 block later
    """
    The blockchain did not automatically withdraw account creation tax (3 ĞD) because this feature has been removed
    """
    Then dave should have 5 ĞD

  Scenario: Create a new account without enough funds then retry with enough funds
    When alice sends 2 ĞD to eve
    Then eve should have 2 ĞD
    When 1 block later
    """
    The blockchain did not automatically destroy Eve account for Eve not having enough funds to pay the new account tax
    Because this feature has been removed
    """
    Then eve should have 2 ĞD
    When alice send 5 ĞD to eve
    Then eve should have 7 ĞD
    When 1 block later
    """
    The blockchain did not automatically withdraw account creation tax (3 ĞD) because this feature has been removed
    """
    Then eve should have 7 ĞD

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
    The blockchain did not automatically withdraw account creation tax (3 ĞD) because this feature has been removed
    """
    Then eve should have 5 ĞD
