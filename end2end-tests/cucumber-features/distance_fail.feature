@setup.bad_distance

Feature: Distance fail
# Two cliques of six members each have five received certifications per member.
# Five issuers from the first clique certify Ferdie. He reaches 6 / 12 referees.

  Scenario: an unvalidated member fails the distance rule
    Then treasury should contain 1 ĞD
    When alice sends 7 ĞD to ferdie
    Then alice should have 0 ĞD reserved
    Then alice should have 2 ĞD
    When bob sends 750 cĞD to ferdie
    When alice creates identity for ferdie
    Then ferdie identity should be unconfirmed
    Then ferdie should be certified by alice
    When ferdie confirms his identity with pseudo "ferdie"
    Then ferdie identity should be unvalidated
    When bob certifies ferdie
    When charlie certifies ferdie
    When one certifies ferdie
    When two certifies ferdie
    Then ferdie should be certified by bob
    Then ferdie should have 0 ĞD reserved
    # 700 + 750 - 2(one transaction fee, not a member yet)
    Then ferdie should have 1448 cĞD
    When ferdie requests distance evaluation
    Then ferdie should have 10 ĞD reserved
    # 1448 - 1000 - 2(one transaction fee, not a member yet)
    Then ferdie should have 446 cĞD
    When the distance request enters the computation pool
    Then treasury should contain 104 cĞD
    When alice runs distance oracle
    Then the oracle should compute 50 percent accessibility
    When the distance result is applied
    Then ferdie should be certified by alice
    Then ferdie should be certified by bob
    # The distance rule is failed
    Then ferdie identity should be unvalidated
    # Ferdie gets his evaluation deposit back
    Then ferdie should have 0 ĞD reserved
    Then ferdie should have 1446 cĞD
    # A negative distance result does not pay the treasury
    Then treasury should contain 104 cĞD
