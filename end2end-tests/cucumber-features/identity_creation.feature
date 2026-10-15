Feature: Identity creation

  Scenario: alice invites a new member to join the web of trust
    # 6 ĞD covers:
    # - existential deposit (1 ĞD)
    # - transaction fees (below 1 ĞD)
    When alice sends 7 ĞD to dave
    # Alice is treasury funder for 1 ĞD => 10-1-7 = 2 (minus TODO fees)
    Then alice should have 0 ĞD reserved
    Then alice should have 2 ĞD
    When bob sends 750 cĞD to dave
    When charlie sends 6 ĞD to eve
    When alice creates identity for dave
    Then dave identity should be unconfirmed
    Then dave should be certified by alice
    When dave confirms his identity with pseudo "dave"
    Then dave identity should be unvalidated
    When bob certifies dave
    When one certifies dave
    When two certifies dave
    When charlie certifies dave
    Then dave should be certified by bob
    Then dave should be certified by charlie
    Then dave should have 0 ĞD reserved
    # 700 + 750 - 2(one transaction fee, not a member yet)
    Then dave should have 1448 cĞD
    When dave requests distance evaluation
    Then dave should have 10 ĞD reserved
    # 1448 - 1000 - 2(one transaction fee, not a member yet)
    Then dave should have 446 cĞD
    When the distance request enters the computation pool
    When alice runs distance oracle
    Then the oracle should compute 100 percent accessibility
    When the distance result is applied
    Then dave identity should be member
    Then dave should have 0 ĞD reserved
    Then dave should have 1446 cĞD
