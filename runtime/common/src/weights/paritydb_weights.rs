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

//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-11-18 (Y/M/D)
//! HOSTNAME: `raspberrypi`, CPU: `ARMv7 Processor rev 3 (v7l)`
//!
//! DATABASE: `ParityDb`, RUNTIME: `Ğdev`
//! BLOCK-NUM: `BlockId::Number(85630)`
//! SKIP-WRITE: `false`, SKIP-READ: `false`, WARMUPS: `1`
//! STATE-VERSION: `V1`, STATE-CACHE-SIZE: `0`
//! WEIGHT-PATH: `.`
//! METRIC: `Average`, WEIGHT-MUL: `2`, WEIGHT-ADD: `0`

// Executed Command:
//   ./duniter
//   benchmark
//   storage
//   -d=/mnt/ssd1/duniter-v2s/t1
//   --chain=gdev
//   --mul=2
//   --weight-path=.
//   --state-version=1

/// Storage DB weights for the `Ğdev` runtime and `ParityDb`.
pub mod constants {
    use frame_support::{
        parameter_types,
        weights::{constants, RuntimeDbWeight},
    };

    parameter_types! {
        /// `ParityDB` can be enabled with a feature flag, but is still experimental. These weights
        /// are available for brave runtime engineers who may want to try this out as default.
        pub const ParityDbWeight: RuntimeDbWeight = RuntimeDbWeight {
            /// Time to read one storage item.
            /// Calculated by multiplying the *Average* of all values with `2` and adding `0`.
            ///
            /// Stats nanoseconds:
            ///   Min, Max: 62_017, 5_238_182
			///   Average:  125_949
            ///   Median:   124_943
            ///   Std-Dev:  19659.02
            ///
            /// Percentiles nanoseconds:
            ///   99th: 151_424
            ///   95th: 143_017
            ///   75th: 133_498
            read: 250_000 * constants::WEIGHT_PER_NANOS,

            /// Time to write one storage item.
            /// Calculated by multiplying the *Average* of all values with `2` and adding `0`.
            ///
            /// Stats nanoseconds:
            ///   Min, Max: 88_054, 107_065_367
			///   Average:  419_064
            ///   Median:   424_994
            ///   Std-Dev:  423253.1
            ///
            /// Percentiles nanoseconds:
            ///   99th: 611_825
            ///   95th: 512_789
            ///   75th: 457_938
            write: 840_000 * constants::WEIGHT_PER_NANOS,
        };
    }

    #[cfg(test)]
    mod test_db_weights {
        use super::constants::ParityDbWeight as W;
        use frame_support::weights::constants;

        /// Checks that all weights exist and have sane values.
        // NOTE: If this test fails but you are sure that the generated values are fine,
        // you can delete it.
        #[test]
        fn bound() {
            // At least 1 µs.
            assert!(
                W::get().reads(1) >= constants::WEIGHT_PER_MICROS,
                "Read weight should be at least 1 µs."
            );
            assert!(
                W::get().writes(1) >= constants::WEIGHT_PER_MICROS,
                "Write weight should be at least 1 µs."
            );
            // At most 1 ms.
            assert!(
                W::get().reads(1) <= constants::WEIGHT_PER_MILLIS,
                "Read weight should be at most 1 ms."
            );
            assert!(
                W::get().writes(1) <= constants::WEIGHT_PER_MILLIS,
                "Write weight should be at most 1 ms."
            );
        }
    }
}
