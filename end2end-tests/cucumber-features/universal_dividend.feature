@genesis.wot

Feature: Universal Dividend

    Scenario: Eligibility at genesis
        When 2 blocks later

        # Members
        Then alice should be eligible to UD
        Then bob should be eligible to UD
        Then charlie should be eligible to UD

        # Not members
        Then eve should not be eligible to UD
        Then ferdie should not be eligible to UD
