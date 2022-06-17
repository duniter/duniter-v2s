@genesis.wot

Feature: Certification

    Scenario: Dave certifies Alice
        When dave certifies alice
        Then alice should be certified by dave
