@genesis.default
Feature: Balance transfer all

  Scenario: If bob sends all his ĞDs to Dave
    When bob sends all her ĞDs to dave
    """
    Bob is a member, as such he is not allowed to empty his account completely,
    if he tries to do so, the existence deposit (2 ĞD) must remain.
    """
    Then bob should have 2 ĞD
    """
    10 ĞD (initial Bob balance) - 2 ĞD (Existential deposit) - 0.02 ĞD (transaction fees)
    """
    Then dave should have 798 cĞD
