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
    fn on_initialize() -> Weight;
    fn expire_memberships(_i: u32) -> Weight;
}

// Insecure weights implementation, use it for tests only!
impl WeightInfo for () {
    fn on_initialize() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 4_012_000 picoseconds.
        Weight::from_parts(4_629_000, 0).saturating_add(Weight::from_parts(0, 0))
    }

    fn expire_memberships(i: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `567 + i * (23 ±0)`
        //  Estimated: `6583 + i * (2499 ±0)`
        // Minimum execution time: 86_925_000 picoseconds.
        Weight::from_parts(89_056_000, 0)
            .saturating_add(Weight::from_parts(0, 6583))
            // Standard Error: 2_429_589
            .saturating_add(Weight::from_parts(295_368_241, 0).saturating_mul(i.into()))
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().reads((2_u64).saturating_mul(i.into())))
            .saturating_add(RocksDbWeight::get().writes(5))
            .saturating_add(RocksDbWeight::get().writes((1_u64).saturating_mul(i.into())))
            .saturating_add(Weight::from_parts(0, 2499).saturating_mul(i.into()))
    }
}
