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

use crate::{Balance, BlockNumber};
use sp_runtime::Perbill;

/// This determines the average expected block time that we are targeting.
// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
// up by `pallet_babe` to implement `fn slot_duration()`.
// Change this to adjust the block time.
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

// The BABE epoch configuration at genesis.
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
