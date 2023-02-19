# Duniter oneshot account pallet

Duniter provides light accounts without `AccountInfo` (nonce, consumers, providers, sufficients, free, reserved, misc_frozen, fee_frozen) that can only be consumed once. This should reduce transaction weight and then fees. The use case is anonymous accounts or physical supports.