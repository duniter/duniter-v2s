# Duniter quota pallet

Duniter identity system allows to allocate quota and refund transaction fees when not consumed.

## General behavior

Quota system is plugged to transactions fees which is a rather critical aspect of substrate.
That's why in `duniter-account` pallet `OnChargeTransaction` implementation, the default behavior is preserved, and refunds are added to a queue handeled in `on_idle`.

## Path for a refund

This is what happens on a transaction:

- `frame-executive` calls `OnChargeTransaction` implementations
- `duniter-account` `OnChargeTransaction` implementation is called, and if an identity is linked to the account who pays the fees, `request_refund` is called
- `request_refund` implementation of `quota` pallet determines whether the fees are eligible for refund based on the identity and then call `queue_refund`
- `queue_refund` adds a refund to the `RefundQueue` which will be processed in `on_idle`
- during `on_idle`, `quota` pallet processes the refund queue within the supplied weight limit with `process_refund_queue`
- for each refund in the `RefundQueue`, `try_refund` is called
- it first tries to use quotas to refund fees with `spend_quota`
- if a certain amount of quotas has been spend, it actually performs the refund with `do_refund`, taking currency from the `RefundAccount` to give it back to the account who paid the fee

The conditions for a refund to happen are:

1. an identity is linked to the account who pays the fees
1. some quotas are defined for the identity and have a non-null value after update


## TODO

- [ ] sanity test checking that only member identities have quota