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

There are **81** user calls from **21** pallets.

### Account - 1

#### unlink_identity - 0

<details><summary><code>unlink_identity()</code></summary>

Taking 0.0113 % of a block.

```rust
```
</details>


See [`Pallet::unlink_identity`].

### Scheduler - 2

#### schedule - 0

<details><summary><code>schedule(when, maybe_periodic, priority, call)</code></summary>

Taking 0.013 % of a block.

```rust
when: BlockNumberFor<T>
maybe_periodic: Option<schedule::Period<BlockNumberFor<T>>>
priority: schedule::Priority
call: Box<<T as Config>::RuntimeCall>
```
</details>


See [`Pallet::schedule`].

#### cancel - 1

<details><summary><code>cancel(when, index)</code></summary>

Taking 0.0195 % of a block.

```rust
when: BlockNumberFor<T>
index: u32
```
</details>


See [`Pallet::cancel`].

#### schedule_named - 2

<details><summary><code>schedule_named(id, when, maybe_periodic, priority, call)</code></summary>

Taking 0.0203 % of a block.

```rust
id: TaskName
when: BlockNumberFor<T>
maybe_periodic: Option<schedule::Period<BlockNumberFor<T>>>
priority: schedule::Priority
call: Box<<T as Config>::RuntimeCall>
```
</details>


See [`Pallet::schedule_named`].

#### cancel_named - 3

<details><summary><code>cancel_named(id)</code></summary>

Taking 0.0209 % of a block.

```rust
id: TaskName
```
</details>


See [`Pallet::cancel_named`].

#### schedule_after - 4

<details><summary><code>schedule_after(after, maybe_periodic, priority, call)</code></summary>

No weight available.

```rust
after: BlockNumberFor<T>
maybe_periodic: Option<schedule::Period<BlockNumberFor<T>>>
priority: schedule::Priority
call: Box<<T as Config>::RuntimeCall>
```
</details>


See [`Pallet::schedule_after`].

#### schedule_named_after - 5

<details><summary><code>schedule_named_after(id, after, maybe_periodic, priority, call)</code></summary>

No weight available.

```rust
id: TaskName
after: BlockNumberFor<T>
maybe_periodic: Option<schedule::Period<BlockNumberFor<T>>>
priority: schedule::Priority
call: Box<<T as Config>::RuntimeCall>
```
</details>


See [`Pallet::schedule_named_after`].

### Babe - 3

#### report_equivocation - 0

<details><summary><code>report_equivocation(equivocation_proof, key_owner_proof)</code></summary>

No weight available.

```rust
equivocation_proof: Box<EquivocationProof<HeaderFor<T>>>
key_owner_proof: T::KeyOwnerProof
```
</details>


See [`Pallet::report_equivocation`].

### Balances - 6

#### transfer_allow_death - 0

<details><summary><code>transfer_allow_death(dest, value)</code></summary>

Taking 0.0216 % of a block.

```rust
dest: AccountIdLookupOf<T>
value: T::Balance
```
</details>


See [`Pallet::transfer_allow_death`].

#### transfer_keep_alive - 3

<details><summary><code>transfer_keep_alive(dest, value)</code></summary>

Taking 0.0232 % of a block.

```rust
dest: AccountIdLookupOf<T>
value: T::Balance
```
</details>


See [`Pallet::transfer_keep_alive`].

#### transfer_all - 4

<details><summary><code>transfer_all(dest, keep_alive)</code></summary>

Taking 0.0184 % of a block.

```rust
dest: AccountIdLookupOf<T>
keep_alive: bool
```
</details>


See [`Pallet::transfer_all`].

#### force_set_balance - 8

<details><summary><code>force_set_balance(who, new_free)</code></summary>

No weight available.

```rust
who: AccountIdLookupOf<T>
new_free: T::Balance
```
</details>


See [`Pallet::force_set_balance`].

### OneshotAccount - 7

#### create_oneshot_account - 0

<details><summary><code>create_oneshot_account(dest, value)</code></summary>

Taking 0.0126 % of a block.

```rust
dest: <T::Lookup as StaticLookup>::Source
value: <T::Currency as Currency<T::AccountId>>::Balance
```
</details>


See [`Pallet::create_oneshot_account`].

#### consume_oneshot_account - 1

<details><summary><code>consume_oneshot_account(block_height, dest)</code></summary>

Taking 0.0214 % of a block.

```rust
block_height: BlockNumberFor<T>
dest: Account<<T::Lookup as StaticLookup>::Source>
```
</details>


See [`Pallet::consume_oneshot_account`].

#### consume_oneshot_account_with_remaining - 2

<details><summary><code>consume_oneshot_account_with_remaining(block_height, dest, remaining_to, balance)</code></summary>

Taking 0.0295 % of a block.

```rust
block_height: BlockNumberFor<T>
dest: Account<<T::Lookup as StaticLookup>::Source>
remaining_to: Account<<T::Lookup as StaticLookup>::Source>
balance: <T::Currency as Currency<T::AccountId>>::Balance
```
</details>


See [`Pallet::consume_oneshot_account_with_remaining`].

### SmithMembers - 10

#### invite_smith - 0

<details><summary><code>invite_smith(receiver)</code></summary>

Taking 0.0265 % of a block.

```rust
receiver: T::IdtyIndex
```
</details>


See [`Pallet::invite_smith`].

#### accept_invitation - 1

<details><summary><code>accept_invitation()</code></summary>

Taking 0.0133 % of a block.

```rust
```
</details>


See [`Pallet::accept_invitation`].

#### certify_smith - 2

<details><summary><code>certify_smith(receiver)</code></summary>

Taking 0.0234 % of a block.

```rust
receiver: T::IdtyIndex
```
</details>


See [`Pallet::certify_smith`].

### AuthorityMembers - 11

#### go_offline - 0

<details><summary><code>go_offline()</code></summary>

Taking 0.0188 % of a block.

```rust
```
</details>


See [`Pallet::go_offline`].

#### go_online - 1

<details><summary><code>go_online()</code></summary>

Taking 0.0225 % of a block.

```rust
```
</details>


See [`Pallet::go_online`].

#### set_session_keys - 2

<details><summary><code>set_session_keys(keys)</code></summary>

Taking 0.0296 % of a block.

```rust
keys: T::Keys
```
</details>


See [`Pallet::set_session_keys`].

#### remove_member_from_blacklist - 4

<details><summary><code>remove_member_from_blacklist(member_id)</code></summary>

Taking 0.0125 % of a block.

```rust
member_id: T::MemberId
```
</details>


See [`Pallet::remove_member_from_blacklist`].

### Grandpa - 16

#### report_equivocation - 0

<details><summary><code>report_equivocation(equivocation_proof, key_owner_proof)</code></summary>

No weight available.

```rust
equivocation_proof: Box<EquivocationProof<T::Hash, BlockNumberFor<T>>>
key_owner_proof: T::KeyOwnerProof
```
</details>


See [`Pallet::report_equivocation`].

### UpgradeOrigin - 21

#### dispatch_as_root_unchecked_weight - 1

<details><summary><code>dispatch_as_root_unchecked_weight(call, weight)</code></summary>

No weight available.

```rust
call: Box<<T as Config>::Call>
weight: Weight
```
</details>


See [`Pallet::dispatch_as_root_unchecked_weight`].

### Preimage - 22

#### note_preimage - 0

<details><summary><code>note_preimage(bytes)</code></summary>

Taking 0.5106 % of a block.

```rust
bytes: Vec<u8>
```
</details>


See [`Pallet::note_preimage`].

#### unnote_preimage - 1

<details><summary><code>unnote_preimage(hash)</code></summary>

Taking 0.0211 % of a block.

```rust
hash: T::Hash
```
</details>


See [`Pallet::unnote_preimage`].

#### request_preimage - 2

<details><summary><code>request_preimage(hash)</code></summary>

Taking 0.0136 % of a block.

```rust
hash: T::Hash
```
</details>


See [`Pallet::request_preimage`].

#### unrequest_preimage - 3

<details><summary><code>unrequest_preimage(hash)</code></summary>

Taking 0.0195 % of a block.

```rust
hash: T::Hash
```
</details>


See [`Pallet::unrequest_preimage`].

#### ensure_updated - 4

<details><summary><code>ensure_updated(hashes)</code></summary>

Taking 21.0381 % of a block.

```rust
hashes: Vec<T::Hash>
```
</details>


See [`Pallet::ensure_updated`].

### TechnicalCommittee - 23

#### execute - 1

<details><summary><code>execute(proposal, length_bound)</code></summary>

Taking 0.006 % of a block.

```rust
proposal: Box<<T as Config<I>>::Proposal>
length_bound: u32
```
</details>


See [`Pallet::execute`].

#### propose - 2

<details><summary><code>propose(threshold, proposal, length_bound)</code></summary>

No weight available.

```rust
threshold: MemberCount
proposal: Box<<T as Config<I>>::Proposal>
length_bound: u32
```
</details>


See [`Pallet::propose`].

#### vote - 3

<details><summary><code>vote(proposal, index, approve)</code></summary>

Taking 0.0144 % of a block.

```rust
proposal: T::Hash
index: ProposalIndex
approve: bool
```
</details>


See [`Pallet::vote`].

#### close - 6

<details><summary><code>close(proposal_hash, index, proposal_weight_bound, length_bound)</code></summary>

No weight available.

```rust
proposal_hash: T::Hash
index: ProposalIndex
proposal_weight_bound: Weight
length_bound: u32
```
</details>


See [`Pallet::close`].

### UniversalDividend - 30

#### claim_uds - 0

<details><summary><code>claim_uds()</code></summary>

Taking 0.0224 % of a block.

```rust
```
</details>


See [`Pallet::claim_uds`].

#### transfer_ud - 1

<details><summary><code>transfer_ud(dest, value)</code></summary>

Taking 0.027 % of a block.

```rust
dest: <T::Lookup as StaticLookup>::Source
value: BalanceOf<T>
```
</details>


See [`Pallet::transfer_ud`].

#### transfer_ud_keep_alive - 2

<details><summary><code>transfer_ud_keep_alive(dest, value)</code></summary>

Taking 0.0193 % of a block.

```rust
dest: <T::Lookup as StaticLookup>::Source
value: BalanceOf<T>
```
</details>


See [`Pallet::transfer_ud_keep_alive`].

### Identity - 41

#### create_identity - 0

<details><summary><code>create_identity(owner_key)</code></summary>

Taking 0.0969 % of a block.

```rust
owner_key: T::AccountId
```
</details>


See [`Pallet::create_identity`].

#### confirm_identity - 1

<details><summary><code>confirm_identity(idty_name)</code></summary>

Taking 0.0365 % of a block.

```rust
idty_name: IdtyName
```
</details>


See [`Pallet::confirm_identity`].

#### change_owner_key - 3

<details><summary><code>change_owner_key(new_key, new_key_sig)</code></summary>

Taking 0.0507 % of a block.

```rust
new_key: T::AccountId
new_key_sig: T::Signature
```
</details>


See [`Pallet::change_owner_key`].

#### revoke_identity - 4

<details><summary><code>revoke_identity(idty_index, revocation_key, revocation_sig)</code></summary>

Taking 0.0487 % of a block.

```rust
idty_index: T::IdtyIndex
revocation_key: T::AccountId
revocation_sig: T::Signature
```
</details>


See [`Pallet::revoke_identity`].

#### fix_sufficients - 7

<details><summary><code>fix_sufficients(owner_key, inc)</code></summary>

Taking 0.0113 % of a block.

```rust
owner_key: T::AccountId
inc: bool
```
</details>


See [`Pallet::fix_sufficients`].

#### link_account - 8

<details><summary><code>link_account(account_id, payload_sig)</code></summary>

Taking 0.0182 % of a block.

```rust
account_id: T::AccountId
payload_sig: T::Signature
```
</details>


See [`Pallet::link_account`].

### Certification - 43

#### add_cert - 0

<details><summary><code>add_cert(receiver)</code></summary>

Taking 0.0398 % of a block.

```rust
receiver: T::IdtyIndex
```
</details>


See [`Pallet::add_cert`].

#### renew_cert - 3

<details><summary><code>renew_cert(receiver)</code></summary>

Taking 0.0324 % of a block.

```rust
receiver: T::IdtyIndex
```
</details>


See [`Pallet::renew_cert`].

#### del_cert - 1

<details><summary><code>del_cert(issuer, receiver)</code></summary>

Taking 0.0276 % of a block.

```rust
issuer: T::IdtyIndex
receiver: T::IdtyIndex
```
</details>


See [`Pallet::del_cert`].

#### remove_all_certs_received_by - 2

<details><summary><code>remove_all_certs_received_by(idty_index)</code></summary>

Taking 7.8043 % of a block.

```rust
idty_index: T::IdtyIndex
```
</details>


See [`Pallet::remove_all_certs_received_by`].

### Distance - 44

#### request_distance_evaluation - 0

<details><summary><code>request_distance_evaluation()</code></summary>

Taking 0.0354 % of a block.

```rust
```
</details>


See [`Pallet::request_distance_evaluation`].

#### request_distance_evaluation_for - 4

<details><summary><code>request_distance_evaluation_for(target)</code></summary>

Taking 0.0367 % of a block.

```rust
target: T::IdtyIndex
```
</details>


See [`Pallet::request_distance_evaluation_for`].

#### update_evaluation - 1

<details><summary><code>update_evaluation(computation_result)</code></summary>

Taking 0.0351 % of a block.

```rust
computation_result: ComputationResult
```
</details>


See [`Pallet::update_evaluation`].

#### force_update_evaluation - 2

<details><summary><code>force_update_evaluation(evaluator, computation_result)</code></summary>

Taking 0.018 % of a block.

```rust
evaluator: <T as frame_system::Config>::AccountId
computation_result: ComputationResult
```
</details>


See [`Pallet::force_update_evaluation`].

#### force_valid_distance_status - 3

<details><summary><code>force_valid_distance_status(identity)</code></summary>

Taking 0.0301 % of a block.

```rust
identity: <T as pallet_identity::Config>::IdtyIndex
```
</details>


See [`Pallet::force_valid_distance_status`].

### AtomicSwap - 50

#### create_swap - 0

<details><summary><code>create_swap(target, hashed_proof, action, duration)</code></summary>

No weight available.

```rust
target: T::AccountId
hashed_proof: HashedProof
action: T::SwapAction
duration: BlockNumberFor<T>
```
</details>


See [`Pallet::create_swap`].

#### claim_swap - 1

<details><summary><code>claim_swap(proof, action)</code></summary>

No weight available.

```rust
proof: Vec<u8>
action: T::SwapAction
```
</details>


See [`Pallet::claim_swap`].

#### cancel_swap - 2

<details><summary><code>cancel_swap(target, hashed_proof)</code></summary>

No weight available.

```rust
target: T::AccountId
hashed_proof: HashedProof
```
</details>


See [`Pallet::cancel_swap`].

### Multisig - 51

#### as_multi_threshold_1 - 0

<details><summary><code>as_multi_threshold_1(other_signatories, call)</code></summary>

Taking 0.0045 % of a block.

```rust
other_signatories: Vec<T::AccountId>
call: Box<<T as Config>::RuntimeCall>
```
</details>


See [`Pallet::as_multi_threshold_1`].

#### as_multi - 1

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


See [`Pallet::as_multi`].

#### approve_as_multi - 2

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


See [`Pallet::approve_as_multi`].

#### cancel_as_multi - 3

<details><summary><code>cancel_as_multi(threshold, other_signatories, timepoint, call_hash)</code></summary>

Taking 0.0135 % of a block.

```rust
threshold: u16
other_signatories: Vec<T::AccountId>
timepoint: Timepoint<BlockNumberFor<T>>
call_hash: [u8; 32]
```
</details>


See [`Pallet::cancel_as_multi`].

### ProvideRandomness - 52

#### request - 0

<details><summary><code>request(randomness_type, salt)</code></summary>

Taking 0.0393 % of a block.

```rust
randomness_type: RandomnessType
salt: H256
```
</details>


See [`Pallet::request`].

### Proxy - 53

#### proxy - 0

<details><summary><code>proxy(real, force_proxy_type, call)</code></summary>

Taking 0.0063 % of a block.

```rust
real: AccountIdLookupOf<T>
force_proxy_type: Option<T::ProxyType>
call: Box<<T as Config>::RuntimeCall>
```
</details>


See [`Pallet::proxy`].

#### add_proxy - 1

<details><summary><code>add_proxy(delegate, proxy_type, delay)</code></summary>

Taking 0.0129 % of a block.

```rust
delegate: AccountIdLookupOf<T>
proxy_type: T::ProxyType
delay: BlockNumberFor<T>
```
</details>


See [`Pallet::add_proxy`].

#### remove_proxy - 2

<details><summary><code>remove_proxy(delegate, proxy_type, delay)</code></summary>

Taking 0.0133 % of a block.

```rust
delegate: AccountIdLookupOf<T>
proxy_type: T::ProxyType
delay: BlockNumberFor<T>
```
</details>


See [`Pallet::remove_proxy`].

#### remove_proxies - 3

<details><summary><code>remove_proxies()</code></summary>

Taking 0.0129 % of a block.

```rust
```
</details>


See [`Pallet::remove_proxies`].

#### create_pure - 4

<details><summary><code>create_pure(proxy_type, delay, index)</code></summary>

Taking 0.0141 % of a block.

```rust
proxy_type: T::ProxyType
delay: BlockNumberFor<T>
index: u16
```
</details>


See [`Pallet::create_pure`].

#### kill_pure - 5

<details><summary><code>kill_pure(spawner, proxy_type, index, height, ext_index)</code></summary>

Taking 0.0125 % of a block.

```rust
spawner: AccountIdLookupOf<T>
proxy_type: T::ProxyType
index: u16
height: BlockNumberFor<T>
ext_index: u32
```
</details>


See [`Pallet::kill_pure`].

#### announce - 6

<details><summary><code>announce(real, call_hash)</code></summary>

Taking 0.0218 % of a block.

```rust
real: AccountIdLookupOf<T>
call_hash: CallHashOf<T>
```
</details>


See [`Pallet::announce`].

#### remove_announcement - 7

<details><summary><code>remove_announcement(real, call_hash)</code></summary>

Taking 0.0198 % of a block.

```rust
real: AccountIdLookupOf<T>
call_hash: CallHashOf<T>
```
</details>


See [`Pallet::remove_announcement`].

#### reject_announcement - 8

<details><summary><code>reject_announcement(delegate, call_hash)</code></summary>

Taking 0.02 % of a block.

```rust
delegate: AccountIdLookupOf<T>
call_hash: CallHashOf<T>
```
</details>


See [`Pallet::reject_announcement`].

#### proxy_announced - 9

<details><summary><code>proxy_announced(delegate, real, force_proxy_type, call)</code></summary>

Taking 0.0235 % of a block.

```rust
delegate: AccountIdLookupOf<T>
real: AccountIdLookupOf<T>
force_proxy_type: Option<T::ProxyType>
call: Box<<T as Config>::RuntimeCall>
```
</details>


See [`Pallet::proxy_announced`].

### Utility - 54

#### batch - 0

<details><summary><code>batch(calls)</code></summary>

Taking 0.2728 % of a block.

```rust
calls: Vec<<T as Config>::RuntimeCall>
```
</details>


See [`Pallet::batch`].

#### as_derivative - 1

<details><summary><code>as_derivative(index, call)</code></summary>

Taking 0.004 % of a block.

```rust
index: u16
call: Box<<T as Config>::RuntimeCall>
```
</details>


See [`Pallet::as_derivative`].

#### batch_all - 2

<details><summary><code>batch_all(calls)</code></summary>

Taking 0.2935 % of a block.

```rust
calls: Vec<<T as Config>::RuntimeCall>
```
</details>


See [`Pallet::batch_all`].

#### force_batch - 4

<details><summary><code>force_batch(calls)</code></summary>

Taking 0.3104 % of a block.

```rust
calls: Vec<<T as Config>::RuntimeCall>
```
</details>


See [`Pallet::force_batch`].

#### with_weight - 5

<details><summary><code>with_weight(call, weight)</code></summary>

No weight available.

```rust
call: Box<<T as Config>::RuntimeCall>
weight: Weight
```
</details>


See [`Pallet::with_weight`].

### Treasury - 55

#### propose_spend - 0

<details><summary><code>propose_spend(value, beneficiary)</code></summary>

Taking 0.0183 % of a block.

```rust
value: BalanceOf<T, I>
beneficiary: AccountIdLookupOf<T>
```
</details>


See [`Pallet::propose_spend`].

#### spend_local - 3

<details><summary><code>spend_local(amount, beneficiary)</code></summary>

Taking 0.0036 % of a block.

```rust
amount: BalanceOf<T, I>
beneficiary: AccountIdLookupOf<T>
```
</details>


See [`Pallet::spend_local`].

#### remove_approval - 4

<details><summary><code>remove_approval(proposal_id)</code></summary>

Taking 0.0111 % of a block.

```rust
proposal_id: ProposalIndex
```
</details>


See [`Pallet::remove_approval`].

#### spend - 5

<details><summary><code>spend(asset_kind, amount, beneficiary, valid_from)</code></summary>

Taking 0.0035 % of a block.

```rust
asset_kind: Box<T::AssetKind>
amount: AssetBalanceOf<T, I>
beneficiary: Box<BeneficiaryLookupOf<T, I>>
valid_from: Option<BlockNumberFor<T>>
```
</details>


See [`Pallet::spend`].

#### payout - 6

<details><summary><code>payout(index)</code></summary>

Taking 0.0326 % of a block.

```rust
index: SpendIndex
```
</details>


See [`Pallet::payout`].

#### check_status - 7

<details><summary><code>check_status(index)</code></summary>

Taking 0.011 % of a block.

```rust
index: SpendIndex
```
</details>


See [`Pallet::check_status`].

#### void_spend - 8

<details><summary><code>void_spend(index)</code></summary>

Taking 0.011 % of a block.

```rust
index: SpendIndex
```
</details>


See [`Pallet::void_spend`].



## Root calls

There are **18** root calls from **8** pallets.

### System - 0

#### set_heap_pages - 1

<details><summary><code>set_heap_pages(pages)</code></summary>

Taking 0.0169 % of a block.

```rust
pages: u64
```
</details>


See [`Pallet::set_heap_pages`].

#### set_code - 2

<details><summary><code>set_code(code)</code></summary>

Taking 3.9604 % of a block.

```rust
code: Vec<u8>
```
</details>


See [`Pallet::set_code`].

#### set_code_without_checks - 3

<details><summary><code>set_code_without_checks(code)</code></summary>

No weight available.

```rust
code: Vec<u8>
```
</details>


See [`Pallet::set_code_without_checks`].

#### set_storage - 4

<details><summary><code>set_storage(items)</code></summary>

Taking 5.9169 % of a block.

```rust
items: Vec<KeyValue>
```
</details>


See [`Pallet::set_storage`].

#### kill_storage - 5

<details><summary><code>kill_storage(keys)</code></summary>

Taking 5.8899 % of a block.

```rust
keys: Vec<Key>
```
</details>


See [`Pallet::kill_storage`].

#### kill_prefix - 6

<details><summary><code>kill_prefix(prefix, subkeys)</code></summary>

Taking 7.0785 % of a block.

```rust
prefix: Key
subkeys: u32
```
</details>


See [`Pallet::kill_prefix`].

#### authorize_upgrade - 9

<details><summary><code>authorize_upgrade(code_hash)</code></summary>

Taking 0.0098 % of a block.

```rust
code_hash: T::Hash
```
</details>


See [`Pallet::authorize_upgrade`].

#### authorize_upgrade_without_checks - 10

<details><summary><code>authorize_upgrade_without_checks(code_hash)</code></summary>

No weight available.

```rust
code_hash: T::Hash
```
</details>


See [`Pallet::authorize_upgrade_without_checks`].

#### apply_authorized_upgrade - 11

<details><summary><code>apply_authorized_upgrade(code)</code></summary>

Taking 4.1178 % of a block.

```rust
code: Vec<u8>
```
</details>


See [`Pallet::apply_authorized_upgrade`].

### Babe - 3

#### plan_config_change - 2

<details><summary><code>plan_config_change(config)</code></summary>

No weight available.

```rust
config: NextConfigDescriptor
```
</details>


See [`Pallet::plan_config_change`].

### Balances - 6

#### force_transfer - 2

<details><summary><code>force_transfer(source, dest, value)</code></summary>

Taking 0.0347 % of a block.

```rust
source: AccountIdLookupOf<T>
dest: AccountIdLookupOf<T>
value: T::Balance
```
</details>


See [`Pallet::force_transfer`].

#### force_unreserve - 5

<details><summary><code>force_unreserve(who, amount)</code></summary>

Taking 0.0128 % of a block.

```rust
who: AccountIdLookupOf<T>
amount: T::Balance
```
</details>


See [`Pallet::force_unreserve`].

### AuthorityMembers - 11

#### remove_member - 3

<details><summary><code>remove_member(member_id)</code></summary>

Taking 0.073 % of a block.

```rust
member_id: T::MemberId
```
</details>


See [`Pallet::remove_member`].

### Grandpa - 16

#### note_stalled - 2

<details><summary><code>note_stalled(delay, best_finalized_block_number)</code></summary>

No weight available.

```rust
delay: BlockNumberFor<T>
best_finalized_block_number: BlockNumberFor<T>
```
</details>


See [`Pallet::note_stalled`].

### TechnicalCommittee - 23

#### set_members - 0

<details><summary><code>set_members(new_members, prime, old_count)</code></summary>

Taking 0.175 % of a block.

```rust
new_members: Vec<T::AccountId>
prime: Option<T::AccountId>
old_count: MemberCount
```
</details>


See [`Pallet::set_members`].

#### disapprove_proposal - 5

<details><summary><code>disapprove_proposal(proposal_hash)</code></summary>

Taking 0.0236 % of a block.

```rust
proposal_hash: T::Hash
```
</details>


See [`Pallet::disapprove_proposal`].

### Identity - 41

#### prune_item_identities_names - 6

<details><summary><code>prune_item_identities_names(names)</code></summary>

Taking 5.9553 % of a block.

```rust
names: Vec<IdtyName>
```
</details>


See [`Pallet::prune_item_identities_names`].

### Utility - 54

#### dispatch_as - 3

<details><summary><code>dispatch_as(as_origin, call)</code></summary>

Taking 0.005 % of a block.

```rust
as_origin: Box<T::PalletsOrigin>
call: Box<<T as Config>::RuntimeCall>
```
</details>


See [`Pallet::dispatch_as`].






## Disabled calls

There are **4** disabled calls from **2** pallets.

### System - 0

#### remark - 0

<details><summary><code>remark(remark)</code></summary>

Taking 0.0946 % of a block.

```rust
remark: Vec<u8>
```
</details>


See [`Pallet::remark`].

#### remark_with_event - 7

<details><summary><code>remark_with_event(remark)</code></summary>

Taking 0.3505 % of a block.

```rust
remark: Vec<u8>
```
</details>


See [`Pallet::remark_with_event`].

### Session - 15

#### set_keys - 0

<details><summary><code>set_keys(keys, proof)</code></summary>

Taking 0.0406 % of a block.

```rust
keys: T::Keys
proof: Vec<u8>
```
</details>


See [`Pallet::set_keys`].

#### purge_keys - 1

<details><summary><code>purge_keys()</code></summary>

Taking 0.0351 % of a block.

```rust
```
</details>


See [`Pallet::purge_keys`].

