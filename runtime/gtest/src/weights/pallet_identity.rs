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
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 32.0.0
//! DATE: 2024-09-09, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `bgallois-ms7d43`, CPU: `12th Gen Intel(R) Core(TM) i3-12100F`
//! WASM-EXECUTION: `Compiled`, CHAIN: `Some("dev")`, DB CACHE: 1024

// Executed Command:
// target/release/duniter
// benchmark
// pallet
// --chain=dev
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

/// Weight functions for `pallet_identity`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_identity::WeightInfo for WeightInfo<T> {
	/// Storage: `Identity::IdentityIndexOf` (r:2 w:1)
	/// Proof: `Identity::IdentityIndexOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Identity::Identities` (r:2 w:2)
	/// Proof: `Identity::Identities` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Certification::StorageIdtyCertMeta` (r:2 w:2)
	/// Proof: `Certification::StorageIdtyCertMeta` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(93), added: 2568, mode: `MaxEncodedLen`)
	/// Storage: `Identity::NextIdtyIndex` (r:1 w:1)
	/// Proof: `Identity::NextIdtyIndex` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Identity::IdentityChangeSchedule` (r:1 w:1)
	/// Proof: `Identity::IdentityChangeSchedule` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Identity::CounterForIdentities` (r:1 w:1)
	/// Proof: `Identity::CounterForIdentities` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `Certification::CertsRemovableOn` (r:1 w:1)
	/// Proof: `Certification::CertsRemovableOn` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Certification::CertsByReceiver` (r:1 w:1)
	/// Proof: `Certification::CertsByReceiver` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Quota::IdtyQuota` (r:0 w:1)
	/// Proof: `Quota::IdtyQuota` (`max_values`: None, `max_size`: Some(24), added: 2499, mode: `MaxEncodedLen`)
	fn create_identity() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1099`
		//  Estimated: `7039`
		// Minimum execution time: 56_179_000 picoseconds.
		Weight::from_parts(58_673_000, 0)
			.saturating_add(Weight::from_parts(0, 7039))
			.saturating_add(T::DbWeight::get().reads(12))
			.saturating_add(T::DbWeight::get().writes(12))
	}
	/// Storage: `Identity::IdentityIndexOf` (r:1 w:0)
	/// Proof: `Identity::IdentityIndexOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Identity::Identities` (r:1 w:1)
	/// Proof: `Identity::Identities` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Identity::IdentitiesNames` (r:1 w:1)
	/// Proof: `Identity::IdentitiesNames` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Identity::IdentityChangeSchedule` (r:2 w:2)
	/// Proof: `Identity::IdentityChangeSchedule` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn confirm_identity() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `822`
		//  Estimated: `6762`
		// Minimum execution time: 28_257_000 picoseconds.
		Weight::from_parts(29_816_000, 0)
			.saturating_add(Weight::from_parts(0, 6762))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: `Identity::IdentityIndexOf` (r:2 w:2)
	/// Proof: `Identity::IdentityIndexOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Identity::Identities` (r:1 w:1)
	/// Proof: `Identity::Identities` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `System::BlockHash` (r:1 w:0)
	/// Proof: `System::BlockHash` (`max_values`: None, `max_size`: Some(44), added: 2519, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:2 w:2)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(93), added: 2568, mode: `MaxEncodedLen`)
	fn change_owner_key() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `793`
		//  Estimated: `6733`
		// Minimum execution time: 71_115_000 picoseconds.
		Weight::from_parts(73_816_000, 0)
			.saturating_add(Weight::from_parts(0, 6733))
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	/// Storage: `Identity::Identities` (r:1 w:1)
	/// Proof: `Identity::Identities` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `System::BlockHash` (r:1 w:0)
	/// Proof: `System::BlockHash` (`max_values`: None, `max_size`: Some(44), added: 2519, mode: `MaxEncodedLen`)
	/// Storage: `Identity::IdentityChangeSchedule` (r:2 w:2)
	/// Proof: `Identity::IdentityChangeSchedule` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Membership::Membership` (r:1 w:1)
	/// Proof: `Membership::Membership` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Quota::IdtyQuota` (r:0 w:1)
	/// Proof: `Quota::IdtyQuota` (`max_values`: None, `max_size`: Some(24), added: 2499, mode: `MaxEncodedLen`)
	fn revoke_identity() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `731`
		//  Estimated: `6671`
		// Minimum execution time: 62_588_000 picoseconds.
		Weight::from_parts(63_816_000, 0)
			.saturating_add(Weight::from_parts(0, 6671))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	/// Storage: `Identity::IdentitiesNames` (r:0 w:999)
	/// Proof: `Identity::IdentitiesNames` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `i` is `[2, 1000]`.
	fn prune_item_identities_names(i: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 3_391_000 picoseconds.
		Weight::from_parts(3_543_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
			// Standard Error: 1_202
			.saturating_add(Weight::from_parts(1_239_262, 0).saturating_mul(i.into()))
			.saturating_add(T::DbWeight::get().writes(1))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(i.into())))
	}
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(93), added: 2568, mode: `MaxEncodedLen`)
	fn fix_sufficients() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `104`
		//  Estimated: `3558`
		// Minimum execution time: 5_920_000 picoseconds.
		Weight::from_parts(6_366_000, 0)
			.saturating_add(Weight::from_parts(0, 3558))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Identity::IdentityIndexOf` (r:1 w:0)
	/// Proof: `Identity::IdentityIndexOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `System::BlockHash` (r:1 w:0)
	/// Proof: `System::BlockHash` (`max_values`: None, `max_size`: Some(44), added: 2519, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(93), added: 2568, mode: `MaxEncodedLen`)
	fn link_account() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `436`
		//  Estimated: `3901`
		// Minimum execution time: 49_082_000 picoseconds.
		Weight::from_parts(50_732_000, 0)
			.saturating_add(Weight::from_parts(0, 3901))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn on_initialize() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 85_000 picoseconds.
		Weight::from_parts(96_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	/// Storage: `Identity::Identities` (r:1 w:0)
	/// Proof: `Identity::Identities` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn do_revoke_identity_noop() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `360`
		//  Estimated: `3825`
		// Minimum execution time: 3_515_000 picoseconds.
		Weight::from_parts(3_673_000, 0)
			.saturating_add(Weight::from_parts(0, 3825))
			.saturating_add(T::DbWeight::get().reads(1))
	}
	/// Storage: `Identity::Identities` (r:1 w:1)
	/// Proof: `Identity::Identities` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Identity::IdentityChangeSchedule` (r:2 w:2)
	/// Proof: `Identity::IdentityChangeSchedule` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Membership::Membership` (r:1 w:1)
	/// Proof: `Membership::Membership` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Membership::CounterForMembership` (r:1 w:1)
	/// Proof: `Membership::CounterForMembership` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `Membership::MembershipsExpireOn` (r:1 w:1)
	/// Proof: `Membership::MembershipsExpireOn` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `UniversalDividend::CurrentUdIndex` (r:1 w:0)
	/// Proof: `UniversalDividend::CurrentUdIndex` (`max_values`: Some(1), `max_size`: Some(2), added: 497, mode: `MaxEncodedLen`)
	/// Storage: `SmithMembers::Smiths` (r:5 w:5)
	/// Proof: `SmithMembers::Smiths` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `AuthorityMembers::Members` (r:1 w:1)
	/// Proof: `AuthorityMembers::Members` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `AuthorityMembers::OnlineAuthorities` (r:1 w:1)
	/// Proof: `AuthorityMembers::OnlineAuthorities` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AuthorityMembers::OutgoingAuthorities` (r:1 w:1)
	/// Proof: `AuthorityMembers::OutgoingAuthorities` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AuthorityMembers::IncomingAuthorities` (r:1 w:1)
	/// Proof: `AuthorityMembers::IncomingAuthorities` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Session::NextKeys` (r:1 w:1)
	/// Proof: `Session::NextKeys` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(93), added: 2568, mode: `MaxEncodedLen`)
	/// Storage: `Quota::IdtyQuota` (r:0 w:1)
	/// Proof: `Quota::IdtyQuota` (`max_values`: None, `max_size`: Some(24), added: 2499, mode: `MaxEncodedLen`)
	/// Storage: `Session::KeyOwner` (r:0 w:4)
	/// Proof: `Session::KeyOwner` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn do_revoke_identity() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1930`
		//  Estimated: `15295`
		// Minimum execution time: 88_168_000 picoseconds.
		Weight::from_parts(93_506_000, 0)
			.saturating_add(Weight::from_parts(0, 15295))
			.saturating_add(T::DbWeight::get().reads(18))
			.saturating_add(T::DbWeight::get().writes(22))
	}
	/// Storage: `Identity::Identities` (r:1 w:0)
	/// Proof: `Identity::Identities` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn do_remove_identity_noop() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `360`
		//  Estimated: `3825`
		// Minimum execution time: 3_390_000 picoseconds.
		Weight::from_parts(3_530_000, 0)
			.saturating_add(Weight::from_parts(0, 3825))
			.saturating_add(T::DbWeight::get().reads(1))
	}
	/// Storage: `Identity::Identities` (r:1 w:1)
	/// Proof: `Identity::Identities` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Identity::CounterForIdentities` (r:1 w:1)
	/// Proof: `Identity::CounterForIdentities` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:2 w:2)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(93), added: 2568, mode: `MaxEncodedLen`)
	/// Storage: `Membership::Membership` (r:1 w:1)
	/// Proof: `Membership::Membership` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Membership::CounterForMembership` (r:1 w:1)
	/// Proof: `Membership::CounterForMembership` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `Membership::MembershipsExpireOn` (r:1 w:1)
	/// Proof: `Membership::MembershipsExpireOn` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SmithMembers::Smiths` (r:5 w:5)
	/// Proof: `SmithMembers::Smiths` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `AuthorityMembers::Members` (r:1 w:1)
	/// Proof: `AuthorityMembers::Members` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `AuthorityMembers::OnlineAuthorities` (r:1 w:1)
	/// Proof: `AuthorityMembers::OnlineAuthorities` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AuthorityMembers::OutgoingAuthorities` (r:1 w:1)
	/// Proof: `AuthorityMembers::OutgoingAuthorities` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AuthorityMembers::IncomingAuthorities` (r:1 w:1)
	/// Proof: `AuthorityMembers::IncomingAuthorities` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Session::NextKeys` (r:1 w:1)
	/// Proof: `Session::NextKeys` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Certification::CertsByReceiver` (r:1 w:1)
	/// Proof: `Certification::CertsByReceiver` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Certification::StorageIdtyCertMeta` (r:6 w:6)
	/// Proof: `Certification::StorageIdtyCertMeta` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Identity::IdentityIndexOf` (r:0 w:1)
	/// Proof: `Identity::IdentityIndexOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Quota::IdtyQuota` (r:0 w:1)
	/// Proof: `Quota::IdtyQuota` (`max_values`: None, `max_size`: Some(24), added: 2499, mode: `MaxEncodedLen`)
	/// Storage: `Session::KeyOwner` (r:0 w:4)
	/// Proof: `Session::KeyOwner` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn do_remove_identity() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2355`
		//  Estimated: `18195`
		// Minimum execution time: 128_934_000 picoseconds.
		Weight::from_parts(132_596_000, 0)
			.saturating_add(Weight::from_parts(0, 18195))
			.saturating_add(T::DbWeight::get().reads(24))
			.saturating_add(T::DbWeight::get().writes(30))
	}
	/// Storage: `Membership::Membership` (r:1 w:1)
	/// Proof: `Membership::Membership` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Membership::CounterForMembership` (r:1 w:1)
	/// Proof: `Membership::CounterForMembership` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `Membership::MembershipsExpireOn` (r:1 w:1)
	/// Proof: `Membership::MembershipsExpireOn` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Identity::Identities` (r:1 w:1)
	/// Proof: `Identity::Identities` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Identity::IdentityChangeSchedule` (r:2 w:2)
	/// Proof: `Identity::IdentityChangeSchedule` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `UniversalDividend::CurrentUdIndex` (r:1 w:0)
	/// Proof: `UniversalDividend::CurrentUdIndex` (`max_values`: Some(1), `max_size`: Some(2), added: 497, mode: `MaxEncodedLen`)
	/// Storage: `SmithMembers::Smiths` (r:5 w:5)
	/// Proof: `SmithMembers::Smiths` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `AuthorityMembers::Members` (r:1 w:1)
	/// Proof: `AuthorityMembers::Members` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `AuthorityMembers::OnlineAuthorities` (r:1 w:1)
	/// Proof: `AuthorityMembers::OnlineAuthorities` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AuthorityMembers::OutgoingAuthorities` (r:1 w:1)
	/// Proof: `AuthorityMembers::OutgoingAuthorities` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AuthorityMembers::IncomingAuthorities` (r:1 w:1)
	/// Proof: `AuthorityMembers::IncomingAuthorities` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Session::NextKeys` (r:1 w:1)
	/// Proof: `Session::NextKeys` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(93), added: 2568, mode: `MaxEncodedLen`)
	/// Storage: `Certification::CertsByReceiver` (r:1 w:1)
	/// Proof: `Certification::CertsByReceiver` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Certification::StorageIdtyCertMeta` (r:6 w:6)
	/// Proof: `Certification::StorageIdtyCertMeta` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Quota::IdtyQuota` (r:0 w:1)
	/// Proof: `Quota::IdtyQuota` (`max_values`: None, `max_size`: Some(24), added: 2499, mode: `MaxEncodedLen`)
	/// Storage: `Session::KeyOwner` (r:0 w:4)
	/// Proof: `Session::KeyOwner` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn do_remove_identity_handler() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2372`
		//  Estimated: `18212`
		// Minimum execution time: 124_614_000 picoseconds.
		Weight::from_parts(133_121_000, 0)
			.saturating_add(Weight::from_parts(0, 18212))
			.saturating_add(T::DbWeight::get().reads(25))
			.saturating_add(T::DbWeight::get().writes(29))
	}
	/// Storage: `Identity::Identities` (r:1 w:1)
	/// Proof: `Identity::Identities` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Identity::IdentityChangeSchedule` (r:2 w:2)
	/// Proof: `Identity::IdentityChangeSchedule` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn membership_removed() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `441`
		//  Estimated: `6381`
		// Minimum execution time: 13_448_000 picoseconds.
		Weight::from_parts(14_160_000, 0)
			.saturating_add(Weight::from_parts(0, 6381))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: `Identity::IdentityChangeSchedule` (r:1 w:0)
	/// Proof: `Identity::IdentityChangeSchedule` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn prune_identities_noop() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `165`
		//  Estimated: `3630`
		// Minimum execution time: 2_555_000 picoseconds.
		Weight::from_parts(2_794_000, 0)
			.saturating_add(Weight::from_parts(0, 3630))
			.saturating_add(T::DbWeight::get().reads(1))
	}
	/// Storage: `Identity::IdentityChangeSchedule` (r:1 w:1)
	/// Proof: `Identity::IdentityChangeSchedule` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Identity::Identities` (r:1 w:0)
	/// Proof: `Identity::Identities` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn prune_identities_none() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `363`
		//  Estimated: `3828`
		// Minimum execution time: 5_568_000 picoseconds.
		Weight::from_parts(5_799_000, 0)
			.saturating_add(Weight::from_parts(0, 3828))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Identity::IdentityChangeSchedule` (r:1 w:1)
	/// Proof: `Identity::IdentityChangeSchedule` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Identity::Identities` (r:1 w:1)
	/// Proof: `Identity::Identities` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Identity::CounterForIdentities` (r:1 w:1)
	/// Proof: `Identity::CounterForIdentities` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(93), added: 2568, mode: `MaxEncodedLen`)
	/// Storage: `Membership::Membership` (r:1 w:1)
	/// Proof: `Membership::Membership` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Certification::CertsByReceiver` (r:1 w:0)
	/// Proof: `Certification::CertsByReceiver` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Identity::IdentityIndexOf` (r:0 w:1)
	/// Proof: `Identity::IdentityIndexOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Quota::IdtyQuota` (r:0 w:1)
	/// Proof: `Quota::IdtyQuota` (`max_values`: None, `max_size`: Some(24), added: 2499, mode: `MaxEncodedLen`)
	fn prune_identities_err() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `952`
		//  Estimated: `4417`
		// Minimum execution time: 29_403_000 picoseconds.
		Weight::from_parts(30_245_000, 0)
			.saturating_add(Weight::from_parts(0, 4417))
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(7))
	}
}