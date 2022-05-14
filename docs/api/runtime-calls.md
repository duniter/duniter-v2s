# Runtime calls

Calls are categorized according to the dispatch origin they require:

1. User calls: the dispatch origin for this kind of call must be Signed by
the transactor. This is the only call category that can be submitted with an extrinsic.
1. Root calls: This kind of call requires a special origin that can only be invoked
through on-chain governance mechanisms.
1. Inherent calls: This kind of call is invoked by the author of the block itself
(usually automatically by the node).


## User calls

There are **53** user calls organized in **18** pallets.

### 2: Scheduler

<details><summary>0: schedule(when, maybe_periodic, priority, call)</summary>
<p>

### Index

`0`

### Documentation

Anonymously schedule a task.

### Types of parameters

```rust
when: T::BlockNumber,
maybe_periodic: Option<schedule::Period<T::BlockNumber>>,
priority: schedule::Priority,
call: Box<CallOrHashOf<T>>
```

</p>
</details>

<details><summary>1: cancel(when, index)</summary>
<p>

### Index

`1`

### Documentation

Cancel an anonymously scheduled task.

### Types of parameters

```rust
when: T::BlockNumber,
index: u32
```

</p>
</details>

<details><summary>2: schedule_named(id, when, maybe_periodic, priority, call)</summary>
<p>

### Index

`2`

### Documentation

Schedule a named task.

### Types of parameters

```rust
id: Vec<u8>,
when: T::BlockNumber,
maybe_periodic: Option<schedule::Period<T::BlockNumber>>,
priority: schedule::Priority,
call: Box<CallOrHashOf<T>>
```

</p>
</details>

<details><summary>3: cancel_named(id)</summary>
<p>

### Index

`3`

### Documentation

Cancel a named scheduled task.

### Types of parameters

```rust
id: Vec<u8>
```

</p>
</details>

<details><summary>4: schedule_after(after, maybe_periodic, priority, call)</summary>
<p>

### Index

`4`

### Documentation

Anonymously schedule a task after a delay.


### Types of parameters

```rust
after: T::BlockNumber,
maybe_periodic: Option<schedule::Period<T::BlockNumber>>,
priority: schedule::Priority,
call: Box<CallOrHashOf<T>>
```

</p>
</details>

<details><summary>5: schedule_named_after(id, after, maybe_periodic, priority, call)</summary>
<p>

### Index

`5`

### Documentation

Schedule a named task after a delay.


### Types of parameters

```rust
id: Vec<u8>,
after: T::BlockNumber,
maybe_periodic: Option<schedule::Period<T::BlockNumber>>,
priority: schedule::Priority,
call: Box<CallOrHashOf<T>>
```

</p>
</details>


### 3: Babe

<details><summary>0: report_equivocation(equivocation_proof, key_owner_proof)</summary>
<p>

### Index

`0`

### Documentation

Report authority equivocation/misbehavior. This method will verify
the equivocation proof and validate the given key ownership proof
against the extracted offender. If both are valid, the offence will
be reported.

### Types of parameters

```rust
equivocation_proof: Box<EquivocationProof<T::Header>>,
key_owner_proof: T::KeyOwnerProof
```

</p>
</details>


### 6: Balances

<details><summary>0: transfer(dest, value)</summary>
<p>

### Index

`0`

### Documentation

Transfer some liquid free balance to another account.

`transfer` will set the `FreeBalance` of the sender and receiver.
If the sender's account is below the existential deposit as a result
of the transfer, the account will be reaped.

The dispatch origin for this call must be `Signed` by the transactor.


### Types of parameters

```rust
dest: <T::Lookup as StaticLookup>::Source,
value: T::Balance
```

</p>
</details>

<details><summary>3: transfer_keep_alive(dest, value)</summary>
<p>

### Index

`3`

### Documentation

Same as the [`transfer`] call, but with a check that the transfer will not kill the
origin account.

99% of the time you want [`transfer`] instead.

[`transfer`]: struct.Pallet.html#method.transfer

### Types of parameters

```rust
dest: <T::Lookup as StaticLookup>::Source,
value: T::Balance
```

</p>
</details>

<details><summary>4: transfer_all(dest, keep_alive)</summary>
<p>

### Index

`4`

### Documentation

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

### Types of parameters

```rust
dest: <T::Lookup as StaticLookup>::Source,
keep_alive: bool
```

</p>
</details>


### 10: AuthorityMembers

<details><summary>0: go_offline()</summary>
<p>

### Index

`0`

### Documentation



</p>
</details>

<details><summary>1: go_online()</summary>
<p>

### Index

`1`

### Documentation



</p>
</details>

<details><summary>2: set_session_keys(keys)</summary>
<p>

### Index

`2`

### Documentation



### Types of parameters

```rust
keys: T::KeysWrapper
```

</p>
</details>


### 15: Grandpa

<details><summary>0: report_equivocation(equivocation_proof, key_owner_proof)</summary>
<p>

### Index

`0`

### Documentation

Report voter equivocation/misbehavior. This method will verify the
equivocation proof and validate the given key ownership proof
against the extracted offender. If both are valid, the offence
will be reported.

### Types of parameters

```rust
equivocation_proof: Box<EquivocationProof<T::Hash, T::BlockNumber>>,
key_owner_proof: T::KeyOwnerProof
```

</p>
</details>


### 31: UniversalDividend

<details><summary>0: transfer_ud(dest, value)</summary>
<p>

### Index

`0`

### Documentation

Transfer some liquid free balance to another account, in milliUD.

### Types of parameters

```rust
dest: <T::Lookup as StaticLookup>::Source,
value: BalanceOf<T>
```

</p>
</details>

<details><summary>1: transfer_ud_keep_alive(dest, value)</summary>
<p>

### Index

`1`

### Documentation

Transfer some liquid free balance to another account, in milliUD.

### Types of parameters

```rust
dest: <T::Lookup as StaticLookup>::Source,
value: BalanceOf<T>
```

</p>
</details>


### 41: Identity

<details><summary>0: create_identity(owner_key)</summary>
<p>

### Index

`0`

### Documentation



### Types of parameters

```rust
owner_key: T::AccountId
```

</p>
</details>

<details><summary>1: confirm_identity(idty_name)</summary>
<p>

### Index

`1`

### Documentation



### Types of parameters

```rust
idty_name: IdtyName
```

</p>
</details>

<details><summary>2: validate_identity(idty_index)</summary>
<p>

### Index

`2`

### Documentation



### Types of parameters

```rust
idty_index: T::IdtyIndex
```

</p>
</details>

<details><summary>3: revoke_identity(payload, payload_sig)</summary>
<p>

### Index

`3`

### Documentation



### Types of parameters

```rust
payload: RevocationPayload<T::AccountId, T::Hash>,
payload_sig: T::RevocationSignature
```

</p>
</details>


### 42: Membership

<details><summary>1: request_membership(metadata)</summary>
<p>

### Index

`1`

### Documentation



### Types of parameters

```rust
metadata: T::MetaData
```

</p>
</details>

<details><summary>3: renew_membership(maybe_idty_id)</summary>
<p>

### Index

`3`

### Documentation



### Types of parameters

```rust
maybe_idty_id: Option<T::IdtyId>
```

</p>
</details>


### 43: Cert

<details><summary>1: add_cert(receiver)</summary>
<p>

### Index

`1`

### Documentation



### Types of parameters

```rust
receiver: T::AccountId
```

</p>
</details>


### 52: SmithsMembership

<details><summary>1: request_membership(metadata)</summary>
<p>

### Index

`1`

### Documentation



### Types of parameters

```rust
metadata: T::MetaData
```

</p>
</details>

<details><summary>3: renew_membership(maybe_idty_id)</summary>
<p>

### Index

`3`

### Documentation



### Types of parameters

```rust
maybe_idty_id: Option<T::IdtyId>
```

</p>
</details>

<details><summary>4: revoke_membership(maybe_idty_id)</summary>
<p>

### Index

`4`

### Documentation



### Types of parameters

```rust
maybe_idty_id: Option<T::IdtyId>
```

</p>
</details>


### 53: SmithsCert

<details><summary>1: add_cert(receiver)</summary>
<p>

### Index

`1`

### Documentation



### Types of parameters

```rust
receiver: T::AccountId
```

</p>
</details>


### 54: SmithsCollective

<details><summary>1: execute(proposal, length_bound)</summary>
<p>

### Index

`1`

### Documentation

Dispatch a proposal from a member using the `Member` origin.

Origin must be a member of the collective.


### Types of parameters

```rust
proposal: Box<<T as Config<I>>::Proposal>,
length_bound: u32
```

</p>
</details>

<details><summary>2: propose(threshold, proposal, length_bound)</summary>
<p>

### Index

`2`

### Documentation

Add a new proposal to either be voted on or executed directly.

Requires the sender to be member.

`threshold` determines whether `proposal` is executed directly (`threshold < 2`)
or put up for voting.


### Types of parameters

```rust
threshold: MemberCount,
proposal: Box<<T as Config<I>>::Proposal>,
length_bound: u32
```

</p>
</details>

<details><summary>3: vote(proposal, index, approve)</summary>
<p>

### Index

`3`

### Documentation

Add an aye or nay vote for the sender to the given proposal.

Requires the sender to be a member.

Transaction fees will be waived if the member is voting on any particular proposal
for the first time and the call is successful. Subsequent vote changes will charge a
fee.

### Types of parameters

```rust
proposal: T::Hash,
index: ProposalIndex,
approve: bool
```

</p>
</details>

<details><summary>4: close(proposal_hash, index, proposal_weight_bound, length_bound)</summary>
<p>

### Index

`4`

### Documentation

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


### Types of parameters

```rust
proposal_hash: T::Hash,
index: ProposalIndex,
proposal_weight_bound: Weight,
length_bound: u32
```

</p>
</details>


### 60: AtomicSwap

<details><summary>0: create_swap(target, hashed_proof, action, duration)</summary>
<p>

### Index

`0`

### Documentation

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

### Types of parameters

```rust
target: T::AccountId,
hashed_proof: HashedProof,
action: T::SwapAction,
duration: T::BlockNumber
```

</p>
</details>

<details><summary>1: claim_swap(proof, action)</summary>
<p>

### Index

`1`

### Documentation

Claim an atomic swap.

The dispatch origin for this call must be _Signed_.

- `proof`: Revealed proof of the claim.
- `action`: Action defined in the swap, it must match the entry in blockchain. Otherwise
  the operation fails. This is used for weight calculation.

### Types of parameters

```rust
proof: Vec<u8>,
action: T::SwapAction
```

</p>
</details>

<details><summary>2: cancel_swap(target, hashed_proof)</summary>
<p>

### Index

`2`

### Documentation

Cancel an atomic swap. Only possible after the originally set duration has passed.

The dispatch origin for this call must be _Signed_.

- `target`: Target of the original atomic swap.
- `hashed_proof`: Hashed proof of the original atomic swap.

### Types of parameters

```rust
target: T::AccountId,
hashed_proof: HashedProof
```

</p>
</details>


### 61: Multisig

<details><summary>0: as_multi_threshold_1(other_signatories, call)</summary>
<p>

### Index

`0`

### Documentation

Immediately dispatch a multi-signature call using a single approval from the caller.

The dispatch origin for this call must be _Signed_.

- `other_signatories`: The accounts (other than the sender) who are part of the
multi-signature, but do not participate in the approval process.
- `call`: The call to be executed.

Result is equivalent to the dispatched result.


### Types of parameters

```rust
other_signatories: Vec<T::AccountId>,
call: Box<<T as Config>::Call>
```

</p>
</details>

<details><summary>1: as_multi(threshold, other_signatories, maybe_timepoint, call, store_call, max_weight)</summary>
<p>

### Index

`1`

### Documentation

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


### Types of parameters

```rust
threshold: u16,
other_signatories: Vec<T::AccountId>,
maybe_timepoint: Option<Timepoint<T::BlockNumber>>,
call: OpaqueCall<T>,
store_call: bool,
max_weight: Weight
```

</p>
</details>

<details><summary>2: approve_as_multi(threshold, other_signatories, maybe_timepoint, call_hash, max_weight)</summary>
<p>

### Index

`2`

### Documentation

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


### Types of parameters

```rust
threshold: u16,
other_signatories: Vec<T::AccountId>,
maybe_timepoint: Option<Timepoint<T::BlockNumber>>,
call_hash: [u8; 32],
max_weight: Weight
```

</p>
</details>

<details><summary>3: cancel_as_multi(threshold, other_signatories, timepoint, call_hash)</summary>
<p>

### Index

`3`

### Documentation

Cancel a pre-existing, on-going multisig transaction. Any deposit reserved previously
for this operation will be unreserved on success.

The dispatch origin for this call must be _Signed_.

- `threshold`: The total number of approvals for this dispatch before it is executed.
- `other_signatories`: The accounts (other than the sender) who can approve this
dispatch. May not be empty.
- `timepoint`: The timepoint (block number and transaction index) of the first approval
transaction for this dispatch.
- `call_hash`: The hash of the call to be executed.


### Types of parameters

```rust
threshold: u16,
other_signatories: Vec<T::AccountId>,
timepoint: Timepoint<T::BlockNumber>,
call_hash: [u8; 32]
```

</p>
</details>


### 62: ProvideRandomness

<details><summary>0: request(randomness_type, salt)</summary>
<p>

### Index

`0`

### Documentation

Request a randomness

### Types of parameters

```rust
randomness_type: RandomnessType,
salt: H256
```

</p>
</details>


### 63: Proxy

<details><summary>0: proxy(real, force_proxy_type, call)</summary>
<p>

### Index

`0`

### Documentation

Dispatch the given `call` from an account that the sender is authorised for through
`add_proxy`.

Removes any corresponding announcement(s).

The dispatch origin for this call must be _Signed_.

Parameters:
- `real`: The account that the proxy will make a call on behalf of.
- `force_proxy_type`: Specify the exact proxy type to be used and checked for this call.
- `call`: The call to be made by the `real` account.


### Types of parameters

```rust
real: T::AccountId,
force_proxy_type: Option<T::ProxyType>,
call: Box<<T as Config>::Call>
```

</p>
</details>

<details><summary>1: add_proxy(delegate, proxy_type, delay)</summary>
<p>

### Index

`1`

### Documentation

Register a proxy account for the sender that is able to make calls on its behalf.

The dispatch origin for this call must be _Signed_.

Parameters:
- `proxy`: The account that the `caller` would like to make a proxy.
- `proxy_type`: The permissions allowed for this proxy account.
- `delay`: The announcement period required of the initial proxy. Will generally be
zero.


### Types of parameters

```rust
delegate: T::AccountId,
proxy_type: T::ProxyType,
delay: T::BlockNumber
```

</p>
</details>

<details><summary>2: remove_proxy(delegate, proxy_type, delay)</summary>
<p>

### Index

`2`

### Documentation

Unregister a proxy account for the sender.

The dispatch origin for this call must be _Signed_.

Parameters:
- `proxy`: The account that the `caller` would like to remove as a proxy.
- `proxy_type`: The permissions currently enabled for the removed proxy account.


### Types of parameters

```rust
delegate: T::AccountId,
proxy_type: T::ProxyType,
delay: T::BlockNumber
```

</p>
</details>

<details><summary>3: remove_proxies()</summary>
<p>

### Index

`3`

### Documentation

Unregister all proxy accounts for the sender.

The dispatch origin for this call must be _Signed_.

WARNING: This may be called on accounts created by `anonymous`, however if done, then
the unreserved fees will be inaccessible. **All access to this account will be lost.**


</p>
</details>

<details><summary>4: anonymous(proxy_type, delay, index)</summary>
<p>

### Index

`4`

### Documentation

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


### Types of parameters

```rust
proxy_type: T::ProxyType,
delay: T::BlockNumber,
index: u16
```

</p>
</details>

<details><summary>5: kill_anonymous(spawner, proxy_type, index, height, ext_index)</summary>
<p>

### Index

`5`

### Documentation

Removes a previously spawned anonymous proxy.

WARNING: **All access to this account will be lost.** Any funds held in it will be
inaccessible.

Requires a `Signed` origin, and the sender account must have been created by a call to
`anonymous` with corresponding parameters.

- `spawner`: The account that originally called `anonymous` to create this account.
- `index`: The disambiguation index originally passed to `anonymous`. Probably `0`.
- `proxy_type`: The proxy type originally passed to `anonymous`.
- `height`: The height of the chain when the call to `anonymous` was processed.
- `ext_index`: The extrinsic index in which the call to `anonymous` was processed.

Fails with `NoPermission` in case the caller is not a previously created anonymous
account whose `anonymous` call has corresponding parameters.


### Types of parameters

```rust
spawner: T::AccountId,
proxy_type: T::ProxyType,
index: u16,
height: T::BlockNumber,
ext_index: u32
```

</p>
</details>

<details><summary>6: announce(real, call_hash)</summary>
<p>

### Index

`6`

### Documentation

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


### Types of parameters

```rust
real: T::AccountId,
call_hash: CallHashOf<T>
```

</p>
</details>

<details><summary>7: remove_announcement(real, call_hash)</summary>
<p>

### Index

`7`

### Documentation

Remove a given announcement.

May be called by a proxy account to remove a call they previously announced and return
the deposit.

The dispatch origin for this call must be _Signed_.

Parameters:
- `real`: The account that the proxy will make a call on behalf of.
- `call_hash`: The hash of the call to be made by the `real` account.


### Types of parameters

```rust
real: T::AccountId,
call_hash: CallHashOf<T>
```

</p>
</details>

<details><summary>8: reject_announcement(delegate, call_hash)</summary>
<p>

### Index

`8`

### Documentation

Remove the given announcement of a delegate.

May be called by a target (proxied) account to remove a call that one of their delegates
(`delegate`) has announced they want to execute. The deposit is returned.

The dispatch origin for this call must be _Signed_.

Parameters:
- `delegate`: The account that previously announced the call.
- `call_hash`: The hash of the call to be made.


### Types of parameters

```rust
delegate: T::AccountId,
call_hash: CallHashOf<T>
```

</p>
</details>

<details><summary>9: proxy_announced(delegate, real, force_proxy_type, call)</summary>
<p>

### Index

`9`

### Documentation

Dispatch the given `call` from an account that the sender is authorized for through
`add_proxy`.

Removes any corresponding announcement(s).

The dispatch origin for this call must be _Signed_.

Parameters:
- `real`: The account that the proxy will make a call on behalf of.
- `force_proxy_type`: Specify the exact proxy type to be used and checked for this call.
- `call`: The call to be made by the `real` account.


### Types of parameters

```rust
delegate: T::AccountId,
real: T::AccountId,
force_proxy_type: Option<T::ProxyType>,
call: Box<<T as Config>::Call>
```

</p>
</details>


### 64: Utility

<details><summary>0: batch(calls)</summary>
<p>

### Index

`0`

### Documentation

Send a batch of dispatch calls.

May be called from any origin.

- `calls`: The calls to be dispatched from the same origin. The number of call must not
  exceed the constant: `batched_calls_limit` (available in constant metadata).

If origin is root then call are dispatch without checking origin filter. (This includes
bypassing `frame_system::Config::BaseCallFilter`).


### Types of parameters

```rust
calls: Vec<<T as Config>::Call>
```

</p>
</details>

<details><summary>1: as_derivative(index, call)</summary>
<p>

### Index

`1`

### Documentation

Send a call through an indexed pseudonym of the sender.

Filter from origin are passed along. The call will be dispatched with an origin which
use the same filter as the origin of this call.

NOTE: If you need to ensure that any account-based filtering is not honored (i.e.
because you expect `proxy` to have been used prior in the call stack and you do not want
the call restrictions to apply to any sub-accounts), then use `as_multi_threshold_1`
in the Multisig pallet instead.

NOTE: Prior to version *12, this was called `as_limited_sub`.

The dispatch origin for this call must be _Signed_.

### Types of parameters

```rust
index: u16,
call: Box<<T as Config>::Call>
```

</p>
</details>

<details><summary>2: batch_all(calls)</summary>
<p>

### Index

`2`

### Documentation

Send a batch of dispatch calls and atomically execute them.
The whole transaction will rollback and fail if any of the calls failed.

May be called from any origin.

- `calls`: The calls to be dispatched from the same origin. The number of call must not
  exceed the constant: `batched_calls_limit` (available in constant metadata).

If origin is root then call are dispatch without checking origin filter. (This includes
bypassing `frame_system::Config::BaseCallFilter`).


### Types of parameters

```rust
calls: Vec<<T as Config>::Call>
```

</p>
</details>


### 65: Treasury

<details><summary>0: propose_spend(value, beneficiary)</summary>
<p>

### Index

`0`

### Documentation

Put forward a suggestion for spending. A deposit proportional to the value
is reserved and slashed if the proposal is rejected. It is returned once the
proposal is awarded.


### Types of parameters

```rust
value: BalanceOf<T, I>,
beneficiary: <T::Lookup as StaticLookup>::Source
```

</p>
</details>



## Root calls

There are **28** root calls organized in **12** pallets.

### 0: System

<details><summary>0: fill_block(ratio)</summary>
<p>

### Index

`0`

### Documentation

A dispatch that will fill the block weight up to the given ratio.

### Types of parameters

```rust
ratio: Perbill
```

</p>
</details>

<details><summary>2: set_heap_pages(pages)</summary>
<p>

### Index

`2`

### Documentation

Set the number of pages in the WebAssembly environment's heap.

### Types of parameters

```rust
pages: u64
```

</p>
</details>

<details><summary>3: set_code(code)</summary>
<p>

### Index

`3`

### Documentation

Set the new runtime code.


### Types of parameters

```rust
code: Vec<u8>
```

</p>
</details>

<details><summary>4: set_code_without_checks(code)</summary>
<p>

### Index

`4`

### Documentation

Set the new runtime code without doing any checks of the given `code`.


### Types of parameters

```rust
code: Vec<u8>
```

</p>
</details>

<details><summary>5: set_storage(items)</summary>
<p>

### Index

`5`

### Documentation

Set some items of storage.

### Types of parameters

```rust
items: Vec<KeyValue>
```

</p>
</details>

<details><summary>6: kill_storage(keys)</summary>
<p>

### Index

`6`

### Documentation

Kill some items from storage.

### Types of parameters

```rust
keys: Vec<Key>
```

</p>
</details>

<details><summary>7: kill_prefix(prefix, subkeys)</summary>
<p>

### Index

`7`

### Documentation

Kill all storage items with a key that starts with the given prefix.

**NOTE:** We rely on the Root origin to provide us the number of subkeys under
the prefix we are removing to accurately calculate the weight of this function.

### Types of parameters

```rust
prefix: Key,
subkeys: u32
```

</p>
</details>


### 3: Babe

<details><summary>2: plan_config_change(config)</summary>
<p>

### Index

`2`

### Documentation

Plan an epoch config change. The epoch config change is recorded and will be enacted on
the next call to `enact_epoch_change`. The config will be activated one epoch after.
Multiple calls to this method will replace any existing planned config change that had
not been enacted yet.

### Types of parameters

```rust
config: NextConfigDescriptor
```

</p>
</details>


### 6: Balances

<details><summary>1: set_balance(who, new_free, new_reserved)</summary>
<p>

### Index

`1`

### Documentation

Set the balances of a given account.

This will alter `FreeBalance` and `ReservedBalance` in storage. it will
also alter the total issuance of the system (`TotalIssuance`) appropriately.
If the new free or reserved balance is below the existential deposit,
it will reset the account nonce (`frame_system::AccountNonce`).

The dispatch origin for this call is `root`.

### Types of parameters

```rust
who: <T::Lookup as StaticLookup>::Source,
new_free: T::Balance,
new_reserved: T::Balance
```

</p>
</details>

<details><summary>2: force_transfer(source, dest, value)</summary>
<p>

### Index

`2`

### Documentation

Exactly as `transfer`, except the origin must be root and the source account may be
specified.

### Types of parameters

```rust
source: <T::Lookup as StaticLookup>::Source,
dest: <T::Lookup as StaticLookup>::Source,
value: T::Balance
```

</p>
</details>

<details><summary>5: force_unreserve(who, amount)</summary>
<p>

### Index

`5`

### Documentation

Unreserve some balance from a user by force.

Can only be called by ROOT.

### Types of parameters

```rust
who: <T::Lookup as StaticLookup>::Source,
amount: T::Balance
```

</p>
</details>


### 10: AuthorityMembers

<details><summary>3: prune_account_id_of(members_ids)</summary>
<p>

### Index

`3`

### Documentation



### Types of parameters

```rust
members_ids: Vec<T::MemberId>
```

</p>
</details>

<details><summary>4: remove_member(member_id)</summary>
<p>

### Index

`4`

### Documentation



### Types of parameters

```rust
member_id: T::MemberId
```

</p>
</details>


### 15: Grandpa

<details><summary>2: note_stalled(delay, best_finalized_block_number)</summary>
<p>

### Index

`2`

### Documentation

Note that the current authority set of the GRANDPA finality gadget has
stalled. This will trigger a forced authority set change at the beginning
of the next session, to be enacted `delay` blocks after that. The delay
should be high enough to safely assume that the block signalling the
forced change will not be re-orged (e.g. 1000 blocks). The GRANDPA voters
will start the new authority set using the given finalized block as base.
Only callable by root.

### Types of parameters

```rust
delay: T::BlockNumber,
best_finalized_block_number: T::BlockNumber
```

</p>
</details>


### 41: Identity

<details><summary>4: remove_identity(idty_index, idty_name)</summary>
<p>

### Index

`4`

### Documentation



### Types of parameters

```rust
idty_index: T::IdtyIndex,
idty_name: Option<IdtyName>
```

</p>
</details>

<details><summary>5: prune_item_identities_names(names)</summary>
<p>

### Index

`5`

### Documentation



### Types of parameters

```rust
names: Vec<IdtyName>
```

</p>
</details>

<details><summary>6: prune_item_identity_index_of(accounts_ids)</summary>
<p>

### Index

`6`

### Documentation



### Types of parameters

```rust
accounts_ids: Vec<T::AccountId>
```

</p>
</details>


### 42: Membership

<details><summary>0: force_request_membership(idty_id, metadata)</summary>
<p>

### Index

`0`

### Documentation



### Types of parameters

```rust
idty_id: T::IdtyId,
metadata: T::MetaData
```

</p>
</details>


### 43: Cert

<details><summary>0: force_add_cert(issuer, receiver, verify_rules)</summary>
<p>

### Index

`0`

### Documentation



### Types of parameters

```rust
issuer: T::IdtyIndex,
receiver: T::IdtyIndex,
verify_rules: bool
```

</p>
</details>

<details><summary>2: del_cert(issuer, receiver)</summary>
<p>

### Index

`2`

### Documentation



### Types of parameters

```rust
issuer: T::IdtyIndex,
receiver: T::IdtyIndex
```

</p>
</details>

<details><summary>3: remove_all_certs_received_by(idty_index)</summary>
<p>

### Index

`3`

### Documentation



### Types of parameters

```rust
idty_index: T::IdtyIndex
```

</p>
</details>


### 52: SmithsMembership

<details><summary>0: force_request_membership(idty_id, metadata)</summary>
<p>

### Index

`0`

### Documentation



### Types of parameters

```rust
idty_id: T::IdtyId,
metadata: T::MetaData
```

</p>
</details>


### 53: SmithsCert

<details><summary>0: force_add_cert(issuer, receiver, verify_rules)</summary>
<p>

### Index

`0`

### Documentation



### Types of parameters

```rust
issuer: T::IdtyIndex,
receiver: T::IdtyIndex,
verify_rules: bool
```

</p>
</details>

<details><summary>2: del_cert(issuer, receiver)</summary>
<p>

### Index

`2`

### Documentation



### Types of parameters

```rust
issuer: T::IdtyIndex,
receiver: T::IdtyIndex
```

</p>
</details>

<details><summary>3: remove_all_certs_received_by(idty_index)</summary>
<p>

### Index

`3`

### Documentation



### Types of parameters

```rust
idty_index: T::IdtyIndex
```

</p>
</details>


### 54: SmithsCollective

<details><summary>0: set_members(new_members, prime, old_count)</summary>
<p>

### Index

`0`

### Documentation

Set the collective's membership.

- `new_members`: The new member list. Be nice to the chain and provide it sorted.
- `prime`: The prime member whose vote sets the default.
- `old_count`: The upper bound for the previous number of members in storage. Used for
  weight estimation.

Requires root origin.

NOTE: Does not enforce the expected `MaxMembers` limit on the amount of members, but
      the weight estimations rely on it to estimate dispatchable weight.

# WARNING:

The `pallet-collective` can also be managed by logic outside of the pallet through the
implementation of the trait [`ChangeMembers`].
Any call to `set_members` must be careful that the member set doesn't get out of sync
with other logic managing the member set.


### Types of parameters

```rust
new_members: Vec<T::AccountId>,
prime: Option<T::AccountId>,
old_count: MemberCount
```

</p>
</details>

<details><summary>5: disapprove_proposal(proposal_hash)</summary>
<p>

### Index

`5`

### Documentation

Disapprove a proposal, close, and remove it from the system, regardless of its current
state.

Must be called by the Root origin.

Parameters:
* `proposal_hash`: The hash of the proposal that should be disapproved.


### Types of parameters

```rust
proposal_hash: T::Hash
```

</p>
</details>


### 64: Utility

<details><summary>3: dispatch_as(as_origin, call)</summary>
<p>

### Index

`3`

### Documentation

Dispatches a function call with a provided origin.

The dispatch origin for this call must be _Root_.


### Types of parameters

```rust
as_origin: Box<T::PalletsOrigin>,
call: Box<<T as Config>::Call>
```

</p>
</details>

