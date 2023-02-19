# Duniter identity pallet

Duniter has a builtin identity system that does not work with external registrar compared to [parity identity pallet](https://github.com/paritytech/substrate/tree/master/frame/identity).

## Duniter identity

A Duniter identity contains:

- its **owner key** (that can change)
- an optional **old owner key** with the date of the key change
- a **status** that can be
  - created (by an existing identity)
  - confirmed (by owner, comes with a name)
  - validated (that has become member in the allowed timeframe)

It also contains:

- the block number at which it can emit its **next certification**
- the block number at which it can be **removed from storage**

It also contains attached data defined by the runtime that can be for example

- the number of the first UD it is eligible to

### Name

Each identity is declared with a name emited on confirmation event. Duniter keeps a list of identity names hash to ensure unicity. 

### Owner key

The idea of the owner key is to allow the user to keep a fixed identity while changing the keys for security reasons. For example when a device with the keys might have been compromised. There is a limit to the frequency of owner key change and the old owner key can still revoke the identity for a given period.

### Status / removable date

The status is a temporary value allowing to prune identities before they become member. When an identity is not valiated (not member of the WoT for instance), it can be removed when the date is reached. The remove date of a validated identity is block zero.

### Next certification

The next certification is a rate limit to the emission of certification (and then identity creation).

### Revokation

Revoking an identity basically means deleting it.