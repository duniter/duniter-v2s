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

use common_runtime::constants::*;
use common_runtime::{Balance, BlockNumber};
use frame_support::parameter_types;
use frame_support::weights::constants::WEIGHT_PER_SECOND;
use sp_arithmetic::Permill;
use sp_runtime::transaction_validity::TransactionPriority;

parameter_types! {
    pub const BlockHashCount: BlockNumber = 2400;
    /// We allow for 2 seconds of compute with a 6 second average block time.
    pub BlockWeights: frame_system::limits::BlockWeights = frame_system::limits::BlockWeights
        ::with_sensible_defaults(2 * WEIGHT_PER_SECOND, NORMAL_DISPATCH_RATIO);
    pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
        ::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
    pub const SS58Prefix: u16 = 42;
}

/*************/
/* CONSENSUS */
/*************/

// Authority discovery
parameter_types! {
    pub const MaxAuthorities: u32 = 100;
}

// Authorship
parameter_types! {
    pub const UncleGenerations: u32 = 0;
}

// Timestamp
parameter_types! {
    pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

// Babe
pub const EPOCH_DURATION_IN_SLOTS: BlockNumber = 4 * HOURS;
parameter_types! {
    pub const EpochDuration: u64 = EPOCH_DURATION_IN_SLOTS as u64;
    pub const ExpectedBlockTime: u64 = MILLISECS_PER_BLOCK;
    pub const ReportLongevity: u64 = 168 * EpochDuration::get();
}

// ImOnline
parameter_types! {
    pub const ImOnlineUnsignedPriority: TransactionPriority = TransactionPriority::max_value();
    pub const MaxPeerInHeartbeats: u32 = 10_000;
    pub const MaxPeerDataEncodingSize: u32 = 1_000;
}

/*********/
/* MONEY */
/*********/

// Balances
frame_support::parameter_types! {
    pub const ExistentialDeposit: Balance = 200;
    pub const MaxLocks: u32 = 50;
}

// Transaction payment
frame_support::parameter_types! {
    pub const TransactionByteFee: Balance = 0;
}

// Universal dividend
parameter_types! {
    // 0.002_381_440 = 0.0488^2
    pub const SquareMoneyGrowthRate: Permill = Permill::from_parts(2_381_440);
    pub const UdCreationPeriod: BlockNumber = DAYS;
    // TODO: this value will depend on the date of the migration
    pub const UdFirstReeval: BlockNumber = 45 * DAYS;
    pub const UdReevalPeriod: Balance = 182;
    pub const UdReevalPeriodInBlocks: BlockNumber = 2_620_800; // 86400 * 182 / 6
}

/*******/
/* WOT */
/*******/

parameter_types! {
    pub const WotFirstCertIssuableOn: BlockNumber = 30 * DAYS;
    pub const WotMinCertForMembership: u32 = 5;
    pub const WotMinCertForCreateIdtyRight: u32 = 5;
}

// Identity
parameter_types! {
    pub const ConfirmPeriod: BlockNumber = 14 * DAYS;
    pub const IdtyCreationPeriod: BlockNumber = MONTHS;
    pub const ValidationPeriod: BlockNumber = YEARS;
}

// Membership
parameter_types! {
    pub const MembershipPeriod: BlockNumber = YEARS;
    pub const PendingMembershipPeriod: BlockNumber = 2 * MONTHS;
    pub const RenewablePeriod: BlockNumber = 2 * MONTHS;
}

// Certification
parameter_types! {
    pub const CertPeriod: BlockNumber = 5 * DAYS;
    pub const MaxByIssuer: u32 = 100;
    pub const MinReceivedCertToBeAbleToIssueCert: u32 = 5;
    pub const CertRenewablePeriod: BlockNumber = 6 * MONTHS;
    pub const ValidityPeriod: BlockNumber = 2 * YEARS;
}

/******************/
/* SMITHS SUB-WOT */
/******************/

parameter_types! {
    pub const SmithsWotFirstCertIssuableOn: BlockNumber = 30 * DAYS;
    pub const SmithsWotMinCertForMembership: u32 = 3;
}

// Membership
parameter_types! {
    pub const SmithMembershipPeriod: BlockNumber = 73 * DAYS;
    pub const SmithPendingMembershipPeriod: BlockNumber = 12 * DAYS;
    pub const SmithRenewablePeriod: BlockNumber = 12 * DAYS;
}

// Certification
parameter_types! {
    pub const SmithCertPeriod: BlockNumber = 5 * DAYS;
    pub const SmithMaxByIssuer: u32 = 12;
    pub const SmithMinReceivedCertToBeAbleToIssueCert: u32 = 5;
    pub const SmithCertRenewablePeriod: BlockNumber = 12 * DAYS;
    pub const SmithValidityPeriod: BlockNumber = 146 * DAYS;
}

// Multisig
parameter_types! {
    pub const MaxSignatories: u16 = 5;
}
