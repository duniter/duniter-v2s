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

//! Autogenerated weights for `pallet_provide_randomness`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 47.0.0
//! DATE: 2025-04-09, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
// --output=./runtime/gtest/src/weights/

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_provide_randomness`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_provide_randomness::WeightInfo for WeightInfo<T> {
	/// Storage: `ProvideRandomness::CounterForRequestsIds` (r:1 w:1)
	/// Proof: `ProvideRandomness::CounterForRequestsIds` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(93), added: 2568, mode: `MaxEncodedLen`)
	/// Storage: `ProvideRandomness::RequestIdProvider` (r:1 w:1)
	/// Proof: `ProvideRandomness::RequestIdProvider` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `ProvideRandomness::RequestsIds` (r:1 w:1)
	/// Proof: `ProvideRandomness::RequestsIds` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Babe::EpochIndex` (r:1 w:0)
	/// Proof: `Babe::EpochIndex` (`max_values`: Some(1), `max_size`: Some(8), added: 503, mode: `MaxEncodedLen`)
	/// Storage: `ProvideRandomness::NexEpochHookIn` (r:1 w:0)
	/// Proof: `ProvideRandomness::NexEpochHookIn` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `ProvideRandomness::RequestsReadyAtEpoch` (r:1 w:1)
	/// Proof: `ProvideRandomness::RequestsReadyAtEpoch` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn request() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `298`
		//  Estimated: `3763`
		// Minimum execution time: 40_436_000 picoseconds.
		Weight::from_parts(41_114_000, 0)
			.saturating_add(Weight::from_parts(0, 3763))
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	/// Storage: `ProvideRandomness::RequestsReadyAtNextBlock` (r:1 w:1)
	/// Proof: `ProvideRandomness::RequestsReadyAtNextBlock` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Babe::AuthorVrfRandomness` (r:1 w:0)
	/// Proof: `Babe::AuthorVrfRandomness` (`max_values`: Some(1), `max_size`: Some(33), added: 528, mode: `MaxEncodedLen`)
	/// Storage: `ProvideRandomness::RequestsIds` (r:100 w:100)
	/// Proof: `ProvideRandomness::RequestsIds` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `ProvideRandomness::CounterForRequestsIds` (r:1 w:1)
	/// Proof: `ProvideRandomness::CounterForRequestsIds` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `ProvideRandomness::NexEpochHookIn` (r:1 w:1)
	/// Proof: `ProvideRandomness::NexEpochHookIn` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// The range of component `i` is `[1, 100]`.
	fn on_initialize(i: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `341 + i * (59 ±0)`
		//  Estimated: `1827 + i * (2535 ±0)`
		// Minimum execution time: 17_085_000 picoseconds.
		Weight::from_parts(16_264_018, 0)
			.saturating_add(Weight::from_parts(0, 1827))
			// Standard Error: 5_038
			.saturating_add(Weight::from_parts(5_015_846, 0).saturating_mul(i.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(i.into())))
			.saturating_add(T::DbWeight::get().writes(3))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(i.into())))
			.saturating_add(Weight::from_parts(0, 2535).saturating_mul(i.into()))
	}
	/// Storage: `ProvideRandomness::RequestsReadyAtNextBlock` (r:1 w:0)
	/// Proof: `ProvideRandomness::RequestsReadyAtNextBlock` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `ProvideRandomness::NexEpochHookIn` (r:1 w:1)
	/// Proof: `ProvideRandomness::NexEpochHookIn` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Babe::EpochIndex` (r:1 w:0)
	/// Proof: `Babe::EpochIndex` (`max_values`: Some(1), `max_size`: Some(8), added: 503, mode: `MaxEncodedLen`)
	/// Storage: `ProvideRandomness::RequestsReadyAtEpoch` (r:1 w:1)
	/// Proof: `ProvideRandomness::RequestsReadyAtEpoch` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Babe::NextRandomness` (r:1 w:0)
	/// Proof: `Babe::NextRandomness` (`max_values`: Some(1), `max_size`: Some(32), added: 527, mode: `MaxEncodedLen`)
	/// Storage: `Babe::EpochStart` (r:1 w:0)
	/// Proof: `Babe::EpochStart` (`max_values`: Some(1), `max_size`: Some(8), added: 503, mode: `MaxEncodedLen`)
	/// Storage: `ProvideRandomness::RequestsIds` (r:100 w:100)
	/// Proof: `ProvideRandomness::RequestsIds` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `ProvideRandomness::CounterForRequestsIds` (r:1 w:1)
	/// Proof: `ProvideRandomness::CounterForRequestsIds` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// The range of component `i` is `[1, 100]`.
	fn on_initialize_epoch(i: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `351 + i * (59 ±0)`
		//  Estimated: `3817 + i * (2535 ±0)`
		// Minimum execution time: 18_695_000 picoseconds.
		Weight::from_parts(16_499_706, 0)
			.saturating_add(Weight::from_parts(0, 3817))
			// Standard Error: 7_032
			.saturating_add(Weight::from_parts(5_391_937, 0).saturating_mul(i.into()))
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(i.into())))
			.saturating_add(T::DbWeight::get().writes(3))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(i.into())))
			.saturating_add(Weight::from_parts(0, 2535).saturating_mul(i.into()))
	}
}
