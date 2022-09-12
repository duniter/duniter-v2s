# Manual for wallet developers

This functional documentation presents how wallets can interact with the blockchain.
It is intended to complete the [runtime calls documentation](./runtime-calls.md) in a runtime-specific way to fit the real needs of wallet developers.

Only ĞDev is covered for now.

## Notations

1 ĞD = 100 units

## Account existence

An account exists if and only if it contains at least the existential deposit (2 ĞD).

## Become member

Only use `identity` pallet. The `membership` calls are disabled.

1. The account that wants to gain membership needs to exists.
1. Any account that already has membership and respects the identity creation period can create an identity for another account, using `identity.createIdentity`.
1. The account has to confirm its identity with a name, using `identity.confirmIdentity`. The name must be ASCII alphanumeric, punctuation or space characters: ``/^[-!"#$%&'()*+,-./:;<=>?@[\]^_`{|}~a-zA-Z0-9 ]{3,64}$/`` (additionally, trailing spaces and double spaces are forbidden, as a phishing countermeasure). If the name is already used, the call will fail.

## Revoke an identity

Revoking an identity makes it lose its membership, hence UD creation and governance rights. Other data such as balance will remain.

This feature is useful in case the user has lost their private key since the revocation document can be made in advance.

### Generate the revocation payload

1. Scale-encode the revocation payload, that is the concatenation of the 32-bits public key and the genesis block hash.
2. Store this payload and its signature.

### Effectively revoke the identity

1. From any origin that can pay the fee, use `identity.revokeIdentity` with the revocation payload.
