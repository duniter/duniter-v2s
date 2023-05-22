
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-05-19 (Y/M/D)
//! HOSTNAME: `benjamin-xps139380`, CPU: `Intel(R) Core(TM) i7-8565U CPU @ 1.80GHz`
//!
//! SHORT-NAME: `extrinsic`, LONG-NAME: `ExtrinsicBase`, RUNTIME: `Development`
//! WARMUPS: `10`, REPEAT: `100`
//! WEIGHT-PATH: `runtime/common/src/weights/`
//! WEIGHT-METRIC: `Average`, WEIGHT-MUL: `1.0`, WEIGHT-ADD: `0`

// Executed Command:
//   target/release/duniter
//   benchmark
//   overhead
//   --chain=gdev-benchmark
//   --execution=wasm
//   --wasm-execution=compiled
//   --weight-path=runtime/common/src/weights/
//   --warmup=10

use sp_core::parameter_types;
use sp_weights::{constants::WEIGHT_PER_NANOS, Weight};

parameter_types! {
	/// Time to execute a NO-OP extrinsic, for example `System::remark`.
	/// Calculated by multiplying the *Average* with `1.0` and adding `0`.
	///
	/// Stats nanoseconds:
	///   Min, Max: 203_855, 206_882
	///   Average:  204_580
	///   Median:   204_521
	///   Std-Dev:  405.03
	///
	/// Percentiles nanoseconds:
	///   99th: 205_581
	///   95th: 205_130
	///   75th: 204_772
	pub const ExtrinsicBaseWeight: Weight = WEIGHT_PER_NANOS.saturating_mul(204_580);
}

#[cfg(test)]
mod test_weights {
	use sp_weights::constants;

	/// Checks that the weight exists and is sane.
	// NOTE: If this test fails but you are sure that the generated values are fine,
	// you can delete it.
	#[test]
	fn sane() {
		let w = super::ExtrinsicBaseWeight::get();

		// At least 10 µs.
		assert!(
			w.ref_time() >= 10u64 * constants::WEIGHT_PER_MICROS.ref_time(),
			"Weight should be at least 10 µs."
		);
		// At most 1 ms.
		assert!(
			w.ref_time() <= constants::WEIGHT_PER_MILLIS.ref_time(),
			"Weight should be at most 1 ms."
		);
	}
}
