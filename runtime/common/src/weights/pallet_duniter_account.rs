








//	NEED TO BE REPLACED BY FILE GENERATED WITH REFERENCE MACHINE 








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

//! Autogenerated weights for `common_runtime::duniter_account`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-03-24, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `benjamin-xps139380`, CPU: `Intel(R) Core(TM) i7-8565U CPU @ 1.80GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/duniter
// benchmark
// pallet
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=common_runtime::duniter_account
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --header=./file_header.txt
// --output=./runtime/common/src/weights/pallet_duniter_account.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `common_runtime::duniter_account`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_duniter_account::WeightInfo for WeightInfo<T> {
	// Storage: Account PendingNewAccounts (r:1 w:0)
	// Storage: ProvideRandomness RequestIdProvider (r:1 w:1)
	// Storage: ProvideRandomness RequestsIds (r:1 w:1)
	// Storage: ProvideRandomness CounterForRequestsIds (r:1 w:1)
	// Storage: Babe EpochIndex (r:1 w:0)
	// Storage: ProvideRandomness NexEpochHookIn (r:1 w:0)
	// Storage: ProvideRandomness RequestsReadyAtEpoch (r:1 w:1)
	// Storage: Account PendingRandomIdAssignments (r:0 w:1)
	/// The range of component `i` is `[0, 1]`.
	fn on_initialize_sufficient(i: u32, ) -> Weight {
		// Minimum execution time: 12_958 nanoseconds.
		Weight::from_ref_time(14_907_902 as u64)
			// Standard Error: 550_025
			.saturating_add(Weight::from_ref_time(79_482_297 as u64).saturating_mul(i as u64))
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().reads((6 as u64).saturating_mul(i as u64)))
			.saturating_add(T::DbWeight::get().writes((6 as u64).saturating_mul(i as u64)))
	}
	// Storage: Account PendingNewAccounts (r:1 w:0)
	// Storage: ProvideRandomness RequestIdProvider (r:1 w:1)
	// Storage: ProvideRandomness RequestsIds (r:1 w:1)
	// Storage: ProvideRandomness CounterForRequestsIds (r:1 w:1)
	// Storage: Babe EpochIndex (r:1 w:0)
	// Storage: ProvideRandomness NexEpochHookIn (r:1 w:0)
	// Storage: ProvideRandomness RequestsReadyAtEpoch (r:1 w:1)
	// Storage: Account PendingRandomIdAssignments (r:0 w:1)
	/// The range of component `i` is `[0, 1]`.
	fn on_initialize_with_balance(i: u32, ) -> Weight {
		// Minimum execution time: 12_965 nanoseconds.
		Weight::from_ref_time(16_754_718 as u64)
			// Standard Error: 1_790_537
			.saturating_add(Weight::from_ref_time(164_043_481 as u64).saturating_mul(i as u64))
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().reads((6 as u64).saturating_mul(i as u64)))
			.saturating_add(T::DbWeight::get().writes((6 as u64).saturating_mul(i as u64)))
	}
	// Storage: Account PendingNewAccounts (r:1 w:0)
	/// The range of component `i` is `[0, 1]`.
	fn on_initialize_no_balance(i: u32, ) -> Weight {
		// Minimum execution time: 12_912 nanoseconds.
		Weight::from_ref_time(13_846_469 as u64)
			// Standard Error: 115_598
			.saturating_add(Weight::from_ref_time(67_524_530 as u64).saturating_mul(i as u64))
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes((1 as u64).saturating_mul(i as u64)))
	}
	// Storage: Account PendingRandomIdAssignments (r:1 w:1)
	fn on_filled_randomness_pending() -> Weight {
		// Minimum execution time: 66_963 nanoseconds.
		Weight::from_ref_time(69_757_000 as u64)
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Account PendingRandomIdAssignments (r:1 w:0)
	fn on_filled_randomness_no_pending() -> Weight {
		// Minimum execution time: 16_088 nanoseconds.
		Weight::from_ref_time(27_963_000 as u64)
			.saturating_add(T::DbWeight::get().reads(1 as u64))
	}
}