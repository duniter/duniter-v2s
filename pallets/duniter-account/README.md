# Duniter account pallet

Duniter customizes the `AccountData` of the `Balances` Substrate pallet. In particular, it adds the fields `random_id` and `linked_idty`. 

## RandomID

The RandomId field was added with the idea to provide a unique id that can not be controlled by user to serve as a basis for robust identification. The discussion is available on the forum.

https://forum.duniter.org/t/la-solution-pour-des-identicones-securisees-le-random-id/9126

## Account creation fee

DuniterAccount defines a creation fee that is preleved to the account one block after its creation. This fee goes to the treasury.

## Sufficient 

DuniterAccount tweaks the substrate AccountInfo to allow identity accounts to exist without existential deposit. This allows to spare the creation fee.

## Linked identity

Duniter offers the possibility to link an account to an identity with the `linked_idty` field. It allows to request refund of transaction fees in `OnChargeTransaction`.