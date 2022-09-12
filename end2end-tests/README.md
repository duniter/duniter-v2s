# Duniter-v2s end2end tests

## Cucumber functional tests

We use [cucumber] to be able to describe test scenarios in human language.

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

### Create a new functional test

To create a new test scenario, simply create a new file with a name of your choice in the
`/cucumber-features` folder and give it the extension `.feature`.

Read in the sections below which users are available and which operations you can write.

If you want to write things that are not yet interpreted, make sure you standardize as much as
possible the way you write actions and assertions, in order to facilitate their future technical
interpretation.

Feel free to add comments to explain your scenario:

```gherkin
Feature: My awesome feature

  Scenario: If something like this happens, then we should observe that
    Given Something
     """
      This is a comment, you can write whatever you want here, this part of the text will not be
      interpreted, but it allows you to explain your scenario so that the developers interpret
      it correctly.
      """
    When Something's happening
    Then We should observe that
```

### Steps

Each scenario is a list of steps. In our context (blockchain), only the `When` and `Then` steps make sense, `Given` being the genesis.

#### When

List of possible actions:

- transfer: `alice sends 5 ĞD to bob`
- transfer_ud: `alice sends 3 UD to bob`
- transfer_all: `alice sends all her ĞDs to bob`

#### Then

-  Check that a user has exactly a specific balance

    Usage: `{user} have {amount} {unit}`

    Example: `alice should have 10 ĞD`

-  Check the current UD amount

    Usage: `Current UD amount should be {amount}.{cents} ĞD`

    Example: `Current UD amount should be 10.00 ĞD`

-  Check the monetary mass

    Usage: `Monetary mass should be {amount}.{cents} ĞD`

    Example: `Monetary mass should be 30.00 ĞD`

### Test users

8 test users are provided derived from the same [dev mnemonic](https://docs.substrate.io/v3/getting-started/glossary/#dev-phrase)

```
bottom drive obey lake curtain smoke basket hold race lonely fit walk
```

with the derivation path `//Alice`, `//Bob`...

- alice `5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY`
- bob `5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty`
- charlie `5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y`
- dave `5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy`
- eve `5HGjWAeFDfFCWPsjFQdVV2Msvz2XtMktvgocEZcCj68kUMaw`
- ferdie `5CiPPseXPECbkjWCa6MnjNokrgYjMqmKndv2rSnekmSK2DjL`
- one `5Fxune7f71ZbpP2FoY3mhYcmM596Erhv1gRue4nsPwkxMR4n`
- two `5CUjxa4wVKMj3FqKdqAUf7zcEMr4MYAjXeWmUf44B41neLmJ`

### Currency amounts

Amounts must be expressed as an integer of `ĞD` or `UD`, decimal numbers are not supported.
If you need more precision, you can express amounts in cents of ĞD (write `cĞD`), or in thousandths
of UD (write `mUD`).

### Genesis state

Each scenario bootstraps its own blockchain with its own genesis state.

By default, all scenarios use the same configuration for the genesis, which is located in the file
`/cucumber-genesis/default.json`.

You can define a custom genesis state for each scenario with the tag `@genesis.confName`.

The genesis configuration must then be defined in a json file located at
`/cucumber-genesis/confName.json`.

You can also define a custom genesis at the feature level, all the scenarios of this feature will
then inherit the genesis configuration.

### ignoreErrors

For some scenarios, you may need to perform an action (When) that fails voluntarily, in this case you must add the tag @ignoreErrors to your scenario, otherwise it will be considered as failed

### Run cucumber functional tests

The cucumber tests use the last debug binary in your `target` folder. Make sure this binary corresponds to the executable you want to test by running `cargo build` before.

To run the cucumber tests, you will need to have the rust toolchain installed locally.

To run all the scenarios (there are many) use the command: `cargo cucumber`

You can filter the `.feature` files to run with the option `i`, for instance:

```
cargo cucumber -i monetary*
```

will only run `.feature` files that start with `"monetary"`.

The features will be tested in parallel and logs files will be written in the `end2end-tests` folder.
If you get an `Error: Timeout`, look at the logs to understand why Duniter did not launch successfully. You can also set the environment variable `DUNITER_END2END_TESTS_SPAWN_NODE_TIMEOUT` to increase the timeout for node spawn.

### Contribute to the code that runs the tests

Cucumber is not magic, we have to write code that interprets the Gherkin text and performs the right
actions accordingly.

The rust code that interprets the Gherkin text is in this file:
`end2end-tests/tests/cucumber_tests.rs`.

To contribute to this section, read the [Cucumber Rust Book].

To interact with the node, we use exclusively RPC requests, the RPC requests are realized in
functions defined in `end2end-tests/tests/common`.

To realize the RPC requests, we use the rust crate [subxt](https://github.com/paritytech/subxt).

#### Upgrade metadata

To work, the integration tests need to have the runtime metadata up to date, here is how to update
them:

```bash
subxt metadata -f bytes > resources/metadata.scale
```

If you don't have subxt, install it: `cargo install subxt-cli`

[BDD]: https://en.wikipedia.org/wiki/Behavior-driven_development
[cucumber]: https://cucumber.io/
[Cucumber Rust Book]: https://cucumber-rs.github.io/cucumber/current/writing/index.html
[Gherkin]: https://cucumber.io/docs/gherkin/reference
