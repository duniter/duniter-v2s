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
use common_runtime::constants::*;
use common_runtime::{Balance, BlockNumber};
use frame_support::parameter_types;
use frame_support::weights::constants::WEIGHT_REF_TIME_PER_SECOND;
use sp_arithmetic::Perbill;
use sp_runtime::transaction_validity::TransactionPriority;

parameter_types! {
    pub const BlockHashCount: BlockNumber = 2400;
    /// We allow for 2 seconds of compute with a 6 second average block time.
    pub BlockWeights: frame_system::limits::BlockWeights = block_weights((Weight::from_parts(WEIGHT_REF_TIME_PER_SECOND, 0) * 2)
        .set_proof_size(5 * 1024 * 1024), NORMAL_DISPATCH_RATIO);
    pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
        ::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
    pub const SS58Prefix: u16 = 42;
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

// Babe
pub const EPOCH_DURATION_IN_SLOTS: BlockNumber = HOURS;
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
parameter_types! {
    // we take 100 of existential deposit to mimic duniter v1
    // and avoid loosing too many accounts during the migration
    pub const ExistentialDeposit: Balance = 100;
    pub const MaxLocks: u32 = 50;
}

// Universal dividend
parameter_types! {
    // 0.002_381_440 = 0.0488^2
    pub const SquareMoneyGrowthRate: Perbill = Perbill::from_parts(2_381_440);
    pub const UdCreationPeriod: BlockNumber = 86_400_000; // 1 day
    pub const UdReevalPeriod: BlockNumber = 7 * 86_400_000; // 7 days
}

/*******/
/* WOT */
/*******/

parameter_types! {
    pub const WotFirstCertIssuableOn: BlockNumber = DAYS;
    pub const WotMinCertForMembership: u32 = 5;
    pub const MinReceivedCertToBeAbleToIssueCert: u32 = 5;
    pub const WotMinCertForCreateIdtyRight: u32 = 5;
}

// Identity
parameter_types! {
    pub const ChangeOwnerKeyPeriod: BlockNumber = MONTHS;
    pub const ConfirmPeriod: BlockNumber = DAYS;
    pub const IdtyCreationPeriod: BlockNumber = DAYS;
}

// Membership
parameter_types! {
    pub const MembershipPeriod: BlockNumber = 73 * DAYS;
    pub const PendingMembershipPeriod: BlockNumber = 12 * DAYS;
}

// Certification
parameter_types! {
    pub const CertPeriod: BlockNumber = DAYS;
    pub const MaxByIssuer: u32 = 100;
    pub const ValidityPeriod: BlockNumber = 146 * DAYS;
}

/******************/
/* SMITHS SUB-WOT */
/******************/

parameter_types! {
    pub const SmithWotFirstCertIssuableOn: BlockNumber = DAYS;
    pub const SmithWotMinCertForMembership: u32 = 3;
}

// Membership
parameter_types! {
    pub const SmithMembershipPeriod: BlockNumber = 73 * DAYS;
    pub const SmithPendingMembershipPeriod: BlockNumber = 12 * DAYS;
}

// Certification
parameter_types! {
    pub const SmithCertPeriod: BlockNumber = DAYS;
    pub const SmithMaxByIssuer: u32 = 100;
    pub const SmithMinReceivedCertToBeAbleToIssueCert: u32 = 5;
    pub const SmithValidityPeriod: BlockNumber = 146 * DAYS;
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
