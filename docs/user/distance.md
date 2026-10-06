# Distance rule evaluation

The [distance rule](https://duniter.org/blog/duniter-deep-dive-wot/) is computationally too heavy to be handled by the runtime. Therefore it is computed offchain using the distance oracle.

Distance evaluation is operated on a voluntary basis by individual smiths. Since evaluators can lie or make errors, the result considered for applying the distance rule is the median of results published by the different evaluators.

## How it works

Online validators who run a distance oracle will compute the distance rule for any distance evaluation request. However, they can publish their result only when writing a block. This is why distance evaluation is a probabilistic process, that may fail to provide a result if there are not enough validators running an oracle.

For an evaluation period of N blocks, in order to guarantee a probability less or equal to P that evaluation fails, the network needs at least the following proportion of validators to run an oracle: 1-P^(1/N).

Each request temporarily holds `Distance.EvaluationPrice` on the requester account. The runtime returns this deposit for every outcome, including a negative result. Ordinary transaction fees still apply.

A negative result leaves the identity unvalidated or its existing membership unchanged. A new request for that identity must wait for `Distance.DistanceRetryPeriod`, starting at the block that applies the negative result. The default is 48 hours, expressed as 28,800 blocks at the nominal six-second block time. Test builds with the `fast` feature use two minutes. The runtime parameter can be changed through a runtime upgrade.

The delay applies to the identity index. Changing the requester or the identity's owner key does not bypass it. `Distance.NextEvaluationOn` stores the first block at which another request is allowed. An earlier request returns `DistanceRetryPeriodNotRespected`.

If no oracle submits a result, the runtime returns the deposit without starting a retry delay. The requester can try again, subject to the existing certification and membership renewal checks.

Applications can use a server's precomputed distance as an estimate. The network's evaluation remains authoritative. A stale or inaccurate estimate can result in a retry delay, but it cannot cause the evaluation deposit to be lost.

## Running distance evaluation

Any smith member authoring blocks can run a distance evaluation oracle. It is better to have a machine more powerful than the reference machine.

Create a service from this command line, run by the same user as Duniter, on the same system:

    /absolute/path/to/duniter distance-oracle --interval <duration>

The duration is the number of seconds between two evaluations. It should be less than the duration of a distance evaluation period. If it is equal, your node may not have the time to evaluate distance.

The oracle communicates with Duniter using its RPC API and using temporary files. Without additional (unsupported) configuration, both must run on the same filesystem. The node also needs to be forging blocks for the evaluations to be published.

### Additional Duniter configuration

Duniter should keep states at least one distance evaluation period old. If this is more than the default 256 and your node is not already an archive (`--state-pruning archive`), use the option `--state-pruning <blocks>`.
