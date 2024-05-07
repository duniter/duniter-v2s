# Runtime events

There are **138** events from **35** pallets.

<ul>
<li>System - 0
<ul>
<li>
<details>
<summary>
<code>ExtrinsicSuccess(dispatch_info)</code> - 0</summary>
An extrinsic completed successfully.

```rust
dispatch_info: DispatchInfo
```

</details>
</li>
<li>
<details>
<summary>
<code>ExtrinsicFailed(dispatch_error, dispatch_info)</code> - 1</summary>
An extrinsic failed.

```rust
dispatch_error: DispatchError
dispatch_info: DispatchInfo
```

</details>
</li>
<li>
<details>
<summary>
<code>CodeUpdated()</code> - 2</summary>
`:code` was updated.

```rust
no args
```

</details>
</li>
<li>
<details>
<summary>
<code>NewAccount(account)</code> - 3</summary>
A new account was created.

```rust
account: T::AccountId
```

</details>
</li>
<li>
<details>
<summary>
<code>KilledAccount(account)</code> - 4</summary>
An account was reaped.

```rust
account: T::AccountId
```

</details>
</li>
<li>
<details>
<summary>
<code>Remarked(sender, hash)</code> - 5</summary>
On on-chain remark happened.

```rust
sender: T::AccountId
hash: T::Hash
```

</details>
</li>
<li>
<details>
<summary>
<code>UpgradeAuthorized(code_hash, check_version)</code> - 6</summary>
An upgrade was authorized.

```rust
code_hash: T::Hash
check_version: bool
```

</details>
</li>
</ul>
</li>
<li>Account - 1
<ul>
<li>
<details>
<summary>
<code>AccountLinked(who, identity)</code> - 0</summary>
account linked to identity

```rust
who: T::AccountId
identity: IdtyIdOf<T>
```

</details>
</li>
<li>
<details>
<summary>
<code>AccountUnlinked()</code> - 1</summary>
The account was unlinked from its identity.

```rust
: T::AccountId
```

</details>
</li>
</ul>
</li>
<li>Scheduler - 2
<ul>
<li>
<details>
<summary>
<code>Scheduled(when, index)</code> - 0</summary>
Scheduled some task.

```rust
when: BlockNumberFor<T>
index: u32
```

</details>
</li>
<li>
<details>
<summary>
<code>Canceled(when, index)</code> - 1</summary>
Canceled some task.

```rust
when: BlockNumberFor<T>
index: u32
```

</details>
</li>
<li>
<details>
<summary>
<code>Dispatched(task, id, result)</code> - 2</summary>
Dispatched some task.

```rust
task: TaskAddress<BlockNumberFor<T>>
id: Option<TaskName>
result: DispatchResult
```

</details>
</li>
<li>
<details>
<summary>
<code>RetrySet(task, id, period, retries)</code> - 3</summary>
Set a retry configuration for some task.

```rust
task: TaskAddress<BlockNumberFor<T>>
id: Option<TaskName>
period: BlockNumberFor<T>
retries: u8
```

</details>
</li>
<li>
<details>
<summary>
<code>RetryCancelled(task, id)</code> - 4</summary>
Cancel a retry configuration for some task.

```rust
task: TaskAddress<BlockNumberFor<T>>
id: Option<TaskName>
```

</details>
</li>
<li>
<details>
<summary>
<code>CallUnavailable(task, id)</code> - 5</summary>
The call for the provided hash was not found so the task has been aborted.

```rust
task: TaskAddress<BlockNumberFor<T>>
id: Option<TaskName>
```

</details>
</li>
<li>
<details>
<summary>
<code>PeriodicFailed(task, id)</code> - 6</summary>
The given task was unable to be renewed since the agenda is full at that block.

```rust
task: TaskAddress<BlockNumberFor<T>>
id: Option<TaskName>
```

</details>
</li>
<li>
<details>
<summary>
<code>RetryFailed(task, id)</code> - 7</summary>
The given task was unable to be retried since the agenda is full at that block or there
was not enough weight to reschedule it.

```rust
task: TaskAddress<BlockNumberFor<T>>
id: Option<TaskName>
```

</details>
</li>
<li>
<details>
<summary>
<code>PermanentlyOverweight(task, id)</code> - 8</summary>
The given task can never be executed since it is overweight.

```rust
task: TaskAddress<BlockNumberFor<T>>
id: Option<TaskName>
```

</details>
</li>
</ul>
</li>
<li>Babe - 3
<ul>
</ul>
</li>
<li>Timestamp - 4
<ul>
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
<code>Endowed(account, free_balance)</code> - 0</summary>
An account was created with some free balance.

```rust
account: T::AccountId
free_balance: T::Balance
```

</details>
</li>
<li>
<details>
<summary>
<code>DustLost(account, amount)</code> - 1</summary>
An account was removed whose balance was non-zero but below ExistentialDeposit,
resulting in an outright loss.

```rust
account: T::AccountId
amount: T::Balance
```

</details>
</li>
<li>
<details>
<summary>
<code>Transfer(from, to, amount)</code> - 2</summary>
Transfer succeeded.

```rust
from: T::AccountId
to: T::AccountId
amount: T::Balance
```

</details>
</li>
<li>
<details>
<summary>
<code>BalanceSet(who, free)</code> - 3</summary>
A balance was set by root.

```rust
who: T::AccountId
free: T::Balance
```

</details>
</li>
<li>
<details>
<summary>
<code>Reserved(who, amount)</code> - 4</summary>
Some balance was reserved (moved from free to reserved).

```rust
who: T::AccountId
amount: T::Balance
```

</details>
</li>
<li>
<details>
<summary>
<code>Unreserved(who, amount)</code> - 5</summary>
Some balance was unreserved (moved from reserved to free).

```rust
who: T::AccountId
amount: T::Balance
```

</details>
</li>
<li>
<details>
<summary>
<code>ReserveRepatriated(from, to, amount, destination_status)</code> - 6</summary>
Some balance was moved from the reserve of the first account to the second account.
Final argument indicates the destination balance type.

```rust
from: T::AccountId
to: T::AccountId
amount: T::Balance
destination_status: Status
```

</details>
</li>
<li>
<details>
<summary>
<code>Deposit(who, amount)</code> - 7</summary>
Some amount was deposited (e.g. for transaction fees).

```rust
who: T::AccountId
amount: T::Balance
```

</details>
</li>
<li>
<details>
<summary>
<code>Withdraw(who, amount)</code> - 8</summary>
Some amount was withdrawn from the account (e.g. for transaction fees).

```rust
who: T::AccountId
amount: T::Balance
```

</details>
</li>
<li>
<details>
<summary>
<code>Slashed(who, amount)</code> - 9</summary>
Some amount was removed from the account (e.g. for misbehavior).

```rust
who: T::AccountId
amount: T::Balance
```

</details>
</li>
<li>
<details>
<summary>
<code>Minted(who, amount)</code> - 10</summary>
Some amount was minted into an account.

```rust
who: T::AccountId
amount: T::Balance
```

</details>
</li>
<li>
<details>
<summary>
<code>Burned(who, amount)</code> - 11</summary>
Some amount was burned from an account.

```rust
who: T::AccountId
amount: T::Balance
```

</details>
</li>
<li>
<details>
<summary>
<code>Suspended(who, amount)</code> - 12</summary>
Some amount was suspended from an account (it can be restored later).

```rust
who: T::AccountId
amount: T::Balance
```

</details>
</li>
<li>
<details>
<summary>
<code>Restored(who, amount)</code> - 13</summary>
Some amount was restored into an account.

```rust
who: T::AccountId
amount: T::Balance
```

</details>
</li>
<li>
<details>
<summary>
<code>Upgraded(who)</code> - 14</summary>
An account was upgraded.

```rust
who: T::AccountId
```

</details>
</li>
<li>
<details>
<summary>
<code>Issued(amount)</code> - 15</summary>
Total issuance was increased by `amount`, creating a credit to be balanced.

```rust
amount: T::Balance
```

</details>
</li>
<li>
<details>
<summary>
<code>Rescinded(amount)</code> - 16</summary>
Total issuance was decreased by `amount`, creating a debt to be balanced.

```rust
amount: T::Balance
```

</details>
</li>
<li>
<details>
<summary>
<code>Locked(who, amount)</code> - 17</summary>
Some balance was locked.

```rust
who: T::AccountId
amount: T::Balance
```

</details>
</li>
<li>
<details>
<summary>
<code>Unlocked(who, amount)</code> - 18</summary>
Some balance was unlocked.

```rust
who: T::AccountId
amount: T::Balance
```

</details>
</li>
<li>
<details>
<summary>
<code>Frozen(who, amount)</code> - 19</summary>
Some balance was frozen.

```rust
who: T::AccountId
amount: T::Balance
```

</details>
</li>
<li>
<details>
<summary>
<code>Thawed(who, amount)</code> - 20</summary>
Some balance was thawed.

```rust
who: T::AccountId
amount: T::Balance
```

</details>
</li>
<li>
<details>
<summary>
<code>TotalIssuanceForced(old, new)</code> - 21</summary>
The `TotalIssuance` was forcefully changed.

```rust
old: T::Balance
new: T::Balance
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
<code>TransactionFeePaid(who, actual_fee, tip)</code> - 0</summary>
A transaction fee `actual_fee`, of which `tip` was added to the minimum inclusion fee,
has been paid by `who`.

```rust
who: T::AccountId
actual_fee: BalanceOf<T>
tip: BalanceOf<T>
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
<code>OneshotAccountCreated(account, balance, creator)</code> - 0</summary>
A oneshot account was created.

```rust
account: T::AccountId
balance: <T::Currency as Currency<T::AccountId>>::Balance
creator: T::AccountId
```

</details>
</li>
<li>
<details>
<summary>
<code>OneshotAccountConsumed(account, dest1, dest2)</code> - 1</summary>
A oneshot account was consumed.

```rust
account: T::AccountId
dest1: (T::AccountId,<T::Currency as Currency<T::AccountId>>::Balance,)
dest2: Option<
(T::AccountId,<T::Currency as Currency<T::AccountId>>::Balance,)
>
```

</details>
</li>
<li>
<details>
<summary>
<code>Withdraw(account, balance)</code> - 2</summary>
A withdrawal was executed on a oneshot account.

```rust
account: T::AccountId
balance: <T::Currency as Currency<T::AccountId>>::Balance
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
<code>Refunded(who, identity, amount)</code> - 0</summary>
Transaction fees were refunded.

```rust
who: T::AccountId
identity: IdtyId<T>
amount: BalanceOf<T>
```

</details>
</li>
<li>
<details>
<summary>
<code>NoQuotaForIdty()</code> - 1</summary>
No more quota available for refund.

```rust
: IdtyId<T>
```

</details>
</li>
<li>
<details>
<summary>
<code>NoMoreCurrencyForRefund()</code> - 2</summary>
No more currency available for refund.
This scenario should never occur if the fees are intended for the refund account.

```rust
no args
```

</details>
</li>
<li>
<details>
<summary>
<code>RefundFailed()</code> - 3</summary>
The refund has failed.
This scenario should rarely occur, except when the account was destroyed in the interim between the request and the refund.

```rust
: T::AccountId
```

</details>
</li>
<li>
<details>
<summary>
<code>RefundQueueFull()</code> - 4</summary>
Refund queue was full.

```rust
no args
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
<code>InvitationSent(issuer, receiver)</code> - 0</summary>
An identity is being inivited to become a smith.

```rust
issuer: T::IdtyIndex
receiver: T::IdtyIndex
```

</details>
</li>
<li>
<details>
<summary>
<code>InvitationAccepted(idty_index)</code> - 1</summary>
The invitation has been accepted.

```rust
idty_index: T::IdtyIndex
```

</details>
</li>
<li>
<details>
<summary>
<code>SmithCertAdded(issuer, receiver)</code> - 2</summary>
Certification received

```rust
issuer: T::IdtyIndex
receiver: T::IdtyIndex
```

</details>
</li>
<li>
<details>
<summary>
<code>SmithCertRemoved(issuer, receiver)</code> - 3</summary>
Certification lost

```rust
issuer: T::IdtyIndex
receiver: T::IdtyIndex
```

</details>
</li>
<li>
<details>
<summary>
<code>SmithMembershipAdded(idty_index)</code> - 4</summary>
A smith gathered enough certifications to become an authority (can call `go_online()`).

```rust
idty_index: T::IdtyIndex
```

</details>
</li>
<li>
<details>
<summary>
<code>SmithMembershipRemoved(idty_index)</code> - 5</summary>
A smith has been removed from the smiths set.

```rust
idty_index: T::IdtyIndex
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
<code>IncomingAuthorities(members)</code> - 0</summary>
List of members scheduled to join the set of authorities in the next session.

```rust
members: Vec<T::MemberId>
```

</details>
</li>
<li>
<details>
<summary>
<code>OutgoingAuthorities(members)</code> - 1</summary>
List of members leaving the set of authorities in the next session.

```rust
members: Vec<T::MemberId>
```

</details>
</li>
<li>
<details>
<summary>
<code>MemberGoOffline(member)</code> - 2</summary>
A member will leave the set of authorities in 2 sessions.

```rust
member: T::MemberId
```

</details>
</li>
<li>
<details>
<summary>
<code>MemberGoOnline(member)</code> - 3</summary>
A member will join the set of authorities in 2 sessions.

```rust
member: T::MemberId
```

</details>
</li>
<li>
<details>
<summary>
<code>MemberRemoved(member)</code> - 4</summary>
A member, who no longer has authority rights, will be removed from the authority set in 2 sessions.

```rust
member: T::MemberId
```

</details>
</li>
<li>
<details>
<summary>
<code>MemberRemovedFromBlacklist(member)</code> - 5</summary>
A member has been removed from the blacklist.

```rust
member: T::MemberId
```

</details>
</li>
<li>
<details>
<summary>
<code>MemberAddedToBlacklist(member)</code> - 6</summary>
A member has been blacklisted.

```rust
member: T::MemberId
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
<li>
<details>
<summary>
<code>Offence(kind, timeslot)</code> - 0</summary>
An offense was reported during the specified time slot. This event is not deposited for duplicate slashes.

```rust
kind: Kind
timeslot: OpaqueTimeSlot
```

</details>
</li>
</ul>
</li>
<li>Historical - 14
<ul>
</ul>
</li>
<li>Session - 15
<ul>
<li>
<details>
<summary>
<code>NewSession(session_index)</code> - 0</summary>
New session has happened. Note that the argument is the session index, not the
block number as the type might suggest.

```rust
session_index: SessionIndex
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
<code>NewAuthorities(authority_set)</code> - 0</summary>
New authority set has been applied.

```rust
authority_set: AuthorityList
```

</details>
</li>
<li>
<details>
<summary>
<code>Paused()</code> - 1</summary>
Current authority set has been paused.

```rust
no args
```

</details>
</li>
<li>
<details>
<summary>
<code>Resumed()</code> - 2</summary>
Current authority set has been resumed.

```rust
no args
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
<code>HeartbeatReceived(authority_id)</code> - 0</summary>
A new heartbeat was received from `AuthorityId`.

```rust
authority_id: T::AuthorityId
```

</details>
</li>
<li>
<details>
<summary>
<code>AllGood()</code> - 1</summary>
At the end of the session, no offence was committed.

```rust
no args
```

</details>
</li>
<li>
<details>
<summary>
<code>SomeOffline(offline)</code> - 2</summary>
At the end of the session, at least one validator was found to be offline.

```rust
offline: Vec<IdentificationTuple<T>>
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
<li>
<details>
<summary>
<code>Sudid(sudo_result)</code> - 0</summary>
A sudo call just took place.

```rust
sudo_result: DispatchResult
```

</details>
</li>
<li>
<details>
<summary>
<code>KeyChanged(old, new)</code> - 1</summary>
The sudo key has been updated.

```rust
old: Option<T::AccountId>
new: T::AccountId
```

</details>
</li>
<li>
<details>
<summary>
<code>KeyRemoved()</code> - 2</summary>
The key was permanently removed.

```rust
no args
```

</details>
</li>
<li>
<details>
<summary>
<code>SudoAsDone(sudo_result)</code> - 3</summary>
A [sudo_as](Pallet::sudo_as) call just took place.

```rust
sudo_result: DispatchResult
```

</details>
</li>
</ul>
</li>
<li>UpgradeOrigin - 21
<ul>
<li>
<details>
<summary>
<code>DispatchedAsRoot(result)</code> - 0</summary>
A call was dispatched as root from an upgradable origin

```rust
result: DispatchResult
```

</details>
</li>
</ul>
</li>
<li>Preimage - 22
<ul>
<li>
<details>
<summary>
<code>Noted(hash)</code> - 0</summary>
A preimage has been noted.

```rust
hash: T::Hash
```

</details>
</li>
<li>
<details>
<summary>
<code>Requested(hash)</code> - 1</summary>
A preimage has been requested.

```rust
hash: T::Hash
```

</details>
</li>
<li>
<details>
<summary>
<code>Cleared(hash)</code> - 2</summary>
A preimage has ben cleared.

```rust
hash: T::Hash
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
<code>Proposed(account, proposal_index, proposal_hash, threshold)</code> - 0</summary>
A motion (given hash) has been proposed (by given account) with a threshold (given
`MemberCount`).

```rust
account: T::AccountId
proposal_index: ProposalIndex
proposal_hash: T::Hash
threshold: MemberCount
```

</details>
</li>
<li>
<details>
<summary>
<code>Voted(account, proposal_hash, voted, yes, no)</code> - 1</summary>
A motion (given hash) has been voted on by given account, leaving
a tally (yes votes and no votes given respectively as `MemberCount`).

```rust
account: T::AccountId
proposal_hash: T::Hash
voted: bool
yes: MemberCount
no: MemberCount
```

</details>
</li>
<li>
<details>
<summary>
<code>Approved(proposal_hash)</code> - 2</summary>
A motion was approved by the required threshold.

```rust
proposal_hash: T::Hash
```

</details>
</li>
<li>
<details>
<summary>
<code>Disapproved(proposal_hash)</code> - 3</summary>
A motion was not approved by the required threshold.

```rust
proposal_hash: T::Hash
```

</details>
</li>
<li>
<details>
<summary>
<code>Executed(proposal_hash, result)</code> - 4</summary>
A motion was executed; result will be `Ok` if it returned without error.

```rust
proposal_hash: T::Hash
result: DispatchResult
```

</details>
</li>
<li>
<details>
<summary>
<code>MemberExecuted(proposal_hash, result)</code> - 5</summary>
A single member did some action; result will be `Ok` if it returned without error.

```rust
proposal_hash: T::Hash
result: DispatchResult
```

</details>
</li>
<li>
<details>
<summary>
<code>Closed(proposal_hash, yes, no)</code> - 6</summary>
A proposal was closed because its threshold was reached or after its duration was up.

```rust
proposal_hash: T::Hash
yes: MemberCount
no: MemberCount
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
<code>NewUdCreated(amount, index, monetary_mass, members_count)</code> - 0</summary>
A new universal dividend is created.

```rust
amount: BalanceOf<T>
index: UdIndex
monetary_mass: BalanceOf<T>
members_count: BalanceOf<T>
```

</details>
</li>
<li>
<details>
<summary>
<code>UdReevalued(new_ud_amount, monetary_mass, members_count)</code> - 1</summary>
The universal dividend has been re-evaluated.

```rust
new_ud_amount: BalanceOf<T>
monetary_mass: BalanceOf<T>
members_count: BalanceOf<T>
```

</details>
</li>
<li>
<details>
<summary>
<code>UdsAutoPaid(count, total, who)</code> - 2</summary>
DUs were automatically transferred as part of a member removal.

```rust
count: UdIndex
total: BalanceOf<T>
who: T::AccountId
```

</details>
</li>
<li>
<details>
<summary>
<code>UdsClaimed(count, total, who)</code> - 3</summary>
A member claimed his UDs.

```rust
count: UdIndex
total: BalanceOf<T>
who: T::AccountId
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
<code>IdtyCreated(idty_index, owner_key)</code> - 0</summary>
A new identity has been created.

```rust
idty_index: T::IdtyIndex
owner_key: T::AccountId
```

</details>
</li>
<li>
<details>
<summary>
<code>IdtyConfirmed(idty_index, owner_key, name)</code> - 1</summary>
An identity has been confirmed by its owner.

```rust
idty_index: T::IdtyIndex
owner_key: T::AccountId
name: IdtyName
```

</details>
</li>
<li>
<details>
<summary>
<code>IdtyValidated(idty_index)</code> - 2</summary>
An identity has been validated.

```rust
idty_index: T::IdtyIndex
```

</details>
</li>
<li>
<details>
<summary>
<code>IdtyChangedOwnerKey(idty_index, new_owner_key)</code> - 3</summary>


```rust
idty_index: T::IdtyIndex
new_owner_key: T::AccountId
```

</details>
</li>
<li>
<details>
<summary>
<code>IdtyRevoked(idty_index, reason)</code> - 4</summary>
An identity has been revoked.

```rust
idty_index: T::IdtyIndex
reason: RevocationReason
```

</details>
</li>
<li>
<details>
<summary>
<code>IdtyRemoved(idty_index, reason)</code> - 5</summary>
An identity has been removed.

```rust
idty_index: T::IdtyIndex
reason: RemovalReason
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
<code>MembershipAdded(member, expire_on)</code> - 0</summary>
A membership was added.

```rust
member: T::IdtyId
expire_on: BlockNumberFor<T>
```

</details>
</li>
<li>
<details>
<summary>
<code>MembershipRenewed(member, expire_on)</code> - 1</summary>
A membership was renewed.

```rust
member: T::IdtyId
expire_on: BlockNumberFor<T>
```

</details>
</li>
<li>
<details>
<summary>
<code>MembershipRemoved(member, reason)</code> - 2</summary>
A membership was removed.

```rust
member: T::IdtyId
reason: MembershipRemovalReason
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
<code>CertAdded(issuer, receiver)</code> - 0</summary>
A new certification was added.

```rust
issuer: T::IdtyIndex
receiver: T::IdtyIndex
```

</details>
</li>
<li>
<details>
<summary>
<code>CertRemoved(issuer, receiver, expiration)</code> - 1</summary>
A certification was removed.

```rust
issuer: T::IdtyIndex
receiver: T::IdtyIndex
expiration: bool
```

</details>
</li>
<li>
<details>
<summary>
<code>CertRenewed(issuer, receiver)</code> - 2</summary>
A certification was renewed.

```rust
issuer: T::IdtyIndex
receiver: T::IdtyIndex
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
<code>EvaluationRequested(idty_index, who)</code> - 0</summary>
A distance evaluation was requested.

```rust
idty_index: T::IdtyIndex
who: T::AccountId
```

</details>
</li>
<li>
<details>
<summary>
<code>EvaluatedValid(idty_index, distance)</code> - 1</summary>
Distance rule was found valid.

```rust
idty_index: T::IdtyIndex
distance: Perbill
```

</details>
</li>
<li>
<details>
<summary>
<code>EvaluatedInvalid(idty_index, distance)</code> - 2</summary>
Distance rule was found invalid.

```rust
idty_index: T::IdtyIndex
distance: Perbill
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
<code>NewSwap(account, proof, swap)</code> - 0</summary>
Swap created.

```rust
account: T::AccountId
proof: HashedProof
swap: PendingSwap<T>
```

</details>
</li>
<li>
<details>
<summary>
<code>SwapClaimed(account, proof, success)</code> - 1</summary>
Swap claimed. The last parameter indicates whether the execution succeeds.

```rust
account: T::AccountId
proof: HashedProof
success: bool
```

</details>
</li>
<li>
<details>
<summary>
<code>SwapCancelled(account, proof)</code> - 2</summary>
Swap cancelled.

```rust
account: T::AccountId
proof: HashedProof
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
<code>NewMultisig(approving, multisig, call_hash)</code> - 0</summary>
A new multisig operation has begun.

```rust
approving: T::AccountId
multisig: T::AccountId
call_hash: CallHash
```

</details>
</li>
<li>
<details>
<summary>
<code>MultisigApproval(approving, timepoint, multisig, call_hash)</code> - 1</summary>
A multisig operation has been approved by someone.

```rust
approving: T::AccountId
timepoint: Timepoint<BlockNumberFor<T>>
multisig: T::AccountId
call_hash: CallHash
```

</details>
</li>
<li>
<details>
<summary>
<code>MultisigExecuted(approving, timepoint, multisig, call_hash, result)</code> - 2</summary>
A multisig operation has been executed.

```rust
approving: T::AccountId
timepoint: Timepoint<BlockNumberFor<T>>
multisig: T::AccountId
call_hash: CallHash
result: DispatchResult
```

</details>
</li>
<li>
<details>
<summary>
<code>MultisigCancelled(cancelling, timepoint, multisig, call_hash)</code> - 3</summary>
A multisig operation has been cancelled.

```rust
cancelling: T::AccountId
timepoint: Timepoint<BlockNumberFor<T>>
multisig: T::AccountId
call_hash: CallHash
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
<code>FilledRandomness(request_id, randomness)</code> - 0</summary>
A request for randomness was fulfilled.

```rust
request_id: RequestId
randomness: H256
```

</details>
</li>
<li>
<details>
<summary>
<code>RequestedRandomness(request_id, salt, r#type)</code> - 1</summary>
A request for randomness was made.

```rust
request_id: RequestId
salt: H256
r#type: RandomnessType
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
<code>ProxyExecuted(result)</code> - 0</summary>
A proxy was executed correctly, with the given.

```rust
result: DispatchResult
```

</details>
</li>
<li>
<details>
<summary>
<code>PureCreated(pure, who, proxy_type, disambiguation_index)</code> - 1</summary>
A pure account has been created by new proxy with given
disambiguation index and proxy type.

```rust
pure: T::AccountId
who: T::AccountId
proxy_type: T::ProxyType
disambiguation_index: u16
```

</details>
</li>
<li>
<details>
<summary>
<code>Announced(real, proxy, call_hash)</code> - 2</summary>
An announcement was placed to make a call in the future.

```rust
real: T::AccountId
proxy: T::AccountId
call_hash: CallHashOf<T>
```

</details>
</li>
<li>
<details>
<summary>
<code>ProxyAdded(delegator, delegatee, proxy_type, delay)</code> - 3</summary>
A proxy was added.

```rust
delegator: T::AccountId
delegatee: T::AccountId
proxy_type: T::ProxyType
delay: BlockNumberFor<T>
```

</details>
</li>
<li>
<details>
<summary>
<code>ProxyRemoved(delegator, delegatee, proxy_type, delay)</code> - 4</summary>
A proxy was removed.

```rust
delegator: T::AccountId
delegatee: T::AccountId
proxy_type: T::ProxyType
delay: BlockNumberFor<T>
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
<code>BatchInterrupted(index, error)</code> - 0</summary>
Batch of dispatches did not complete fully. Index of first failing dispatch given, as
well as the error.

```rust
index: u32
error: DispatchError
```

</details>
</li>
<li>
<details>
<summary>
<code>BatchCompleted()</code> - 1</summary>
Batch of dispatches completed fully with no error.

```rust
no args
```

</details>
</li>
<li>
<details>
<summary>
<code>BatchCompletedWithErrors()</code> - 2</summary>
Batch of dispatches completed but has errors.

```rust
no args
```

</details>
</li>
<li>
<details>
<summary>
<code>ItemCompleted()</code> - 3</summary>
A single item within a Batch of dispatches has completed with no error.

```rust
no args
```

</details>
</li>
<li>
<details>
<summary>
<code>ItemFailed(error)</code> - 4</summary>
A single item within a Batch of dispatches has completed with error.

```rust
error: DispatchError
```

</details>
</li>
<li>
<details>
<summary>
<code>DispatchedAs(result)</code> - 5</summary>
A call was dispatched.

```rust
result: DispatchResult
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
<code>Proposed(proposal_index)</code> - 0</summary>
New proposal.

```rust
proposal_index: ProposalIndex
```

</details>
</li>
<li>
<details>
<summary>
<code>Spending(budget_remaining)</code> - 1</summary>
We have ended a spend period and will now allocate funds.

```rust
budget_remaining: BalanceOf<T, I>
```

</details>
</li>
<li>
<details>
<summary>
<code>Awarded(proposal_index, award, account)</code> - 2</summary>
Some funds have been allocated.

```rust
proposal_index: ProposalIndex
award: BalanceOf<T, I>
account: T::AccountId
```

</details>
</li>
<li>
<details>
<summary>
<code>Rejected(proposal_index, slashed)</code> - 3</summary>
A proposal was rejected; funds were slashed.

```rust
proposal_index: ProposalIndex
slashed: BalanceOf<T, I>
```

</details>
</li>
<li>
<details>
<summary>
<code>Burnt(burnt_funds)</code> - 4</summary>
Some of our funds have been burnt.

```rust
burnt_funds: BalanceOf<T, I>
```

</details>
</li>
<li>
<details>
<summary>
<code>Rollover(rollover_balance)</code> - 5</summary>
Spending has finished; this is the amount that rolls over until next spend.

```rust
rollover_balance: BalanceOf<T, I>
```

</details>
</li>
<li>
<details>
<summary>
<code>Deposit(value)</code> - 6</summary>
Some funds have been deposited.

```rust
value: BalanceOf<T, I>
```

</details>
</li>
<li>
<details>
<summary>
<code>SpendApproved(proposal_index, amount, beneficiary)</code> - 7</summary>
A new spend proposal has been approved.

```rust
proposal_index: ProposalIndex
amount: BalanceOf<T, I>
beneficiary: T::AccountId
```

</details>
</li>
<li>
<details>
<summary>
<code>UpdatedInactive(reactivated, deactivated)</code> - 8</summary>
The inactive funds of the pallet have been updated.

```rust
reactivated: BalanceOf<T, I>
deactivated: BalanceOf<T, I>
```

</details>
</li>
<li>
<details>
<summary>
<code>AssetSpendApproved(index, asset_kind, amount, beneficiary, valid_from, expire_at)</code> - 9</summary>
A new asset spend proposal has been approved.

```rust
index: SpendIndex
asset_kind: T::AssetKind
amount: AssetBalanceOf<T, I>
beneficiary: T::Beneficiary
valid_from: BlockNumberFor<T>
expire_at: BlockNumberFor<T>
```

</details>
</li>
<li>
<details>
<summary>
<code>AssetSpendVoided(index)</code> - 10</summary>
An approved spend was voided.

```rust
index: SpendIndex
```

</details>
</li>
<li>
<details>
<summary>
<code>Paid(index, payment_id)</code> - 11</summary>
A payment happened.

```rust
index: SpendIndex
payment_id: <T::Paymaster as Pay>::Id
```

</details>
</li>
<li>
<details>
<summary>
<code>PaymentFailed(index, payment_id)</code> - 12</summary>
A payment failed and can be retried.

```rust
index: SpendIndex
payment_id: <T::Paymaster as Pay>::Id
```

</details>
</li>
<li>
<details>
<summary>
<code>SpendProcessed(index)</code> - 13</summary>
A spend was processed and removed from the storage. It might have been successfully
paid or it may have expired.

```rust
index: SpendIndex
```

</details>
</li>
</ul>
</li>
</ul>