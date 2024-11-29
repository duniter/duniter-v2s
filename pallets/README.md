# Duniter pallets

Duniter uses some [parity pallets](https://github.com/duniter/substrate/tree/master/frame) available in our substrate fork, and some defined here. Each pallet has its own readme, but here is a summary:

## Business processes pallets

These pallets are at the core of Duniter/Ğ1 currency

- **[`authority-members`](https://doc-duniter-org.ipns.pagu.re/pallet_authority_members/index.html)** Duniter authorities are not selected with staking but through a smith web of trust.
- **[`certification`](https://doc-duniter-org.ipns.pagu.re/pallet_certification/index.html)** Certifications are the "edges" of Duniter's dynamic directed graph. They mean the acceptation of a Licence.
- **[`duniter-account`](https://doc-duniter-org.ipns.pagu.re/pallet_duniter_account/index.html)** Duniter customized the `AccountData` defined in the `Balances` pallet to introduce a `linked_idty`.
- **[`duniter-wot`](https://doc-duniter-org.ipns.pagu.re/pallet_duniter_wot/index.html)** Merges identities, membership, certifications and distance pallets to implement Duniter Web of Trust.
- **[`distance`](https://doc-duniter-org.ipns.pagu.re/pallet_distance/index.html)** Publishes median of distance computation results provided by inherents coming from `distance-oracle` workers.
- **[`identity`](https://doc-duniter-org.ipns.pagu.re/pallet_identity/index.html)** Identities are the "nodes" of Duniter's dynamic directed graph. They are one-to-one mapping to human being.
- **[`membership`](https://doc-duniter-org.ipns.pagu.re/pallet_membership/index.html)** Membership defines the state of identities. They can be member or not of the different WoTs.
- **[`universal-dividend`](https://doc-duniter-org.ipns.pagu.re/pallet_universal_dividend/index.html)** UD is at the basis of Ğ1 "libre currency". It is both a kind of "basic income" and a measure unit.

## Functional pallets

- **[`duniter-test-parameters`](https://doc-duniter-org.ipns.pagu.re/pallet_duniter_test_parameters/index.html)** Test parameters only used in ĞDev to allow tweaking parameters more easily.
- **[`offences`](https://doc-duniter-org.ipns.pagu.re/pallet_offences/index.html)** Sorts offences that will be executed by the `authority-members` pallet.
- **[`oneshot-account`](https://doc-duniter-org.ipns.pagu.re/pallet_oneshot_account/index.html)** Oneshot accounts are light accounts only used once for anonymity or convenience use case.
- **[`provide-randomness`](https://doc-duniter-org.ipns.pagu.re/pallet_provide_randomness/index.html)** Lets blockchain users ask for a verifiable random number.
- **[`session-benchmarking`](https://doc-duniter-org.ipns.pagu.re/pallet_session_benchmarking/index.html)** Benchmarks the session pallet.
- **[`upgrade-origin`](https://doc-duniter-org.ipns.pagu.re/pallet_upgrade_origin/index.html)** Allows some origins to dispatch a call as root.
