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
//! SHORT-NAME: `block`, LONG-NAME: `BlockExecution`, RUNTIME: `Ğdev`
//! WARMUPS: `10`, REPEAT: `100`
//! WEIGHT-PATH: `.`
//! WEIGHT-METRIC: `Average`, WEIGHT-MUL: `1`, WEIGHT-ADD: `0`

// Executed Command:
//   ./duniter
//   benchmark
//   overhead
//   --chain=gdev
//   --execution=wasm
//   --wasm-execution=interpreted-i-know-what-i-do
//   --weight-path=.
//   --warmup=10
//   --repeat=100

use frame_support::{
	parameter_types,
	weights::{constants::WEIGHT_PER_NANOS, Weight},
};

parameter_types! {
	/// Time to execute an empty block.
	/// Calculated by multiplying the *Average* with `1` and adding `0`.
	///
	/// Stats nanoseconds:
	///   Min, Max: 23_866_638, 90_077_105
	///   Average:  24_871_527
	///   Median:   23_915_377
	///   Std-Dev:  6645558.32
	///
	/// Percentiles nanoseconds:
	///   99th: 30_529_787
	///   95th: 27_134_555
	///   75th: 23_951_395
	pub const BlockExecutionWeight: Weight = 24_871_527 * WEIGHT_PER_NANOS;
}

#[cfg(test)]
mod test_weights {
	use frame_support::weights::constants;

	/// Checks that the weight exists and is sane.
	// NOTE: If this test fails but you are sure that the generated values are fine,
	// you can delete it.
	#[test]
	fn sane() {
		let w = super::BlockExecutionWeight::get();

		// At least 100 µs.
		assert!(w >= 100 * constants::WEIGHT_PER_MICROS, "Weight should be at least 100 µs.");
		// At most 50 ms.
		assert!(w <= 50 * constants::WEIGHT_PER_MILLIS, "Weight should be at most 50 ms.");
	}
}
