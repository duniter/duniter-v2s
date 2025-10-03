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

use frame_support::weights::{Weight, constants::RocksDbWeight};

/// Weight functions needed for pallet_universal_dividend.
pub trait WeightInfo {
    fn claim_uds(i: u32) -> Weight;
    fn transfer_ud() -> Weight;
    fn transfer_ud_keep_alive() -> Weight;
    fn on_removed_member(i: u32) -> Weight;
}

// Insecure weights implementation, use it for tests only!
impl WeightInfo for () {
    fn claim_uds(i: u32) -> Weight {
        Weight::from_parts(32_514_000, 0)
            // Standard Error: 32_000
            .saturating_add(Weight::from_parts(8_000, 0).saturating_mul(i as u64))
            .saturating_add(RocksDbWeight::get().reads(4))
            .saturating_add(RocksDbWeight::get().writes(1))
    }

    // Storage: UniversalDividend CurrentUd (r:1 w:0)
    // Storage: System Account (r:1 w:1)
    fn transfer_ud() -> Weight {
        Weight::from_parts(53_401_000, 0)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(2))
    }

    // Storage: UniversalDividend CurrentUd (r:1 w:0)
    // Storage: System Account (r:1 w:1)
    fn transfer_ud_keep_alive() -> Weight {
        Weight::from_parts(33_420_000, 0)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(2))
    }

    fn on_removed_member(i: u32) -> Weight {
        Weight::from_parts(32_514_000, 0)
            // Standard Error: 32_000
            .saturating_add(Weight::from_parts(8_000, 0).saturating_mul(i as u64))
            .saturating_add(RocksDbWeight::get().reads(4))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
}
