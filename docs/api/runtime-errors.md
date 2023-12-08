# Runtime errors

There are **178** errors from **37** pallets.

<ul>
<li>System - 0
<ul>
<li>
<details>
<summary>
<code>InvalidSpecName</code> - 0</summary>
The name of specification does not match between the current runtime
and the new runtime.
</details>
</li>
<li>
<details>
<summary>
<code>SpecVersionNeedsToIncrease</code> - 1</summary>
The specification version is not allowed to decrease between the current runtime
and the new runtime.
</details>
</li>
<li>
<details>
<summary>
<code>FailedToExtractRuntimeVersion</code> - 2</summary>
Failed to extract the runtime version from the new runtime.

Either calling `Core_version` or decoding `RuntimeVersion` failed.
</details>
</li>
<li>
<details>
<summary>
<code>NonDefaultComposite</code> - 3</summary>
Suicide called when the account has non-default composite data.
</details>
</li>
<li>
<details>
<summary>
<code>NonZeroRefCount</code> - 4</summary>
There is a non-zero reference count preventing the account from being purged.
</details>
</li>
<li>
<details>
<summary>
<code>CallFiltered</code> - 5</summary>
The origin filter prevent the call to be dispatched.
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
<code>FailedToSchedule</code> - 0</summary>
Failed to schedule a call
</details>
</li>
<li>
<details>
<summary>
<code>NotFound</code> - 1</summary>
Cannot find the scheduled call.
</details>
</li>
<li>
<details>
<summary>
<code>TargetBlockNumberInPast</code> - 2</summary>
Given target block number is in the past.
</details>
</li>
<li>
<details>
<summary>
<code>RescheduleNoChange</code> - 3</summary>
Reschedule failed because it does not change scheduled time.
</details>
</li>
<li>
<details>
<summary>
<code>Named</code> - 4</summary>
Attempt to use a non-named function on a named task.
</details>
</li>
</ul>
</li>
<li>Babe - 3
<ul>
<li>
<details>
<summary>
<code>InvalidEquivocationProof</code> - 0</summary>
An equivocation proof provided as part of an equivocation report is invalid.
</details>
</li>
<li>
<details>
<summary>
<code>InvalidKeyOwnershipProof</code> - 1</summary>
A key ownership proof provided as part of an equivocation report is invalid.
</details>
</li>
<li>
<details>
<summary>
<code>DuplicateOffenceReport</code> - 2</summary>
A given equivocation report is valid but already previously reported.
</details>
</li>
<li>
<details>
<summary>
<code>InvalidConfiguration</code> - 3</summary>
Submitted configuration is invalid.
</details>
</li>
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
<code>VestingBalance</code> - 0</summary>
Vesting balance too high to send value.
</details>
</li>
<li>
<details>
<summary>
<code>LiquidityRestrictions</code> - 1</summary>
Account liquidity restrictions prevent withdrawal.
</details>
</li>
<li>
<details>
<summary>
<code>InsufficientBalance</code> - 2</summary>
Balance too low to send value.
</details>
</li>
<li>
<details>
<summary>
<code>ExistentialDeposit</code> - 3</summary>
Value too low to create account due to existential deposit.
</details>
</li>
<li>
<details>
<summary>
<code>Expendability</code> - 4</summary>
Transfer/payment would kill account.
</details>
</li>
<li>
<details>
<summary>
<code>ExistingVestingSchedule</code> - 5</summary>
A vesting schedule already exists for this account.
</details>
</li>
<li>
<details>
<summary>
<code>DeadAccount</code> - 6</summary>
Beneficiary account must pre-exist.
</details>
</li>
<li>
<details>
<summary>
<code>TooManyReserves</code> - 7</summary>
Number of named reserves exceed `MaxReserves`.
</details>
</li>
<li>
<details>
<summary>
<code>TooManyHolds</code> - 8</summary>
Number of holds exceed `MaxHolds`.
</details>
</li>
<li>
<details>
<summary>
<code>TooManyFreezes</code> - 9</summary>
Number of freezes exceed `MaxFreezes`.
</details>
</li>
</ul>
</li>
<li>TransactionPayment - 32
<ul>
</ul>
</li>
<li>OneshotAccount - 7
<ul>
<li>
<details>
<summary>
<code>BlockHeightInFuture</code> - 0</summary>
Block height is in the future.
</details>
</li>
<li>
<details>
<summary>
<code>BlockHeightTooOld</code> - 1</summary>
Block height is too old.
</details>
</li>
<li>
<details>
<summary>
<code>DestAccountNotExist</code> - 2</summary>
Destination account does not exist.
</details>
</li>
<li>
<details>
<summary>
<code>ExistentialDeposit</code> - 3</summary>
Destination account has a balance less than the existential deposit.
</details>
</li>
<li>
<details>
<summary>
<code>InsufficientBalance</code> - 4</summary>
Source account has insufficient balance.
</details>
</li>
<li>
<details>
<summary>
<code>OneshotAccountAlreadyCreated</code> - 5</summary>
Destination oneshot account already exists.
</details>
</li>
<li>
<details>
<summary>
<code>OneshotAccountNotExist</code> - 6</summary>
Source oneshot account does not exist.
</details>
</li>
</ul>
</li>
<li>Quota - 66
<ul>
</ul>
</li>
<li>AuthorityMembers - 10
<ul>
<li>
<details>
<summary>
<code>AlreadyIncoming</code> - 0</summary>
Member already incoming
</details>
</li>
<li>
<details>
<summary>
<code>AlreadyOnline</code> - 1</summary>
Member already online
</details>
</li>
<li>
<details>
<summary>
<code>AlreadyOutgoing</code> - 2</summary>
Member already outgoing
</details>
</li>
<li>
<details>
<summary>
<code>MemberIdNotFound</code> - 3</summary>
Owner key is invalid as a member.
</details>
</li>
<li>
<details>
<summary>
<code>MemberBlacklisted</code> - 4</summary>
Member is blacklisted
</details>
</li>
<li>
<details>
<summary>
<code>MemberNotBlacklisted</code> - 5</summary>
Member is not blacklisted
</details>
</li>
<li>
<details>
<summary>
<code>MemberNotFound</code> - 6</summary>
Member not found
</details>
</li>
<li>
<details>
<summary>
<code>NotOnlineNorIncoming</code> - 7</summary>
Neither online nor scheduled
</details>
</li>
<li>
<details>
<summary>
<code>NotMember</code> - 8</summary>
Not member
</details>
</li>
<li>
<details>
<summary>
<code>SessionKeysNotProvided</code> - 9</summary>
Session keys not provided
</details>
</li>
<li>
<details>
<summary>
<code>TooManyAuthorities</code> - 10</summary>
Too many authorities.
</details>
</li>
</ul>
</li>
<li>Authorship - 11
<ul>
</ul>
</li>
<li>Offences - 12
<ul>
</ul>
</li>
<li>Historical - 13
<ul>
</ul>
</li>
<li>Session - 14
<ul>
<li>
<details>
<summary>
<code>InvalidProof</code> - 0</summary>
Invalid ownership proof.
</details>
</li>
<li>
<details>
<summary>
<code>NoAssociatedValidatorId</code> - 1</summary>
No associated validator ID for account.
</details>
</li>
<li>
<details>
<summary>
<code>DuplicatedKey</code> - 2</summary>
Registered duplicate key.
</details>
</li>
<li>
<details>
<summary>
<code>NoKeys</code> - 3</summary>
No keys are associated with this account.
</details>
</li>
<li>
<details>
<summary>
<code>NoAccount</code> - 4</summary>
Key setting account is not live, so it's impossible to associate keys.
</details>
</li>
</ul>
</li>
<li>Grandpa - 15
<ul>
<li>
<details>
<summary>
<code>PauseFailed</code> - 0</summary>
Attempt to signal GRANDPA pause when the authority set isn't live
(either paused or already pending pause).
</details>
</li>
<li>
<details>
<summary>
<code>ResumeFailed</code> - 1</summary>
Attempt to signal GRANDPA resume when the authority set isn't paused
(either live or already pending resume).
</details>
</li>
<li>
<details>
<summary>
<code>ChangePending</code> - 2</summary>
Attempt to signal GRANDPA change with one already pending.
</details>
</li>
<li>
<details>
<summary>
<code>TooSoon</code> - 3</summary>
Cannot signal forced change so soon after last.
</details>
</li>
<li>
<details>
<summary>
<code>InvalidKeyOwnershipProof</code> - 4</summary>
A key ownership proof provided as part of an equivocation report is invalid.
</details>
</li>
<li>
<details>
<summary>
<code>InvalidEquivocationProof</code> - 5</summary>
An equivocation proof provided as part of an equivocation report is invalid.
</details>
</li>
<li>
<details>
<summary>
<code>DuplicateOffenceReport</code> - 6</summary>
A given equivocation report is valid but already previously reported.
</details>
</li>
</ul>
</li>
<li>ImOnline - 16
<ul>
<li>
<details>
<summary>
<code>InvalidKey</code> - 0</summary>
Non existent public key.
</details>
</li>
<li>
<details>
<summary>
<code>DuplicatedHeartbeat</code> - 1</summary>
Duplicated heartbeat.
</details>
</li>
</ul>
</li>
<li>AuthorityDiscovery - 17
<ul>
</ul>
</li>
<li>Sudo - 20
<ul>
<li>
<details>
<summary>
<code>RequireSudo</code> - 0</summary>
Sender must be the Sudo account
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
<code>TooBig</code> - 0</summary>
Preimage is too large to store on-chain.
</details>
</li>
<li>
<details>
<summary>
<code>AlreadyNoted</code> - 1</summary>
Preimage has already been noted on-chain.
</details>
</li>
<li>
<details>
<summary>
<code>NotAuthorized</code> - 2</summary>
The user is not authorized to perform this action.
</details>
</li>
<li>
<details>
<summary>
<code>NotNoted</code> - 3</summary>
The preimage cannot be removed since it has not yet been noted.
</details>
</li>
<li>
<details>
<summary>
<code>Requested</code> - 4</summary>
A preimage may not be removed when there are outstanding requests.
</details>
</li>
<li>
<details>
<summary>
<code>NotRequested</code> - 5</summary>
The preimage request cannot be removed since no outstanding requests exist.
</details>
</li>
</ul>
</li>
<li>TechnicalCommittee - 23
<ul>
<li>
<details>
<summary>
<code>NotMember</code> - 0</summary>
Account is not a member
</details>
</li>
<li>
<details>
<summary>
<code>DuplicateProposal</code> - 1</summary>
Duplicate proposals not allowed
</details>
</li>
<li>
<details>
<summary>
<code>ProposalMissing</code> - 2</summary>
Proposal must exist
</details>
</li>
<li>
<details>
<summary>
<code>WrongIndex</code> - 3</summary>
Mismatched index
</details>
</li>
<li>
<details>
<summary>
<code>DuplicateVote</code> - 4</summary>
Duplicate vote ignored
</details>
</li>
<li>
<details>
<summary>
<code>AlreadyInitialized</code> - 5</summary>
Members are already initialized!
</details>
</li>
<li>
<details>
<summary>
<code>TooEarly</code> - 6</summary>
The close call was made too early, before the end of the voting.
</details>
</li>
<li>
<details>
<summary>
<code>TooManyProposals</code> - 7</summary>
There can only be a maximum of `MaxProposals` active proposals.
</details>
</li>
<li>
<details>
<summary>
<code>WrongProposalWeight</code> - 8</summary>
The given weight bound for the proposal was too low.
</details>
</li>
<li>
<details>
<summary>
<code>WrongProposalLength</code> - 9</summary>
The given length bound for the proposal was too low.
</details>
</li>
</ul>
</li>
<li>UniversalDividend - 30
<ul>
<li>
<details>
<summary>
<code>AccountNotAllowedToClaimUds</code> - 0</summary>
This account is not allowed to claim UDs.
</details>
</li>
</ul>
</li>
<li>Wot - 40
<ul>
<li>
<details>
<summary>
<code>NotEnoughCertsToClaimMembership</code> - 0</summary>
Insufficient certifications received to claim membership.
</details>
</li>
<li>
<details>
<summary>
<code>DistanceNotOk</code> - 1</summary>
Distance has not received a positive evaluation.
</details>
</li>
<li>
<details>
<summary>
<code>IdtyNotAllowedToRequestMembership</code> - 2</summary>
Identity is not allowed to request membership.
</details>
</li>
<li>
<details>
<summary>
<code>IdtyNotAllowedToRenewMembership</code> - 3</summary>
Identity not allowed to renew membership.
</details>
</li>
<li>
<details>
<summary>
<code>IdtyCreationPeriodNotRespected</code> - 4</summary>
Identity creation period not respected.
</details>
</li>
<li>
<details>
<summary>
<code>NotEnoughReceivedCertsToCreateIdty</code> - 5</summary>
Insufficient received certifications to create identity.
</details>
</li>
<li>
<details>
<summary>
<code>MaxEmittedCertsReached</code> - 6</summary>
Maximum number of emitted certifications reached.
</details>
</li>
<li>
<details>
<summary>
<code>NotAllowedToChangeIdtyAddress</code> - 7</summary>
Not allowed to change identity address.
</details>
</li>
<li>
<details>
<summary>
<code>NotAllowedToRemoveIdty</code> - 8</summary>
Not allowed to remove identity.
</details>
</li>
<li>
<details>
<summary>
<code>IssuerCanNotEmitCert</code> - 9</summary>
Issuer cannot emit a certification because it is not validated.
</details>
</li>
<li>
<details>
<summary>
<code>CertToUndefined</code> - 10</summary>
Cannot issue a certification to an identity without membership or pending membership.
</details>
</li>
<li>
<details>
<summary>
<code>IdtyNotFound</code> - 11</summary>
Issuer or receiver not found.
</details>
</li>
</ul>
</li>
<li>Identity - 41
<ul>
<li>
<details>
<summary>
<code>IdtyAlreadyConfirmed</code> - 0</summary>
Identity already confirmed.
</details>
</li>
<li>
<details>
<summary>
<code>IdtyAlreadyCreated</code> - 1</summary>
Identity already created.
</details>
</li>
<li>
<details>
<summary>
<code>IdtyAlreadyValidated</code> - 2</summary>
Identity already validated.
</details>
</li>
<li>
<details>
<summary>
<code>IdtyIndexNotFound</code> - 3</summary>
Identity index not found.
</details>
</li>
<li>
<details>
<summary>
<code>IdtyNameAlreadyExist</code> - 4</summary>
Identity name already exists.
</details>
</li>
<li>
<details>
<summary>
<code>IdtyNameInvalid</code> - 5</summary>
Invalid identity name.
</details>
</li>
<li>
<details>
<summary>
<code>IdtyNotConfirmedByOwner</code> - 6</summary>
Identity not confirmed by its owner.
</details>
</li>
<li>
<details>
<summary>
<code>IdtyNotFound</code> - 7</summary>
Identity not found.
</details>
</li>
<li>
<details>
<summary>
<code>InvalidSignature</code> - 8</summary>
Invalid payload signature.
</details>
</li>
<li>
<details>
<summary>
<code>InvalidRevocationKey</code> - 9</summary>
Invalid revocation key.
</details>
</li>
<li>
<details>
<summary>
<code>NotRespectIdtyCreationPeriod</code> - 10</summary>
Identity creation period is not respected.
</details>
</li>
<li>
<details>
<summary>
<code>OwnerKeyAlreadyRecentlyChanged</code> - 11</summary>
Owner key already changed recently.
</details>
</li>
<li>
<details>
<summary>
<code>OwnerKeyAlreadyUsed</code> - 12</summary>
Owner key already used.
</details>
</li>
<li>
<details>
<summary>
<code>ProhibitedToRevertToAnOldKey</code> - 13</summary>
Reverting to an old key is prohibited.
</details>
</li>
</ul>
</li>
<li>Membership - 42
<ul>
<li>
<details>
<summary>
<code>IdtyIdNotFound</code> - 0</summary>
Identity ID not found.
</details>
</li>
<li>
<details>
<summary>
<code>MembershipAlreadyAcquired</code> - 1</summary>
Membership already acquired.
</details>
</li>
<li>
<details>
<summary>
<code>MembershipAlreadyRequested</code> - 2</summary>
Membership already requested.
</details>
</li>
<li>
<details>
<summary>
<code>MembershipNotFound</code> - 3</summary>
Membership not found.
</details>
</li>
<li>
<details>
<summary>
<code>MembershipRequestNotFound</code> - 4</summary>
Membership request not found.
</details>
</li>
</ul>
</li>
<li>Cert - 43
<ul>
<li>
<details>
<summary>
<code>CannotCertifySelf</code> - 0</summary>
Identity cannot certify itself
</details>
</li>
<li>
<details>
<summary>
<code>IssuedTooManyCert</code> - 1</summary>
Identity has already issued the maximum number of certifications
</details>
</li>
<li>
<details>
<summary>
<code>IssuerNotFound</code> - 2</summary>
Issuer not found
</details>
</li>
<li>
<details>
<summary>
<code>NotEnoughCertReceived</code> - 3</summary>
Insufficient certifications received.
</details>
</li>
<li>
<details>
<summary>
<code>NotRespectCertPeriod</code> - 4</summary>
Identity has issued a certification too recently.
</details>
</li>
</ul>
</li>
<li>Distance - 44
<ul>
<li>
<details>
<summary>
<code>AlreadyInEvaluation</code> - 0</summary>
Distance is already under evaluation.
</details>
</li>
<li>
<details>
<summary>
<code>TooManyEvaluationsByAuthor</code> - 1</summary>
Too many evaluations requested by author.
</details>
</li>
<li>
<details>
<summary>
<code>TooManyEvaluationsInBlock</code> - 2</summary>
Too many evaluations for this block.
</details>
</li>
<li>
<details>
<summary>
<code>NoAuthor</code> - 3</summary>
No author for this block.
</details>
</li>
<li>
<details>
<summary>
<code>NoIdentity</code> - 4</summary>
Caller has no identity.
</details>
</li>
<li>
<details>
<summary>
<code>QueueFull</code> - 5</summary>
Evaluation queue is full.
</details>
</li>
<li>
<details>
<summary>
<code>TooManyEvaluators</code> - 6</summary>
Too many evaluators in the current evaluation pool.
</details>
</li>
<li>
<details>
<summary>
<code>WrongResultLength</code> - 7</summary>
Evaluation result has a wrong length.
</details>
</li>
</ul>
</li>
<li>SmithSubWot - 50
<ul>
<li>
<details>
<summary>
<code>NotEnoughCertsToClaimMembership</code> - 0</summary>
Insufficient certifications received to claim membership.
</details>
</li>
<li>
<details>
<summary>
<code>DistanceNotOk</code> - 1</summary>
Distance has not received a positive evaluation.
</details>
</li>
<li>
<details>
<summary>
<code>IdtyNotAllowedToRequestMembership</code> - 2</summary>
Identity is not allowed to request membership.
</details>
</li>
<li>
<details>
<summary>
<code>IdtyNotAllowedToRenewMembership</code> - 3</summary>
Identity not allowed to renew membership.
</details>
</li>
<li>
<details>
<summary>
<code>IdtyCreationPeriodNotRespected</code> - 4</summary>
Identity creation period not respected.
</details>
</li>
<li>
<details>
<summary>
<code>NotEnoughReceivedCertsToCreateIdty</code> - 5</summary>
Insufficient received certifications to create identity.
</details>
</li>
<li>
<details>
<summary>
<code>MaxEmittedCertsReached</code> - 6</summary>
Maximum number of emitted certifications reached.
</details>
</li>
<li>
<details>
<summary>
<code>NotAllowedToChangeIdtyAddress</code> - 7</summary>
Not allowed to change identity address.
</details>
</li>
<li>
<details>
<summary>
<code>NotAllowedToRemoveIdty</code> - 8</summary>
Not allowed to remove identity.
</details>
</li>
<li>
<details>
<summary>
<code>IssuerCanNotEmitCert</code> - 9</summary>
Issuer cannot emit a certification because it is not validated.
</details>
</li>
<li>
<details>
<summary>
<code>CertToUndefined</code> - 10</summary>
Cannot issue a certification to an identity without membership or pending membership.
</details>
</li>
<li>
<details>
<summary>
<code>IdtyNotFound</code> - 11</summary>
Issuer or receiver not found.
</details>
</li>
</ul>
</li>
<li>SmithMembership - 52
<ul>
<li>
<details>
<summary>
<code>IdtyIdNotFound</code> - 0</summary>
Identity ID not found.
</details>
</li>
<li>
<details>
<summary>
<code>MembershipAlreadyAcquired</code> - 1</summary>
Membership already acquired.
</details>
</li>
<li>
<details>
<summary>
<code>MembershipAlreadyRequested</code> - 2</summary>
Membership already requested.
</details>
</li>
<li>
<details>
<summary>
<code>MembershipNotFound</code> - 3</summary>
Membership not found.
</details>
</li>
<li>
<details>
<summary>
<code>MembershipRequestNotFound</code> - 4</summary>
Membership request not found.
</details>
</li>
</ul>
</li>
<li>SmithCert - 53
<ul>
<li>
<details>
<summary>
<code>CannotCertifySelf</code> - 0</summary>
Identity cannot certify itself
</details>
</li>
<li>
<details>
<summary>
<code>IssuedTooManyCert</code> - 1</summary>
Identity has already issued the maximum number of certifications
</details>
</li>
<li>
<details>
<summary>
<code>IssuerNotFound</code> - 2</summary>
Issuer not found
</details>
</li>
<li>
<details>
<summary>
<code>NotEnoughCertReceived</code> - 3</summary>
Insufficient certifications received.
</details>
</li>
<li>
<details>
<summary>
<code>NotRespectCertPeriod</code> - 4</summary>
Identity has issued a certification too recently.
</details>
</li>
</ul>
</li>
<li>AtomicSwap - 60
<ul>
<li>
<details>
<summary>
<code>AlreadyExist</code> - 0</summary>
Swap already exists.
</details>
</li>
<li>
<details>
<summary>
<code>InvalidProof</code> - 1</summary>
Swap proof is invalid.
</details>
</li>
<li>
<details>
<summary>
<code>ProofTooLarge</code> - 2</summary>
Proof is too large.
</details>
</li>
<li>
<details>
<summary>
<code>SourceMismatch</code> - 3</summary>
Source does not match.
</details>
</li>
<li>
<details>
<summary>
<code>AlreadyClaimed</code> - 4</summary>
Swap has already been claimed.
</details>
</li>
<li>
<details>
<summary>
<code>NotExist</code> - 5</summary>
Swap does not exist.
</details>
</li>
<li>
<details>
<summary>
<code>ClaimActionMismatch</code> - 6</summary>
Claim action mismatch.
</details>
</li>
<li>
<details>
<summary>
<code>DurationNotPassed</code> - 7</summary>
Duration has not yet passed for the swap to be cancelled.
</details>
</li>
</ul>
</li>
<li>Multisig - 61
<ul>
<li>
<details>
<summary>
<code>MinimumThreshold</code> - 0</summary>
Threshold must be 2 or greater.
</details>
</li>
<li>
<details>
<summary>
<code>AlreadyApproved</code> - 1</summary>
Call is already approved by this signatory.
</details>
</li>
<li>
<details>
<summary>
<code>NoApprovalsNeeded</code> - 2</summary>
Call doesn't need any (more) approvals.
</details>
</li>
<li>
<details>
<summary>
<code>TooFewSignatories</code> - 3</summary>
There are too few signatories in the list.
</details>
</li>
<li>
<details>
<summary>
<code>TooManySignatories</code> - 4</summary>
There are too many signatories in the list.
</details>
</li>
<li>
<details>
<summary>
<code>SignatoriesOutOfOrder</code> - 5</summary>
The signatories were provided out of order; they should be ordered.
</details>
</li>
<li>
<details>
<summary>
<code>SenderInSignatories</code> - 6</summary>
The sender was contained in the other signatories; it shouldn't be.
</details>
</li>
<li>
<details>
<summary>
<code>NotFound</code> - 7</summary>
Multisig operation not found when attempting to cancel.
</details>
</li>
<li>
<details>
<summary>
<code>NotOwner</code> - 8</summary>
Only the account that originally created the multisig is able to cancel it.
</details>
</li>
<li>
<details>
<summary>
<code>NoTimepoint</code> - 9</summary>
No timepoint was given, yet the multisig operation is already underway.
</details>
</li>
<li>
<details>
<summary>
<code>WrongTimepoint</code> - 10</summary>
A different timepoint was given to the multisig operation that is underway.
</details>
</li>
<li>
<details>
<summary>
<code>UnexpectedTimepoint</code> - 11</summary>
A timepoint was given, yet no multisig operation is underway.
</details>
</li>
<li>
<details>
<summary>
<code>MaxWeightTooLow</code> - 12</summary>
The maximum weight information provided was too low.
</details>
</li>
<li>
<details>
<summary>
<code>AlreadyStored</code> - 13</summary>
The data to be stored is already stored.
</details>
</li>
</ul>
</li>
<li>ProvideRandomness - 62
<ul>
<li>
<details>
<summary>
<code>QueueFull</code> - 0</summary>
Request randomness queue is full.
</details>
</li>
</ul>
</li>
<li>Proxy - 63
<ul>
<li>
<details>
<summary>
<code>TooMany</code> - 0</summary>
There are too many proxies registered or too many announcements pending.
</details>
</li>
<li>
<details>
<summary>
<code>NotFound</code> - 1</summary>
Proxy registration not found.
</details>
</li>
<li>
<details>
<summary>
<code>NotProxy</code> - 2</summary>
Sender is not a proxy of the account to be proxied.
</details>
</li>
<li>
<details>
<summary>
<code>Unproxyable</code> - 3</summary>
A call which is incompatible with the proxy type's filter was attempted.
</details>
</li>
<li>
<details>
<summary>
<code>Duplicate</code> - 4</summary>
Account is already a proxy.
</details>
</li>
<li>
<details>
<summary>
<code>NoPermission</code> - 5</summary>
Call may not be made by proxy because it may escalate its privileges.
</details>
</li>
<li>
<details>
<summary>
<code>Unannounced</code> - 6</summary>
Announcement, if made at all, was made too recently.
</details>
</li>
<li>
<details>
<summary>
<code>NoSelfProxy</code> - 7</summary>
Cannot add self as proxy.
</details>
</li>
</ul>
</li>
<li>Utility - 64
<ul>
<li>
<details>
<summary>
<code>TooManyCalls</code> - 0</summary>
Too many calls batched.
</details>
</li>
</ul>
</li>
<li>Treasury - 65
<ul>
<li>
<details>
<summary>
<code>InsufficientProposersBalance</code> - 0</summary>
Proposer's balance is too low.
</details>
</li>
<li>
<details>
<summary>
<code>InvalidIndex</code> - 1</summary>
No proposal or bounty at that index.
</details>
</li>
<li>
<details>
<summary>
<code>TooManyApprovals</code> - 2</summary>
Too many approvals in the queue.
</details>
</li>
<li>
<details>
<summary>
<code>InsufficientPermission</code> - 3</summary>
The spend origin is valid but the amount it is allowed to spend is lower than the
amount to be spent.
</details>
</li>
<li>
<details>
<summary>
<code>ProposalNotApproved</code> - 4</summary>
Proposal has not been approved.
</details>
</li>
</ul>
</li>
</ul>