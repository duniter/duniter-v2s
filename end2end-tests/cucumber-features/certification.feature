@genesis.wot

Feature: Certification

    Scenario: Dave certifies Alice
        When 2 blocks later
        When dave certifies alice
        Then alice should be certified by dave
