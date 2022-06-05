Feature: Balance transfer

  Scenario: Create a new account with enough founds
    When alice send 5 ĞD to dave
    Then dave should have 5 ĞD
    When 1 block later
    """
    The blockchain should automatically withdraw account creation tax (3 ĞD)
    """
    Then dave should have 2 ĞD

  Scenario: Create a new account without enough founds then retry with enough founds
    When alice send 2 ĞD to eve
    Then eve should have 2 ĞD
    When 1 block later
    """
    The blockchain should automatically destroy Evec account
    because Eve not have enough founds to pay the new account tax
    """
    Then eve should have 0 ĞD
    When alice send 5 ĞD to eve
    Then eve should have 5 ĞD
    When 1 block later
    """
    The blockchain should automatically withdraw account creation tax (3 ĞD)
    """
    Then eve should have 2 ĞD

  Scenario: Create a new account without any founds
    Then eve should have 0 ĞD
    When eve send 0 ĞD to alice
    Then alice should have 10 ĞD
    When alice send 5 ĞD to eve
    Then eve should have 5 ĞD
    When 1 block later
    """
    The blockchain should automatically withdraw account creation tax (3 ĞD)
    """
    Then eve should have 2 ĞD
