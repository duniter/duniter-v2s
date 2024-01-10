# Distance rule evaluation

The [distance rule](https://duniter.org/blog/duniter-deep-dive-wot/) is computationally too heavy to be handled by the runtime. Therefore it is computed offchain using the distance oracle.

Distance evaluation is operated on a voluntary basis by individual smiths. Since evaluators can lie or make errors, the result considered for applying the distance rule is the median of results published by the different evaluators.

## Running distance evaluation

Any smith member authoring blocks can run a distance evaluation oracle. It is better to have a machine more powerful than the reference machine.

The simplest way is to run the oracle on the same machine as Duniter.

Add this line to your cron with the command `crontab -e`: (add option `-u <user>` to edit another user's cron)

    4,24,44 * * * * nice -n 2 /absolute/path/to/duniter distance-oracle

The precise hours don't matter so you can pick random values, but it should run at least one time per hour, and running it more often decreases the risk of problem in case of missing blocks or temporary network failure.

If the evaluation ran successfully in a session, the next runs in the same session won't re-evaluate the same data.

The `nice -n 2` lowers the oracle's priority, so that Duniter has the priority even when the oracle wants to use all the cores.

### Additional Duniter configuration

Duniter should keep states at least one session old, that is 600 blocks (while 256 is the default). Use the option `--state-pruning 600` if your node is not already an archive (`--state-pruning archive`).
