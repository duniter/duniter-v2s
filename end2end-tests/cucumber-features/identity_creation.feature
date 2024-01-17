Feature: Identity creation

  Scenario: alice invites a new member to join the web of trust
    # 6 ĞD covers:
    # - account creation fees (3 ĞD)
    # - existential deposit (1 ĞD)
    # - transaction fees (below 1 ĞD)
    When alice sends 7 ĞD to dave
    # Alice is treasury funder for 1 ĞD => 10-1-7 = 2 (minus TODO fees)
    Then alice should have 199 cĞD
    When bob sends 750 cĞD to dave
    When charlie sends 6 ĞD to eve
    # alice last certification is counted from block zero
    # then next cert can be done after cert_period, which is 15
    When 15 block later
    When alice creates identity for dave
    Then dave identity should be unconfirmed
    Then dave should be certified by alice
    When dave confirms his identity with pseudo "dave"
    Then dave identity should be unvalidated
    When 3 block later
    When bob certifies dave
    When charlie certifies dave
    Then dave should be certified by bob
    Then dave should be certified by charlie
    When dave requests distance evaluation
    Then dave should have distance result in 2 sessions
    When 30 blocks later
    Then dave should have distance result in 1 session
    When alice runs distance oracle
    When 30 blocks later
    Then dave identity should be member
