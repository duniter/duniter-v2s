
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 32.0.0
//! DATE: 2024-07-03 (Y/M/D)
//! HOSTNAME: `bgallois-ms7d43`, CPU: `12th Gen Intel(R) Core(TM) i3-12100F`
//!
//! DATABASE: `ParityDb`, RUNTIME: `Ğ1 Local Testnet`
//! BLOCK-NUM: `BlockId::Number(0)`
//! SKIP-WRITE: `false`, SKIP-READ: `false`, WARMUPS: `1`
//! STATE-VERSION: `V1`, STATE-CACHE-SIZE: ``
//! WEIGHT-PATH: `./runtime/g1/src/weights/`
//! METRIC: `Average`, WEIGHT-MUL: `2.0`, WEIGHT-ADD: `0`

// Executed Command:
//   target/release/duniter
//   benchmark
//   storage
//   --chain=dev
//   --mul=2
//   --weight-path=./runtime/g1/src/weights/
//   --state-version=1
//   --database=paritydb

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
			/// Calculated by multiplying the *Average* of all values with `2.0` and adding `0`.
			///
			/// Stats nanoseconds:
			///   Min, Max: 747, 1_117_031
			///   Average:  8_389
			///   Median:   1_201
			///   Std-Dev:  88763.87
			///
			/// Percentiles nanoseconds:
			///   99th: 5_272
			///   95th: 1_844
			///   75th: 1_385
			read: 16_778 * constants::WEIGHT_REF_TIME_PER_NANOS,

			/// Time to write one storage item.
			/// Calculated by multiplying the *Average* of all values with `2.0` and adding `0`.
			///
			/// Stats nanoseconds:
			///   Min, Max: 4_012, 7_069_882
			///   Average:  54_938
			///   Median:   10_035
			///   Std-Dev:  561653.53
			///
			/// Percentiles nanoseconds:
			///   99th: 21_117
			///   95th: 15_204
			///   75th: 12_204
			write: 109_876 * constants::WEIGHT_REF_TIME_PER_NANOS,
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
