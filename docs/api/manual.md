# Manual for wallet developers

This functional documentation presents how wallets can interact with the blockchain.
It is intended to complete the [runtime calls documentation](./runtime-calls.md) in a runtime-specific way to fit the real needs of wallet developers.

NOTE : a more detailed doc is available at <https://duniter.org/wiki/duniter-v2/doc/>

## Notations

1 ĞD = 100 units

## Account existence

An account exists if and only if it contains at least the existential deposit (`balances.existentialDeposit` = 1 ĞD).

## Become member

Only use `identity` pallet.

1. The account that wants to gain membership needs to exists.
1. Any account that already has membership and respects the identity creation period can create an identity for another account, using `identity.createIdentity`.
1. The account has to confirm its identity with a name, using `identity.confirmIdentity`. The name must be ASCII alphanumeric, punctuation or space characters: `` /^[-!"#$%&'()*+,-./:;<=>?@[\]^_`{|}~a-zA-Z0-9 ]{3,64}$/ `` (additionally, trailing spaces and double spaces are forbidden, as a phishing countermeasure). If the name is already used, the call will fail.
1. 4 different member accounts must certify the account using `cert.addCert`.
1. The distance evaluation must be requested for the pending identity using `distance.requestDistanceEvaluation`.
1. 3 distance sessions later, if the distance rule is respected, identity is validated automatically.

## Change key

A member can request a key change via the `identity.change_onwner_key` call. It needs the following SCALE encoded (see SCALE encoding section below) payload:

- The new owner key payload prefix (rust definition: `b"icok"`)
- the genesis block hash. (rust type `[u8; 32]` (`H256`))
- The identity index (rust type `u64`)
- The old key (rust type `u64`)

This payload must be signed with the new key.

## Revoke an identity

Revoking an identity makes it lose its membership, hence UD creation and governance rights. Other data such as balance will remain.

This feature is useful in case the user has lost their private key since the revocation document can be made in advance.

### Generate the revocation payload

The revocation needs this SCALE encoded (see SCALE encoding section below) payload:

- The revocation payload prefix (rust definition: `b"revo"`)
- The identity index (rust type `u64`)
- the genesis block hash. (rust type `[u8; 32]` (`H256`))

This payload must be signed with the corresponding revocation key.

### Effectively revoke the identity

1. From any origin that can pay the fee, use `identity.revokeIdentity` with the revocation payload.

## SCALE encoding

SCALE codec documentation: https://docs.substrate.io/reference/scale-codec/.

At the end of this documentation you'll find links to SCALE codec implementation for other languages.
