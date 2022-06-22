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
    fn on_initialize() -> Weight;
    fn transfer_ud() -> Weight;
    fn transfer_ud_keep_alive() -> Weight;
}

// Insecure weights implementation, use it for tests only!
impl WeightInfo for () {
    // Storage: (r:0 w:0)
    fn on_initialize() -> Weight {
        2_260_000 as Weight
    }
    // Storage: UniversalDividend CurrentUd (r:1 w:0)
    // Storage: System Account (r:1 w:1)
    // Storage: Account PendingNewAccounts (r:0 w:1)
    fn transfer_ud() -> Weight {
        (53_401_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
    }
    // Storage: UniversalDividend CurrentUd (r:1 w:0)
    // Storage: System Account (r:1 w:1)
    // Storage: Account PendingNewAccounts (r:0 w:1)
    fn transfer_ud_keep_alive() -> Weight {
        (33_420_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
    }
}
