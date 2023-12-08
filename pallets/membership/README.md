# Duniter membership pallet

Duniter membership is related to duniter Web of Trust and more specific than [parity membership pallet](https://github.com/paritytech/substrate/tree/master/frame/membership). It is used only internally by the identity, WoT, and distance pallets. In particular, it is adding the concept of "pending membership" which is an intermediate state where the identity is waiting to become member.

## Main Web of Trust

When used in conjunction with the main Web of Trust, the membership pallet is combined with the Duniter-WoT pallet and the identity pallet, resulting in the following functionality:

- `request_membership` is automatically invoked when confirming identity and should not be called manually. It will add the identity to the pending membership.
- `claim_membership` is automatically triggered during identity validation and should not be manually invoked. This process requires a pending membership, sufficient membership certificates, and a valid distance. Membership is granted upon successful completion of these requirements.
- `renew_membership` requires a valid membership and a valid distance status to extend the validity period of a membership.
- `revoke_membership` is automatically executed when a membership expires and should not be called manually.

In practice, a new user creates an account, confirms identity using the `confirm_identity` call from the identity pallet, and is added to the pending membership. The user then validates it identity using the `validate_identity` call from the identity pallet, triggering a membership claim. If the certification and distance requirements are met, the identity is granted membership.

## Sub Web of Trust Smith

Functionality related to the Smith Web of Trust involves the following:

- `request_membership` requires a validated identity for the member to be added to the pending membership.
- `claim_membership` requires a pending membership, sufficient smith certificates, and a valid distance status for the identity to be included among the authority members.
- `renew_membership` needs a valid membership and a valid distance status to extend the membership's validity period.
- `revoke_membership` requires a valid origin to revoke the membership.

In practice, a user must complete all steps to gain membership in the main Web Of Trust. They can then manually request smith membership using the `request_membership` call of the membership pallet. The membership can be claimed using the `claim_membership` call from the membership pallet, and if the identity meets smith certificate and distance requirements, it will be added to the authority members.