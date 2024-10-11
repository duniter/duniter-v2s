@genesis.bad_distance

Feature: Distance fail
#
# WoT:
#
# H<->G<->E<->D<->C-->B
# ^   ^           ^   ^
#  \ /             \ /
#  v v             v v
#   I               A
#
# Every member is referee. Referee count = 8; 80% = 6.4
# Certs from Alice and Bob do not ensure the fulfilling of the distance rule
# because the newcomer would reach only 6 members up to G.


  Scenario: an unvalidated member fails the distance rule
    Then treasury should contain 1 ĞD
    When alice sends 7 ĞD to ferdie
    Then alice should have 0 ĞD reserved
    Then alice should have 199 cĞD
    When bob sends 750 cĞD to ferdie
    When 15 block later
    When alice creates identity for ferdie
    Then ferdie identity should be unconfirmed
    Then ferdie should be certified by alice
    When ferdie confirms his identity with pseudo "ferdie"
    Then ferdie identity should be unvalidated
    When 3 block later
    When bob certifies ferdie
    Then ferdie should be certified by bob
    Then ferdie should have 0 ĞD reserved
    Then ferdie should have 1449 cĞD
    When ferdie requests distance evaluation
    Then ferdie should have 10 ĞD reserved
    Then ferdie should have 449 cĞD
    When 7 blocks later
    Then treasury should contain 102 cĞD
    When alice runs distance oracle
    When 7 blocks later
    Then ferdie should be certified by alice
    Then ferdie should be certified by bob
    # The distance rule is failed
    Then ferdie identity should be unvalidated
    # Ferdie got his reserve slashed
    Then ferdie should have 0 ĞD reserved
    Then ferdie should have 449 cĞD
    # Slashed amount is transfered to treasury
    Then treasury should contain 1102 cĞD
