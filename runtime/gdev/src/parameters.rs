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
use frame_support::traits::EitherOfDiverse;
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
parameter_types! {
    pub const ExpectedBlockTime: u64 = MILLISECS_PER_BLOCK;
    pub const ReportLongevity: u64 = 168 * HOURS as u64;
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
    pub const ExistentialDeposit: Balance = 100;
    pub const MaxLocks: u32 = 50;
}

// Universal dividend
parameter_types! {
    // 0.002_381_440 = 0.0488^2
    pub const SquareMoneyGrowthRate: Perbill = Perbill::from_parts(2_381_440);
}

/*******/
/* WOT */
/*******/

// Identity
frame_support::parameter_types! {
    pub const ChangeOwnerKeyPeriod: BlockNumber = 7 * DAYS;
}

/*************/
/* UTILITIES */
/*************/

// Multisig
parameter_types! {
    pub const MaxSignatories: u16 = 10;
}

// Treasury
pub type TreasuryApproveOrigin = EitherOfDiverse<
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionMoreThan<AccountId, TechnicalCommitteeInstance, 1, 2>,
>; // Root is needed for benchmarking
pub type TreasuryRejectOrigin =
    pallet_collective::EnsureProportionMoreThan<AccountId, TechnicalCommitteeInstance, 1, 3>;
