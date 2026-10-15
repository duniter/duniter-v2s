@setup.default
Feature: Balance transfer all

  Scenario: A member can transfer all funds while keeping their identity
    When bob sends all his ĞDs to dave
    Then bob should have 0 cĞD
    Then bob identity should be member
    Then bob should be eligible to UD
    Then dave should have 998 cĞD
