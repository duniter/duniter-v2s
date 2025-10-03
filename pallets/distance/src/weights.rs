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

pub trait WeightInfo {
    fn request_distance_evaluation() -> Weight;
    fn request_distance_evaluation_for() -> Weight;
    fn update_evaluation(i: u32) -> Weight;
    fn force_update_evaluation(i: u32) -> Weight;
    fn force_valid_distance_status() -> Weight;
    fn on_initialize_overhead() -> Weight;
    fn do_evaluation_overhead() -> Weight;
    fn do_evaluation_success() -> Weight;
    fn do_evaluation_failure() -> Weight;
    fn on_finalize() -> Weight;
}

// Insecure weights implementation, use it for tests only!
impl WeightInfo for () {
    fn request_distance_evaluation() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1280`
        //  Estimated: `4745`
        // Minimum execution time: 876_053_000 picoseconds.
        Weight::from_parts(898_445_000, 0)
            .saturating_add(Weight::from_parts(0, 4745))
            .saturating_add(RocksDbWeight::get().reads(8))
            .saturating_add(RocksDbWeight::get().writes(3))
    }

    fn request_distance_evaluation_for() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1485`
        //  Estimated: `7425`
        // Minimum execution time: 1_118_982_000 picoseconds.
        Weight::from_parts(1_292_782_000, 0)
            .saturating_add(Weight::from_parts(0, 7425))
            .saturating_add(RocksDbWeight::get().reads(10))
            .saturating_add(RocksDbWeight::get().writes(3))
    }

    fn update_evaluation(i: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `773 + i * (10 ±0)`
        //  Estimated: `2256 + i * (10 ±0)`
        // Minimum execution time: 463_878_000 picoseconds.
        Weight::from_parts(743_823_548, 0)
            .saturating_add(Weight::from_parts(0, 2256))
            // Standard Error: 292_144
            .saturating_add(Weight::from_parts(1_326_639, 0).saturating_mul(i.into()))
            .saturating_add(RocksDbWeight::get().reads(6))
            .saturating_add(RocksDbWeight::get().writes(3))
            .saturating_add(Weight::from_parts(0, 10).saturating_mul(i.into()))
    }

    fn force_update_evaluation(i: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `612 + i * (10 ±0)`
        //  Estimated: `2095 + i * (10 ±0)`
        // Minimum execution time: 208_812_000 picoseconds.
        Weight::from_parts(257_150_521, 0)
            .saturating_add(Weight::from_parts(0, 2095))
            // Standard Error: 53_366
            .saturating_add(Weight::from_parts(1_841_329, 0).saturating_mul(i.into()))
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(1))
            .saturating_add(Weight::from_parts(0, 10).saturating_mul(i.into()))
    }

    fn force_valid_distance_status() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1181`
        //  Estimated: `7121`
        // Minimum execution time: 873_892_000 picoseconds.
        Weight::from_parts(1_081_510_000, 0)
            .saturating_add(Weight::from_parts(0, 7121))
            .saturating_add(RocksDbWeight::get().reads(7))
            .saturating_add(RocksDbWeight::get().writes(5))
    }

    fn do_evaluation_success() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `612 + i * (10 ±0)`
        //  Estimated: `2095 + i * (10 ±0)`
        // Minimum execution time: 208_812_000 picoseconds.
        Weight::from_parts(257_150_521, 0)
            .saturating_add(Weight::from_parts(0, 2095))
            // Standard Error: 53_366
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(1))
    }

    fn do_evaluation_failure() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `612 + i * (10 ±0)`
        //  Estimated: `2095 + i * (10 ±0)`
        // Minimum execution time: 208_812_000 picoseconds.
        Weight::from_parts(257_150_521, 0)
            .saturating_add(Weight::from_parts(0, 2095))
            // Standard Error: 53_366
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(1))
    }

    fn do_evaluation_overhead() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `612 + i * (10 ±0)`
        //  Estimated: `2095 + i * (10 ±0)`
        // Minimum execution time: 208_812_000 picoseconds.
        Weight::from_parts(257_150_521, 0)
            .saturating_add(Weight::from_parts(0, 2095))
            // Standard Error: 53_366
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(1))
    }

    fn on_initialize_overhead() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `170`
        //  Estimated: `1655`
        // Minimum execution time: 93_595_000 picoseconds.
        Weight::from_parts(109_467_000, 0)
            .saturating_add(Weight::from_parts(0, 1655))
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(1))
    }

    fn on_finalize() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `170`
        //  Estimated: `1655`
        // Minimum execution time: 93_595_000 picoseconds.
        Weight::from_parts(109_467_000, 0)
            .saturating_add(Weight::from_parts(0, 1655))
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
}
