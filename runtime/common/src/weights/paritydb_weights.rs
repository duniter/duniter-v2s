
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2024-01-24 (Y/M/D)
//! HOSTNAME: `bgallois-ms7d43`, CPU: `12th Gen Intel(R) Core(TM) i3-12100F`
//!
//! DATABASE: `ParityDb`, RUNTIME: `Ğdev Local Testnet`
//! BLOCK-NUM: `BlockId::Number(0)`
//! SKIP-WRITE: `false`, SKIP-READ: `false`, WARMUPS: `1`
//! STATE-VERSION: `V1`, STATE-CACHE-SIZE: ``
//! WEIGHT-PATH: `./runtime/common/src/weights/`
//! METRIC: `Average`, WEIGHT-MUL: `2.0`, WEIGHT-ADD: `0`

// Executed Command:
//   target/release/duniter
//   benchmark
//   storage
//   --chain=dev
//   --mul=2
//   --weight-path=./runtime/common/src/weights/
//   --state-version=1

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
			///   Min, Max: 951, 1_277_318
			///   Average:  11_592
			///   Median:   1_567
			///   Std-Dev:  111445.56
			///
			/// Percentiles nanoseconds:
			///   99th: 10_193
			///   95th: 2_928
			///   75th: 1_855
			read: 23_184 * constants::WEIGHT_REF_TIME_PER_NANOS,

			/// Time to write one storage item.
			/// Calculated by multiplying the *Average* of all values with `2.0` and adding `0`.
			///
			/// Stats nanoseconds:
			///   Min, Max: 3_824, 6_440_944
			///   Average:  58_440
			///   Median:   8_907
			///   Std-Dev:  561954.34
			///
			/// Percentiles nanoseconds:
			///   99th: 18_256
			///   95th: 13_376
			///   75th: 10_858
			write: 116_880 * constants::WEIGHT_REF_TIME_PER_NANOS,
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
