// Copyright 2021-2022 Axiom-Team
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

//! Autogenerated weights for `pallet_multisig`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 32.0.0
//! DATE: 2025-01-22, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `bgallois-ms7d43`, CPU: `12th Gen Intel(R) Core(TM) i3-12100F`
//! WASM-EXECUTION: `Compiled`, CHAIN: `None`, DB CACHE: 1024

// Executed Command:
// target/release/duniter
// benchmark
// pallet
// --genesis-builder=spec-genesis
// --steps=50
// --repeat=20
// --pallet=*
// --extrinsic=*
// --wasm-execution=compiled
// --heap-pages=4096
// --header=./file_header.txt
// --output=./runtime/g1/src/weights/

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_multisig`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_multisig::WeightInfo for WeightInfo<T> {
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_threshold_1(z: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 5_837_000 picoseconds.
		Weight::from_parts(6_128_523, 0)
			.saturating_add(Weight::from_parts(0, 0))
			// Standard Error: 2
			.saturating_add(Weight::from_parts(324, 0).saturating_mul(z.into()))
	}
	/// Storage: `Multisig::Multisigs` (r:1 w:1)
	/// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(457), added: 2932, mode: `MaxEncodedLen`)
	/// The range of component `s` is `[2, 10]`.
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_create(s: u32, z: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `125 + s * (5 ±0)`
		//  Estimated: `3922`
		// Minimum execution time: 26_711_000 picoseconds.
		Weight::from_parts(26_026_874, 0)
			.saturating_add(Weight::from_parts(0, 3922))
			// Standard Error: 5_830
			.saturating_add(Weight::from_parts(178_952, 0).saturating_mul(s.into()))
			// Standard Error: 5
			.saturating_add(Weight::from_parts(1_169, 0).saturating_mul(z.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Multisig::Multisigs` (r:1 w:1)
	/// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(457), added: 2932, mode: `MaxEncodedLen`)
	/// The range of component `s` is `[3, 10]`.
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_approve(s: u32, z: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `240`
		//  Estimated: `3922`
		// Minimum execution time: 15_581_000 picoseconds.
		Weight::from_parts(15_333_720, 0)
			.saturating_add(Weight::from_parts(0, 3922))
			// Standard Error: 3_745
			.saturating_add(Weight::from_parts(102_346, 0).saturating_mul(s.into()))
			// Standard Error: 2
			.saturating_add(Weight::from_parts(1_166, 0).saturating_mul(z.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Multisig::Multisigs` (r:1 w:1)
	/// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(457), added: 2932, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(93), added: 2568, mode: `MaxEncodedLen`)
	/// The range of component `s` is `[2, 10]`.
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_complete(s: u32, z: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `259 + s * (37 ±0)`
		//  Estimated: `3922`
		// Minimum execution time: 28_305_000 picoseconds.
		Weight::from_parts(27_996_672, 0)
			.saturating_add(Weight::from_parts(0, 3922))
			// Standard Error: 6_158
			.saturating_add(Weight::from_parts(202_739, 0).saturating_mul(s.into()))
			// Standard Error: 5
			.saturating_add(Weight::from_parts(1_195, 0).saturating_mul(z.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: `Multisig::Multisigs` (r:1 w:1)
	/// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(457), added: 2932, mode: `MaxEncodedLen`)
	/// The range of component `s` is `[2, 10]`.
	/// The range of component `z` is `[0, 10000]`.
	fn approve_as_multi_create(s: u32, z: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `125 + s * (5 ±0)`
		//  Estimated: `3922`
		// Minimum execution time: 23_926_000 picoseconds.
		Weight::from_parts(25_141_266, 0)
			.saturating_add(Weight::from_parts(0, 3922))
			// Standard Error: 5_251
			.saturating_add(Weight::from_parts(153_725, 0).saturating_mul(s.into()))
			// Standard Error: 4
			.saturating_add(Weight::from_parts(10, 0).saturating_mul(z.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Multisig::Multisigs` (r:1 w:1)
	/// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(457), added: 2932, mode: `MaxEncodedLen`)
	/// The range of component `s` is `[2, 10]`.
	/// The range of component `z` is `[0, 10000]`.
	fn approve_as_multi_approve(s: u32, z: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `240`
		//  Estimated: `3922`
		// Minimum execution time: 13_680_000 picoseconds.
		Weight::from_parts(14_213_245, 0)
			.saturating_add(Weight::from_parts(0, 3922))
			// Standard Error: 3_037
			.saturating_add(Weight::from_parts(108_757, 0).saturating_mul(s.into()))
			// Standard Error: 2
			.saturating_add(Weight::from_parts(15, 0).saturating_mul(z.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Multisig::Multisigs` (r:1 w:1)
	/// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(457), added: 2932, mode: `MaxEncodedLen`)
	/// The range of component `s` is `[2, 10]`.
	/// The range of component `z` is `[0, 10000]`.
	fn cancel_as_multi(s: u32, z: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `323 + s * (5 ±0)`
		//  Estimated: `3922`
		// Minimum execution time: 24_772_000 picoseconds.
		Weight::from_parts(25_953_083, 0)
			.saturating_add(Weight::from_parts(0, 3922))
			// Standard Error: 5_132
			.saturating_add(Weight::from_parts(144_988, 0).saturating_mul(s.into()))
			// Standard Error: 4
			.saturating_add(Weight::from_parts(7, 0).saturating_mul(z.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}
