Feature: Identity creation

  Scenario: alice invites a new member to join the web of trust
    # 6 ĞD covers:
    # - account creation fees (3 ĞD) 
    # - existential deposit (2 ĞD)
    # - transaction fees (below 1 ĞD)
    When alice sends 6 ĞD to dave
    When bob sends 6 ĞD to eve
    # alice last certification is counted from block zero
    # then next cert can be done after cert_period, which is 15
    When 15 block later
    When alice creates identity for dave
    Then dave identity should be created
    Then dave should be certified by alice
    When dave confirms his identity with pseudo "dave"
    Then dave identity should be confirmed
    When 3 block later
    When bob certifies dave
    When charlie certifies dave
    Then dave should be certified by bob
    Then dave should be certified by charlie
    When 3 block later
    When eve validates dave identity
    Then dave identity should be validated
