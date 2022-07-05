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

use crate::{Balance, BlockNumber};
use frame_support::weights::constants::WEIGHT_PER_MICROS;
use frame_support::weights::Weight;
use sp_runtime::Perbill;

/// This determines the average expected block time that we are targeting.
/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
/// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
/// up by `pallet_babe` to implement `fn slot_duration()`.
///
/// Change this to adjust the block time.
pub const MILLISECS_PER_BLOCK: u64 = 6000;
pub const SECS_PER_BLOCK: u64 = MILLISECS_PER_BLOCK / 1_000;

// NOTE: Currently it is not possible to change the slot duration after the chain has started.
//       Attempting to do so will brick block production.
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;
const SECS_PER_YEAR: u64 = 31_557_600; // (365.25 * 24 * 60 * 60)
pub const MONTHS: BlockNumber = (SECS_PER_YEAR / (12 * SECS_PER_BLOCK)) as BlockNumber;
pub const YEARS: BlockNumber = (SECS_PER_YEAR / SECS_PER_BLOCK) as BlockNumber;

// 1 in 4 blocks (on average, not counting collisions) will be primary babe blocks.
pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);

/// The BABE epoch configuration at genesis.
pub const BABE_GENESIS_EPOCH_CONFIG: sp_consensus_babe::BabeEpochConfiguration =
    sp_consensus_babe::BabeEpochConfiguration {
        c: PRIMARY_PROBABILITY,
        allowed_slots: sp_consensus_babe::AllowedSlots::PrimaryAndSecondaryVRFSlots,
    };

pub const DEPOSIT_PER_BYTE: Balance = 1;
pub const DEPOSIT_PER_ITEM: Balance = 100;

// Compute storage deposit per items and bytes
pub const fn deposit(items: u32, bytes: u32) -> Balance {
    items as Balance * DEPOSIT_PER_ITEM + (bytes as Balance * DEPOSIT_PER_BYTE)
}

// Maximal weight proportion of normal extrinsics per block
pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

// WEIGHTS CONSTANTS //

// Read DB weights
pub const READ_WEIGHTS: Weight = 250 * WEIGHT_PER_MICROS; // ~250 µs

// Write DB weights
pub const WRITE_WEIGHTS: Weight = 1_000 * WEIGHT_PER_MICROS; // ~1000 µs

// Execution cost of everything outside of the call itself:
// signature verification, pre_dispatch and post_dispatch
pub const EXTRINSIC_BASE_WEIGHTS: Weight = READ_WEIGHTS + WRITE_WEIGHTS;

// DB weights
frame_support::parameter_types! {
    pub const DbWeight: frame_support::weights::RuntimeDbWeight = frame_support::weights::RuntimeDbWeight {
        read: READ_WEIGHTS,
        write: WRITE_WEIGHTS,
    };
}

// Block weights limits
pub fn block_weights(
    expected_block_weight: Weight,
    normal_ratio: sp_arithmetic::Perbill,
) -> frame_system::limits::BlockWeights {
    let normal_weight = normal_ratio * expected_block_weight;
    frame_system::limits::BlockWeights::builder()
        .for_class(frame_support::weights::DispatchClass::Normal, |weights| {
            weights.base_extrinsic = EXTRINSIC_BASE_WEIGHTS;
            weights.max_total = normal_weight.into();
        })
        .for_class(
            frame_support::weights::DispatchClass::Operational,
            |weights| {
                weights.base_extrinsic = EXTRINSIC_BASE_WEIGHTS;
                weights.max_total = expected_block_weight.into();
                weights.reserved = (expected_block_weight - normal_weight).into();
            },
        )
        .avg_block_initialization(sp_arithmetic::Perbill::from_percent(10))
        .build()
        .expect("Fatal error: invalid BlockWeights configuration")
}
