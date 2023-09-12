# Distance pallet

The distance pallet uses results provided in a file by the `distance-oracle` offchain worker.
At some point an inherent is called to submit the results of this file to the blockchain.
The pallet then selects the median of the results (reach perbill) of an evaluation pool and fills the storage with the result status.
The status of an identity can be:

- inexistant: distance evaluation has not been requested or has expired
- pending: distance evaluation for this identity has been requested and is waiting two sessions for results
- valid: distance has been evaluated positively for this identity

The result of the evaluation is used by `duniter-wot` pallet to determine if an identity can get / should loose membership to the web of trust. 

## Process

Any account can request a distance evaluation for a given identity provided it has enough currency to be reserved. In this case, the distance status is marked as pending and in the next session, inherents can start to publish results. 

This is how a result is published:

1. local worker creates a file containing the result of computation
1. inherent is created with the data from this file
1. author is registered as an evaluator
1. the result is added to the current evaluation pool
1. a flag is set to prevent other distance evaluation in the same block

On each new session:

1. old results set to expire at this session do expire
1. results from the current pool (previous session's result pool) are taken and for each identity
    - the median of the distance results for this identity is chosen
    - if the distance is ok, the distance is marked as valid
    - if the distance is ko, the result for this identity is removed and reserved currency is slashed (from the account which requested the evaluation)

Then, in other pallets, when a membership is claimed, it is possible to look if there is a valid evaluation of distance for this identity.

## Pools

Evaluation pools are made of two components:

- a set of evaluators
- a vec of results

The evaluation are separated in three pools (N - 2 is the session index):

- pool number N - 1 % 3: results from the previous session used in the current one (let empty for next session)
- pool number N + 0 % 3: inherent results are added there
- pool number N + 1 % 3: identities are added there for evaluation