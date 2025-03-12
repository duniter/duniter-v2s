# Runtime Storage

There are **139** storages from **35** pallets.

<ul>

<li>System - 0
<ul>

<li>
<details>
<summary>
<code>Account</code>
</summary>
 The full account information for a particular account ID.

```rust
key: sp_core::crypto::AccountId32
value: frame_system::AccountInfo<U32, pallet_duniter_account::types::AccountData<U64, U32>>
```

</details>
</li>

<li>
<details>
<summary>
<code>ExtrinsicCount</code>
</summary>
 Total extrinsics count for the current block.

```rust
value: Option<U32>
```

</details>
</li>

<li>
<details>
<summary>
<code>InherentsApplied</code>
</summary>
 Whether all inherents have been applied.

```rust
value: Bool
```

</details>
</li>

<li>
<details>
<summary>
<code>BlockWeight</code>
</summary>
 The current weight for the block.

```rust
value: frame_support::dispatch::PerDispatchClass<sp_weights::weight_v2::Weight>
```

</details>
</li>

<li>
<details>
<summary>
<code>AllExtrinsicsLen</code>
</summary>
 Total length (in bytes) for all extrinsics put together, for the current block.

```rust
value: Option<U32>
```

</details>
</li>

<li>
<details>
<summary>
<code>BlockHash</code>
</summary>
 Map of block numbers to block hashes.

```rust
key: U32
value: primitive_types::H256
```

</details>
</li>

<li>
<details>
<summary>
<code>ExtrinsicData</code>
</summary>
 Extrinsics data for the current block (maps an extrinsic's index to its data).

```rust
key: U32
value: Vec<U8>
```

</details>
</li>

<li>
<details>
<summary>
<code>Number</code>
</summary>
 The current block number being processed. Set by `execute_block`.

```rust
value: U32
```

</details>
</li>

<li>
<details>
<summary>
<code>ParentHash</code>
</summary>
 Hash of the previous block.

```rust
value: primitive_types::H256
```

</details>
</li>

<li>
<details>
<summary>
<code>Digest</code>
</summary>
 Digest of the current block, also part of the block header.

```rust
value: sp_runtime::generic::digest::Digest
```

</details>
</li>

<li>
<details>
<summary>
<code>Events</code>
</summary>
 Events deposited for the current block.

 NOTE: The item is unbound and should therefore never be read on chain.
 It could otherwise inflate the PoV size of a block.

 Events have a large in-memory size. Box the events to not go out-of-memory
 just in case someone still reads them from within the runtime.

```rust
value: Vec<frame_system::EventRecord<gdev_runtime::RuntimeEvent, primitive_types::H256>>
```

</details>
</li>

<li>
<details>
<summary>
<code>EventCount</code>
</summary>
 The number of events in the `Events<T>` list.

```rust
value: U32
```

</details>
</li>

<li>
<details>
<summary>
<code>EventTopics</code>
</summary>
 Mapping between a topic (represented by T::Hash) and a vector of indexes
 of events in the `<Events<T>>` list.

 All topic vectors have deterministic storage locations depending on the topic. This
 allows light-clients to leverage the changes trie storage tracking mechanism and
 in case of changes fetch the list of events of interest.

 The value has the type `(BlockNumberFor<T>, EventIndex)` because if we used only just
 the `EventIndex` then in case if the topic has the same contents on the next block
 no notification will be triggered thus the event might be lost.

```rust
key: primitive_types::H256
value: Vec<(U32, U32)>
```

</details>
</li>

<li>
<details>
<summary>
<code>LastRuntimeUpgrade</code>
</summary>
 Stores the `spec_version` and `spec_name` of when the last runtime upgrade happened.

```rust
value: Option<frame_system::LastRuntimeUpgradeInfo>
```

</details>
</li>

<li>
<details>
<summary>
<code>UpgradedToU32RefCount</code>
</summary>
 True if we have upgraded so that `type RefCount` is `u32`. False (default) if not.

```rust
value: Bool
```

</details>
</li>

<li>
<details>
<summary>
<code>UpgradedToTripleRefCount</code>
</summary>
 True if we have upgraded so that AccountInfo contains three types of `RefCount`. False
 (default) if not.

```rust
value: Bool
```

</details>
</li>

<li>
<details>
<summary>
<code>ExecutionPhase</code>
</summary>
 The execution phase of the block.

```rust
value: Option<frame_system::Phase>
```

</details>
</li>

<li>
<details>
<summary>
<code>AuthorizedUpgrade</code>
</summary>
 `Some` if a code upgrade has been authorized.

```rust
value: Option<frame_system::CodeUpgradeAuthorization<>>
```

</details>
</li>

</ul>
</li>

<li>Account - 1
<ul>

</ul>
</li>

<li>Scheduler - 2
<ul>

<li>
<details>
<summary>
<code>IncompleteSince</code>
</summary>


```rust
value: Option<U32>
```

</details>
</li>

<li>
<details>
<summary>
<code>Agenda</code>
</summary>
 Items to be executed, indexed by the block number that they should be executed on.

```rust
key: U32
value: bounded_collections::bounded_vec::BoundedVec<Option<pallet_scheduler::Scheduled<[U8; 32], frame_support::traits::preimages::Bounded<gdev_runtime::RuntimeCall, sp_runtime::traits::BlakeTwo256>, U32, gdev_runtime::OriginCaller, sp_core::crypto::AccountId32>>, >
```

</details>
</li>

<li>
<details>
<summary>
<code>Retries</code>
</summary>
 Retry configurations for items to be executed, indexed by task address.

```rust
key: (U32, U32)
value: pallet_scheduler::RetryConfig<U32>
```

</details>
</li>

<li>
<details>
<summary>
<code>Lookup</code>
</summary>
 Lookup from a name to the block number and index of the task.

 For v3 -> v4 the previously unbounded identities are Blake2-256 hashed to form the v4
 identities.

```rust
key: [U8; 32]
value: (U32, U32)
```

</details>
</li>

</ul>
</li>

<li>Babe - 3
<ul>

<li>
<details>
<summary>
<code>EpochIndex</code>
</summary>
 Current epoch index.

```rust
value: U64
```

</details>
</li>

<li>
<details>
<summary>
<code>Authorities</code>
</summary>
 Current epoch authorities.

```rust
value: bounded_collections::weak_bounded_vec::WeakBoundedVec<(sp_consensus_babe::app::Public, U64), >
```

</details>
</li>

<li>
<details>
<summary>
<code>GenesisSlot</code>
</summary>
 The slot at which the first epoch actually started. This is 0
 until the first block of the chain.

```rust
value: sp_consensus_slots::Slot
```

</details>
</li>

<li>
<details>
<summary>
<code>CurrentSlot</code>
</summary>
 Current slot number.

```rust
value: sp_consensus_slots::Slot
```

</details>
</li>

<li>
<details>
<summary>
<code>Randomness</code>
</summary>
 The epoch randomness for the *current* epoch.

 # Security

 This MUST NOT be used for gambling, as it can be influenced by a
 malicious validator in the short term. It MAY be used in many
 cryptographic protocols, however, so long as one remembers that this
 (like everything else on-chain) it is public. For example, it can be
 used where a number is needed that cannot have been chosen by an
 adversary, for purposes such as public-coin zero-knowledge proofs.

```rust
value: [U8; 32]
```

</details>
</li>

<li>
<details>
<summary>
<code>PendingEpochConfigChange</code>
</summary>
 Pending epoch configuration change that will be applied when the next epoch is enacted.

```rust
value: Option<sp_consensus_babe::digests::NextConfigDescriptor>
```

</details>
</li>

<li>
<details>
<summary>
<code>NextRandomness</code>
</summary>
 Next epoch randomness.

```rust
value: [U8; 32]
```

</details>
</li>

<li>
<details>
<summary>
<code>NextAuthorities</code>
</summary>
 Next epoch authorities.

```rust
value: bounded_collections::weak_bounded_vec::WeakBoundedVec<(sp_consensus_babe::app::Public, U64), >
```

</details>
</li>

<li>
<details>
<summary>
<code>SegmentIndex</code>
</summary>
 Randomness under construction.

 We make a trade-off between storage accesses and list length.
 We store the under-construction randomness in segments of up to
 `UNDER_CONSTRUCTION_SEGMENT_LENGTH`.

 Once a segment reaches this length, we begin the next one.
 We reset all segments and return to `0` at the beginning of every
 epoch.

```rust
value: U32
```

</details>
</li>

<li>
<details>
<summary>
<code>UnderConstruction</code>
</summary>
 TWOX-NOTE: `SegmentIndex` is an increasing integer, so this is okay.

```rust
key: U32
value: bounded_collections::bounded_vec::BoundedVec<[U8; 32], >
```

</details>
</li>

<li>
<details>
<summary>
<code>Initialized</code>
</summary>
 Temporary value (cleared at block finalization) which is `Some`
 if per-block initialization has already been called for current block.

```rust
value: Option<Option<sp_consensus_babe::digests::PreDigest>>
```

</details>
</li>

<li>
<details>
<summary>
<code>AuthorVrfRandomness</code>
</summary>
 This field should always be populated during block processing unless
 secondary plain slots are enabled (which don't contain a VRF output).

 It is set in `on_finalize`, before it will contain the value from the last block.

```rust
value: Option<[U8; 32]>
```

</details>
</li>

<li>
<details>
<summary>
<code>EpochStart</code>
</summary>
 The block numbers when the last and current epoch have started, respectively `N-1` and
 `N`.
 NOTE: We track this is in order to annotate the block number when a given pool of
 entropy was fixed (i.e. it was known to chain observers). Since epochs are defined in
 slots, which may be skipped, the block numbers may not line up with the slot numbers.

```rust
value: (U32, U32)
```

</details>
</li>

<li>
<details>
<summary>
<code>Lateness</code>
</summary>
 How late the current block is compared to its parent.

 This entry is populated as part of block execution and is cleaned up
 on block finalization. Querying this storage entry outside of block
 execution context should always yield zero.

```rust
value: U32
```

</details>
</li>

<li>
<details>
<summary>
<code>EpochConfig</code>
</summary>
 The configuration for the current epoch. Should never be `None` as it is initialized in
 genesis.

```rust
value: Option<sp_consensus_babe::BabeEpochConfiguration>
```

</details>
</li>

<li>
<details>
<summary>
<code>NextEpochConfig</code>
</summary>
 The configuration for the next epoch, `None` if the config will not change
 (you can fallback to `EpochConfig` instead in that case).

```rust
value: Option<sp_consensus_babe::BabeEpochConfiguration>
```

</details>
</li>

<li>
<details>
<summary>
<code>SkippedEpochs</code>
</summary>
 A list of the last 100 skipped epochs and the corresponding session index
 when the epoch was skipped.

 This is only used for validating equivocation proofs. An equivocation proof
 must contains a key-ownership proof for a given session, therefore we need a
 way to tie together sessions and epoch indices, i.e. we need to validate that
 a validator was the owner of a given key on a given session, and what the
 active epoch index was during that session.

```rust
value: bounded_collections::bounded_vec::BoundedVec<(U64, U32), >
```

</details>
</li>

</ul>
</li>

<li>Timestamp - 4
<ul>

<li>
<details>
<summary>
<code>Now</code>
</summary>
 The current time for the current block.

```rust
value: U64
```

</details>
</li>

<li>
<details>
<summary>
<code>DidUpdate</code>
</summary>
 Whether the timestamp has been updated in this block.

 This value is updated to `true` upon successful submission of a timestamp by a node.
 It is then checked at the end of each block execution in the `on_finalize` hook.

```rust
value: Bool
```

</details>
</li>

</ul>
</li>

<li>Parameters - 5
<ul>

<li>
<details>
<summary>
<code>ParametersStorage</code>
</summary>


```rust
value: pallet_duniter_test_parameters::types::Parameters<U32, U32, U64, U32>
```

</details>
</li>

</ul>
</li>

<li>Balances - 6
<ul>

<li>
<details>
<summary>
<code>TotalIssuance</code>
</summary>
 The total units issued in the system.

```rust
value: U64
```

</details>
</li>

<li>
<details>
<summary>
<code>InactiveIssuance</code>
</summary>
 The total units of outstanding deactivated balance in the system.

```rust
value: U64
```

</details>
</li>

<li>
<details>
<summary>
<code>Account</code>
</summary>
 The Balances pallet example of storing the balance of an account.

 # Example

 ```nocompile
  impl pallet_balances::Config for Runtime {
    type AccountStore = StorageMapShim<Self::Account<Runtime>, frame_system::Provider<Runtime>, AccountId, Self::AccountData<Balance>>
  }
 ```

 You can also store the balance of an account in the `System` pallet.

 # Example

 ```nocompile
  impl pallet_balances::Config for Runtime {
   type AccountStore = System
  }
 ```

 But this comes with tradeoffs, storing account balances in the system pallet stores
 `frame_system` data alongside the account data contrary to storing account balances in the
 `Balances` pallet, which uses a `StorageMap` to store balances data only.
 NOTE: This is only used in the case that this pallet is used to store balances.

```rust
key: sp_core::crypto::AccountId32
value: pallet_balances::types::AccountData<U64>
```

</details>
</li>

<li>
<details>
<summary>
<code>Locks</code>
</summary>
 Any liquidity locks on some account balances.
 NOTE: Should only be accessed when setting, changing and freeing a lock.

 Use of locks is deprecated in favour of freezes. See `https://github.com/paritytech/substrate/pull/12951/`

```rust
key: sp_core::crypto::AccountId32
value: bounded_collections::weak_bounded_vec::WeakBoundedVec<pallet_balances::types::BalanceLock<U64>, >
```

</details>
</li>

<li>
<details>
<summary>
<code>Reserves</code>
</summary>
 Named reserves on some account balances.

 Use of reserves is deprecated in favour of holds. See `https://github.com/paritytech/substrate/pull/12951/`

```rust
key: sp_core::crypto::AccountId32
value: bounded_collections::bounded_vec::BoundedVec<pallet_balances::types::ReserveData<[U8; 8], U64>, >
```

</details>
</li>

<li>
<details>
<summary>
<code>Holds</code>
</summary>
 Holds on account balances.

```rust
key: sp_core::crypto::AccountId32
value: bounded_collections::bounded_vec::BoundedVec<frame_support::traits::tokens::misc::IdAmount<gdev_runtime::RuntimeHoldReason, U64>, >
```

</details>
</li>

<li>
<details>
<summary>
<code>Freezes</code>
</summary>
 Freeze locks on account balances.

```rust
key: sp_core::crypto::AccountId32
value: bounded_collections::bounded_vec::BoundedVec<frame_support::traits::tokens::misc::IdAmount<(), U64>, >
```

</details>
</li>

</ul>
</li>

<li>TransactionPayment - 32
<ul>

<li>
<details>
<summary>
<code>NextFeeMultiplier</code>
</summary>


```rust
value: sp_arithmetic::fixed_point::FixedU128
```

</details>
</li>

<li>
<details>
<summary>
<code>StorageVersion</code>
</summary>


```rust
value: pallet_transaction_payment::Releases
```

</details>
</li>

</ul>
</li>

<li>OneshotAccount - 7
<ul>

<li>
<details>
<summary>
<code>OneshotAccounts</code>
</summary>
 The balance for each oneshot account.

```rust
key: sp_core::crypto::AccountId32
value: U64
```

</details>
</li>

</ul>
</li>

<li>Quota - 66
<ul>

<li>
<details>
<summary>
<code>IdtyQuota</code>
</summary>
 The quota for each identity.

```rust
key: U32
value: pallet_quota::pallet::Quota<U32, U64>
```

</details>
</li>

<li>
<details>
<summary>
<code>RefundQueue</code>
</summary>
 The fees waiting to be refunded.

```rust
value: bounded_collections::bounded_vec::BoundedVec<pallet_quota::pallet::Refund<sp_core::crypto::AccountId32, U32, U64>, >
```

</details>
</li>

</ul>
</li>

<li>SmithMembers - 10
<ul>

<li>
<details>
<summary>
<code>Smiths</code>
</summary>
 The Smith metadata for each identity.

```rust
key: U32
value: pallet_smith_members::types::SmithMeta<U32>
```

</details>
</li>

<li>
<details>
<summary>
<code>ExpiresOn</code>
</summary>
 The indexes of Smith to remove at a given session.

```rust
key: U32
value: Vec<U32>
```

</details>
</li>

<li>
<details>
<summary>
<code>CurrentSession</code>
</summary>
 The current session index.

```rust
value: U32
```

</details>
</li>

</ul>
</li>

<li>AuthorityMembers - 11
<ul>

<li>
<details>
<summary>
<code>IncomingAuthorities</code>
</summary>
 The incoming authorities.

```rust
value: Vec<U32>
```

</details>
</li>

<li>
<details>
<summary>
<code>OnlineAuthorities</code>
</summary>
 The online authorities.

```rust
value: Vec<U32>
```

</details>
</li>

<li>
<details>
<summary>
<code>OutgoingAuthorities</code>
</summary>
 The outgoing authorities.

```rust
value: Vec<U32>
```

</details>
</li>

<li>
<details>
<summary>
<code>Members</code>
</summary>
 The member data.

```rust
key: U32
value: pallet_authority_members::types::MemberData<sp_core::crypto::AccountId32>
```

</details>
</li>

<li>
<details>
<summary>
<code>Blacklist</code>
</summary>
 The blacklisted authorities.

```rust
value: Vec<U32>
```

</details>
</li>

</ul>
</li>

<li>Authorship - 12
<ul>

<li>
<details>
<summary>
<code>Author</code>
</summary>
 Author of current block.

```rust
value: Option<sp_core::crypto::AccountId32>
```

</details>
</li>

</ul>
</li>

<li>Offences - 13
<ul>

<li>
<details>
<summary>
<code>Reports</code>
</summary>
 The primary structure that holds all offence records keyed by report identifiers.

```rust
key: primitive_types::H256
value: sp_staking::offence::OffenceDetails<sp_core::crypto::AccountId32, (sp_core::crypto::AccountId32, common_runtime::entities::ValidatorFullIdentification)>
```

</details>
</li>

<li>
<details>
<summary>
<code>ConcurrentReportsIndex</code>
</summary>
 A vector of reports of the same kind that happened at the same time slot.

```rust
key: ([U8; 16], Vec<U8>)
value: Vec<primitive_types::H256>
```

</details>
</li>

</ul>
</li>

<li>Historical - 14
<ul>

<li>
<details>
<summary>
<code>HistoricalSessions</code>
</summary>
 Mapping from historical session indices to session-data root hash and validator count.

```rust
key: U32
value: (primitive_types::H256, U32)
```

</details>
</li>

<li>
<details>
<summary>
<code>StoredRange</code>
</summary>
 The range of historical sessions we store. [first, last)

```rust
value: Option<(U32, U32)>
```

</details>
</li>

</ul>
</li>

<li>Session - 15
<ul>

<li>
<details>
<summary>
<code>Validators</code>
</summary>
 The current set of validators.

```rust
value: Vec<sp_core::crypto::AccountId32>
```

</details>
</li>

<li>
<details>
<summary>
<code>CurrentIndex</code>
</summary>
 Current index of the session.

```rust
value: U32
```

</details>
</li>

<li>
<details>
<summary>
<code>QueuedChanged</code>
</summary>
 True if the underlying economic identities or weighting behind the validators
 has changed in the queued validator set.

```rust
value: Bool
```

</details>
</li>

<li>
<details>
<summary>
<code>QueuedKeys</code>
</summary>
 The queued keys for the next session. When the next session begins, these keys
 will be used to determine the validator's session keys.

```rust
value: Vec<(sp_core::crypto::AccountId32, gdev_runtime::opaque::SessionKeys)>
```

</details>
</li>

<li>
<details>
<summary>
<code>DisabledValidators</code>
</summary>
 Indices of disabled validators.

 The vec is always kept sorted so that we can find whether a given validator is
 disabled using binary search. It gets cleared when `on_session_ending` returns
 a new set of identities.

```rust
value: Vec<U32>
```

</details>
</li>

<li>
<details>
<summary>
<code>NextKeys</code>
</summary>
 The next session keys for a validator.

```rust
key: sp_core::crypto::AccountId32
value: gdev_runtime::opaque::SessionKeys
```

</details>
</li>

<li>
<details>
<summary>
<code>KeyOwner</code>
</summary>
 The owner of a key. The key is the `KeyTypeId` + the encoded key.

```rust
key: (sp_core::crypto::KeyTypeId, Vec<U8>)
value: sp_core::crypto::AccountId32
```

</details>
</li>

</ul>
</li>

<li>Grandpa - 16
<ul>

<li>
<details>
<summary>
<code>State</code>
</summary>
 State of the current authority set.

```rust
value: pallet_grandpa::StoredState<U32>
```

</details>
</li>

<li>
<details>
<summary>
<code>PendingChange</code>
</summary>
 Pending change: (signaled at, scheduled change).

```rust
value: Option<pallet_grandpa::StoredPendingChange<U32, >>
```

</details>
</li>

<li>
<details>
<summary>
<code>NextForced</code>
</summary>
 next block number where we can force a change.

```rust
value: Option<U32>
```

</details>
</li>

<li>
<details>
<summary>
<code>Stalled</code>
</summary>
 `true` if we are currently stalled.

```rust
value: Option<(U32, U32)>
```

</details>
</li>

<li>
<details>
<summary>
<code>CurrentSetId</code>
</summary>
 The number of changes (both in terms of keys and underlying economic responsibilities)
 in the "set" of Grandpa validators from genesis.

```rust
value: U64
```

</details>
</li>

<li>
<details>
<summary>
<code>SetIdSession</code>
</summary>
 A mapping from grandpa set ID to the index of the *most recent* session for which its
 members were responsible.

 This is only used for validating equivocation proofs. An equivocation proof must
 contains a key-ownership proof for a given session, therefore we need a way to tie
 together sessions and GRANDPA set ids, i.e. we need to validate that a validator
 was the owner of a given key on a given session, and what the active set ID was
 during that session.

 TWOX-NOTE: `SetId` is not under user control.

```rust
key: U64
value: U32
```

</details>
</li>

<li>
<details>
<summary>
<code>Authorities</code>
</summary>
 The current list of authorities.

```rust
value: bounded_collections::weak_bounded_vec::WeakBoundedVec<(sp_consensus_grandpa::app::Public, U64), >
```

</details>
</li>

</ul>
</li>

<li>ImOnline - 17
<ul>

<li>
<details>
<summary>
<code>HeartbeatAfter</code>
</summary>
 The block number after which it's ok to send heartbeats in the current
 session.

 At the beginning of each session we set this to a value that should fall
 roughly in the middle of the session duration. The idea is to first wait for
 the validators to produce a block in the current session, so that the
 heartbeat later on will not be necessary.

 This value will only be used as a fallback if we fail to get a proper session
 progress estimate from `NextSessionRotation`, as those estimates should be
 more accurate then the value we calculate for `HeartbeatAfter`.

```rust
value: U32
```

</details>
</li>

<li>
<details>
<summary>
<code>Keys</code>
</summary>
 The current set of keys that may issue a heartbeat.

```rust
value: bounded_collections::weak_bounded_vec::WeakBoundedVec<pallet_im_online::sr25519::app_sr25519::Public, >
```

</details>
</li>

<li>
<details>
<summary>
<code>ReceivedHeartbeats</code>
</summary>
 For each session index, we keep a mapping of `SessionIndex` and `AuthIndex`.

```rust
key: (U32, U32)
value: Bool
```

</details>
</li>

<li>
<details>
<summary>
<code>AuthoredBlocks</code>
</summary>
 For each session index, we keep a mapping of `ValidatorId<T>` to the
 number of blocks authored by the given authority.

```rust
key: (U32, sp_core::crypto::AccountId32)
value: U32
```

</details>
</li>

</ul>
</li>

<li>AuthorityDiscovery - 18
<ul>

<li>
<details>
<summary>
<code>Keys</code>
</summary>
 Keys of the current authority set.

```rust
value: bounded_collections::weak_bounded_vec::WeakBoundedVec<sp_authority_discovery::app::Public, >
```

</details>
</li>

<li>
<details>
<summary>
<code>NextKeys</code>
</summary>
 Keys of the next authority set.

```rust
value: bounded_collections::weak_bounded_vec::WeakBoundedVec<sp_authority_discovery::app::Public, >
```

</details>
</li>

</ul>
</li>

<li>Sudo - 20
<ul>

<li>
<details>
<summary>
<code>Key</code>
</summary>
 The `AccountId` of the sudo key.

```rust
value: Option<sp_core::crypto::AccountId32>
```

</details>
</li>

</ul>
</li>

<li>UpgradeOrigin - 21
<ul>

</ul>
</li>

<li>Preimage - 22
<ul>

<li>
<details>
<summary>
<code>StatusFor</code>
</summary>
 The request status of a given hash.

```rust
key: primitive_types::H256
value: pallet_preimage::OldRequestStatus<sp_core::crypto::AccountId32, U64>
```

</details>
</li>

<li>
<details>
<summary>
<code>RequestStatusFor</code>
</summary>
 The request status of a given hash.

```rust
key: primitive_types::H256
value: pallet_preimage::RequestStatus<sp_core::crypto::AccountId32, ()>
```

</details>
</li>

<li>
<details>
<summary>
<code>PreimageFor</code>
</summary>


```rust
key: (primitive_types::H256, U32)
value: bounded_collections::bounded_vec::BoundedVec<U8, >
```

</details>
</li>

</ul>
</li>

<li>TechnicalCommittee - 23
<ul>

<li>
<details>
<summary>
<code>Proposals</code>
</summary>
 The hashes of the active proposals.

```rust
value: bounded_collections::bounded_vec::BoundedVec<primitive_types::H256, >
```

</details>
</li>

<li>
<details>
<summary>
<code>ProposalOf</code>
</summary>
 Actual proposal for a given hash, if it's current.

```rust
key: primitive_types::H256
value: gdev_runtime::RuntimeCall
```

</details>
</li>

<li>
<details>
<summary>
<code>CostOf</code>
</summary>
 Consideration cost created for publishing and storing a proposal.

 Determined by [Config::Consideration] and may be not present for certain proposals (e.g. if
 the proposal count at the time of creation was below threshold N).

```rust
key: primitive_types::H256
value: (sp_core::crypto::AccountId32, ())
```

</details>
</li>

<li>
<details>
<summary>
<code>Voting</code>
</summary>
 Votes on a given proposal, if it is ongoing.

```rust
key: primitive_types::H256
value: pallet_collective::Votes<sp_core::crypto::AccountId32, U32>
```

</details>
</li>

<li>
<details>
<summary>
<code>ProposalCount</code>
</summary>
 Proposals so far.

```rust
value: U32
```

</details>
</li>

<li>
<details>
<summary>
<code>Members</code>
</summary>
 The current members of the collective. This is stored sorted (just by value).

```rust
value: Vec<sp_core::crypto::AccountId32>
```

</details>
</li>

<li>
<details>
<summary>
<code>Prime</code>
</summary>
 The prime member that helps determine the default vote behavior in case of abstentions.

```rust
value: Option<sp_core::crypto::AccountId32>
```

</details>
</li>

</ul>
</li>

<li>UniversalDividend - 30
<ul>

<li>
<details>
<summary>
<code>CurrentUd</code>
</summary>
 The current Universal Dividend value.

```rust
value: U64
```

</details>
</li>

<li>
<details>
<summary>
<code>CurrentUdIndex</code>
</summary>
 The current Universal Dividend index.

```rust
value: U16
```

</details>
</li>

<li>
<details>
<summary>
<code>MonetaryMass</code>
</summary>
 The total quantity of money created by Universal Dividend, excluding potential money destruction.

```rust
value: U64
```

</details>
</li>

<li>
<details>
<summary>
<code>NextReeval</code>
</summary>
 The next Universal Dividend re-evaluation.

```rust
value: Option<U64>
```

</details>
</li>

<li>
<details>
<summary>
<code>NextUd</code>
</summary>
 The next Universal Dividend creation.

```rust
value: Option<U64>
```

</details>
</li>

<li>
<details>
<summary>
<code>PastReevals</code>
</summary>
 The past Universal Dividend re-evaluations.

```rust
value: bounded_collections::bounded_vec::BoundedVec<(U16, U64), >
```

</details>
</li>

</ul>
</li>

<li>Wot - 40
<ul>

</ul>
</li>

<li>Identity - 41
<ul>

<li>
<details>
<summary>
<code>Identities</code>
</summary>
 The identity value for each identity.

```rust
key: U32
value: pallet_identity::types::IdtyValue<U32, sp_core::crypto::AccountId32, common_runtime::entities::IdtyData>
```

</details>
</li>

<li>
<details>
<summary>
<code>CounterForIdentities</code>
</summary>
Counter for the related counted storage map

```rust
value: U32
```

</details>
</li>

<li>
<details>
<summary>
<code>IdentityIndexOf</code>
</summary>
 The identity associated with each account.

```rust
key: sp_core::crypto::AccountId32
value: U32
```

</details>
</li>

<li>
<details>
<summary>
<code>IdentitiesNames</code>
</summary>
 The name associated with each identity.

```rust
key: pallet_identity::types::IdtyName
value: U32
```

</details>
</li>

<li>
<details>
<summary>
<code>NextIdtyIndex</code>
</summary>
 The identity index to assign to the next created identity.

```rust
value: U32
```

</details>
</li>

<li>
<details>
<summary>
<code>IdentityChangeSchedule</code>
</summary>
 The identities to remove at a given block.

```rust
key: U32
value: Vec<U32>
```

</details>
</li>

</ul>
</li>

<li>Membership - 42
<ul>

<li>
<details>
<summary>
<code>Membership</code>
</summary>
 The membership data for each identity.

```rust
key: U32
value: sp_membership::MembershipData<U32>
```

</details>
</li>

<li>
<details>
<summary>
<code>CounterForMembership</code>
</summary>
Counter for the related counted storage map

```rust
value: U32
```

</details>
</li>

<li>
<details>
<summary>
<code>MembershipsExpireOn</code>
</summary>
 The identities of memberships to expire at a given block.

```rust
key: U32
value: Vec<U32>
```

</details>
</li>

</ul>
</li>

<li>Certification - 43
<ul>

<li>
<details>
<summary>
<code>StorageIdtyCertMeta</code>
</summary>
 The certification metadata for each issuer.

```rust
key: U32
value: pallet_certification::types::IdtyCertMeta<U32>
```

</details>
</li>

<li>
<details>
<summary>
<code>CertsByReceiver</code>
</summary>
 The certifications for each receiver.

```rust
key: U32
value: Vec<(U32, U32)>
```

</details>
</li>

<li>
<details>
<summary>
<code>CertsRemovableOn</code>
</summary>
 The certifications that should expire at a given block.

```rust
key: U32
value: Vec<(U32, U32)>
```

</details>
</li>

</ul>
</li>

<li>Distance - 44
<ul>

<li>
<details>
<summary>
<code>EvaluationPool0</code>
</summary>
 The first evaluation pool for distance evaluation queuing identities to evaluate for a given
 evaluator account.

```rust
value: pallet_distance::types::EvaluationPool<sp_core::crypto::AccountId32, U32>
```

</details>
</li>

<li>
<details>
<summary>
<code>EvaluationPool1</code>
</summary>
 The second evaluation pool for distance evaluation queuing identities to evaluate for a given
 evaluator account.

```rust
value: pallet_distance::types::EvaluationPool<sp_core::crypto::AccountId32, U32>
```

</details>
</li>

<li>
<details>
<summary>
<code>EvaluationPool2</code>
</summary>
 The third evaluation pool for distance evaluation queuing identities to evaluate for a given
 evaluator account.

```rust
value: pallet_distance::types::EvaluationPool<sp_core::crypto::AccountId32, U32>
```

</details>
</li>

<li>
<details>
<summary>
<code>EvaluationBlock</code>
</summary>
 The block at which the distance is evaluated.

```rust
value: primitive_types::H256
```

</details>
</li>

<li>
<details>
<summary>
<code>PendingEvaluationRequest</code>
</summary>
 The pending evaluation requesters.

```rust
key: U32
value: sp_core::crypto::AccountId32
```

</details>
</li>

<li>
<details>
<summary>
<code>DidUpdate</code>
</summary>
 Store if the evaluation was updated in this block.

```rust
value: Bool
```

</details>
</li>

<li>
<details>
<summary>
<code>CurrentPeriodIndex</code>
</summary>
 The current evaluation period index.

```rust
value: U32
```

</details>
</li>

</ul>
</li>

<li>AtomicSwap - 50
<ul>

<li>
<details>
<summary>
<code>PendingSwaps</code>
</summary>


```rust
key: (sp_core::crypto::AccountId32, [U8; 32])
value: pallet_atomic_swap::PendingSwap<>
```

</details>
</li>

</ul>
</li>

<li>Multisig - 51
<ul>

<li>
<details>
<summary>
<code>Multisigs</code>
</summary>
 The set of open multisig operations.

```rust
key: (sp_core::crypto::AccountId32, [U8; 32])
value: pallet_multisig::Multisig<U32, U64, sp_core::crypto::AccountId32, >
```

</details>
</li>

</ul>
</li>

<li>ProvideRandomness - 52
<ul>

<li>
<details>
<summary>
<code>NexEpochHookIn</code>
</summary>
 The number of blocks before the next epoch.

```rust
value: U8
```

</details>
</li>

<li>
<details>
<summary>
<code>RequestIdProvider</code>
</summary>
 The request ID.

```rust
value: U64
```

</details>
</li>

<li>
<details>
<summary>
<code>RequestsReadyAtNextBlock</code>
</summary>
 The requests that will be fulfilled at the next block.

```rust
value: Vec<pallet_provide_randomness::types::Request>
```

</details>
</li>

<li>
<details>
<summary>
<code>RequestsReadyAtEpoch</code>
</summary>
 The requests that will be fulfilled at the next epoch.

```rust
key: U64
value: Vec<pallet_provide_randomness::types::Request>
```

</details>
</li>

<li>
<details>
<summary>
<code>RequestsIds</code>
</summary>
 The requests being processed.

```rust
key: U64
value: ()
```

</details>
</li>

<li>
<details>
<summary>
<code>CounterForRequestsIds</code>
</summary>
Counter for the related counted storage map

```rust
value: U32
```

</details>
</li>

</ul>
</li>

<li>Proxy - 53
<ul>

<li>
<details>
<summary>
<code>Proxies</code>
</summary>
 The set of account proxies. Maps the account which has delegated to the accounts
 which are being delegated to, together with the amount held on deposit.

```rust
key: sp_core::crypto::AccountId32
value: (bounded_collections::bounded_vec::BoundedVec<pallet_proxy::ProxyDefinition<sp_core::crypto::AccountId32, gdev_runtime::ProxyType, U32>, >, U64)
```

</details>
</li>

<li>
<details>
<summary>
<code>Announcements</code>
</summary>
 The announcements made by the proxy (key).

```rust
key: sp_core::crypto::AccountId32
value: (bounded_collections::bounded_vec::BoundedVec<pallet_proxy::Announcement<sp_core::crypto::AccountId32, primitive_types::H256, U32>, >, U64)
```

</details>
</li>

</ul>
</li>

<li>Utility - 54
<ul>

</ul>
</li>

<li>Treasury - 55
<ul>

<li>
<details>
<summary>
<code>ProposalCount</code>
</summary>
 DEPRECATED: associated with `spend_local` call and will be removed in May 2025.
 Refer to <https://github.com/paritytech/polkadot-sdk/pull/5961> for migration to `spend`.

 Number of proposals that have been made.

```rust
value: U32
```

</details>
</li>

<li>
<details>
<summary>
<code>Proposals</code>
</summary>
 DEPRECATED: associated with `spend_local` call and will be removed in May 2025.
 Refer to <https://github.com/paritytech/polkadot-sdk/pull/5961> for migration to `spend`.

 Proposals that have been made.

```rust
key: U32
value: pallet_treasury::Proposal<sp_core::crypto::AccountId32, U64>
```

</details>
</li>

<li>
<details>
<summary>
<code>Deactivated</code>
</summary>
 The amount which has been reported as inactive to Currency.

```rust
value: U64
```

</details>
</li>

<li>
<details>
<summary>
<code>Approvals</code>
</summary>
 DEPRECATED: associated with `spend_local` call and will be removed in May 2025.
 Refer to <https://github.com/paritytech/polkadot-sdk/pull/5961> for migration to `spend`.

 Proposal indices that have been approved but not yet awarded.

```rust
value: bounded_collections::bounded_vec::BoundedVec<U32, >
```

</details>
</li>

<li>
<details>
<summary>
<code>SpendCount</code>
</summary>
 The count of spends that have been made.

```rust
value: U32
```

</details>
</li>

<li>
<details>
<summary>
<code>Spends</code>
</summary>
 Spends that have been approved and being processed.

```rust
key: U32
value: pallet_treasury::SpendStatus<(), U64, sp_core::crypto::AccountId32, U32, ()>
```

</details>
</li>

<li>
<details>
<summary>
<code>LastSpendPeriod</code>
</summary>
 The blocknumber for the last triggered spend period.

```rust
value: Option<U32>
```

</details>
</li>

</ul>
</li>

</ul>
