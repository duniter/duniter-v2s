Feature: Identity creation

  Scenario: alice invites a new member to join the web of trust
    # 6 ĞD covers:
    # - account creation fees (3 ĞD) 
    # - existential deposit (2 ĞD)
    # - transaction fees (below 1 ĞD)
    When alice sends 6 ĞD to ferdie
    # alice last certification is counted from block zero
    # then next cert can be done after cert_period
    When 15 block later
    When alice creates identity for ferdie
    Then ferdie identity should be created
    When ferdie confirms his identity with pseudo "Ferdie"
    Then ferdie identity should be confirmed 
