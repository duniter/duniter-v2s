// Copyright 2021 Axiom-Team
//
// This file is part of Duniter-v2S.
//
// Duniter-v2S is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, version 3 of the License.
//
// Duniter-v2S is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with Duniter-v2S. If not, see <https://www.gnu.org/licenses/>.

use crate::*;
use common_runtime::{Moment, constants::*};
use frame_support::{parameter_types, weights::constants::WEIGHT_REF_TIME_PER_SECOND};
use sp_runtime::transaction_validity::TransactionPriority;

parameter_types! {
    pub const BlockHashCount: BlockNumber = 2400;
    /// We allow for 2 seconds of compute with a 6 second average block time.
    pub BlockWeights: frame_system::limits::BlockWeights = frame_system::limits::BlockWeights::with_sensible_defaults(Weight::from_parts(WEIGHT_REF_TIME_PER_SECOND * 2u64, u64::MAX), NORMAL_DISPATCH_RATIO);
    pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
        ::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
    pub const SS58Prefix: u16 = 4450;
}

/*************/
/* CONSENSUS */
/*************/

// Authority discovery
parameter_types! {
    pub const MaxAuthorities: u32 = 32;
}

// Authorship
parameter_types! {
    pub const UncleGenerations: u32 = 0;
}

// Timestamp
parameter_types! {
    pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

// Distance
parameter_types! {
    pub const MinAccessibleReferees: Perbill = Perbill::from_percent(80);
    pub const MaxRefereeDistance: u32 = 5;
    pub const EvaluationPeriod: u32 = common_runtime::param_duration!(40, 5 * MINUTES);
}

// Babe
pub const EPOCH_DURATION_IN_SLOTS: BlockNumber =
    common_runtime::param_duration!(HOURS, 2 * MINUTES);
parameter_types! {
    pub const EpochDuration: u64 = EPOCH_DURATION_IN_SLOTS as u64;
    pub const ExpectedBlockTime: u64 = MILLISECS_PER_BLOCK;
    // Keep offence/equivocation reports valid long enough to tolerate network delays
    // and temporary outages, reducing missed-slash risk from short reporting windows.
    // TODO: temporarily use 1h for chain launch, must be changed to 28d when the chain is stable
    pub const ReportLongevity: BlockNumber =
        common_runtime::param_duration!(28 * 24 * EPOCH_DURATION_IN_SLOTS, 10 * EPOCH_DURATION_IN_SLOTS);
}

// ImOnline
parameter_types! {
    pub const ImOnlineUnsignedPriority: TransactionPriority = TransactionPriority::MAX;
    pub const MaxPeerInHeartbeats: u32 = 10_000;
    pub const MaxPeerDataEncodingSize: u32 = 1_000;
}

/*********/
/* MONEY */
/*********/

// Balances
// Why? Pallet treasury benchmarks are broken because the spend
// value is hardcoded 100 in benchmark and the account is not provided enough funds
// to exist if ED > 100.
#[cfg(feature = "runtime-benchmarks")]
frame_support::parameter_types! {
    pub const ExistentialDeposit: Balance = 100;
    pub const MaxLocks: u32 = 50;
}
#[cfg(not(feature = "runtime-benchmarks"))]
frame_support::parameter_types! {
    pub const ExistentialDeposit: Balance = 100;
    pub const MaxLocks: u32 = 50;
}

// Universal dividend
parameter_types! {
    // (1.1**0.5-1)**2 = 0.0023823036
    pub const SquareMoneyGrowthRate: Perbill = Perbill::from_parts(2_382_304);
    pub const UdCreationPeriod: Moment = common_runtime::param_duration!(86_400_000, 10 * 60_000); // 1 day
    pub const UdReevalPeriod: Moment =
        common_runtime::param_duration!(15_778_800_000, 30 * 60_000); // 1/2 year
}

/*******/
/* WOT */
/*******/

parameter_types! {
    pub const WotFirstCertIssuableOn: BlockNumber = 0;
    pub const WotMinCertForMembership: u32 = 5;
    pub const WotMinCertForCreateIdtyRight: u32 = 5;
}

// Identity
parameter_types! {
    pub const ChangeOwnerKeyPeriod: BlockNumber =
        common_runtime::param_duration!(6 * MONTHS, 30 * MINUTES);
    pub const ConfirmPeriod: BlockNumber =
        common_runtime::param_duration!(2 * MONTHS, 10 * MINUTES);
    pub const IdtyCreationPeriod: BlockNumber =
        common_runtime::param_duration!(5 * DAYS, 15 * MINUTES);
    pub const AutorevocationPeriod: BlockNumber =
        common_runtime::param_duration!(YEARS, 60 * MINUTES);
    pub const DeletionPeriod: BlockNumber =
        common_runtime::param_duration!(10 * YEARS, 120 * MINUTES);
}

// Membership
parameter_types! {
    pub const MembershipPeriod: BlockNumber =
        common_runtime::param_duration!(YEARS, 60 * MINUTES);
    pub const MembershipRenewalPeriod: BlockNumber =
        common_runtime::param_duration!(2 * MONTHS, 20 * MINUTES);
}

// Certification
parameter_types! {
    pub const CertPeriod: BlockNumber =
        common_runtime::param_duration!(5 * DAYS, 15 * MINUTES);
    pub const MaxByIssuer: u32 = 100;
    pub const MinReceivedCertToBeAbleToIssueCert: u32 = 5;
    pub const ValidityPeriod: BlockNumber =
        common_runtime::param_duration!(2 * YEARS, 120 * MINUTES);
}

/******************/
/* SMITH-MEMBERS */
/******************/

parameter_types! {
    pub const SmithWotMinCertForMembership: u32 = 3;
    pub const SmithMaxByIssuer: u32 = 12;
    pub const SmithInactivityMaxDuration: u32 =
        common_runtime::param_duration!(3 * 24 * 30, 15); // 3 months (24 sessions/day)
}

/*************/
/* UTILITIES */
/*************/

// Multisig
parameter_types! {
    pub const MaxSignatories: u16 = 10;
}

// Treasury
pub type TreasuryApproveOrigin =
    pallet_collective::EnsureProportionMoreThan<AccountId, TechnicalCommitteeInstance, 1, 2>;
pub type TreasuryRejectOrigin =
    pallet_collective::EnsureProportionMoreThan<AccountId, TechnicalCommitteeInstance, 1, 3>;
