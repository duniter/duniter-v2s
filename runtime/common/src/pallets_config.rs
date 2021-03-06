// Copyright 2021 Axiom-Team
//
// This file is part of Substrate-Libre-Currency.
//
// Substrate-Libre-Currency is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, version 3 of the License.
//
// Substrate-Libre-Currency is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with Substrate-Libre-Currency. If not, see <https://www.gnu.org/licenses/>.

#[macro_export]
macro_rules! pallets_config {
    {$($custom:tt)*} => {
        $($custom)*

        // SYSTEM //

        parameter_types! {
            pub const Version: RuntimeVersion = VERSION;
        }

        impl frame_system::Config for Runtime {
            /// The basic call filter to use in dispatchable.
            type BaseCallFilter = BaseCallFilter;
            /// Block & extrinsics weights: base values and limits.
            type BlockWeights = BlockWeights;
            /// The maximum length of a block (in bytes).
            type BlockLength = BlockLength;
            /// The identifier used to distinguish between accounts.
            type AccountId = AccountId;
            /// The aggregated dispatch type that is available for extrinsics.
            type Call = Call;
            /// The lookup mechanism to get account ID from whatever is passed in dispatchers.
            type Lookup = AccountIdLookup<AccountId, ()>;
            /// The index type for storing how many extrinsics an account has signed.
            type Index = Index;
            /// The index type for blocks.
            type BlockNumber = BlockNumber;
            /// The type for hashing blocks and tries.
            type Hash = Hash;
            /// The hashing algorithm used.
            type Hashing = BlakeTwo256;
            /// The header type.
            type Header = generic::Header<BlockNumber, BlakeTwo256>;
            /// The ubiquitous event type.
            type Event = Event;
            /// The ubiquitous origin type.
            type Origin = Origin;
            /// Maximum number of block number to block hash mappings to keep (oldest pruned first).
            type BlockHashCount = BlockHashCount;
            /// The weight of database operations that the runtime can invoke.
            type DbWeight = DbWeight;
            /// Version of the runtime.
            type Version = Version;
            /// Converts a module to the index of the module in `construct_runtime!`.
            ///
            /// This type is being generated by `construct_runtime!`.
            type PalletInfo = PalletInfo;
            /// What to do if a new account is created.
            type OnNewAccount = ();
            /// What to do if an account is fully reaped from the system.
            type OnKilledAccount = ();
            /// The data to be stored in an account.
            type AccountData = pallet_duniter_account::AccountData<Balance>;
            /// Weight information for the extrinsics of this pallet.
            type SystemWeightInfo = common_runtime::weights::frame_system::WeightInfo<Runtime>;
            /// This is used as an identifier of the chain. 42 is the generic substrate prefix.
            type SS58Prefix = SS58Prefix;
            /// The set code logic, just the default since we're not a parachain.
            type OnSetCode = ();
            type MaxConsumers = frame_support::traits::ConstU32<16>;
        }

        // SCHEDULER //

        parameter_types! {
            pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) *
                BlockWeights::get().max_block;
            pub const MaxScheduledPerBlock: u32 = 50;
            pub const NoPreimagePostponement: Option<u32> = Some(10);
        }
        impl pallet_scheduler::Config for Runtime {
            type Event = Event;
            type Origin = Origin;
            type PalletsOrigin = OriginCaller;
            type Call = Call;
            type MaximumWeight = MaximumSchedulerWeight;
            type ScheduleOrigin = EnsureRoot<AccountId>;
            type OriginPrivilegeCmp = EqualPrivilegeOnly;
            type MaxScheduledPerBlock = MaxScheduledPerBlock;
            type WeightInfo = common_runtime::weights::pallet_scheduler::WeightInfo<Runtime>;
            type PreimageProvider = Preimage;
            type NoPreimagePostponement = ();
        }

        // ACCOUNT //

        impl pallet_duniter_account::Config for Runtime {
            type AccountIdToSalt = sp_runtime::traits::ConvertInto;
            type Event = Event;
            type MaxNewAccountsPerBlock = frame_support::pallet_prelude::ConstU32<1>;
            type NewAccountPrice = frame_support::traits::ConstU64<300>;
        }

        // BLOCK CREATION //

        impl pallet_babe::Config for Runtime {
            type EpochDuration = EpochDuration;
            type ExpectedBlockTime = ExpectedBlockTime;

            // session module is the trigger
            type EpochChangeTrigger = pallet_babe::ExternalTrigger;

            type DisabledValidators = Session;

            type KeyOwnerProofSystem = Historical;

            type KeyOwnerProof = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
                KeyTypeId,
                pallet_babe::AuthorityId,
            )>>::Proof;

            type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
                KeyTypeId,
                pallet_babe::AuthorityId,
            )>>::IdentificationTuple;

            type HandleEquivocation =
                pallet_babe::EquivocationHandler<Self::KeyOwnerIdentification, Offences, ReportLongevity>;

            type WeightInfo = common_runtime::weights::pallet_babe::WeightInfo<Runtime>;

            type MaxAuthorities = MaxAuthorities;
        }

        impl pallet_timestamp::Config for Runtime {
            type Moment = u64;
            type OnTimestampSet = Babe;
            type MinimumPeriod = MinimumPeriod;
            type WeightInfo = common_runtime::weights::pallet_timestamp::WeightInfo<Runtime>;
        }

        // MONEY MANAGEMENT //

        impl pallet_balances::Config for Runtime {
            type MaxLocks = MaxLocks;
            type MaxReserves = frame_support::pallet_prelude::ConstU32<5>;
            type ReserveIdentifier = [u8; 8];
            /// The type for recording an account's balance.
            type Balance = Balance;
            /// The ubiquitous event type.
            type Event = Event;
            type DustRemoval = Treasury;
            type ExistentialDeposit = ExistentialDeposit;
            type AccountStore = Account;
            type WeightInfo = common_runtime::weights::pallet_balances::WeightInfo<Runtime>;
        }

        pub struct HandleFees;
        type NegativeImbalance = <Balances as frame_support::traits::Currency<AccountId>>::NegativeImbalance;
        impl frame_support::traits::OnUnbalanced<NegativeImbalance> for HandleFees {
            fn on_nonzero_unbalanced(amount: NegativeImbalance) {
                use frame_support::traits::Currency as _;

                if let Some(author) = Authorship::author() {
                    Balances::resolve_creating(&author, amount);
                }
            }
        }
        impl pallet_transaction_payment::Config for Runtime {
            type OnChargeTransaction = CurrencyAdapter<Balances, HandleFees>;
            type OperationalFeeMultiplier = frame_support::traits::ConstU8<5>;
            type WeightToFee = common_runtime::fees::WeightToFeeImpl<Balance>;
            type LengthToFee = common_runtime::fees::LengthToFeeImpl<Balance>;
            type FeeMultiplierUpdate = ();
        }

        // CONSENSUS  //

        impl pallet_authority_discovery::Config for Runtime {
            type MaxAuthorities = MaxAuthorities;
        }
        impl pallet_authority_members::Config for Runtime {
            type Event = Event;
            type KeysWrapper = opaque::SessionKeysWrapper;
            type IsMember = SmithsMembership;
            type OnNewSession = OnNewSessionHandler<Runtime>;
            type OnRemovedMember = OnRemovedAuthorityMemberHandler<Runtime>;
            type MemberId = IdtyIndex;
            type MemberIdOf = common_runtime::providers::IdentityIndexOf<Self>;
            type MaxAuthorities = MaxAuthorities;
            type MaxKeysLife = frame_support::pallet_prelude::ConstU32<1_500>;
            type MaxOfflineSessions = frame_support::pallet_prelude::ConstU32<2_400>;
            type RemoveMemberOrigin = EnsureRoot<Self::AccountId>;
        }
        impl pallet_authorship::Config for Runtime {
            type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Babe>;
            type UncleGenerations = UncleGenerations;
            type FilterUncle = ();
            type EventHandler = ImOnline;
        }
        impl pallet_im_online::Config for Runtime {
            type AuthorityId = ImOnlineId;
            type Event = Event;
            type ValidatorSet = Historical;
            type NextSessionRotation = Babe;
            type ReportUnresponsiveness = Offences;
            type UnsignedPriority = ImOnlineUnsignedPriority;
            type WeightInfo = pallet_im_online::weights::SubstrateWeight<Runtime>;
            type MaxKeys = MaxAuthorities;
            type MaxPeerInHeartbeats = MaxPeerInHeartbeats;
            type MaxPeerDataEncodingSize = MaxPeerDataEncodingSize;
        }
        impl pallet_offences::Config for Runtime {
            type Event = Event;
            type IdentificationTuple = pallet_session::historical::IdentificationTuple<Self>;
            type OnOffenceHandler = ();
        }
        impl pallet_session::Config for Runtime {
            type Event = Event;
            type ValidatorId = AccountId;
            type ValidatorIdOf = sp_runtime::traits::ConvertInto;
            type ShouldEndSession = Babe;
            type NextSessionRotation = Babe;
            type SessionManager = pallet_session::historical::NoteHistoricalRoot<Self, AuthorityMembers>;
            type SessionHandler = <opaque::SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
            type Keys = opaque::SessionKeys;
            type WeightInfo = pallet_session::weights::SubstrateWeight<Runtime>;
        }
        impl pallet_session::historical::Config for Runtime {
            type FullIdentification = ValidatorFullIdentification;
            type FullIdentificationOf = FullIdentificationOfImpl;
        }
        impl pallet_grandpa::Config for Runtime {
            type Event = Event;
            type Call = Call;

            type KeyOwnerProofSystem = ();

            type KeyOwnerProof =
                <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;

            type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
                KeyTypeId,
                GrandpaId,
            )>>::IdentificationTuple;

            type HandleEquivocation = ();

            type WeightInfo = common_runtime::weights::pallet_grandpa::WeightInfo<Runtime>;

            type MaxAuthorities = MaxAuthorities;
        }

        // ONCHAIN??GOVERNANCE //

		#[cfg(feature = "runtime-benchmarks")]
		parameter_types! {
			pub const WorstCaseOrigin: pallet_collective::RawOrigin<AccountId, TechnicalCommitteeInstance> =
				pallet_collective::RawOrigin::<AccountId, TechnicalCommitteeInstance>::Members(2, 3);
		}

		impl pallet_upgrade_origin::Config for Runtime {
			type Event = Event;
			type Call = Call;
			type UpgradableOrigin = pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCommitteeInstance, 2, 3>;
			type WeightInfo = common_runtime::weights::pallet_upgrade_origin::WeightInfo<Runtime>;
			#[cfg(feature = "runtime-benchmarks")]
			type WorstCaseOriginType = pallet_collective::RawOrigin<AccountId, TechnicalCommitteeInstance>;
			#[cfg(feature = "runtime-benchmarks")]
			type WorstCaseOrigin = WorstCaseOrigin;
		}

        parameter_types! {
            pub const PreimageMaxSize: u32 = 4096 * 1024;
            pub const PreimageBaseDeposit: Balance = deposit(2, 64);
            pub const PreimageByteDeposit: Balance = deposit(0, 1);
        }

        impl pallet_preimage::Config for Runtime {
            type WeightInfo = pallet_preimage::weights::SubstrateWeight<Runtime>;
            type Event = Event;
            type Currency = Balances;
            type ManagerOrigin = EnsureRoot<AccountId>;
            type MaxSize = PreimageMaxSize;
            type BaseDeposit = PreimageBaseDeposit;
            type ByteDeposit = PreimageByteDeposit;
        }

        // UTILITIES //

        impl pallet_atomic_swap::Config for Runtime {
            type Event = Event;
            type SwapAction = pallet_atomic_swap::BalanceSwapAction<AccountId, Balances>;
            type ProofLimit = frame_support::traits::ConstU32<1_024>;
        }

        impl pallet_provide_randomness::Config for Runtime {
            type Currency = Balances;
            type Event = Event;
            type GetCurrentEpochIndex = GetCurrentEpochIndex<Self>;
            type MaxRequests = frame_support::traits::ConstU32<100>;
            type RequestPrice = frame_support::traits::ConstU64<2_000>;
            type OnFilledRandomness = Account;
            type OnUnbalanced = Treasury;
            type ParentBlockRandomness = pallet_babe::ParentBlockRandomness<Self>;
            type RandomnessFromOneEpochAgo = pallet_babe::RandomnessFromOneEpochAgo<Self>;
        }

        parameter_types! {
            // One storage item; key size 32, value size 8; .
            pub const ProxyDepositBase: Balance = deposit(1, 8);
            // Additional storage item size of 33 bytes.
            pub const ProxyDepositFactor: Balance = deposit(0, 33);
            pub const AnnouncementDepositBase: Balance = deposit(1, 8);
            pub const AnnouncementDepositFactor: Balance = deposit(0, 66);
        }
        impl pallet_proxy::Config for Runtime {
            type Event = Event;
            type Call = Call;
            type Currency = Balances;
            type ProxyType = ProxyType;
            type ProxyDepositBase = ProxyDepositBase;
            type ProxyDepositFactor = ProxyDepositFactor;
            type MaxProxies = frame_support::traits::ConstU32<32>;
            type MaxPending = frame_support::traits::ConstU32<32>;
            type CallHasher = BlakeTwo256;
            type AnnouncementDepositBase = AnnouncementDepositBase;
            type AnnouncementDepositFactor = AnnouncementDepositFactor;
            type WeightInfo = common_runtime::weights::pallet_proxy::WeightInfo<Self>;
        }

        parameter_types! {
            pub const DepositBase: Balance = DEPOSIT_PER_ITEM;
            pub const DepositFactor: Balance = DEPOSIT_PER_BYTE * 32;
        }
        impl pallet_multisig::Config for Runtime {
            type Event = Event;
            type Call = Call;
            type Currency = Balances;
            type DepositBase = DepositBase;
            type DepositFactor = DepositFactor;
            type MaxSignatories = MaxSignatories;
            type WeightInfo = common_runtime::weights::pallet_multisig::WeightInfo<Self>;
        }

        impl pallet_utility::Config for Runtime {
            type Event = Event;
            type Call = Call;
            type PalletsOrigin = OriginCaller;
            type WeightInfo = pallet_utility::weights::SubstrateWeight<Self>;
        }

        parameter_types! {
            pub const Burn: Permill = Permill::zero();
            pub const ProposalBond: Permill = Permill::from_percent(1);
            pub const ProposalBondMaximum: Option<Balance> = None;
            pub const SpendPeriod: BlockNumber = DAYS;
            // Treasury account address:
            // gdev/gtest: 5EYCAe5ijiYfyeZ2JJCGq56LmPyNRAKzpG4QkoQkkQNB5e6Z
            pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
        }
        impl pallet_treasury::Config for Runtime {
            type ApproveOrigin = TreasuryApproveOrigin;
            type Burn = Burn;
            type BurnDestination = ();
            type Currency = Balances;
            type Event = Event;
            type OnSlash = Treasury;
            type ProposalBond = ProposalBond;
            type ProposalBondMinimum = frame_support::traits::ConstU64<10_000>;
            type ProposalBondMaximum = ProposalBondMaximum;
            type MaxApprovals = frame_support::traits::ConstU32<100>;
            type PalletId = TreasuryPalletId;
            type RejectOrigin = TreasuryRejectOrigin;
            type SpendFunds = TreasurySpendFunds<Self>;
            type SpendPeriod = SpendPeriod;
            type WeightInfo = pallet_treasury::weights::SubstrateWeight<Self>;
        }

        // UNIVERSAL??DIVIDEND //

		pub struct MembersCount;
		impl frame_support::pallet_prelude::Get<Balance> for MembersCount {
			fn get() -> Balance {
				<Membership as sp_membership::traits::MembersCount>::members_count() as Balance
			}
		}

        impl pallet_universal_dividend::Config for Runtime {
            type BlockNumberIntoBalance = sp_runtime::traits::ConvertInto;
            type Currency = pallet_balances::Pallet<Runtime>;
            type Event = Event;
			type MaxPastReeval = frame_support::traits::ConstU32<4>;
            type MembersCount = MembersCount;
            type MembersStorage = common_runtime::providers::UdMembersStorage<Runtime>;
			type MembersStorageIter = common_runtime::providers::UdMembersStorageIter<Runtime>;
            type SquareMoneyGrowthRate = SquareMoneyGrowthRate;
            type UdCreationPeriod = UdCreationPeriod;
            type UdReevalPeriod = UdReevalPeriod;
            type UnitsPerUd = frame_support::traits::ConstU64<1_000>;
			type WeightInfo = common_runtime::weights::pallet_universal_dividend::WeightInfo<Runtime>;
        }

        // WEB??OF??TRUST //

        use frame_support::instances::Instance1;
        impl pallet_duniter_wot::Config<Instance1> for Runtime {
            type FirstIssuableOn = WotFirstCertIssuableOn;
            type IsSubWot = frame_support::traits::ConstBool<false>;
            type MinCertForMembership = WotMinCertForMembership;
            type MinCertForCreateIdtyRight = WotMinCertForCreateIdtyRight;
        }

        impl pallet_identity::Config for Runtime {
			type ChangeOwnerKeyPeriod = ChangeOwnerKeyPeriod;
            type ConfirmPeriod = ConfirmPeriod;
            type Event = Event;
            type EnsureIdtyCallAllowed = Wot;
            type IdtyCreationPeriod = IdtyCreationPeriod;
			type IdtyData = IdtyData;
            type IdtyIndex = IdtyIndex;
            type IdtyNameValidator = IdtyNameValidatorImpl;
            type NewOwnerKeySigner = <NewOwnerKeySignature as sp_runtime::traits::Verify>::Signer;
			type NewOwnerKeySignature = NewOwnerKeySignature;
            type OnIdtyChange = (common_runtime::handlers::OnIdtyChangeHandler<Runtime>, Wot);
            type RemoveIdentityConsumers = RemoveIdentityConsumersImpl<Self>;
            type RevocationSigner = <Signature as sp_runtime::traits::Verify>::Signer;
            type RevocationSignature = Signature;
        }

        impl pallet_membership::Config<frame_support::instances::Instance1> for Runtime {
			type IsIdtyAllowedToClaimMembership = Wot;
            type IsIdtyAllowedToRenewMembership = Wot;
            type IsIdtyAllowedToRequestMembership = Wot;
            type Event = Event;
            type IdtyId = IdtyIndex;
            type IdtyIdOf = common_runtime::providers::IdentityIndexOf<Self>;
            type MembershipPeriod = MembershipPeriod;
            type MetaData = pallet_duniter_wot::MembershipMetaData<AccountId>;
            type OnEvent = OnMembershipEventHandler<Wot, Runtime>;
            type PendingMembershipPeriod = PendingMembershipPeriod;
            type RevocationPeriod = frame_support::traits::ConstU32<0>;
        }

        impl pallet_certification::Config<Instance1> for Runtime {
            type CertPeriod = CertPeriod;
            type Event = Event;
            type IdtyIndex = IdtyIndex;
            type OwnerKeyOf = Identity;
            type IsCertAllowed = Wot;
            type MaxByIssuer = MaxByIssuer;
            type MinReceivedCertToBeAbleToIssueCert = MinReceivedCertToBeAbleToIssueCert;
            type OnNewcert = Wot;
            type OnRemovedCert = Wot;
            type ValidityPeriod = ValidityPeriod;
        }

        // SMITHS??SUB-WOT //

        use frame_support::instances::Instance2;
        impl pallet_duniter_wot::Config<Instance2> for Runtime {
            type FirstIssuableOn = SmithsWotFirstCertIssuableOn;
            type IsSubWot = frame_support::traits::ConstBool<true>;
            type MinCertForMembership = SmithsWotMinCertForMembership;
            type MinCertForCreateIdtyRight = frame_support::traits::ConstU32<0>;
        }

        impl pallet_membership::Config<Instance2> for Runtime {
			type IsIdtyAllowedToClaimMembership = SmithsSubWot;
            type IsIdtyAllowedToRenewMembership = SmithsSubWot;
            type IsIdtyAllowedToRequestMembership = SmithsSubWot;
            type Event = Event;
            type IdtyId = IdtyIndex;
            type IdtyIdOf = common_runtime::providers::IdentityIndexOf<Self>;
            type MembershipPeriod = SmithMembershipPeriod;
            type MetaData = SmithsMembershipMetaData<opaque::SessionKeysWrapper>;
            type OnEvent = OnSmithMembershipEventHandler<SmithsSubWot, Runtime>;
            type PendingMembershipPeriod = SmithPendingMembershipPeriod;
            type RevocationPeriod = frame_support::traits::ConstU32<0>;
        }

        impl pallet_certification::Config<Instance2> for Runtime {
            type CertPeriod = SmithCertPeriod;
            type Event = Event;
            type IdtyIndex = IdtyIndex;
            type OwnerKeyOf = Identity;
            type IsCertAllowed = SmithsSubWot;
            type MaxByIssuer = SmithMaxByIssuer;
            type MinReceivedCertToBeAbleToIssueCert = SmithMinReceivedCertToBeAbleToIssueCert;
            type OnNewcert = SmithsSubWot;
            type OnRemovedCert = SmithsSubWot;
            type ValidityPeriod = SmithValidityPeriod;
        }

        pub struct TechnicalCommitteeDefaultVote;
        impl pallet_collective::DefaultVote for TechnicalCommitteeDefaultVote {
            fn default_vote(
                _prime_vote: Option<bool>,
                _yes_votes: u32,
                _no_votes: u32,
                _len: u32,
            ) -> bool {
                false
            }
        }
        parameter_types! {
            pub const TechnicalCommitteeMotionDuration: BlockNumber = 7 * DAYS;
        }
        impl pallet_collective::Config<Instance2> for Runtime {
            type Origin = Origin;
            type Proposal = Call;
            type Event = Event;
            type MotionDuration = TechnicalCommitteeMotionDuration;
            type MaxProposals = frame_support::pallet_prelude::ConstU32<20>;
            type MaxMembers = frame_support::pallet_prelude::ConstU32<100>;
            type DefaultVote = TechnicalCommitteeDefaultVote;
            type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
        }
    };
}
