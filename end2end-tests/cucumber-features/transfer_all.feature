@genesis.default
Feature: Balance transfer all

  Scenario: If alice sends all her ĞDs to Dave, Dave will get 8 ĞD
    When alice sends all her ĞDs to dave
    """
    Alice is a smith member, as such she is not allowed to empty her account completely,
    if she tries to do so, the existence deposit (2 ĞD) must remain.
    """
    Then alice should have 2 ĞD
    Then dave should have 8 ĞD
