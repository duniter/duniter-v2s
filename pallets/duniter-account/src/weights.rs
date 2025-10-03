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

use frame_support::weights::{Weight, constants::RocksDbWeight};

/// Weight functions needed for pallet_universal_dividend.
pub trait WeightInfo {
    fn unlink_identity() -> Weight;
    fn on_revoke_identity() -> Weight;
}

// Insecure weights implementation, use it for tests only!
impl WeightInfo for () {
    /// Storage: System Account (r:1 w:0)
    /// Proof: System Account (max_values: None, max_size: Some(126), added: 2601, mode: MaxEncodedLen)
    fn unlink_identity() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `3591`
        // Minimum execution time: 95_130_000 picoseconds.
        Weight::from_parts(110_501_000, 0)
            .saturating_add(Weight::from_parts(0, 3591))
            .saturating_add(RocksDbWeight::get().reads(1))
    }

    fn on_revoke_identity() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `3591`
        // Minimum execution time: 95_130_000 picoseconds.
        Weight::from_parts(110_501_000, 0)
            .saturating_add(Weight::from_parts(0, 3591))
            .saturating_add(RocksDbWeight::get().reads(1))
    }
}
