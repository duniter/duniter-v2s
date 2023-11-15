@genesis.default
Feature: Balance transfer all

  Scenario: If bob sends all his ĞDs to Dave
    When bob sends all his ĞDs to dave
    """
    Bob is a member, as such he is not allowed to empty his account completely,
    if he tries to do so, the existence deposit (1 ĞD) must remain.
    Bob is a member, transaction fees are refunded for him
    101 = existential deposit (100) + fees refunded using quota (001)
    """
    Then bob should have 101 cĞD
    """
    10 ĞD (initial Bob balance) - 1 ĞD (Existential deposit) - 0.02 ĞD (transaction fees)
    """
    Then dave should have 898 cĞD
    # TODO check that the missing cent went to treasury
