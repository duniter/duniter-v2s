Feature: Oneshot account

  Scenario: Simple oneshot consumption
    When alice sends 7 ĞD to oneshot dave
    # Alice is treasury funder for 1 ĞD
    Then alice should have 2 ĞD
    Then dave should have oneshot 7 ĞD
    When oneshot dave consumes into account bob
    Then dave should have oneshot 0 ĞD
    Then bob should have 1699 cĞD
    Then bob should have oneshot 0 ĞD

  Scenario: Double oneshot consumption
    When alice sends 7 ĞD to oneshot dave
    # Alice is treasury funder for 1 ĞD
    Then alice should have 2 ĞD
    Then dave should have oneshot 7 ĞD
    When oneshot dave consumes 4 ĞD into account bob and the rest into oneshot charlie
    Then dave should have oneshot 0 ĞD
    Then bob should have 14 ĞD
    Then bob should have oneshot 0 ĞD
    Then charlie should have 10 ĞD
    Then charlie should have oneshot 299 cĞD
