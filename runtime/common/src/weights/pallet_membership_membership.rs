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

//! Autogenerated weights for `pallet_membership`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-05-16, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `Hugo`, CPU: `Intel(R) Core(TM) i7-8565U CPU @ 1.80GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("gdev-benchmark"), DB CACHE: 1024

// Executed Command:
// ./target/release/duniter
// benchmark
// pallet
// --chain=gdev-benchmark
// --steps=50
// --repeat=20
// --pallet=pallet-membership
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --header=./file_header.txt
// --output=./runtime/common/src/weights/

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_membership`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_membership::WeightInfo for WeightInfo<T> {
	// Storage: Membership PendingMembership (r:1 w:1)
	// Storage: Membership Membership (r:1 w:0)
	// Storage: Parameters ParametersStorage (r:1 w:0)
	// Storage: Membership PendingMembershipsExpireOn (r:1 w:1)
	fn force_request_membership() -> Weight {
		// Minimum execution time: 42_024 nanoseconds.
		Weight::from_ref_time(55_713_000 as u64)
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	// Storage: Identity IdentityIndexOf (r:1 w:0)
	// Storage: Identity Identities (r:1 w:0)
	fn request_membership() -> Weight {
		// Minimum execution time: 19_511 nanoseconds.
		Weight::from_ref_time(37_123_000 as u64)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
	}
	// Storage: Identity IdentityIndexOf (r:1 w:0)
	// Storage: Membership Membership (r:1 w:1)
	// Storage: Cert StorageIdtyCertMeta (r:1 w:0)
	// Storage: Parameters ParametersStorage (r:1 w:0)
	// Storage: Membership PendingMembership (r:1 w:1)
	// Storage: Membership CounterForMembership (r:1 w:1)
	// Storage: Membership MembershipsExpireOn (r:1 w:1)
	fn claim_membership() -> Weight {
		// Minimum execution time: 80_875 nanoseconds.
		Weight::from_ref_time(117_287_000 as u64)
			.saturating_add(T::DbWeight::get().reads(7 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	// Storage: Identity IdentityIndexOf (r:1 w:0)
	// Storage: Membership Membership (r:1 w:1)
	// Storage: Identity Identities (r:1 w:0)
	// Storage: Parameters ParametersStorage (r:1 w:0)
	// Storage: Membership MembershipsExpireOn (r:1 w:1)
	fn renew_membership() -> Weight {
		// Minimum execution time: 69_195 nanoseconds.
		Weight::from_ref_time(91_620_000 as u64)
			.saturating_add(T::DbWeight::get().reads(5 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	// Storage: Identity IdentityIndexOf (r:1 w:0)
	// Storage: Membership Membership (r:1 w:1)
	// Storage: Membership CounterForMembership (r:1 w:1)
	// Storage: Identity Identities (r:1 w:0)
	// Storage: UniversalDividend CurrentUdIndex (r:1 w:0)
	fn revoke_membership() -> Weight {
		// Minimum execution time: 64_419 nanoseconds.
		Weight::from_ref_time(103_812_000 as u64)
			.saturating_add(T::DbWeight::get().reads(5 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
}