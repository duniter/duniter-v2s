// Copyright 2021-2023 Axiom-Team
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

#![allow(clippy::unnecessary_cast)]

use frame_support::weights::{constants::RocksDbWeight, Weight};

/// Weight functions needed for pallet_universal_dividend.
pub trait WeightInfo {
    fn on_initialize(i: u32) -> Weight;
    fn on_initialize_epoch(i: u32) -> Weight;
    fn request() -> Weight;
}

// Insecure weights implementation, use it for tests only!
impl WeightInfo for () {
    // Storage: ProvideRandomness CounterForRequestsIds (r:1 w:1)
    // Storage: ProvideRandomness RequestIdProvider (r:1 w:1)
    // Storage: ProvideRandomness RequestsIds (r:1 w:1)
    // Storage: Babe EpochIndex (r:1 w:0)
    // Storage: ProvideRandomness NexEpochHookIn (r:1 w:0)
    // Storage: ProvideRandomness RequestsReadyAtEpoch (r:1 w:1)
    fn request() -> Weight {
        // Minimum execution time: 321_822 nanoseconds.
        Weight::from_parts(338_919_000 as u64, 0)
            .saturating_add(RocksDbWeight::get().reads(6 as u64))
            .saturating_add(RocksDbWeight::get().writes(4 as u64))
    }

    // Storage: ProvideRandomness RequestsReadyAtNextBlock (r:1 w:1)
    // Storage: Babe AuthorVrfRandomness (r:1 w:0)
    // Storage: ProvideRandomness RequestsIds (r:1 w:1)
    // Storage: ProvideRandomness CounterForRequestsIds (r:1 w:1)
    // Storage: Account PendingRandomIdAssignments (r:1 w:0)
    // Storage: ProvideRandomness NexEpochHookIn (r:1 w:1)
    /// The range of component `i` is `[1, 100]`.
    fn on_initialize(i: u32) -> Weight {
        // Minimum execution time: 175_645 nanoseconds.
        Weight::from_parts(461_442_906 as u64, 0)
            // Standard Error: 1_523_561
            .saturating_add(Weight::from_parts(43_315_015 as u64, 0).saturating_mul(i as u64))
            .saturating_add(RocksDbWeight::get().reads(4 as u64))
            .saturating_add(RocksDbWeight::get().reads((2 as u64).saturating_mul(i as u64)))
            .saturating_add(RocksDbWeight::get().writes(3 as u64))
            .saturating_add(RocksDbWeight::get().writes((1 as u64).saturating_mul(i as u64)))
    }

    fn on_initialize_epoch(i: u32) -> Weight {
        // Minimum execution time: 175_645 nanoseconds.
        Weight::from_parts(461_442_906 as u64, 0)
            // Standard Error: 1_523_561
            .saturating_add(Weight::from_parts(43_315_015 as u64, 0).saturating_mul(i as u64))
            .saturating_add(RocksDbWeight::get().reads(4 as u64))
            .saturating_add(RocksDbWeight::get().reads((2 as u64).saturating_mul(i as u64)))
            .saturating_add(RocksDbWeight::get().writes(3 as u64))
            .saturating_add(RocksDbWeight::get().writes((1 as u64).saturating_mul(i as u64)))
    }
}
