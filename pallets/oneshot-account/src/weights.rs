// Copyright 2021-2022 Axiom-Team
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
    fn create_oneshot_account() -> Weight;
    fn consume_oneshot_account() -> Weight;
    fn consume_oneshot_account_with_remaining() -> Weight;
}

// Insecure weights implementation, use it for tests only!
impl WeightInfo for () {
    // Storage: OneshotAccount OneshotAccounts (r:1 w:1)
    fn create_oneshot_account() -> Weight {
        (Weight::from_parts(45_690_000, 0))
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
    // Storage: OneshotAccount OneshotAccounts (r:1 w:1)
    // Storage: System BlockHash (r:1 w:0)
    // Storage: System Account (r:1 w:1)
    fn consume_oneshot_account() -> Weight {
        (Weight::from_parts(50_060_000, 0))
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(2))
    }
    // Storage: OneshotAccount OneshotAccounts (r:1 w:1)
    // Storage: System BlockHash (r:1 w:0)
    // Storage: System Account (r:2 w:2)
    fn consume_oneshot_account_with_remaining() -> Weight {
        (Weight::from_parts(69_346_000, 0))
            .saturating_add(RocksDbWeight::get().reads(4))
            .saturating_add(RocksDbWeight::get().writes(3))
    }
}
