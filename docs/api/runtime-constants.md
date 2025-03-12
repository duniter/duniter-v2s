# Runtime Constant

There are **69** constants from **35** pallets.

<ul>

<li>System - 0
<ul>

<li>
<details>
<summary>
<code>BlockWeights</code>
</summary>
 Block & extrinsics weights: base values and limits.

```rust
value: frame_system::limits::BlockWeights({ base_block: { ref_time: 431614000, proof_size: 0 }, max_block: { ref_time: 2000000000000, proof_size: 18446744073709551615 }, per_class: { normal: { base_extrinsic: { ref_time: 108157000, proof_size: 0 }, max_extrinsic: Some ({ ref_time: 1299891843000, proof_size: 11990383647911208550 }), max_total: Some ({ ref_time: 1500000000000, proof_size: 13835058055282163711 }), reserved: Some ({ ref_time: 0, proof_size: 0 }) }, operational: { base_extrinsic: { ref_time: 108157000, proof_size: 0 }, max_extrinsic: Some ({ ref_time: 1799891843000, proof_size: 16602069666338596454 }), max_total: Some ({ ref_time: 2000000000000, proof_size: 18446744073709551615 }), reserved: Some ({ ref_time: 500000000000, proof_size: 4611686018427387904 }) }, mandatory: { base_extrinsic: { ref_time: 108157000, proof_size: 0 }, max_extrinsic: None (), max_total: None (), reserved: None () } } })
```

</details>
</li>

<li>
<details>
<summary>
<code>BlockLength</code>
</summary>
 The maximum length of a block (in bytes).

```rust
value: frame_system::limits::BlockLength({ max: { normal: 3932160, operational: 5242880, mandatory: 5242880 } })
```

</details>
</li>

<li>
<details>
<summary>
<code>BlockHashCount</code>
</summary>
 Maximum number of block number to block hash mappings to keep (oldest pruned first).

```rust
value: U32(2400)
```

</details>
</li>

<li>
<details>
<summary>
<code>DbWeight</code>
</summary>
 The weight of runtime database operations the runtime can invoke.

```rust
value: sp_weights::RuntimeDbWeight({ read: 14314000, write: 99642000 })
```

</details>
</li>

<li>
<details>
<summary>
<code>Version</code>
</summary>
 Get the chain's in-code version.

```rust
value: sp_version::RuntimeVersion({ spec_name: ("gdev"), impl_name: ("duniter-gdev"), authoring_version: 1, spec_version: 800, impl_version: 1, apis: ((((104, 122, 212, 74, 211, 127, 3, 194), 1), ((203, 202, 37, 227, 159, 20, 35, 135), 2), ((223, 106, 203, 104, 153, 7, 96, 155), 5), ((55, 227, 151, 252, 124, 145, 245, 228), 2), ((64, 254, 58, 212, 1, 248, 149, 154), 6), ((210, 188, 152, 151, 238, 208, 143, 21), 3), ((247, 139, 39, 139, 229, 63, 69, 76), 2), ((171, 60, 5, 114, 41, 31, 235, 139), 1), ((237, 153, 197, 172, 178, 94, 237, 245), 3), ((188, 157, 137, 144, 79, 91, 146, 63), 1), ((55, 200, 187, 19, 80, 169, 162, 168), 4), ((251, 197, 119, 185, 215, 71, 239, 214), 1))), transaction_version: 1, system_version: 1 })
```

</details>
</li>

<li>
<details>
<summary>
<code>SS58Prefix</code>
</summary>
 The designated SS58 prefix of this chain.

 This replaces the "ss58Format" property declared in the chain spec. Reason is
 that the runtime should know about the prefix in order to make use of it as
 an identifier of the chain.

```rust
value: U16(42)
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
<code>MaximumWeight</code>
</summary>
 The maximum weight that may be scheduled per block for any dispatchables.

```rust
value: sp_weights::weight_v2::Weight({ ref_time: 1600000000000, proof_size: 14757395258967641292 })
```

</details>
</li>

<li>
<details>
<summary>
<code>MaxScheduledPerBlock</code>
</summary>
 The maximum number of scheduled calls in the queue for a single block.

 NOTE:
 + Dependent pallets' benchmarks might require a higher limit for the setting. Set a
 higher limit under `runtime-benchmarks` feature.

```rust
value: U32(50)
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
<code>EpochDuration</code>
</summary>
 The amount of time, in slots, that each epoch should last.
 NOTE: Currently it is not possible to change the epoch duration after
 the chain has started. Attempting to do so will brick block production.

```rust
value: U64(30)
```

</details>
</li>

<li>
<details>
<summary>
<code>ExpectedBlockTime</code>
</summary>
 The expected average block time at which BABE should be creating
 blocks. Since BABE is probabilistic it is not trivial to figure out
 what the expected average block time should be based on the slot
 duration and the security parameter `c` (where `1 - c` represents
 the probability of a slot being empty).

```rust
value: U64(6000)
```

</details>
</li>

<li>
<details>
<summary>
<code>MaxAuthorities</code>
</summary>
 Max number of authorities allowed

```rust
value: U32(32)
```

</details>
</li>

<li>
<details>
<summary>
<code>MaxNominators</code>
</summary>
 The maximum number of nominators for each validator.

```rust
value: U32(64)
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
<code>MinimumPeriod</code>
</summary>
 The minimum period between blocks.

 Be aware that this is different to the *expected* period that the block production
 apparatus provides. Your chosen consensus system will generally work with this to
 determine a sensible block time. For example, in the Aura pallet it will be double this
 period on default settings.

```rust
value: U64(3000)
```

</details>
</li>

</ul>
</li>

<li>Parameters - 5
<ul>

</ul>
</li>

<li>Balances - 6
<ul>

<li>
<details>
<summary>
<code>ExistentialDeposit</code>
</summary>
 The minimum amount required to keep an account open. MUST BE GREATER THAN ZERO!

 If you *really* need it to be zero, you can enable the feature `insecure_zero_ed` for
 this pallet. However, you do so at your own risk: this will open up a major DoS vector.
 In case you have multiple sources of provider references, you may also get unexpected
 behaviour if you set this to zero.

 Bottom line: Do yourself a favour and make it at least one!

```rust
value: U64(100)
```

</details>
</li>

<li>
<details>
<summary>
<code>MaxLocks</code>
</summary>
 The maximum number of locks that should exist on an account.
 Not strictly enforced, but used for weight estimation.

 Use of locks is deprecated in favour of freezes. See `https://github.com/paritytech/substrate/pull/12951/`

```rust
value: U32(50)
```

</details>
</li>

<li>
<details>
<summary>
<code>MaxReserves</code>
</summary>
 The maximum number of named reserves that can exist on an account.

 Use of reserves is deprecated in favour of holds. See `https://github.com/paritytech/substrate/pull/12951/`

```rust
value: U32(5)
```

</details>
</li>

<li>
<details>
<summary>
<code>MaxFreezes</code>
</summary>
 The maximum number of individual freeze locks that can exist on an account at any time.

```rust
value: U32(0)
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
<code>OperationalFeeMultiplier</code>
</summary>
 A fee multiplier for `Operational` extrinsics to compute "virtual tip" to boost their
 `priority`

 This value is multiplied by the `final_fee` to obtain a "virtual tip" that is later
 added to a tip component in regular `priority` calculations.
 It means that a `Normal` transaction can front-run a similarly-sized `Operational`
 extrinsic (with no tip), by including a tip value greater than the virtual tip.

 ```rust,ignore
 // For `Normal`
 let priority = priority_calc(tip);

 // For `Operational`
 let virtual_tip = (inclusion_fee + tip) * OperationalFeeMultiplier;
 let priority = priority_calc(tip + virtual_tip);
 ```

 Note that since we use `final_fee` the multiplier applies also to the regular `tip`
 sent with the transaction. So, not only does the transaction get a priority bump based
 on the `inclusion_fee`, but we also amplify the impact of tips applied to `Operational`
 transactions.

```rust
value: U8(5)
```

</details>
</li>

</ul>
</li>

<li>OneshotAccount - 7
<ul>

</ul>
</li>

<li>Quota - 66
<ul>

<li>
<details>
<summary>
<code>RefundAccount</code>
</summary>
 Account used to refund fees.

```rust
value: sp_core::crypto::AccountId32(((109, 111, 100, 108, 112, 121, 47, 116, 114, 115, 114, 121, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0)))
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
<code>MaxByIssuer</code>
</summary>
 Maximum number of active certifications per issuer.

```rust
value: U32(8)
```

</details>
</li>

<li>
<details>
<summary>
<code>MinCertForMembership</code>
</summary>
 Minimum number of certifications required to become a Smith.

```rust
value: U32(2)
```

</details>
</li>

<li>
<details>
<summary>
<code>SmithInactivityMaxDuration</code>
</summary>
 Maximum duration of inactivity allowed before a Smith is removed.

```rust
value: U32(48)
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
<code>MaxAuthorities</code>
</summary>
 Maximum number of authorities allowed.

```rust
value: U32(32)
```

</details>
</li>

</ul>
</li>

<li>Authorship - 12
<ul>

</ul>
</li>

<li>Offences - 13
<ul>

</ul>
</li>

<li>Historical - 14
<ul>

</ul>
</li>

<li>Session - 15
<ul>

</ul>
</li>

<li>Grandpa - 16
<ul>

<li>
<details>
<summary>
<code>MaxAuthorities</code>
</summary>
 Max Authorities in use

```rust
value: U32(32)
```

</details>
</li>

<li>
<details>
<summary>
<code>MaxNominators</code>
</summary>
 The maximum number of nominators for each validator.

```rust
value: U32(64)
```

</details>
</li>

<li>
<details>
<summary>
<code>MaxSetIdSessionEntries</code>
</summary>
 The maximum number of entries to keep in the set id to session index mapping.

 Since the `SetIdSession` map is only used for validating equivocations this
 value should relate to the bonding duration of whatever staking system is
 being used (if any). If equivocation handling is not enabled then this value
 can be zero.

```rust
value: U64(1000)
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
<code>UnsignedPriority</code>
</summary>
 A configuration for base priority of unsigned transactions.

 This is exposed so that it can be tuned for particular runtime, when
 multiple pallets send unsigned transactions.

```rust
value: U64(18446744073709551615)
```

</details>
</li>

</ul>
</li>

<li>AuthorityDiscovery - 18
<ul>

</ul>
</li>

<li>Sudo - 20
<ul>

</ul>
</li>

<li>UpgradeOrigin - 21
<ul>

</ul>
</li>

<li>Preimage - 22
<ul>

</ul>
</li>

<li>TechnicalCommittee - 23
<ul>

<li>
<details>
<summary>
<code>MaxProposalWeight</code>
</summary>
 The maximum weight of a dispatch call that can be proposed and executed.

```rust
value: sp_weights::weight_v2::Weight({ ref_time: 1000000000000, proof_size: 9223372036854775807 })
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
<code>MaxPastReeval</code>
</summary>
 Maximum number of past UD revaluations to keep in storage.

```rust
value: U32(160)
```

</details>
</li>

<li>
<details>
<summary>
<code>SquareMoneyGrowthRate</code>
</summary>
 Square of the money growth rate per UD reevaluation period.

```rust
value: sp_arithmetic::per_things::Perbill((2381440))
```

</details>
</li>

<li>
<details>
<summary>
<code>UdCreationPeriod</code>
</summary>
 Universal dividend creation period in milliseconds.

```rust
value: U64(60000)
```

</details>
</li>

<li>
<details>
<summary>
<code>UdReevalPeriod</code>
</summary>
 Universal dividend reevaluation period in milliseconds.

```rust
value: U64(1200000)
```

</details>
</li>

</ul>
</li>

<li>Wot - 40
<ul>

<li>
<details>
<summary>
<code>FirstIssuableOn</code>
</summary>
 The block number from which the first certification can be issued.

```rust
value: U32(20)
```

</details>
</li>

<li>
<details>
<summary>
<code>MinCertForMembership</code>
</summary>
 The minimum number of certifications required for membership eligibility.

```rust
value: U32(2)
```

</details>
</li>

<li>
<details>
<summary>
<code>MinCertForCreateIdtyRight</code>
</summary>
 The minimum number of certifications required to create an identity.

```rust
value: U32(2)
```

</details>
</li>

</ul>
</li>

<li>Identity - 41
<ul>

<li>
<details>
<summary>
<code>ConfirmPeriod</code>
</summary>
 The period during which the owner can confirm the new identity.

```rust
value: U32(40)
```

</details>
</li>

<li>
<details>
<summary>
<code>ValidationPeriod</code>
</summary>
 The period during which the identity has to be validated to become a member.

```rust
value: U32(876600)
```

</details>
</li>

<li>
<details>
<summary>
<code>AutorevocationPeriod</code>
</summary>
 The period before which an identity that lost membership is automatically revoked.

```rust
value: U32(438300)
```

</details>
</li>

<li>
<details>
<summary>
<code>DeletionPeriod</code>
</summary>
 The period after which a revoked identity is removed and the keys are freed.

```rust
value: U32(438300)
```

</details>
</li>

<li>
<details>
<summary>
<code>ChangeOwnerKeyPeriod</code>
</summary>
 The minimum duration between two owner key changes to prevent identity theft.

```rust
value: U32(100800)
```

</details>
</li>

<li>
<details>
<summary>
<code>IdtyCreationPeriod</code>
</summary>
 The minimum duration between the creation of two identities by the same creator.
 Should be greater than or equal to the certification period defined in the certification pallet.

```rust
value: U32(50)
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
<code>MembershipPeriod</code>
</summary>
 Maximum lifespan of a single membership (in number of blocks).

```rust
value: U32(1000)
```

</details>
</li>

<li>
<details>
<summary>
<code>MembershipRenewalPeriod</code>
</summary>
 Minimum delay to wait before renewing membership, i.e., asking for distance evaluation.

```rust
value: U32(1000)
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
<code>CertPeriod</code>
</summary>
 The minimum duration (in blocks) between two certifications issued by the same issuer.

```rust
value: U32(15)
```

</details>
</li>

<li>
<details>
<summary>
<code>MaxByIssuer</code>
</summary>
 The maximum number of active certifications that can be issued by a single issuer.

```rust
value: U32(10)
```

</details>
</li>

<li>
<details>
<summary>
<code>MinReceivedCertToBeAbleToIssueCert</code>
</summary>
 The minimum number of certifications received that an identity must have
 to be allowed to issue a certification.

```rust
value: U32(2)
```

</details>
</li>

<li>
<details>
<summary>
<code>ValidityPeriod</code>
</summary>
 The duration (in blocks) for which a certification remains valid.

```rust
value: U32(1000)
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
<code>EvaluationPrice</code>
</summary>
 The amount reserved during evaluation.

```rust
value: U64(1000)
```

</details>
</li>

<li>
<details>
<summary>
<code>EvaluationPeriod</code>
</summary>
 The evaluation period in blocks.
 Since the evaluation uses 3 pools, the total evaluation time will be 3 * EvaluationPeriod.

```rust
value: U32(7)
```

</details>
</li>

<li>
<details>
<summary>
<code>MaxRefereeDistance</code>
</summary>
 The maximum distance used to define a referee's accessibility.
 This value is not used by the runtime but is needed by the client distance oracle.

```rust
value: U32(5)
```

</details>
</li>

<li>
<details>
<summary>
<code>MinAccessibleReferees</code>
</summary>
 The minimum ratio of accessible referees required.

```rust
value: sp_arithmetic::per_things::Perbill((800000000))
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
<code>ProofLimit</code>
</summary>
 Limit of proof size.

 Atomic swap is only atomic if once the proof is revealed, both parties can submit the
 proofs on-chain. If A is the one that generates the proof, then it requires that either:
 - A's blockchain has the same proof length limit as B's blockchain.
 - Or A's blockchain has shorter proof length limit as B's blockchain.

 If B sees A is on a blockchain with larger proof length limit, then it should kindly
 refuse to accept the atomic swap request if A generates the proof, and asks that B
 generates the proof instead.

```rust
value: U32(1024)
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
<code>DepositBase</code>
</summary>
 The base amount of currency needed to reserve for creating a multisig execution or to
 store a dispatch call for later.

 This is held for an additional storage item whose value size is
 `4 + sizeof((BlockNumber, Balance, AccountId))` bytes and whose key size is
 `32 + sizeof(AccountId)` bytes.

```rust
value: U64(100)
```

</details>
</li>

<li>
<details>
<summary>
<code>DepositFactor</code>
</summary>
 The amount of currency needed per unit threshold when creating a multisig execution.

 This is held for adding 32 bytes more into a pre-existing storage value.

```rust
value: U64(32)
```

</details>
</li>

<li>
<details>
<summary>
<code>MaxSignatories</code>
</summary>
 The maximum amount of signatories allowed in the multisig.

```rust
value: U32(10)
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
<code>MaxRequests</code>
</summary>
 Maximum number of not yet filled requests.

```rust
value: U32(100)
```

</details>
</li>

<li>
<details>
<summary>
<code>RequestPrice</code>
</summary>
 The price of a request.

```rust
value: U64(2000)
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
<code>ProxyDepositBase</code>
</summary>
 The base amount of currency needed to reserve for creating a proxy.

 This is held for an additional storage item whose value size is
 `sizeof(Balance)` bytes and whose key size is `sizeof(AccountId)` bytes.

```rust
value: U64(108)
```

</details>
</li>

<li>
<details>
<summary>
<code>ProxyDepositFactor</code>
</summary>
 The amount of currency needed per proxy added.

 This is held for adding 32 bytes plus an instance of `ProxyType` more into a
 pre-existing storage value. Thus, when configuring `ProxyDepositFactor` one should take
 into account `32 + proxy_type.encode().len()` bytes of data.

```rust
value: U64(33)
```

</details>
</li>

<li>
<details>
<summary>
<code>MaxProxies</code>
</summary>
 The maximum amount of proxies allowed for a single account.

```rust
value: U32(32)
```

</details>
</li>

<li>
<details>
<summary>
<code>MaxPending</code>
</summary>
 The maximum amount of time-delayed announcements that are allowed to be pending.

```rust
value: U32(32)
```

</details>
</li>

<li>
<details>
<summary>
<code>AnnouncementDepositBase</code>
</summary>
 The base amount of currency needed to reserve for creating an announcement.

 This is held when a new storage item holding a `Balance` is created (typically 16
 bytes).

```rust
value: U64(108)
```

</details>
</li>

<li>
<details>
<summary>
<code>AnnouncementDepositFactor</code>
</summary>
 The amount of currency needed per announcement made.

 This is held for adding an `AccountId`, `Hash` and `BlockNumber` (typically 68 bytes)
 into a pre-existing storage value.

```rust
value: U64(66)
```

</details>
</li>

</ul>
</li>

<li>Utility - 54
<ul>

<li>
<details>
<summary>
<code>batched_calls_limit</code>
</summary>
 The limit on the number of batched calls.

```rust
value: U32(10922)
```

</details>
</li>

</ul>
</li>

<li>Treasury - 55
<ul>

<li>
<details>
<summary>
<code>SpendPeriod</code>
</summary>
 Period between successive spends.

```rust
value: U32(14400)
```

</details>
</li>

<li>
<details>
<summary>
<code>Burn</code>
</summary>
 Percentage of spare funds (if any) that are burnt per spend period.

```rust
value: sp_arithmetic::per_things::Permill((0))
```

</details>
</li>

<li>
<details>
<summary>
<code>PalletId</code>
</summary>
 The treasury's pallet id, used for deriving its sovereign account ID.

```rust
value: frame_support::PalletId(((112, 121, 47, 116, 114, 115, 114, 121)))
```

</details>
</li>

<li>
<details>
<summary>
<code>MaxApprovals</code>
</summary>
 DEPRECATED: associated with `spend_local` call and will be removed in May 2025.
 Refer to <https://github.com/paritytech/polkadot-sdk/pull/5961> for migration to `spend`.

 The maximum number of approvals that can wait in the spending queue.

 NOTE: This parameter is also used within the Bounties Pallet extension if enabled.

```rust
value: U32(100)
```

</details>
</li>

<li>
<details>
<summary>
<code>PayoutPeriod</code>
</summary>
 The period during which an approved treasury spend has to be claimed.

```rust
value: U32(10)
```

</details>
</li>

</ul>
</li>

</ul>
