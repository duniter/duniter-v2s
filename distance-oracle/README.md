# Distance oracle

> for explanation about the Duniter web of trust, see https://duniter.org/wiki/web-of-trust/deep-dive-wot/

Distance computation on the Duniter web of trust is an expensive operation that should not be included in the runtime for multiple reasons:

- it could exceed the time available for a block computation
- it takes a lot of resource from the host machine
- the result is not critical to the operation of Ğ1

It is then separated into an other program that the user (a duniter smith) can choose to run or not. This program publishes its result in a inherent and the network selects the median of the results given by the smith who published some.

## Structure

This feature is organized in multiple parts:

- **/distance-oracle/** (here): binary executing the distance algorithm
- **/primitives/distance/**: primitive types used both by client and runtime
- **/client/distance/**: exposes the `create_distance_inherent_data_provider` which provides data to the runtime
- **/pallets/distance/**: distance pallet exposing type, traits, storage/calls/hooks executing in the runtime