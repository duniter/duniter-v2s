Feature: Oneshot account

  Scenario: Simple oneshot consumption
    When charlie sends 7 ĞD to dave
    # Cover the oneshot calls fees
    When alice sends 7 ĞD to oneshot dave
    # Alice is treasury funder for 1 ĞD, and member so fees are refunded
    Then alice should have 2 ĞD
    Then dave should have oneshot 7 ĞD
    When oneshot dave consumes into account bob
    Then dave should have oneshot 0 ĞD
    Then bob should have 1698 cĞD
    Then bob should have oneshot 0 ĞD

  Scenario: Double oneshot consumption
    When charlie sends 7 ĞD to dave
    Then charlie should have 299 cĞD
    # Cover the oneshot calls fees
    When alice sends 7 ĞD to oneshot dave
    # Alice is treasury funder for 1 ĞD, and member so fees are refunded
    Then alice should have 2 ĞD
    Then dave should have oneshot 7 ĞD
    When oneshot dave consumes 4 ĞD into account bob and the rest into oneshot charlie
    Then dave should have oneshot 0 ĞD
    Then bob should have 14 ĞD
    Then bob should have oneshot 0 ĞD
    Then charlie should have 299 cĞD
    Then charlie should have oneshot 298 cĞD
