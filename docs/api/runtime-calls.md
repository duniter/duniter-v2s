# Runtime calls

Calls are categorized according to the dispatch origin they require:

1. **User calls**: the dispatch origin for this kind of call must be signed by
   the transactor. This is the only call category that can be submitted with an extrinsic.
1. **Root calls**: This kind of call requires a special origin that can only be invoked
   through on-chain governance mechanisms.
1. **Inherent calls**: This kind of call is invoked by the author of the block itself
   (usually automatically by the node).
1. **Disabled calls**: These calls can not be called directly, they are reserved for internal use by other runtime calls.

We only document user calls below.

There are **65** user calls from **17** pallets.

## Account - 1

### unlink_identity - 0

<details><summary><code>unlink_identity()</code></summary>

Taking 0.0106 % of a block.

```rust
```
</details>



Unlink the identity associated with the account.

## Balances - 6

### transfer_allow_death - 0

<details><summary><code>transfer_allow_death(dest, value)</code></summary>

Taking 0.0186 % of a block.

```rust
dest: AccountIdLookupOf<T>
value: T::Balance
```
</details>



Transfer some liquid free balance to another account.

`transfer_allow_death` will set the `FreeBalance` of the sender and receiver.
If the sender's account is below the existential deposit as a result
of the transfer, the account will be reaped.

The dispatch origin for this call must be `Signed` by the transactor.

### transfer_keep_alive - 3

<details><summary><code>transfer_keep_alive(dest, value)</code></summary>

Taking 0.0119 % of a block.

```rust
dest: AccountIdLookupOf<T>
value: T::Balance
```
</details>



Same as the [`transfer_allow_death`] call, but with a check that the transfer will not
kill the origin account.

99% of the time you want [`transfer_allow_death`] instead.

[`transfer_allow_death`]: struct.Pallet.html#method.transfer

### transfer_all - 4

<details><summary><code>transfer_all(dest, keep_alive)</code></summary>

Taking 0.0122 % of a block.

```rust
dest: AccountIdLookupOf<T>
keep_alive: bool
```
</details>



Transfer the entire transferable balance from the caller account.

NOTE: This function only attempts to transfer _transferable_ balances. This means that
any locked, reserved, or existential deposits (when `keep_alive` is `true`), will not be
transferred by this function. To ensure that this function results in a killed account,
you might need to prepare the account by removing any reference counters, storage
deposits, etc...

The dispatch origin of this call must be Signed.

- `dest`: The recipient of the transfer.
- `keep_alive`: A boolean to determine if the `transfer_all` operation should send all
  of the funds the account has, causing the sender account to be killed (false), or
  transfer everything except at least the existential deposit, which will guarantee to
  keep the sender account alive (true).

## OneshotAccount - 7

### create_oneshot_account - 0

<details><summary><code>create_oneshot_account(dest, value)</code></summary>

Taking 0.0115 % of a block.

```rust
dest: <T::Lookup as StaticLookup>::Source
value: BalanceOf<T>
```
</details>



Create an account that can only be consumed once

- `dest`: The oneshot account to be created.
- `balance`: The balance to be transfered to this oneshot account.

Origin account is kept alive.

### consume_oneshot_account - 1

<details><summary><code>consume_oneshot_account(block_height, dest)</code></summary>

Taking 0.0182 % of a block.

```rust
block_height: BlockNumberFor<T>
dest: Account<<T::Lookup as StaticLookup>::Source>
```
</details>



Consume a oneshot account and transfer its balance to an account

- `block_height`: Must be a recent block number. The limit is `BlockHashCount` in the past. (this is to prevent replay attacks)
- `dest`: The destination account.
- `dest_is_oneshot`: If set to `true`, then a oneshot account is created at `dest`. Else, `dest` has to be an existing account.

### consume_oneshot_account_with_remaining - 2

<details><summary><code>consume_oneshot_account_with_remaining(block_height, dest, remaining_to, balance)</code></summary>

Taking 0.0246 % of a block.

```rust
block_height: BlockNumberFor<T>
dest: Account<<T::Lookup as StaticLookup>::Source>
remaining_to: Account<<T::Lookup as StaticLookup>::Source>
balance: BalanceOf<T>
```
</details>



Consume a oneshot account then transfer some amount to an account,
and the remaining amount to another account.

- `block_height`: Must be a recent block number.
  The limit is `BlockHashCount` in the past. (this is to prevent replay attacks)
- `dest`: The destination account.
- `dest_is_oneshot`: If set to `true`, then a oneshot account is created at `dest`. Else, `dest` has to be an existing account.
- `dest2`: The second destination account.
- `dest2_is_oneshot`: If set to `true`, then a oneshot account is created at `dest2`. Else, `dest2` has to be an existing account.
- `balance1`: The amount transfered to `dest`, the leftover being transfered to `dest2`.

## SmithMembers - 10

### invite_smith - 0

<details><summary><code>invite_smith(receiver)</code></summary>

Taking 0.0214 % of a block.

```rust
receiver: T::IdtyIndex
```
</details>



Invite a member of the Web of Trust to attempt becoming a Smith.

### accept_invitation - 1

<details><summary><code>accept_invitation()</code></summary>

Taking 0.0119 % of a block.

```rust
```
</details>



Accept an invitation to become a Smith (must have been invited first).

### certify_smith - 2

<details><summary><code>certify_smith(receiver)</code></summary>

Taking 0.0255 % of a block.

```rust
receiver: T::IdtyIndex
```
</details>



Certify an invited Smith, which can lead the certified to become a Smith.

## AuthorityMembers - 11

### go_offline - 0

<details><summary><code>go_offline()</code></summary>

Taking 0.0152 % of a block.

```rust
```
</details>



Request to leave the set of validators two sessions later.

### go_online - 1

<details><summary><code>go_online()</code></summary>

Taking 0.0172 % of a block.

```rust
```
</details>



Request to join the set of validators two sessions later.

### set_session_keys - 2

<details><summary><code>set_session_keys(keys)</code></summary>

Taking 0.0225 % of a block.

```rust
keys: T::Keys
```
</details>



Declare new session keys to replace current ones.

## Preimage - 22

### note_preimage - 0

<details><summary><code>note_preimage(bytes)</code></summary>

Taking 0.2846 % of a block.

```rust
bytes: Vec<u8>
```
</details>



Register a preimage on-chain.

If the preimage was previously requested, no fees or deposits are taken for providing
the preimage. Otherwise, a deposit is taken proportional to the size of the preimage.

### unnote_preimage - 1

<details><summary><code>unnote_preimage(hash)</code></summary>

Taking 0.0171 % of a block.

```rust
hash: T::Hash
```
</details>



Clear an unrequested preimage from the runtime storage.

If `len` is provided, then it will be a much cheaper operation.

- `hash`: The hash of the preimage to be removed from the store.
- `len`: The length of the preimage of `hash`.

### request_preimage - 2

<details><summary><code>request_preimage(hash)</code></summary>

Taking 0.0121 % of a block.

```rust
hash: T::Hash
```
</details>



Request a preimage be uploaded to the chain without paying any fees or deposits.

If the preimage requests has already been provided on-chain, we unreserve any deposit
a user may have paid, and take the control of the preimage out of their hands.

### unrequest_preimage - 3

<details><summary><code>unrequest_preimage(hash)</code></summary>

Taking 0.0171 % of a block.

```rust
hash: T::Hash
```
</details>



Clear a previously made request for a preimage.

NOTE: THIS MUST NOT BE CALLED ON `hash` MORE TIMES THAN `request_preimage`.

### ensure_updated - 4

<details><summary><code>ensure_updated(hashes)</code></summary>

Taking 17.5344 % of a block.

```rust
hashes: Vec<T::Hash>
```
</details>



Ensure that the a bulk of pre-images is upgraded.

The caller pays no fee if at least 90% of pre-images were successfully updated.

## TechnicalCommittee - 23

### execute - 1

<details><summary><code>execute(proposal, length_bound)</code></summary>

Taking 0.006 % of a block.

```rust
proposal: Box<<T as Config<I>>::Proposal>
length_bound: u32
```
</details>



Dispatch a proposal from a member using the `Member` origin.

Origin must be a member of the collective.

###### Complexity:
- `O(B + M + P)` where:
- `B` is `proposal` size in bytes (length-fee-bounded)
- `M` members-count (code-bounded)
- `P` complexity of dispatching `proposal`

### propose - 2

<details><summary><code>propose(threshold, proposal, length_bound)</code></summary>

No weight available.

```rust
threshold: MemberCount
proposal: Box<<T as Config<I>>::Proposal>
length_bound: u32
```
</details>



Add a new proposal to either be voted on or executed directly.

Requires the sender to be member.

`threshold` determines whether `proposal` is executed directly (`threshold < 2`)
or put up for voting.

###### Complexity
- `O(B + M + P1)` or `O(B + M + P2)` where:
  - `B` is `proposal` size in bytes (length-fee-bounded)
  - `M` is members-count (code- and governance-bounded)
  - branching is influenced by `threshold` where:
    - `P1` is proposal execution complexity (`threshold < 2`)
    - `P2` is proposals-count (code-bounded) (`threshold >= 2`)

### vote - 3

<details><summary><code>vote(proposal, index, approve)</code></summary>

Taking 0.0121 % of a block.

```rust
proposal: T::Hash
index: ProposalIndex
approve: bool
```
</details>



Add an aye or nay vote for the sender to the given proposal.

Requires the sender to be a member.

Transaction fees will be waived if the member is voting on any particular proposal
for the first time and the call is successful. Subsequent vote changes will charge a
fee.
###### Complexity
- `O(M)` where `M` is members-count (code- and governance-bounded)

### close - 6

<details><summary><code>close(proposal_hash, index, proposal_weight_bound, length_bound)</code></summary>

No weight available.

```rust
proposal_hash: T::Hash
index: ProposalIndex
proposal_weight_bound: Weight
length_bound: u32
```
</details>



Close a vote that is either approved, disapproved or whose voting period has ended.

May be called by any signed account in order to finish voting and close the proposal.

If called before the end of the voting period it will only close the vote if it is
has enough votes to be approved or disapproved.

If called after the end of the voting period abstentions are counted as rejections
unless there is a prime member set and the prime member cast an approval.

If the close operation completes successfully with disapproval, the transaction fee will
be waived. Otherwise execution of the approved operation will be charged to the caller.

+ `proposal_weight_bound`: The maximum amount of weight consumed by executing the closed
proposal.
+ `length_bound`: The upper bound for the length of the proposal in storage. Checked via
`storage::read` so it is `size_of::<u32>() == 4` larger than the pure length.

###### Complexity
- `O(B + M + P1 + P2)` where:
  - `B` is `proposal` size in bytes (length-fee-bounded)
  - `M` is members-count (code- and governance-bounded)
  - `P1` is the complexity of `proposal` preimage.
  - `P2` is proposal-count (code-bounded)

### kill - 7

<details><summary><code>kill(proposal_hash)</code></summary>

Taking 0.0229 % of a block.

```rust
proposal_hash: T::Hash
```
</details>



Disapprove the proposal and burn the cost held for storing this proposal.

Parameters:
- `origin`: must be the `KillOrigin`.
- `proposal_hash`: The hash of the proposal that should be killed.

Emits `Killed` and `ProposalCostBurned` if any cost was held for a given proposal.

### release_proposal_cost - 8

<details><summary><code>release_proposal_cost(proposal_hash)</code></summary>

Taking 0.0066 % of a block.

```rust
proposal_hash: T::Hash
```
</details>



Release the cost held for storing a proposal once the given proposal is completed.

If there is no associated cost for the given proposal, this call will have no effect.

Parameters:
- `origin`: must be `Signed` or `Root`.
- `proposal_hash`: The hash of the proposal.

Emits `ProposalCostReleased` if any cost held for a given proposal.

## UniversalDividend - 30

### claim_uds - 0

<details><summary><code>claim_uds()</code></summary>

Taking 0.02 % of a block.

```rust
```
</details>



Claim Universal Dividends.

### transfer_ud - 1

<details><summary><code>transfer_ud(dest, value)</code></summary>

Taking 0.0194 % of a block.

```rust
dest: <T::Lookup as StaticLookup>::Source
value: BalanceOf<T>
```
</details>



Transfer some liquid free balance to another account, in milliUD.

### transfer_ud_keep_alive - 2

<details><summary><code>transfer_ud_keep_alive(dest, value)</code></summary>

Taking 0.0129 % of a block.

```rust
dest: <T::Lookup as StaticLookup>::Source
value: BalanceOf<T>
```
</details>



Transfer some liquid free balance to another account in milliUD and keep the account alive.

## Identity - 41

### create_identity - 0

<details><summary><code>create_identity(owner_key)</code></summary>

Taking 0.0776 % of a block.

```rust
owner_key: T::AccountId
```
</details>



Create an identity for an existing account

- `owner_key`: the public key corresponding to the identity to be created

The origin must be allowed to create an identity.

### confirm_identity - 1

<details><summary><code>confirm_identity(idty_name)</code></summary>

Taking 0.0298 % of a block.

```rust
idty_name: IdtyName
```
</details>



Confirm the creation of an identity and give it a name

- `idty_name`: the name uniquely associated to this identity. Must match the validation rules defined by the runtime.

The identity must have been created using `create_identity` before it can be confirmed.

### change_owner_key - 3

<details><summary><code>change_owner_key(new_key, new_key_sig)</code></summary>

Taking 0.0388 % of a block.

```rust
new_key: T::AccountId
new_key_sig: T::Signature
```
</details>



Change identity owner key.

- `new_key`: the new owner key.
- `new_key_sig`: the signature of the encoded form of `IdtyIndexAccountIdPayload`.
                 Must be signed by `new_key`.

The origin should be the old identity owner key.

### revoke_identity - 4

<details><summary><code>revoke_identity(idty_index, revocation_key, revocation_sig)</code></summary>

Taking 0.0365 % of a block.

```rust
idty_index: T::IdtyIndex
revocation_key: T::AccountId
revocation_sig: T::Signature
```
</details>



Revoke an identity using a revocation signature

- `idty_index`: the index of the identity to be revoked.
- `revocation_key`: the key used to sign the revocation payload.
- `revocation_sig`: the signature of the encoded form of `RevocationPayload`.
                    Must be signed by `revocation_key`.

Any signed origin can execute this call.

### revoke_identity_legacy - 9

<details><summary><code>revoke_identity_legacy(revocation_document)</code></summary>

Taking 0.04 % of a block.

```rust
revocation_document: Vec<u8>
```
</details>



Revoke an identity using a legacy (DUBP) revocation document

- `revocation document`: the full-length revocation document, signature included

Any signed origin can execute this call.

### link_account - 8

<details><summary><code>link_account(account_id, payload_sig)</code></summary>

Taking 0.0144 % of a block.

```rust
account_id: T::AccountId
payload_sig: T::Signature
```
</details>



Link an account to an identity.

This function links a specified account to an identity, requiring both the account and the
identity to sign the operation.

- `origin` - The origin of the call, which must have an associated identity index.
- `account_id` - The account ID to link, which must sign the payload.
- `payload_sig` - The signature with the linked identity.

## Certification - 43

### add_cert - 0

<details><summary><code>add_cert(receiver)</code></summary>

Taking 0.0324 % of a block.

```rust
receiver: T::IdtyIndex
```
</details>



Add a new certification.

### renew_cert - 3

<details><summary><code>renew_cert(receiver)</code></summary>

Taking 0.0266 % of a block.

```rust
receiver: T::IdtyIndex
```
</details>



Renew an existing certification.

## Distance - 44

### request_distance_evaluation - 0

<details><summary><code>request_distance_evaluation()</code></summary>

Taking 0.0351 % of a block.

```rust
```
</details>



Request evaluation of the caller's identity distance.

This function allows the caller to request an evaluation of their distance.
A positive evaluation will lead to claiming or renewing membership, while a negative
evaluation will result in slashing for the caller.

### request_distance_evaluation_for - 4

<details><summary><code>request_distance_evaluation_for(target)</code></summary>

Taking 0.036 % of a block.

```rust
target: T::IdtyIndex
```
</details>



Request evaluation of a target identity's distance.

This function allows the caller to request an evaluation of a specific target identity's distance.
This action is only permitted for unvalidated identities.

## AtomicSwap - 50

### create_swap - 0

<details><summary><code>create_swap(target, hashed_proof, action, duration)</code></summary>

No weight available.

```rust
target: T::AccountId
hashed_proof: HashedProof
action: T::SwapAction
duration: BlockNumberFor<T>
```
</details>



Register a new atomic swap, declaring an intention to send funds from origin to target
on the current blockchain. The target can claim the fund using the revealed proof. If
the fund is not claimed after `duration` blocks, then the sender can cancel the swap.

The dispatch origin for this call must be _Signed_.

- `target`: Receiver of the atomic swap.
- `hashed_proof`: The blake2_256 hash of the secret proof.
- `balance`: Funds to be sent from origin.
- `duration`: Locked duration of the atomic swap. For safety reasons, it is recommended
  that the revealer uses a shorter duration than the counterparty, to prevent the
  situation where the revealer reveals the proof too late around the end block.

### claim_swap - 1

<details><summary><code>claim_swap(proof, action)</code></summary>

No weight available.

```rust
proof: Vec<u8>
action: T::SwapAction
```
</details>



Claim an atomic swap.

The dispatch origin for this call must be _Signed_.

- `proof`: Revealed proof of the claim.
- `action`: Action defined in the swap, it must match the entry in blockchain. Otherwise
  the operation fails. This is used for weight calculation.

### cancel_swap - 2

<details><summary><code>cancel_swap(target, hashed_proof)</code></summary>

No weight available.

```rust
target: T::AccountId
hashed_proof: HashedProof
```
</details>



Cancel an atomic swap. Only possible after the originally set duration has passed.

The dispatch origin for this call must be _Signed_.

- `target`: Target of the original atomic swap.
- `hashed_proof`: Hashed proof of the original atomic swap.

## Multisig - 51

### as_multi_threshold_1 - 0

<details><summary><code>as_multi_threshold_1(other_signatories, call)</code></summary>

Taking 0.0051 % of a block.

```rust
other_signatories: Vec<T::AccountId>
call: Box<<T as Config>::RuntimeCall>
```
</details>



Immediately dispatch a multi-signature call using a single approval from the caller.

The dispatch origin for this call must be _Signed_.

- `other_signatories`: The accounts (other than the sender) who are part of the
multi-signature, but do not participate in the approval process.
- `call`: The call to be executed.

Result is equivalent to the dispatched result.

###### Complexity
O(Z + C) where Z is the length of the call and C its execution weight.

### as_multi - 1

<details><summary><code>as_multi(threshold, other_signatories, maybe_timepoint, call, max_weight)</code></summary>

No weight available.

```rust
threshold: u16
other_signatories: Vec<T::AccountId>
maybe_timepoint: Option<Timepoint<BlockNumberFor<T>>>
call: Box<<T as Config>::RuntimeCall>
max_weight: Weight
```
</details>



Register approval for a dispatch to be made from a deterministic composite account if
approved by a total of `threshold - 1` of `other_signatories`.

If there are enough, then dispatch the call.

Payment: `DepositBase` will be reserved if this is the first approval, plus
`threshold` times `DepositFactor`. It is returned once this dispatch happens or
is cancelled.

The dispatch origin for this call must be _Signed_.

- `threshold`: The total number of approvals for this dispatch before it is executed.
- `other_signatories`: The accounts (other than the sender) who can approve this
dispatch. May not be empty.
- `maybe_timepoint`: If this is the first approval, then this must be `None`. If it is
not the first approval, then it must be `Some`, with the timepoint (block number and
transaction index) of the first approval transaction.
- `call`: The call to be executed.

NOTE: Unless this is the final approval, you will generally want to use
`approve_as_multi` instead, since it only requires a hash of the call.

Result is equivalent to the dispatched result if `threshold` is exactly `1`. Otherwise
on success, result is `Ok` and the result from the interior call, if it was executed,
may be found in the deposited `MultisigExecuted` event.

###### Complexity
- `O(S + Z + Call)`.
- Up to one balance-reserve or unreserve operation.
- One passthrough operation, one insert, both `O(S)` where `S` is the number of
  signatories. `S` is capped by `MaxSignatories`, with weight being proportional.
- One call encode & hash, both of complexity `O(Z)` where `Z` is tx-len.
- One encode & hash, both of complexity `O(S)`.
- Up to one binary search and insert (`O(logS + S)`).
- I/O: 1 read `O(S)`, up to 1 mutate `O(S)`. Up to one remove.
- One event.
- The weight of the `call`.
- Storage: inserts one item, value size bounded by `MaxSignatories`, with a deposit
  taken for its lifetime of `DepositBase + threshold * DepositFactor`.

### approve_as_multi - 2

<details><summary><code>approve_as_multi(threshold, other_signatories, maybe_timepoint, call_hash, max_weight)</code></summary>

No weight available.

```rust
threshold: u16
other_signatories: Vec<T::AccountId>
maybe_timepoint: Option<Timepoint<BlockNumberFor<T>>>
call_hash: [u8; 32]
max_weight: Weight
```
</details>



Register approval for a dispatch to be made from a deterministic composite account if
approved by a total of `threshold - 1` of `other_signatories`.

Payment: `DepositBase` will be reserved if this is the first approval, plus
`threshold` times `DepositFactor`. It is returned once this dispatch happens or
is cancelled.

The dispatch origin for this call must be _Signed_.

- `threshold`: The total number of approvals for this dispatch before it is executed.
- `other_signatories`: The accounts (other than the sender) who can approve this
dispatch. May not be empty.
- `maybe_timepoint`: If this is the first approval, then this must be `None`. If it is
not the first approval, then it must be `Some`, with the timepoint (block number and
transaction index) of the first approval transaction.
- `call_hash`: The hash of the call to be executed.

NOTE: If this is the final approval, you will want to use `as_multi` instead.

###### Complexity
- `O(S)`.
- Up to one balance-reserve or unreserve operation.
- One passthrough operation, one insert, both `O(S)` where `S` is the number of
  signatories. `S` is capped by `MaxSignatories`, with weight being proportional.
- One encode & hash, both of complexity `O(S)`.
- Up to one binary search and insert (`O(logS + S)`).
- I/O: 1 read `O(S)`, up to 1 mutate `O(S)`. Up to one remove.
- One event.
- Storage: inserts one item, value size bounded by `MaxSignatories`, with a deposit
  taken for its lifetime of `DepositBase + threshold * DepositFactor`.

### cancel_as_multi - 3

<details><summary><code>cancel_as_multi(threshold, other_signatories, timepoint, call_hash)</code></summary>

Taking 0.0117 % of a block.

```rust
threshold: u16
other_signatories: Vec<T::AccountId>
timepoint: Timepoint<BlockNumberFor<T>>
call_hash: [u8; 32]
```
</details>



Cancel a pre-existing, on-going multisig transaction. Any deposit reserved previously
for this operation will be unreserved on success.

The dispatch origin for this call must be _Signed_.

- `threshold`: The total number of approvals for this dispatch before it is executed.
- `other_signatories`: The accounts (other than the sender) who can approve this
dispatch. May not be empty.
- `timepoint`: The timepoint (block number and transaction index) of the first approval
transaction for this dispatch.
- `call_hash`: The hash of the call to be executed.

###### Complexity
- `O(S)`.
- Up to one balance-reserve or unreserve operation.
- One passthrough operation, one insert, both `O(S)` where `S` is the number of
  signatories. `S` is capped by `MaxSignatories`, with weight being proportional.
- One encode & hash, both of complexity `O(S)`.
- One event.
- I/O: 1 read `O(S)`, one remove.
- Storage: removes one item.

## ProvideRandomness - 52

### request - 0

<details><summary><code>request(randomness_type, salt)</code></summary>

Taking 0.0367 % of a block.

```rust
randomness_type: RandomnessType
salt: H256
```
</details>



Request randomness.

## Proxy - 53

### proxy - 0

<details><summary><code>proxy(real, force_proxy_type, call)</code></summary>

Taking 0.0061 % of a block.

```rust
real: AccountIdLookupOf<T>
force_proxy_type: Option<T::ProxyType>
call: Box<<T as Config>::RuntimeCall>
```
</details>



Dispatch the given `call` from an account that the sender is authorised for through
`add_proxy`.

The dispatch origin for this call must be _Signed_.

Parameters:
- `real`: The account that the proxy will make a call on behalf of.
- `force_proxy_type`: Specify the exact proxy type to be used and checked for this call.
- `call`: The call to be made by the `real` account.

### add_proxy - 1

<details><summary><code>add_proxy(delegate, proxy_type, delay)</code></summary>

Taking 0.0114 % of a block.

```rust
delegate: AccountIdLookupOf<T>
proxy_type: T::ProxyType
delay: BlockNumberFor<T>
```
</details>



Register a proxy account for the sender that is able to make calls on its behalf.

The dispatch origin for this call must be _Signed_.

Parameters:
- `proxy`: The account that the `caller` would like to make a proxy.
- `proxy_type`: The permissions allowed for this proxy account.
- `delay`: The announcement period required of the initial proxy. Will generally be
zero.

### remove_proxy - 2

<details><summary><code>remove_proxy(delegate, proxy_type, delay)</code></summary>

Taking 0.0114 % of a block.

```rust
delegate: AccountIdLookupOf<T>
proxy_type: T::ProxyType
delay: BlockNumberFor<T>
```
</details>



Unregister a proxy account for the sender.

The dispatch origin for this call must be _Signed_.

Parameters:
- `proxy`: The account that the `caller` would like to remove as a proxy.
- `proxy_type`: The permissions currently enabled for the removed proxy account.

### remove_proxies - 3

<details><summary><code>remove_proxies()</code></summary>

Taking 0.0113 % of a block.

```rust
```
</details>



Unregister all proxy accounts for the sender.

The dispatch origin for this call must be _Signed_.

WARNING: This may be called on accounts created by `pure`, however if done, then
the unreserved fees will be inaccessible. **All access to this account will be lost.**

### create_pure - 4

<details><summary><code>create_pure(proxy_type, delay, index)</code></summary>

Taking 0.0114 % of a block.

```rust
proxy_type: T::ProxyType
delay: BlockNumberFor<T>
index: u16
```
</details>



Spawn a fresh new account that is guaranteed to be otherwise inaccessible, and
initialize it with a proxy of `proxy_type` for `origin` sender.

Requires a `Signed` origin.

- `proxy_type`: The type of the proxy that the sender will be registered as over the
new account. This will almost always be the most permissive `ProxyType` possible to
allow for maximum flexibility.
- `index`: A disambiguation index, in case this is called multiple times in the same
transaction (e.g. with `utility::batch`). Unless you're using `batch` you probably just
want to use `0`.
- `delay`: The announcement period required of the initial proxy. Will generally be
zero.

Fails with `Duplicate` if this has already been called in this transaction, from the
same sender, with the same parameters.

Fails if there are insufficient funds to pay for deposit.

### kill_pure - 5

<details><summary><code>kill_pure(spawner, proxy_type, index, height, ext_index)</code></summary>

Taking 0.0114 % of a block.

```rust
spawner: AccountIdLookupOf<T>
proxy_type: T::ProxyType
index: u16
height: BlockNumberFor<T>
ext_index: u32
```
</details>



Removes a previously spawned pure proxy.

WARNING: **All access to this account will be lost.** Any funds held in it will be
inaccessible.

Requires a `Signed` origin, and the sender account must have been created by a call to
`pure` with corresponding parameters.

- `spawner`: The account that originally called `pure` to create this account.
- `index`: The disambiguation index originally passed to `pure`. Probably `0`.
- `proxy_type`: The proxy type originally passed to `pure`.
- `height`: The height of the chain when the call to `pure` was processed.
- `ext_index`: The extrinsic index in which the call to `pure` was processed.

Fails with `NoPermission` in case the caller is not a previously created pure
account whose `pure` call has corresponding parameters.

### announce - 6

<details><summary><code>announce(real, call_hash)</code></summary>

Taking 0.0185 % of a block.

```rust
real: AccountIdLookupOf<T>
call_hash: CallHashOf<T>
```
</details>



Publish the hash of a proxy-call that will be made in the future.

This must be called some number of blocks before the corresponding `proxy` is attempted
if the delay associated with the proxy relationship is greater than zero.

No more than `MaxPending` announcements may be made at any one time.

This will take a deposit of `AnnouncementDepositFactor` as well as
`AnnouncementDepositBase` if there are no other pending announcements.

The dispatch origin for this call must be _Signed_ and a proxy of `real`.

Parameters:
- `real`: The account that the proxy will make a call on behalf of.
- `call_hash`: The hash of the call to be made by the `real` account.

### remove_announcement - 7

<details><summary><code>remove_announcement(real, call_hash)</code></summary>

Taking 0.0173 % of a block.

```rust
real: AccountIdLookupOf<T>
call_hash: CallHashOf<T>
```
</details>



Remove a given announcement.

May be called by a proxy account to remove a call they previously announced and return
the deposit.

The dispatch origin for this call must be _Signed_.

Parameters:
- `real`: The account that the proxy will make a call on behalf of.
- `call_hash`: The hash of the call to be made by the `real` account.

### reject_announcement - 8

<details><summary><code>reject_announcement(delegate, call_hash)</code></summary>

Taking 0.0173 % of a block.

```rust
delegate: AccountIdLookupOf<T>
call_hash: CallHashOf<T>
```
</details>



Remove the given announcement of a delegate.

May be called by a target (proxied) account to remove a call that one of their delegates
(`delegate`) has announced they want to execute. The deposit is returned.

The dispatch origin for this call must be _Signed_.

Parameters:
- `delegate`: The account that previously announced the call.
- `call_hash`: The hash of the call to be made.

### proxy_announced - 9

<details><summary><code>proxy_announced(delegate, real, force_proxy_type, call)</code></summary>

Taking 0.0187 % of a block.

```rust
delegate: AccountIdLookupOf<T>
real: AccountIdLookupOf<T>
force_proxy_type: Option<T::ProxyType>
call: Box<<T as Config>::RuntimeCall>
```
</details>



Dispatch the given `call` from an account that the sender is authorized for through
`add_proxy`.

Removes any corresponding announcement(s).

The dispatch origin for this call must be _Signed_.

Parameters:
- `real`: The account that the proxy will make a call on behalf of.
- `force_proxy_type`: Specify the exact proxy type to be used and checked for this call.
- `call`: The call to be made by the `real` account.

## Utility - 54

### batch - 0

<details><summary><code>batch(calls)</code></summary>

Taking 0.1152 % of a block.

```rust
calls: Vec<<T as Config>::RuntimeCall>
```
</details>



Send a batch of dispatch calls.

May be called from any origin except `None`.

- `calls`: The calls to be dispatched from the same origin. The number of call must not
  exceed the constant: `batched_calls_limit` (available in constant metadata).

If origin is root then the calls are dispatched without checking origin filter. (This
includes bypassing `frame_system::Config::BaseCallFilter`).

###### Complexity
- O(C) where C is the number of calls to be batched.

This will return `Ok` in all circumstances. To determine the success of the batch, an
event is deposited. If a call failed and the batch was interrupted, then the
`BatchInterrupted` event is deposited, along with the number of successful calls made
and the error of the failed call. If all were successful, then the `BatchCompleted`
event is deposited.

### as_derivative - 1

<details><summary><code>as_derivative(index, call)</code></summary>

Taking 0.0048 % of a block.

```rust
index: u16
call: Box<<T as Config>::RuntimeCall>
```
</details>



Send a call through an indexed pseudonym of the sender.

Filter from origin are passed along. The call will be dispatched with an origin which
use the same filter as the origin of this call.

NOTE: If you need to ensure that any account-based filtering is not honored (i.e.
because you expect `proxy` to have been used prior in the call stack and you do not want
the call restrictions to apply to any sub-accounts), then use `as_multi_threshold_1`
in the Multisig pallet instead.

NOTE: Prior to version *12, this was called `as_limited_sub`.

The dispatch origin for this call must be _Signed_.

### batch_all - 2

<details><summary><code>batch_all(calls)</code></summary>

Taking 0.123 % of a block.

```rust
calls: Vec<<T as Config>::RuntimeCall>
```
</details>



Send a batch of dispatch calls and atomically execute them.
The whole transaction will rollback and fail if any of the calls failed.

May be called from any origin except `None`.

- `calls`: The calls to be dispatched from the same origin. The number of call must not
  exceed the constant: `batched_calls_limit` (available in constant metadata).

If origin is root then the calls are dispatched without checking origin filter. (This
includes bypassing `frame_system::Config::BaseCallFilter`).

###### Complexity
- O(C) where C is the number of calls to be batched.

### force_batch - 4

<details><summary><code>force_batch(calls)</code></summary>

Taking 0.1152 % of a block.

```rust
calls: Vec<<T as Config>::RuntimeCall>
```
</details>



Send a batch of dispatch calls.
Unlike `batch`, it allows errors and won't interrupt.

May be called from any origin except `None`.

- `calls`: The calls to be dispatched from the same origin. The number of call must not
  exceed the constant: `batched_calls_limit` (available in constant metadata).

If origin is root then the calls are dispatch without checking origin filter. (This
includes bypassing `frame_system::Config::BaseCallFilter`).

###### Complexity
- O(C) where C is the number of calls to be batched.

## Treasury - 55

### spend_local - 3

<details><summary><code>spend_local(amount, beneficiary)</code></summary>

Taking 0.0046 % of a block.

```rust
amount: BalanceOf<T, I>
beneficiary: AccountIdLookupOf<T>
```
</details>



Propose and approve a spend of treasury funds.

###### Dispatch Origin

Must be [`Config::SpendOrigin`] with the `Success` value being at least `amount`.

###### Details
NOTE: For record-keeping purposes, the proposer is deemed to be equivalent to the
beneficiary.

###### Parameters
- `amount`: The amount to be transferred from the treasury to the `beneficiary`.
- `beneficiary`: The destination account for the transfer.

###### Events

Emits [`Event::SpendApproved`] if successful.

### remove_approval - 4

<details><summary><code>remove_approval(proposal_id)</code></summary>

Taking 0.0056 % of a block.

```rust
proposal_id: ProposalIndex
```
</details>



Force a previously approved proposal to be removed from the approval queue.

###### Dispatch Origin

Must be [`Config::RejectOrigin`].

###### Details

The original deposit will no longer be returned.

###### Parameters
- `proposal_id`: The index of a proposal

###### Complexity
- O(A) where `A` is the number of approvals

###### Errors
- [`Error::ProposalNotApproved`]: The `proposal_id` supplied was not found in the
  approval queue, i.e., the proposal has not been approved. This could also mean the
  proposal does not exist altogether, thus there is no way it would have been approved
  in the first place.

### spend - 5

<details><summary><code>spend(asset_kind, amount, beneficiary, valid_from)</code></summary>

Taking 0.0046 % of a block.

```rust
asset_kind: Box<T::AssetKind>
amount: AssetBalanceOf<T, I>
beneficiary: Box<BeneficiaryLookupOf<T, I>>
valid_from: Option<BlockNumberFor<T>>
```
</details>



Propose and approve a spend of treasury funds.

###### Dispatch Origin

Must be [`Config::SpendOrigin`] with the `Success` value being at least
`amount` of `asset_kind` in the native asset. The amount of `asset_kind` is converted
for assertion using the [`Config::BalanceConverter`].

###### Details

Create an approved spend for transferring a specific `amount` of `asset_kind` to a
designated beneficiary. The spend must be claimed using the `payout` dispatchable within
the [`Config::PayoutPeriod`].

###### Parameters
- `asset_kind`: An indicator of the specific asset class to be spent.
- `amount`: The amount to be transferred from the treasury to the `beneficiary`.
- `beneficiary`: The beneficiary of the spend.
- `valid_from`: The block number from which the spend can be claimed. It can refer to
  the past if the resulting spend has not yet expired according to the
  [`Config::PayoutPeriod`]. If `None`, the spend can be claimed immediately after
  approval.

###### Events

Emits [`Event::AssetSpendApproved`] if successful.

### payout - 6

<details><summary><code>payout(index)</code></summary>

Taking 0.0056 % of a block.

```rust
index: SpendIndex
```
</details>



Claim a spend.

###### Dispatch Origin

Must be signed

###### Details

Spends must be claimed within some temporal bounds. A spend may be claimed within one
[`Config::PayoutPeriod`] from the `valid_from` block.
In case of a payout failure, the spend status must be updated with the `check_status`
dispatchable before retrying with the current function.

###### Parameters
- `index`: The spend index.

###### Events

Emits [`Event::Paid`] if successful.

### check_status - 7

<details><summary><code>check_status(index)</code></summary>

Taking 0.0056 % of a block.

```rust
index: SpendIndex
```
</details>



Check the status of the spend and remove it from the storage if processed.

###### Dispatch Origin

Must be signed.

###### Details

The status check is a prerequisite for retrying a failed payout.
If a spend has either succeeded or expired, it is removed from the storage by this
function. In such instances, transaction fees are refunded.

###### Parameters
- `index`: The spend index.

###### Events

Emits [`Event::PaymentFailed`] if the spend payout has failed.
Emits [`Event::SpendProcessed`] if the spend payout has succeed.

### void_spend - 8

<details><summary><code>void_spend(index)</code></summary>

Taking 0.0056 % of a block.

```rust
index: SpendIndex
```
</details>



Void previously approved spend.

###### Dispatch Origin

Must be [`Config::RejectOrigin`].

###### Details

A spend void is only possible if the payout has not been attempted yet.

###### Parameters
- `index`: The spend index.

###### Events

Emits [`Event::AssetSpendVoided`] if successful.


