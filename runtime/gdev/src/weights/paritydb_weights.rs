
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 53.0.0
//! DATE: 2026-02-22 (Y/M/D)
//! HOSTNAME: `sonic`, CPU: `Intel Core Processor (Broadwell, no TSX, IBRS)`
//!
//! DATABASE: `ParityDb`, RUNTIME: `Ğdev Local Testnet`
//! BLOCK-NUM: `BlockId::Number(0)`
//! SKIP-WRITE: `false`, SKIP-READ: `false`, WARMUPS: `1`
//! STATE-VERSION: `V1`, STATE-CACHE-SIZE: ``
//! WEIGHT-PATH: `./runtime/gdev/src/weights/`
//! METRIC: `Average`, WEIGHT-MUL: `2.0`, WEIGHT-ADD: `0`

// Executed Command:
//   target/release/duniter
//   benchmark
//   storage
//   --dev
//   --mul=2
//   --weight-path=./runtime/gdev/src/weights/
//   --state-version=1
//   --database=paritydb
//   --disable-pov-recorder
//   --batch-size=100

/// Storage DB weights for the `Ğdev Local Testnet` runtime and `ParityDb`.
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
			///   Min, Max: 813, 1_437_055
			///   Average:  11_146
			///   Median:   1_943
			///   Std-Dev:  113453.3
			///
			/// Percentiles nanoseconds:
			///   99th: 23_333
			///   95th: 3_036
			///   75th: 2_389
			read: 22_292 * constants::WEIGHT_REF_TIME_PER_NANOS,

			/// Time to write one storage item.
			/// Calculated by multiplying the *Average* of all values with `2.0` and adding `0`.
			///
			/// Stats nanoseconds:
			///   Min, Max: 124_538, 124_538
			///   Average:  124_538
			///   Median:   124_538
			///   Std-Dev:  0.0
			///
			/// Percentiles nanoseconds:
			///   99th: 124_538
			///   95th: 124_538
			///   75th: 124_538
			write: 249_076 * constants::WEIGHT_REF_TIME_PER_NANOS,
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
