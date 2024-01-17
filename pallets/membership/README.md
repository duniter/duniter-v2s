# Duniter membership pallet

Duniter membership is related to duniter Web of Trust and more specific than [parity membership pallet](https://github.com/paritytech/substrate/tree/master/frame/membership). It is used only internally by the identity, WoT, and distance pallets.

## Main Web of Trust

Membership pallet manages all events related to web of trust membership of an identity. It exposes no calls to the user and its features are only available trough distance evaluation provided by distance oracle.
