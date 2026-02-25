
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 49.1.0
//! DATE: 2026-02-23 (Y/M/D)
//! HOSTNAME: `aynuc`, CPU: `Raspberry Pi 4 Model B Rev 1.4`
//!
//! DATABASE: `ParityDb`, RUNTIME: `Ğ1 Local Testnet`
//! BLOCK-NUM: `BlockId::Number(0)`
//! SKIP-WRITE: `false`, SKIP-READ: `false`, WARMUPS: `1`
//! STATE-VERSION: `V1`, STATE-CACHE-SIZE: ``
//! WEIGHT-PATH: `./runtime/g1/src/weights/`
//! METRIC: `Average`, WEIGHT-MUL: `3.0`, WEIGHT-ADD: `0`

// Executed Command:
//   target/release/duniter
//   benchmark
//   storage
//   --dev
//   --mul=3
//   --weight-path=./runtime/g1/src/weights/
//   --state-version=1
//   --database=paritydb
//   --disable-pov-recorder
//   --batch-size=100

/// Storage DB weights for the `Ğ1 Local Testnet` runtime and `ParityDb`.
pub mod constants {
	use frame_support::weights::constants;
	use sp_core::parameter_types;
	use sp_weights::RuntimeDbWeight;

	parameter_types! {
		/// `ParityDB` can be enabled with a feature flag, but is still experimental. These weights
		/// are available for brave runtime engineers who may want to try this out as default.
		pub const ParityDbWeight: RuntimeDbWeight = RuntimeDbWeight {
			/// Time to read one storage item.
			/// Calculated by multiplying the *Average* of all values with `3.0` and adding `0`.
			///
			/// Stats nanoseconds:
			///   Min, Max: 2_185, 6_013_963
			///   Average:  43_706
			///   Median:   5_352
			///   Std-Dev:  478007.55
			///
			/// Percentiles nanoseconds:
			///   99th: 20_740
			///   95th: 8_408
			///   75th: 6_389
			read: 131_118 * constants::WEIGHT_REF_TIME_PER_NANOS,

			/// Time to write one storage item.
			/// Calculated by multiplying the *Average* of all values with `3.0` and adding `0`.
			///
			/// Stats nanoseconds:
			///   Min, Max: 319_145, 319_145
			///   Average:  319_145
			///   Median:   319_145
			///   Std-Dev:  0.0
			///
			/// Percentiles nanoseconds:
			///   99th: 319_145
			///   95th: 319_145
			///   75th: 319_145
			write: 957_435 * constants::WEIGHT_REF_TIME_PER_NANOS,
		};
	}

	#[cfg(test)]
	mod test_db_weights {
		use super::constants::ParityDbWeight as W;
		use sp_weights::constants;

		/// Checks that all weights exist and have sane values.
		// NOTE: If this test fails but you are sure that the generated values are fine,
		// you can delete it.
		#[test]
		fn bound() {
			// At least 1 µs.
			assert!(
				W::get().reads(1).ref_time() >= constants::WEIGHT_REF_TIME_PER_MICROS,
				"Read weight should be at least 1 µs."
			);
			assert!(
				W::get().writes(1).ref_time() >= constants::WEIGHT_REF_TIME_PER_MICROS,
				"Write weight should be at least 1 µs."
			);
			// At most 1 ms.
			assert!(
				W::get().reads(1).ref_time() <= constants::WEIGHT_REF_TIME_PER_MILLIS,
				"Read weight should be at most 1 ms."
			);
			assert!(
				W::get().writes(1).ref_time() <= constants::WEIGHT_REF_TIME_PER_MILLIS,
				"Write weight should be at most 1 ms."
			);
		}
	}
}
