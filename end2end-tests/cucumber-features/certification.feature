@genesis.wot

Feature: Certification

    Scenario: Dave certifies Alice
        When dave certifies alice
        Then alice should be certified by dave

    @ignoreErrors
    Scenario: Dave certifies Alice (but dave is not certified by charlie, this test should fail)
        When dave certifies alice
        Then dave should be certified by charlie