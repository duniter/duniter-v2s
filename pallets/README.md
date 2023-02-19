# Duniter pallets

Duniter uses some [parity pallets](https://github.com/duniter/substrate/tree/master/frame) available in our substrate fork, and some defined here. Each pallet has its own readme, but here is a summary:

## Business processes pallets

These pallets are at the core of Duniter/Ğ1 currency

- **`authority-members`** Duniter authorities are not selected with staking but through a smith web of trust.
- **`certification`** Certifications are the "edges" of Duniter's dynamic directed graph. They mean the acceptation of a Licence.
- **`duniter-account`** Duniter customized the `AccountData` defined in the `Balances` pallet to introduce a `RandomId`.
- **`duniter-wot`** Merges identities, membership, certifications and distance pallets to implement Duniter Web of Trust.
- **`duniter-distance`** Offchain worker used to compute distance criterion.
- **`identity`** Identities are the "nodes" of Duniter's dynamic directed graph. They are one-to-one mapping to human being.
- **`membership`** Membership defines the state of identities. They can be member or not of the different WoTs.
- **`universal-dividend`** UD is at the basis of Ğ1 "libre currency". It is both a kind of "basic income" and a measure unit.

## Functional pallets

- **`duniter-test-parameters`** Test parameters only used in ĞDev to allow tweaking parameters more easily.
- **`oneshot-account`** Oneshot accounts are light accounts only used once for anonimity or convenience use case.
- **`provide-randomness`** Lets blockchain users ask for a verifiable random number.
- **`upgrade-origin`** Allows some origins to dispatch a call as root.