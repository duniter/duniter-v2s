# Runtime calls

Calls are categorized according to the dispatch origin they require:

1. **User calls**: the dispatch origin for this kind of call must be signed by
the transactor. This is the only call category that can be submitted with an extrinsic.
1. **Root calls**: This kind of call requires a special origin that can only be invoked
through on-chain governance mechanisms.
1. **Inherent calls**: This kind of call is invoked by the author of the block itself
(usually automatically by the node).
1. **Disabled calls**: These calls can not be called directly, they are reserved for internal use by other runtime calls.


## User calls

There are **68** user calls from **21** pallets.

### Scheduler - 2

#### schedule - 0

<details><summary><code>schedule(when, maybe_periodic, priority, call)</code></summary>

```rust
when: T::BlockNumber
maybe_periodic: Option<schedule::Period<T::BlockNumber>>
priority: schedule::Priority
call: Box<<T as Config>::RuntimeCall>
```
</details>


Anonymously schedule a task.

#### cancel - 1

<details><summary><code>cancel(when, index)</code></summary>

```rust
when: T::BlockNumber
index: u32
```
</details>


Cancel an anonymously scheduled task.

#### schedule_named - 2

<details><summary><code>schedule_named(id, when, maybe_periodic, priority, call)</code></summary>

```rust
id: TaskName
when: T::BlockNumber
maybe_periodic: Option<schedule::Period<T::BlockNumber>>
priority: schedule::Priority
call: Box<<T as Config>::RuntimeCall>
```
</details>


Schedule a named task.

#### cancel_named - 3

<details><summary><code>cancel_named(id)</code></summary>

```rust
id: TaskName
```
</details>


Cancel a named scheduled task.

#### schedule_after - 4

<details><summary><code>schedule_after(after, maybe_periodic, priority, call)</code></summary>

```rust
after: T::BlockNumber
maybe_periodic: Option<schedule::Period<T::BlockNumber>>
priority: schedule::Priority
call: Box<<T as Config>::RuntimeCall>
```
</details>


Anonymously schedule a task after a delay.


#### schedule_named_after - 5

<details><summary><code>schedule_named_after(id, after, maybe_periodic, priority, call)</code></summary>

```rust
id: TaskName
after: T::BlockNumber
maybe_periodic: Option<schedule::Period<T::BlockNumber>>
priority: schedule::Priority
call: Box<<T as Config>::RuntimeCall>
```
</details>


Schedule a named task after a delay.


### Babe - 3

#### report_equivocation - 0

<details><summary><code>report_equivocation(equivocation_proof, key_owner_proof)</code></summary>

```rust
equivocation_proof: Box<EquivocationProof<T::Header>>
key_owner_proof: T::KeyOwnerProof
```
</details>


Report authority equivocation/misbehavior. This method will verify
the equivocation proof and validate the given key ownership proof
against the extracted offender. If both are valid, the offence will
be reported.

### Balances - 6

#### transfer - 0

<details><summary><code>transfer(dest, value)</code></summary>

```rust
dest: AccountIdLookupOf<T>
value: T::Balance
```
</details>


Transfer some liquid free balance to another account.

`transfer` will set the `FreeBalance` of the sender and receiver.
If the sender's account is below the existential deposit as a result
of the transfer, the account will be reaped.

The dispatch origin for this call must be `Signed` by the transactor.


#### transfer_keep_alive - 3

<details><summary><code>transfer_keep_alive(dest, value)</code></summary>

```rust
dest: AccountIdLookupOf<T>
value: T::Balance
```
</details>


Same as the [`transfer`] call, but with a check that the transfer will not kill the
origin account.

99% of the time you want [`transfer`] instead.

[`transfer`]: struct.Pallet.html#method.transfer

#### transfer_all - 4

<details><summary><code>transfer_all(dest, keep_alive)</code></summary>

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
  keep the sender account alive (true). # <weight>
- O(1). Just like transfer, but reading the user's transferable balance first.
  #</weight>

### OneshotAccount - 7

#### create_oneshot_account - 0

<details><summary><code>create_oneshot_account(dest, value)</code></summary>

```rust
dest: <T::Lookup as StaticLookup>::Source
value: <T::Currency as Currency<T::AccountId>>::Balance
```
</details>


Create an account that can only be consumed once

- `dest`: The oneshot account to be created.
- `balance`: The balance to be transfered to this oneshot account.

Origin account is kept alive.

#### consume_oneshot_account - 1

<details><summary><code>consume_oneshot_account(block_height, dest)</code></summary>

```rust
block_height: T::BlockNumber
dest: Account<<T::Lookup as StaticLookup>::Source>
```
</details>


Consume a oneshot account and transfer its balance to an account

- `block_height`: Must be a recent block number. The limit is `BlockHashCount` in the past. (this is to prevent replay attacks)
- `dest`: The destination account.
- `dest_is_oneshot`: If set to `true`, then a oneshot account is created at `dest`. Else, `dest` has to be an existing account.

#### consume_oneshot_account_with_remaining - 2

<details><summary><code>consume_oneshot_account_with_remaining(block_height, dest, remaining_to, balance)</code></summary>

```rust
block_height: T::BlockNumber
dest: Account<<T::Lookup as StaticLookup>::Source>
remaining_to: Account<<T::Lookup as StaticLookup>::Source>
balance: <T::Currency as Currency<T::AccountId>>::Balance
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

### AuthorityMembers - 10

#### go_offline - 0

<details><summary><code>go_offline()</code></summary>

```rust
```
</details>


ask to leave the set of validators two sessions after

#### go_online - 1

<details><summary><code>go_online()</code></summary>

```rust
```
</details>


ask to join the set of validators two sessions after

#### set_session_keys - 2

<details><summary><code>set_session_keys(keys)</code></summary>

```rust
keys: T::KeysWrapper
```
</details>


declare new session keys to replace current ones

### Grandpa - 15

#### report_equivocation - 0

<details><summary><code>report_equivocation(equivocation_proof, key_owner_proof)</code></summary>

```rust
equivocation_proof: Box<EquivocationProof<T::Hash, T::BlockNumber>>
key_owner_proof: T::KeyOwnerProof
```
</details>


Report voter equivocation/misbehavior. This method will verify the
equivocation proof and validate the given key ownership proof
against the extracted offender. If both are valid, the offence
will be reported.

### UpgradeOrigin - 21

#### dispatch_as_root_unchecked_weight - 1

<details><summary><code>dispatch_as_root_unchecked_weight(call, weight)</code></summary>

```rust
call: Box<<T as Config>::Call>
weight: Weight
```
</details>


Dispatches a function call from root origin.
This function does not check the weight of the call, and instead allows the
caller to specify the weight of the call.

The weight of this call is defined by the caller.

### Preimage - 22

#### note_preimage - 0

<details><summary><code>note_preimage(bytes)</code></summary>

```rust
bytes: Vec<u8>
```
</details>


Register a preimage on-chain.

If the preimage was previously requested, no fees or deposits are taken for providing
the preimage. Otherwise, a deposit is taken proportional to the size of the preimage.

#### unnote_preimage - 1

<details><summary><code>unnote_preimage(hash)</code></summary>

```rust
hash: T::Hash
```
</details>


Clear an unrequested preimage from the runtime storage.

If `len` is provided, then it will be a much cheaper operation.

- `hash`: The hash of the preimage to be removed from the store.
- `len`: The length of the preimage of `hash`.

#### request_preimage - 2

<details><summary><code>request_preimage(hash)</code></summary>

```rust
hash: T::Hash
```
</details>


Request a preimage be uploaded to the chain without paying any fees or deposits.

If the preimage requests has already been provided on-chain, we unreserve any deposit
a user may have paid, and take the control of the preimage out of their hands.

#### unrequest_preimage - 3

<details><summary><code>unrequest_preimage(hash)</code></summary>

```rust
hash: T::Hash
```
</details>


Clear a previously made request for a preimage.

NOTE: THIS MUST NOT BE CALLED ON `hash` MORE TIMES THAN `request_preimage`.

### TechnicalCommittee - 23

#### execute - 1

<details><summary><code>execute(proposal, length_bound)</code></summary>

```rust
proposal: Box<<T as Config<I>>::Proposal>
length_bound: u32
```
</details>


Dispatch a proposal from a member using the `Member` origin.

Origin must be a member of the collective.


#### propose - 2

<details><summary><code>propose(threshold, proposal, length_bound)</code></summary>

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


#### vote - 3

<details><summary><code>vote(proposal, index, approve)</code></summary>

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

#### close_old_weight - 4

<details><summary><code>close_old_weight(proposal_hash, index, proposal_weight_bound, length_bound)</code></summary>

```rust
proposal_hash: T::Hash
index: ProposalIndex
proposal_weight_bound: OldWeight
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


#### close - 6

<details><summary><code>close(proposal_hash, index, proposal_weight_bound, length_bound)</code></summary>

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


### UniversalDividend - 30

#### claim_uds - 0

<details><summary><code>claim_uds()</code></summary>

```rust
```
</details>


Claim Universal Dividends

#### transfer_ud - 1

<details><summary><code>transfer_ud(dest, value)</code></summary>

```rust
dest: <T::Lookup as StaticLookup>::Source
value: BalanceOf<T>
```
</details>


Transfer some liquid free balance to another account, in milliUD.

#### transfer_ud_keep_alive - 2

<details><summary><code>transfer_ud_keep_alive(dest, value)</code></summary>

```rust
dest: <T::Lookup as StaticLookup>::Source
value: BalanceOf<T>
```
</details>


Transfer some liquid free balance to another account, in milliUD.

### Identity - 41

#### create_identity - 0

<details><summary><code>create_identity(owner_key)</code></summary>

```rust
owner_key: T::AccountId
```
</details>


Create an identity for an existing account

- `owner_key`: the public key corresponding to the identity to be created

The origin must be allowed to create an identity.

#### confirm_identity - 1

<details><summary><code>confirm_identity(idty_name)</code></summary>

```rust
idty_name: IdtyName
```
</details>


Confirm the creation of an identity and give it a name

- `idty_name`: the name uniquely associated to this identity. Must match the validation rules defined by the runtime.

The identity must have been created using `create_identity` before it can be confirmed.

#### validate_identity - 2

<details><summary><code>validate_identity(idty_index)</code></summary>

```rust
idty_index: T::IdtyIndex
```
</details>


validate the owned identity (must meet the main wot requirements)

#### change_owner_key - 3

<details><summary><code>change_owner_key(new_key, new_key_sig)</code></summary>

```rust
new_key: T::AccountId
new_key_sig: T::NewOwnerKeySignature
```
</details>


Change identity owner key.

- `new_key`: the new owner key.
- `new_key_sig`: the signature of the encoded form of `NewOwnerKeyPayload`.
                 Must be signed by `new_key`.

The origin should be the old identity owner key.

#### revoke_identity - 4

<details><summary><code>revoke_identity(idty_index, revocation_key, revocation_sig)</code></summary>

```rust
idty_index: T::IdtyIndex
revocation_key: T::AccountId
revocation_sig: T::RevocationSignature
```
</details>


Revoke an identity using a revocation signature

- `idty_index`: the index of the identity to be revoked.
- `revocation_key`: the key used to sign the revocation payload.
- `revocation_sig`: the signature of the encoded form of `RevocationPayload`.
                    Must be signed by `revocation_key`.

Any signed origin can execute this call.

#### fix_sufficients - 7

<details><summary><code>fix_sufficients(owner_key, inc)</code></summary>

```rust
owner_key: T::AccountId
inc: bool
```
</details>


change sufficient ref count for given key

### Membership - 42

#### renew_membership - 3

<details><summary><code>renew_membership(maybe_idty_id)</code></summary>

```rust
maybe_idty_id: Option<T::IdtyId>
```
</details>


extend the validity period of an active membership

### Cert - 43

#### add_cert - 1

<details><summary><code>add_cert(issuer, receiver)</code></summary>

```rust
issuer: T::IdtyIndex
receiver: T::IdtyIndex
```
</details>


Add a new certification or renew an existing one

- `receiver`: the account receiving the certification from the origin

The origin must be allow to certify.

### SmithMembership - 52

#### request_membership - 1

<details><summary><code>request_membership(metadata)</code></summary>

```rust
metadata: T::MetaData
```
</details>


submit a membership request (must have a declared identity)
(only available for sub wot, automatic for main wot)

#### claim_membership - 2

<details><summary><code>claim_membership(maybe_idty_id)</code></summary>

```rust
maybe_idty_id: Option<T::IdtyId>
```
</details>


claim that the previously requested membership fullfills the requirements
(only available for sub wot, automatic for main wot)

#### renew_membership - 3

<details><summary><code>renew_membership(maybe_idty_id)</code></summary>

```rust
maybe_idty_id: Option<T::IdtyId>
```
</details>


extend the validity period of an active membership

#### revoke_membership - 4

<details><summary><code>revoke_membership(maybe_idty_id)</code></summary>

```rust
maybe_idty_id: Option<T::IdtyId>
```
</details>


revoke an active membership
(only available for sub wot, automatic for main wot)

### SmithCert - 53

#### add_cert - 1

<details><summary><code>add_cert(issuer, receiver)</code></summary>

```rust
issuer: T::IdtyIndex
receiver: T::IdtyIndex
```
</details>


Add a new certification or renew an existing one

- `receiver`: the account receiving the certification from the origin

The origin must be allow to certify.

### AtomicSwap - 60

#### create_swap - 0

<details><summary><code>create_swap(target, hashed_proof, action, duration)</code></summary>

```rust
target: T::AccountId
hashed_proof: HashedProof
action: T::SwapAction
duration: T::BlockNumber
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

#### claim_swap - 1

<details><summary><code>claim_swap(proof, action)</code></summary>

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

#### cancel_swap - 2

<details><summary><code>cancel_swap(target, hashed_proof)</code></summary>

```rust
target: T::AccountId
hashed_proof: HashedProof
```
</details>


Cancel an atomic swap. Only possible after the originally set duration has passed.

The dispatch origin for this call must be _Signed_.

- `target`: Target of the original atomic swap.
- `hashed_proof`: Hashed proof of the original atomic swap.

### Multisig - 61

#### as_multi_threshold_1 - 0

<details><summary><code>as_multi_threshold_1(other_signatories, call)</code></summary>

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


#### as_multi - 1

<details><summary><code>as_multi(threshold, other_signatories, maybe_timepoint, call, max_weight)</code></summary>

```rust
threshold: u16
other_signatories: Vec<T::AccountId>
maybe_timepoint: Option<Timepoint<T::BlockNumber>>
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


#### approve_as_multi - 2

<details><summary><code>approve_as_multi(threshold, other_signatories, maybe_timepoint, call_hash, max_weight)</code></summary>

```rust
threshold: u16
other_signatories: Vec<T::AccountId>
maybe_timepoint: Option<Timepoint<T::BlockNumber>>
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


#### cancel_as_multi - 3

<details><summary><code>cancel_as_multi(threshold, other_signatories, timepoint, call_hash)</code></summary>

```rust
threshold: u16
other_signatories: Vec<T::AccountId>
timepoint: Timepoint<T::BlockNumber>
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


### ProvideRandomness - 62

#### request - 0

<details><summary><code>request(randomness_type, salt)</code></summary>

```rust
randomness_type: RandomnessType
salt: H256
```
</details>


Request a randomness

### Proxy - 63

#### proxy - 0

<details><summary><code>proxy(real, force_proxy_type, call)</code></summary>

```rust
real: AccountIdLookupOf<T>
force_proxy_type: Option<T::ProxyType>
call: Box<<T as Config>::RuntimeCall>
```
</details>


Dispatch the given `call` from an account that the sender is authorised for through
`add_proxy`.

Removes any corresponding announcement(s).

The dispatch origin for this call must be _Signed_.

Parameters:
- `real`: The account that the proxy will make a call on behalf of.
- `force_proxy_type`: Specify the exact proxy type to be used and checked for this call.
- `call`: The call to be made by the `real` account.

#### add_proxy - 1

<details><summary><code>add_proxy(delegate, proxy_type, delay)</code></summary>

```rust
delegate: AccountIdLookupOf<T>
proxy_type: T::ProxyType
delay: T::BlockNumber
```
</details>


Register a proxy account for the sender that is able to make calls on its behalf.

The dispatch origin for this call must be _Signed_.

Parameters:
- `proxy`: The account that the `caller` would like to make a proxy.
- `proxy_type`: The permissions allowed for this proxy account.
- `delay`: The announcement period required of the initial proxy. Will generally be
zero.

#### remove_proxy - 2

<details><summary><code>remove_proxy(delegate, proxy_type, delay)</code></summary>

```rust
delegate: AccountIdLookupOf<T>
proxy_type: T::ProxyType
delay: T::BlockNumber
```
</details>


Unregister a proxy account for the sender.

The dispatch origin for this call must be _Signed_.

Parameters:
- `proxy`: The account that the `caller` would like to remove as a proxy.
- `proxy_type`: The permissions currently enabled for the removed proxy account.

#### remove_proxies - 3

<details><summary><code>remove_proxies()</code></summary>

```rust
```
</details>


Unregister all proxy accounts for the sender.

The dispatch origin for this call must be _Signed_.

WARNING: This may be called on accounts created by `pure`, however if done, then
the unreserved fees will be inaccessible. **All access to this account will be lost.**

#### create_pure - 4

<details><summary><code>create_pure(proxy_type, delay, index)</code></summary>

```rust
proxy_type: T::ProxyType
delay: T::BlockNumber
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

#### kill_pure - 5

<details><summary><code>kill_pure(spawner, proxy_type, index, height, ext_index)</code></summary>

```rust
spawner: AccountIdLookupOf<T>
proxy_type: T::ProxyType
index: u16
height: T::BlockNumber
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

#### announce - 6

<details><summary><code>announce(real, call_hash)</code></summary>

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

#### remove_announcement - 7

<details><summary><code>remove_announcement(real, call_hash)</code></summary>

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

#### reject_announcement - 8

<details><summary><code>reject_announcement(delegate, call_hash)</code></summary>

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

#### proxy_announced - 9

<details><summary><code>proxy_announced(delegate, real, force_proxy_type, call)</code></summary>

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

### Utility - 64

#### batch - 0

<details><summary><code>batch(calls)</code></summary>

```rust
calls: Vec<<T as Config>::RuntimeCall>
```
</details>


Send a batch of dispatch calls.

May be called from any origin.

- `calls`: The calls to be dispatched from the same origin. The number of call must not
  exceed the constant: `batched_calls_limit` (available in constant metadata).

If origin is root then call are dispatch without checking origin filter. (This includes
bypassing `frame_system::Config::BaseCallFilter`).


#### as_derivative - 1

<details><summary><code>as_derivative(index, call)</code></summary>

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

#### batch_all - 2

<details><summary><code>batch_all(calls)</code></summary>

```rust
calls: Vec<<T as Config>::RuntimeCall>
```
</details>


Send a batch of dispatch calls and atomically execute them.
The whole transaction will rollback and fail if any of the calls failed.

May be called from any origin.

- `calls`: The calls to be dispatched from the same origin. The number of call must not
  exceed the constant: `batched_calls_limit` (available in constant metadata).

If origin is root then call are dispatch without checking origin filter. (This includes
bypassing `frame_system::Config::BaseCallFilter`).


#### force_batch - 4

<details><summary><code>force_batch(calls)</code></summary>

```rust
calls: Vec<<T as Config>::RuntimeCall>
```
</details>


Send a batch of dispatch calls.
Unlike `batch`, it allows errors and won't interrupt.

May be called from any origin.

- `calls`: The calls to be dispatched from the same origin. The number of call must not
  exceed the constant: `batched_calls_limit` (available in constant metadata).

If origin is root then call are dispatch without checking origin filter. (This includes
bypassing `frame_system::Config::BaseCallFilter`).


### Treasury - 65

#### propose_spend - 0

<details><summary><code>propose_spend(value, beneficiary)</code></summary>

```rust
value: BalanceOf<T, I>
beneficiary: AccountIdLookupOf<T>
```
</details>


Put forward a suggestion for spending. A deposit proportional to the value
is reserved and slashed if the proposal is rejected. It is returned once the
proposal is awarded.


#### spend - 3

<details><summary><code>spend(amount, beneficiary)</code></summary>

```rust
amount: BalanceOf<T, I>
beneficiary: AccountIdLookupOf<T>
```
</details>


Propose and approve a spend of treasury funds.

- `origin`: Must be `SpendOrigin` with the `Success` value being at least `amount`.
- `amount`: The amount to be transferred from the treasury to the `beneficiary`.
- `beneficiary`: The destination account for the transfer.

NOTE: For record-keeping purposes, the proposer is deemed to be equivalent to the
beneficiary.

#### remove_approval - 4

<details><summary><code>remove_approval(proposal_id)</code></summary>

```rust
proposal_id: ProposalIndex
```
</details>


Force a previously approved proposal to be removed from the approval queue.
The original deposit will no longer be returned.

May only be called from `T::RejectOrigin`.
- `proposal_id`: The index of a proposal




## Root calls

There are **26** root calls from **12** pallets.

### System - 0

#### fill_block - 0

<details><summary><code>fill_block(ratio)</code></summary>

```rust
ratio: Perbill
```
</details>


A dispatch that will fill the block weight up to the given ratio.

#### set_heap_pages - 2

<details><summary><code>set_heap_pages(pages)</code></summary>

```rust
pages: u64
```
</details>


Set the number of pages in the WebAssembly environment's heap.

#### set_code - 3

<details><summary><code>set_code(code)</code></summary>

```rust
code: Vec<u8>
```
</details>


Set the new runtime code.


#### set_code_without_checks - 4

<details><summary><code>set_code_without_checks(code)</code></summary>

```rust
code: Vec<u8>
```
</details>


Set the new runtime code without doing any checks of the given `code`.


#### set_storage - 5

<details><summary><code>set_storage(items)</code></summary>

```rust
items: Vec<KeyValue>
```
</details>


Set some items of storage.

#### kill_storage - 6

<details><summary><code>kill_storage(keys)</code></summary>

```rust
keys: Vec<Key>
```
</details>


Kill some items from storage.

#### kill_prefix - 7

<details><summary><code>kill_prefix(prefix, subkeys)</code></summary>

```rust
prefix: Key
subkeys: u32
```
</details>


Kill all storage items with a key that starts with the given prefix.

**NOTE:** We rely on the Root origin to provide us the number of subkeys under
the prefix we are removing to accurately calculate the weight of this function.

### Babe - 3

#### plan_config_change - 2

<details><summary><code>plan_config_change(config)</code></summary>

```rust
config: NextConfigDescriptor
```
</details>


Plan an epoch config change. The epoch config change is recorded and will be enacted on
the next call to `enact_epoch_change`. The config will be activated one epoch after.
Multiple calls to this method will replace any existing planned config change that had
not been enacted yet.

### Balances - 6

#### set_balance - 1

<details><summary><code>set_balance(who, new_free, new_reserved)</code></summary>

```rust
who: AccountIdLookupOf<T>
new_free: T::Balance
new_reserved: T::Balance
```
</details>


Set the balances of a given account.

This will alter `FreeBalance` and `ReservedBalance` in storage. it will
also alter the total issuance of the system (`TotalIssuance`) appropriately.
If the new free or reserved balance is below the existential deposit,
it will reset the account nonce (`frame_system::AccountNonce`).

The dispatch origin for this call is `root`.

#### force_transfer - 2

<details><summary><code>force_transfer(source, dest, value)</code></summary>

```rust
source: AccountIdLookupOf<T>
dest: AccountIdLookupOf<T>
value: T::Balance
```
</details>


Exactly as `transfer`, except the origin must be root and the source account may be
specified.

#### force_unreserve - 5

<details><summary><code>force_unreserve(who, amount)</code></summary>

```rust
who: AccountIdLookupOf<T>
amount: T::Balance
```
</details>


Unreserve some balance from a user by force.

Can only be called by ROOT.

### AuthorityMembers - 10

#### remove_member - 3

<details><summary><code>remove_member(member_id)</code></summary>

```rust
member_id: T::MemberId
```
</details>


remove an identity from the set of authorities

### Grandpa - 15

#### note_stalled - 2

<details><summary><code>note_stalled(delay, best_finalized_block_number)</code></summary>

```rust
delay: T::BlockNumber
best_finalized_block_number: T::BlockNumber
```
</details>


Note that the current authority set of the GRANDPA finality gadget has stalled.

This will trigger a forced authority set change at the beginning of the next session, to
be enacted `delay` blocks after that. The `delay` should be high enough to safely assume
that the block signalling the forced change will not be re-orged e.g. 1000 blocks.
The block production rate (which may be slowed down because of finality lagging) should
be taken into account when choosing the `delay`. The GRANDPA voters based on the new
authority will start voting on top of `best_finalized_block_number` for new finalized
blocks. `best_finalized_block_number` should be the highest of the latest finalized
block of all validators of the new authority set.

Only callable by root.

### TechnicalCommittee - 23

#### set_members - 0

<details><summary><code>set_members(new_members, prime, old_count)</code></summary>

```rust
new_members: Vec<T::AccountId>
prime: Option<T::AccountId>
old_count: MemberCount
```
</details>


Set the collective's membership.

- `new_members`: The new member list. Be nice to the chain and provide it sorted.
- `prime`: The prime member whose vote sets the default.
- `old_count`: The upper bound for the previous number of members in storage. Used for
  weight estimation.

Requires root origin.

NOTE: Does not enforce the expected `MaxMembers` limit on the amount of members, but
      the weight estimations rely on it to estimate dispatchable weight.

WARNING:

The `pallet-collective` can also be managed by logic outside of the pallet through the
implementation of the trait [`ChangeMembers`].
Any call to `set_members` must be careful that the member set doesn't get out of sync
with other logic managing the member set.


#### disapprove_proposal - 5

<details><summary><code>disapprove_proposal(proposal_hash)</code></summary>

```rust
proposal_hash: T::Hash
```
</details>


Disapprove a proposal, close, and remove it from the system, regardless of its current
state.

Must be called by the Root origin.

Parameters:
* `proposal_hash`: The hash of the proposal that should be disapproved.


### Identity - 41

#### remove_identity - 5

<details><summary><code>remove_identity(idty_index, idty_name)</code></summary>

```rust
idty_index: T::IdtyIndex
idty_name: Option<IdtyName>
```
</details>


remove an identity from storage

#### prune_item_identities_names - 6

<details><summary><code>prune_item_identities_names(names)</code></summary>

```rust
names: Vec<IdtyName>
```
</details>


remove identity names from storage

### Membership - 42

#### force_request_membership - 0

<details><summary><code>force_request_membership(idty_id, metadata)</code></summary>

```rust
idty_id: T::IdtyId
metadata: T::MetaData
```
</details>


request membership without checks

### Cert - 43

#### force_add_cert - 0

<details><summary><code>force_add_cert(issuer, receiver, verify_rules)</code></summary>

```rust
issuer: T::IdtyIndex
receiver: T::IdtyIndex
verify_rules: bool
```
</details>


add a certification without checks (only root)

#### del_cert - 2

<details><summary><code>del_cert(issuer, receiver)</code></summary>

```rust
issuer: T::IdtyIndex
receiver: T::IdtyIndex
```
</details>


remove a certification (only root)

#### remove_all_certs_received_by - 3

<details><summary><code>remove_all_certs_received_by(idty_index)</code></summary>

```rust
idty_index: T::IdtyIndex
```
</details>


remove all certifications received by an identity (only root)

### SmithMembership - 52

#### force_request_membership - 0

<details><summary><code>force_request_membership(idty_id, metadata)</code></summary>

```rust
idty_id: T::IdtyId
metadata: T::MetaData
```
</details>


request membership without checks

### SmithCert - 53

#### force_add_cert - 0

<details><summary><code>force_add_cert(issuer, receiver, verify_rules)</code></summary>

```rust
issuer: T::IdtyIndex
receiver: T::IdtyIndex
verify_rules: bool
```
</details>


add a certification without checks (only root)

#### del_cert - 2

<details><summary><code>del_cert(issuer, receiver)</code></summary>

```rust
issuer: T::IdtyIndex
receiver: T::IdtyIndex
```
</details>


remove a certification (only root)

#### remove_all_certs_received_by - 3

<details><summary><code>remove_all_certs_received_by(idty_index)</code></summary>

```rust
idty_index: T::IdtyIndex
```
</details>


remove all certifications received by an identity (only root)

### Utility - 64

#### dispatch_as - 3

<details><summary><code>dispatch_as(as_origin, call)</code></summary>

```rust
as_origin: Box<T::PalletsOrigin>
call: Box<<T as Config>::RuntimeCall>
```
</details>


Dispatches a function call with a provided origin.

The dispatch origin for this call must be _Root_.







## Disabled calls

There are **7** disabled calls from **3** pallets.

### System - 0

#### remark - 1

<details><summary><code>remark(remark)</code></summary>

```rust
remark: Vec<u8>
```
</details>


Make some on-chain remark.


#### remark_with_event - 8

<details><summary><code>remark_with_event(remark)</code></summary>

```rust
remark: Vec<u8>
```
</details>


Make some on-chain remark and emit event.

### Session - 14

#### set_keys - 0

<details><summary><code>set_keys(keys, proof)</code></summary>

```rust
keys: T::Keys
proof: Vec<u8>
```
</details>


Sets the session key(s) of the function caller to `keys`.
Allows an account to set its session key prior to becoming a validator.
This doesn't take effect until the next session.

The dispatch origin of this function must be signed.


#### purge_keys - 1

<details><summary><code>purge_keys()</code></summary>

```rust
```
</details>


Removes any session key(s) of the function caller.

This doesn't take effect until the next session.

The dispatch origin of this function must be Signed and the account must be either be
convertible to a validator ID using the chain's typical addressing system (this usually
means being a controller account) or directly convertible into a validator ID (which
usually means being a stash account).


### Membership - 42

#### request_membership - 1

<details><summary><code>request_membership(metadata)</code></summary>

```rust
metadata: T::MetaData
```
</details>


submit a membership request (must have a declared identity)
(only available for sub wot, automatic for main wot)

#### claim_membership - 2

<details><summary><code>claim_membership(maybe_idty_id)</code></summary>

```rust
maybe_idty_id: Option<T::IdtyId>
```
</details>


claim that the previously requested membership fullfills the requirements
(only available for sub wot, automatic for main wot)

#### revoke_membership - 4

<details><summary><code>revoke_membership(maybe_idty_id)</code></summary>

```rust
maybe_idty_id: Option<T::IdtyId>
```
</details>


revoke an active membership
(only available for sub wot, automatic for main wot)

