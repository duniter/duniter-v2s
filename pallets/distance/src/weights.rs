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

pub trait WeightInfo {
    fn request_distance_evaluation() -> Weight;
    fn update_evaluation(i: u32) -> Weight;
    fn force_update_evaluation(i: u32) -> Weight;
    fn force_set_distance_status() -> Weight;
    fn on_finalize() -> Weight;
}

// Insecure weights implementation, use it for tests only!
impl WeightInfo for () {
    /// Storage: Identity IdentityIndexOf (r:1 w:0)
    /// Proof Skipped: Identity IdentityIndexOf (max_values: None, max_size: None, mode: Measured)
    /// Storage: Distance IdentityDistanceStatus (r:1 w:1)
    /// Proof Skipped: Distance IdentityDistanceStatus (max_values: None, max_size: None, mode: Measured)
    /// Storage: Session CurrentIndex (r:1 w:0)
    /// Proof Skipped: Session CurrentIndex (max_values: Some(1), max_size: None, mode: Measured)
    /// Storage: Distance EvaluationPool2 (r:1 w:1)
    /// Proof Skipped: Distance EvaluationPool2 (max_values: Some(1), max_size: None, mode: Measured)
    /// Storage: System Account (r:1 w:1)
    /// Proof: System Account (max_values: None, max_size: Some(121), added: 2596, mode: MaxEncodedLen)
    /// Storage: Distance DistanceStatusExpireOn (r:1 w:1)
    /// Proof Skipped: Distance DistanceStatusExpireOn (max_values: None, max_size: None, mode: Measured)
    fn request_distance_evaluation() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `935`
        //  Estimated: `4400`
        // Minimum execution time: 28_469_000 picoseconds.
        Weight::from_parts(30_905_000, 0)
            .saturating_add(Weight::from_parts(0, 4400))
            .saturating_add(RocksDbWeight::get().reads(6))
            .saturating_add(RocksDbWeight::get().writes(4))
    }
    /// Storage: Distance DidUpdate (r:1 w:1)
    /// Proof Skipped: Distance DidUpdate (max_values: Some(1), max_size: None, mode: Measured)
    /// Storage: Authorship Author (r:1 w:1)
    /// Proof: Authorship Author (max_values: Some(1), max_size: Some(32), added: 527, mode: MaxEncodedLen)
    /// Storage: System Digest (r:1 w:0)
    /// Proof Skipped: System Digest (max_values: Some(1), max_size: None, mode: Measured)
    /// Storage: Session CurrentIndex (r:1 w:0)
    /// Proof Skipped: Session CurrentIndex (max_values: Some(1), max_size: None, mode: Measured)
    /// Storage: Distance EvaluationPool0 (r:1 w:1)
    /// Proof Skipped: Distance EvaluationPool0 (max_values: Some(1), max_size: None, mode: Measured)
    /// The range of component `i` is `[1, 600]`.
    fn update_evaluation(i: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `744 + i * (10 ±0)`
        //  Estimated: `2228 + i * (10 ±0)`
        // Minimum execution time: 13_870_000 picoseconds.
        Weight::from_parts(17_116_748, 0)
            .saturating_add(Weight::from_parts(0, 2228))
            // Standard Error: 684
            .saturating_add(Weight::from_parts(128_989, 0).saturating_mul(i.into()))
            .saturating_add(RocksDbWeight::get().reads(5))
            .saturating_add(RocksDbWeight::get().writes(3))
            .saturating_add(Weight::from_parts(0, 10).saturating_mul(i.into()))
    }
    /// Storage: Session CurrentIndex (r:1 w:0)
    /// Proof Skipped: Session CurrentIndex (max_values: Some(1), max_size: None, mode: Measured)
    /// Storage: Distance EvaluationPool0 (r:1 w:1)
    /// Proof Skipped: Distance EvaluationPool0 (max_values: Some(1), max_size: None, mode: Measured)
    /// The range of component `i` is `[1, 600]`.
    fn force_update_evaluation(i: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `612 + i * (10 ±0)`
        //  Estimated: `2096 + i * (10 ±0)`
        // Minimum execution time: 8_392_000 picoseconds.
        Weight::from_parts(10_825_908, 0)
            .saturating_add(Weight::from_parts(0, 2096))
            // Standard Error: 326
            .saturating_add(Weight::from_parts(123_200, 0).saturating_mul(i.into()))
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(1))
            .saturating_add(Weight::from_parts(0, 10).saturating_mul(i.into()))
    }
    /// Storage: Session CurrentIndex (r:1 w:0)
    /// Proof Skipped: Session CurrentIndex (max_values: Some(1), max_size: None, mode: Measured)
    /// Storage: Distance DistanceStatusExpireOn (r:1 w:1)
    /// Proof Skipped: Distance DistanceStatusExpireOn (max_values: None, max_size: None, mode: Measured)
    /// Storage: Distance IdentityDistanceStatus (r:0 w:1)
    /// Proof Skipped: Distance IdentityDistanceStatus (max_values: None, max_size: None, mode: Measured)
    fn force_set_distance_status() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `586`
        //  Estimated: `4051`
        // Minimum execution time: 8_099_000 picoseconds.
        Weight::from_parts(8_786_000, 0)
            .saturating_add(Weight::from_parts(0, 4051))
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(2))
    }
    /// Storage: Distance DidUpdate (r:1 w:1)
    /// Proof Skipped: Distance DidUpdate (max_values: Some(1), max_size: None, mode: Measured)
    fn on_finalize() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `170`
        //  Estimated: `1655`
        // Minimum execution time: 3_904_000 picoseconds.
        Weight::from_parts(4_132_000, 0)
            .saturating_add(Weight::from_parts(0, 1655))
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
}
