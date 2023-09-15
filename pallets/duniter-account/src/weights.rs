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
    fn on_initialize_sufficient(i: u32) -> Weight;
    fn on_initialize_with_balance(i: u32) -> Weight;
    fn on_initialize_no_balance(i: u32) -> Weight;
    fn on_filled_randomness_pending() -> Weight;
    fn on_filled_randomness_no_pending() -> Weight;
}

// Insecure weights implementation, use it for tests only!
impl WeightInfo for () {
    // Storage: Account PendingNewAccounts (r:1 w:0)
    // Storage: ProvideRandomness RequestIdProvider (r:1 w:1)
    // Storage: ProvideRandomness RequestsIds (r:1 w:1)
    // Storage: ProvideRandomness CounterForRequestsIds (r:1 w:1)
    // Storage: Babe EpochIndex (r:1 w:0)
    // Storage: ProvideRandomness NexEpochHookIn (r:1 w:0)
    // Storage: ProvideRandomness RequestsReadyAtEpoch (r:1 w:1)
    // Storage: Account PendingRandomIdAssignments (r:0 w:1)
    /// The range of component `i` is `[0, 1]`.
    fn on_initialize_sufficient(i: u32) -> Weight {
        // Minimum execution time: 12_958 nanoseconds.
        Weight::from_parts(14_907_902 as u64, 0)
            // Standard Error: 550_025
            .saturating_add(Weight::from_parts(79_482_297 as u64, 0).saturating_mul(i as u64))
            .saturating_add(RocksDbWeight::get().reads(1 as u64))
            .saturating_add(RocksDbWeight::get().reads((6 as u64).saturating_mul(i as u64)))
            .saturating_add(RocksDbWeight::get().writes((6 as u64).saturating_mul(i as u64)))
    }
    // Storage: Account PendingNewAccounts (r:1 w:0)
    // Storage: ProvideRandomness RequestIdProvider (r:1 w:1)
    // Storage: ProvideRandomness RequestsIds (r:1 w:1)
    // Storage: ProvideRandomness CounterForRequestsIds (r:1 w:1)
    // Storage: Babe EpochIndex (r:1 w:0)
    // Storage: ProvideRandomness NexEpochHookIn (r:1 w:0)
    // Storage: ProvideRandomness RequestsReadyAtEpoch (r:1 w:1)
    // Storage: Account PendingRandomIdAssignments (r:0 w:1)
    /// The range of component `i` is `[0, 1]`.
    fn on_initialize_with_balance(i: u32) -> Weight {
        // Minimum execution time: 12_965 nanoseconds.
        Weight::from_parts(16_754_718 as u64, 0)
            // Standard Error: 1_790_537
            .saturating_add(Weight::from_parts(164_043_481 as u64, 0).saturating_mul(i as u64))
            .saturating_add(RocksDbWeight::get().reads(1 as u64))
            .saturating_add(RocksDbWeight::get().reads((6 as u64).saturating_mul(i as u64)))
            .saturating_add(RocksDbWeight::get().writes((6 as u64).saturating_mul(i as u64)))
    }
    // Storage: Account PendingNewAccounts (r:1 w:0)
    /// The range of component `i` is `[0, 1]`.
    fn on_initialize_no_balance(i: u32) -> Weight {
        // Minimum execution time: 12_912 nanoseconds.
        Weight::from_parts(13_846_469 as u64, 0)
            // Standard Error: 115_598
            .saturating_add(Weight::from_parts(67_524_530 as u64, 0).saturating_mul(i as u64))
            .saturating_add(RocksDbWeight::get().reads(1 as u64))
            .saturating_add(RocksDbWeight::get().writes((1 as u64).saturating_mul(i as u64)))
    }
    // Storage: Account PendingRandomIdAssignments (r:1 w:1)
    fn on_filled_randomness_pending() -> Weight {
        // Minimum execution time: 66_963 nanoseconds.
        Weight::from_parts(69_757_000 as u64, 0)
            .saturating_add(RocksDbWeight::get().reads(1 as u64))
            .saturating_add(RocksDbWeight::get().writes(1 as u64))
    }
    // Storage: Account PendingRandomIdAssignments (r:1 w:0)
    fn on_filled_randomness_no_pending() -> Weight {
        // Minimum execution time: 16_088 nanoseconds.
        Weight::from_parts(27_963_000 as u64, 0)
            .saturating_add(RocksDbWeight::get().reads(1 as u64))
    }
}
