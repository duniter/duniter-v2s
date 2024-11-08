# Distance rule evaluation

The [distance rule](https://duniter.org/blog/duniter-deep-dive-wot/) is computationally too heavy to be handled by the runtime. Therefore it is computed offchain using the distance oracle.

Distance evaluation is operated on a voluntary basis by individual smiths. Since evaluators can lie or make errors, the result considered for applying the distance rule is the median of results published by the different evaluators.

## Running distance evaluation

Any smith member authoring blocks can run a distance evaluation oracle. It is better to have a machine more powerful than the reference machine.

Create a service from this commandline, run by the same user as Duniter, on the same system:

    /absolute/path/to/duniter distance-oracle --interval <duration>

The duration is the number of seconds between two evaluations. It should be less than the duration of a distance evaluation period. If it is equal, your node may not have the time to evaluate distance.

The oracle communicates with Duniter using its RPC API and using temporary files. Without additional (unsupported) configuration, both must run on the same filesystem. The node also needs to be forging blocks for the evaluations to be published.

### Additional Duniter configuration

Duniter should keep states at least one distance evaluation period old. If this is more than the default 256 and your node is not already an archive (`--state-pruning archive`), use the option `--state-pruning <blocks>`.
