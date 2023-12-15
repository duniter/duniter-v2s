# Duniter membership pallet

Duniter membership is related to duniter Web of Trust and more specific than [parity membership pallet](https://github.com/paritytech/substrate/tree/master/frame/membership). It is used only internally by the identity, WoT, and distance pallets.

## Main Web of Trust

When used in conjunction with the main Web of Trust, the membership pallet is combined with the Duniter-WoT pallet and the identity pallet, resulting in the following functionality:

- `claim_membership` requires enough certifications, and a valid distance
- `renew_membership` requires a valid membership and a valid distance status to extend the validity period of a membership.
- `revoke_membership` (to be removed?)

In practice, a new user creates an account, confirms identity using the `confirm_identity` call from the identity pallet. The user then validates its identity using `claim_membership`. If the certification and distance requirements are met, the identity is granted membership.

## Sub Web of Trust Smith

Functionality related to the Smith Web of Trust involves the following:

- `claim_membership` requires to be member of the main wot and have enough smith certifications
- `renew_membership` needs a valid membership to extend the membership's validity period.
- `revoke_membership` requires a valid origin to revoke the membership.

In practice, a user must complete all steps to gain membership in the main Web Of Trust. They can call `claim_membership` to be added to the authority members.
