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

//! Autogenerated weights for `pallet_identity`
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
// --pallet=pallet-identity
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

/// Weight functions for `pallet_identity`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_identity::WeightInfo for WeightInfo<T> {
	// Storage: Identity IdentityIndexOf (r:2 w:1)
	// Storage: Identity Identities (r:2 w:2)
	// Storage: Cert StorageIdtyCertMeta (r:2 w:2)
	// Storage: Parameters ParametersStorage (r:1 w:0)
	// Storage: System Account (r:1 w:1)
	// Storage: Identity NextIdtyIndex (r:1 w:1)
	// Storage: Identity CounterForIdentities (r:1 w:1)
	// Storage: Identity IdentitiesRemovableOn (r:1 w:1)
	// Storage: Cert StorageCertsRemovableOn (r:1 w:1)
	// Storage: Cert CertsByReceiver (r:1 w:1)
	fn create_identity() -> Weight {
		// Minimum execution time: 121_934 nanoseconds.
		Weight::from_ref_time(138_522_000 as u64)
			.saturating_add(T::DbWeight::get().reads(13 as u64))
			.saturating_add(T::DbWeight::get().writes(11 as u64))
	}
	// Storage: Identity IdentityIndexOf (r:1 w:0)
	// Storage: Identity Identities (r:1 w:1)
	// Storage: Identity IdentitiesNames (r:1 w:1)
	// Storage: Membership PendingMembership (r:1 w:1)
	// Storage: Membership Membership (r:1 w:0)
	// Storage: Parameters ParametersStorage (r:1 w:0)
	// Storage: Membership PendingMembershipsExpireOn (r:1 w:1)
	fn confirm_identity() -> Weight {
		// Minimum execution time: 86_584 nanoseconds.
		Weight::from_ref_time(98_246_000 as u64)
			.saturating_add(T::DbWeight::get().reads(7 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	// Storage: Identity Identities (r:1 w:1)
	// Storage: Membership Membership (r:1 w:1)
	// Storage: Cert StorageIdtyCertMeta (r:1 w:0)
	// Storage: Parameters ParametersStorage (r:1 w:0)
	// Storage: Membership PendingMembership (r:1 w:1)
	// Storage: Membership CounterForMembership (r:1 w:1)
	// Storage: Membership MembershipsExpireOn (r:1 w:1)
	// Storage: UniversalDividend CurrentUdIndex (r:1 w:0)
	fn validate_identity() -> Weight {
		// Minimum execution time: 91_201 nanoseconds.
		Weight::from_ref_time(94_836_000 as u64)
			.saturating_add(T::DbWeight::get().reads(8 as u64))
			.saturating_add(T::DbWeight::get().writes(5 as u64))
	}
	// Storage: Identity IdentityIndexOf (r:2 w:2)
	// Storage: Identity Identities (r:1 w:1)
	// Storage: SmithMembership Membership (r:1 w:0)
	// Storage: System BlockHash (r:1 w:0)
	// Storage: System Account (r:2 w:2)
	// Storage: AuthorityMembers Members (r:1 w:0)
	fn change_owner_key() -> Weight {
		// Minimum execution time: 213_666 nanoseconds.
		Weight::from_ref_time(247_995_000 as u64)
			.saturating_add(T::DbWeight::get().reads(8 as u64))
			.saturating_add(T::DbWeight::get().writes(5 as u64))
	}
	// Storage: Identity Identities (r:1 w:1)
	// Storage: SmithMembership Membership (r:1 w:0)
	// Storage: System BlockHash (r:1 w:0)
	// Storage: Membership Membership (r:1 w:1)
	// Storage: Identity CounterForIdentities (r:1 w:1)
	// Storage: System Account (r:2 w:2)
	// Storage: Cert CertsByReceiver (r:1 w:1)
	// Storage: Cert StorageIdtyCertMeta (r:2 w:2)
	// Storage: Parameters ParametersStorage (r:1 w:0)
	// Storage: Identity IdentityIndexOf (r:0 w:1)
	fn revoke_identity() -> Weight {
		// Minimum execution time: 227_151 nanoseconds.
		Weight::from_ref_time(284_771_000 as u64)
			.saturating_add(T::DbWeight::get().reads(11 as u64))
			.saturating_add(T::DbWeight::get().writes(9 as u64))
	}
	// Storage: Identity Identities (r:1 w:1)
	// Storage: SmithMembership Membership (r:1 w:0)
	// Storage: Membership Membership (r:1 w:1)
	// Storage: Identity CounterForIdentities (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: Cert CertsByReceiver (r:1 w:1)
	// Storage: Cert StorageIdtyCertMeta (r:2 w:2)
	// Storage: Parameters ParametersStorage (r:1 w:0)
	// Storage: Identity IdentityIndexOf (r:0 w:1)
	// Storage: Identity IdentitiesNames (r:0 w:1)
	fn remove_identity() -> Weight {
		// Minimum execution time: 110_561 nanoseconds.
		Weight::from_ref_time(119_198_000 as u64)
			.saturating_add(T::DbWeight::get().reads(9 as u64))
			.saturating_add(T::DbWeight::get().writes(9 as u64))
	}
	// Storage: Identity IdentitiesNames (r:0 w:20)
	/// The range of component `i` is `[1, 1000]`.
	fn prune_item_identities_names(i: u32, ) -> Weight {
		// Minimum execution time: 10_675 nanoseconds.
		Weight::from_ref_time(10_809_000 as u64)
			// Standard Error: 3_281
			.saturating_add(Weight::from_ref_time(1_492_089 as u64).saturating_mul(i as u64))
			.saturating_add(T::DbWeight::get().writes((1 as u64).saturating_mul(i as u64)))
	}
	// Storage: System Account (r:1 w:1)
	fn fix_sufficients() -> Weight {
		// Minimum execution time: 23_639 nanoseconds.
		Weight::from_ref_time(29_075_000 as u64)
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
}