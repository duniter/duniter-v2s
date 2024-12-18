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
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 43.0.0
//! DATE: 2024-11-14, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
		// Minimum execution time: 5_207_000 picoseconds.
		Weight::from_parts(5_671_298, 0)
			.saturating_add(Weight::from_parts(0, 0))
			// Standard Error: 2
			.saturating_add(Weight::from_parts(321, 0).saturating_mul(z.into()))
	}
	/// Storage: `Multisig::Multisigs` (r:1 w:1)
	/// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(457), added: 2932, mode: `MaxEncodedLen`)
	/// The range of component `s` is `[2, 10]`.
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_create(s: u32, z: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `125 + s * (5 ±0)`
		//  Estimated: `3922`
		// Minimum execution time: 28_382_000 picoseconds.
		Weight::from_parts(28_468_579, 0)
			.saturating_add(Weight::from_parts(0, 3922))
			// Standard Error: 10_030
			.saturating_add(Weight::from_parts(70_271, 0).saturating_mul(s.into()))
			// Standard Error: 8
			.saturating_add(Weight::from_parts(1_052, 0).saturating_mul(z.into()))
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
		// Minimum execution time: 15_563_000 picoseconds.
		Weight::from_parts(15_235_616, 0)
			.saturating_add(Weight::from_parts(0, 3922))
			// Standard Error: 7_326
			.saturating_add(Weight::from_parts(128_782, 0).saturating_mul(s.into()))
			// Standard Error: 5
			.saturating_add(Weight::from_parts(1_080, 0).saturating_mul(z.into()))
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
		// Minimum execution time: 29_858_000 picoseconds.
		Weight::from_parts(30_559_431, 0)
			.saturating_add(Weight::from_parts(0, 3922))
			// Standard Error: 9_588
			.saturating_add(Weight::from_parts(102_725, 0).saturating_mul(s.into()))
			// Standard Error: 8
			.saturating_add(Weight::from_parts(1_083, 0).saturating_mul(z.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: `Multisig::Multisigs` (r:1 w:1)
	/// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(457), added: 2932, mode: `MaxEncodedLen`)
	/// The range of component `s` is `[2, 10]`.
	fn approve_as_multi_create(s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `130 + s * (5 ±0)`
		//  Estimated: `3922`
		// Minimum execution time: 25_071_000 picoseconds.
		Weight::from_parts(26_371_672, 0)
			.saturating_add(Weight::from_parts(0, 3922))
			// Standard Error: 9_270
			.saturating_add(Weight::from_parts(229_206, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Multisig::Multisigs` (r:1 w:1)
	/// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(457), added: 2932, mode: `MaxEncodedLen`)
	/// The range of component `s` is `[2, 10]`.
	fn approve_as_multi_approve(s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `240`
		//  Estimated: `3922`
		// Minimum execution time: 13_391_000 picoseconds.
		Weight::from_parts(14_298_679, 0)
			.saturating_add(Weight::from_parts(0, 3922))
			// Standard Error: 5_609
			.saturating_add(Weight::from_parts(164_451, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Multisig::Multisigs` (r:1 w:1)
	/// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(457), added: 2932, mode: `MaxEncodedLen`)
	/// The range of component `s` is `[2, 10]`.
	fn cancel_as_multi(s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `328 + s * (5 ±0)`
		//  Estimated: `3922`
		// Minimum execution time: 26_368_000 picoseconds.
		Weight::from_parts(27_511_820, 0)
			.saturating_add(Weight::from_parts(0, 3922))
			// Standard Error: 10_806
			.saturating_add(Weight::from_parts(199_066, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}
