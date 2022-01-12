# Duniter-v2s integration tests

## cucumber functionnal tests

We use [cucumber] to be able to describe test cases in human language.

Cucumber is a specification for running tests in a [BDD] (behavior-driven development) style
workflow.

It assumes involvement of non-technical members on a project and as such provides a human-readable
syntax for the definition of features, via the language [Gherkin]. A typical feature could look
something like this:

```gherkin
Feature: Balance transfer

  Scenario: If alice sends 5 ĞD to Dave, Dave will get 5 ĞD
    Given alice have 10 ĞD
    When alice send 5 ĞD to dave
    Then dave should have 5 ĞD
```

### create a new functional test

To create a new test case, simply create a new file with a name of your choice in the `/features`
folder and give it the extension `.feature`.

Read in the sections below which users are available and which operations you can write.

 [Cucumber Rust Book]

### Test users

6 test users are provided:

- alice
- bob
- charlie
- deve
- eve
- ferdie

### Currency amounts

Amounts must be expressed as an integer of `ĞD` or `UD`, decimal numbers are not supported.
If you need more precision, you can express amounts in cents of ĞD (write `cĞD`), or in thousandths
of UD (write `mUD`).

#### Given

You can give any currency balance to each of the test users, so money will be created ex-nihilo for
that user. Note that this created money is not included in the monetary mass used to revalue the UD
amount.

Usage: `{user} have {amount} {unit}`

Example: `alice have 10 ĞD`

#### When

List of possible actions:

- transfer: `alice send 5 ĞD to bob`
- transfer_ud: `alice send 3 UD to bob`
- transfer_all: `alice sends all her ĞDs to bob`

#### Then

-  Check that a user has exactly a specific balance

    Usage: `{user} have {amount} {unit}`

    Example: `alice should have 10 ĞD`

### Universal dividend creation

#### Then

-  Check the current UD amount

    Usage: `Current UD amount should be {amount}.{cents} ĞD`

    Example: `Current UD amount should be 10.00 ĞD`

### Contribute to the code that runs the tests

Cucumber is not magic, we have to write code that interprets the Gherkin text and performs the right
actions accordingly.

The rust code that interprets the Gherkin text is in this file:
`integration-tests/tests/cucumber_tests.rs`.

To contribute to this section, read the [Cucumber Rust Book].

To interact with the node, we use exclusively RPC requests, the RPC requests are realized in
functions defined in `integration-tests/tests/common`.

To realize the RPC requests, we use the rust crate [subxt](https://github.com/paritytech/subxt).



[BDD]: https://en.wikipedia.org/wiki/Behavior-driven_development
[cucumber]: https://cucumber.io/
[Cucumber Rust Book]: https://cucumber-rs.github.io/cucumber/current/writing/index.html
[Gherkin]: https://cucumber.io/docs/gherkin/reference
