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
		//  Measured:  `1042`
		//  Estimated: `6982`
		// Minimum execution time: 68_835_000 picoseconds.
		Weight::from_parts(70_962_000, 0)
			.saturating_add(Weight::from_parts(0, 6982))
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
		//  Measured:  `784`
		//  Estimated: `6724`
		// Minimum execution time: 33_736_000 picoseconds.
		Weight::from_parts(34_847_000, 0)
			.saturating_add(Weight::from_parts(0, 6724))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: `Identity::IdentityIndexOf` (r:2 w:2)
	/// Proof: `Identity::IdentityIndexOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Identity::Identities` (r:1 w:1)
	/// Proof: `Identity::Identities` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `AuthorityMembers::OnlineAuthorities` (r:1 w:0)
	/// Proof: `AuthorityMembers::OnlineAuthorities` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `System::BlockHash` (r:1 w:0)
	/// Proof: `System::BlockHash` (`max_values`: None, `max_size`: Some(44), added: 2519, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:2 w:2)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(93), added: 2568, mode: `MaxEncodedLen`)
	fn change_owner_key() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `970`
		//  Estimated: `6910`
		// Minimum execution time: 82_861_000 picoseconds.
		Weight::from_parts(84_989_000, 0)
			.saturating_add(Weight::from_parts(0, 6910))
			.saturating_add(T::DbWeight::get().reads(7))
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
		//  Measured:  `673`
		//  Estimated: `6613`
		// Minimum execution time: 67_134_000 picoseconds.
		Weight::from_parts(68_928_000, 0)
			.saturating_add(Weight::from_parts(0, 6613))
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
		// Minimum execution time: 4_306_000 picoseconds.
		Weight::from_parts(4_458_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
			// Standard Error: 1_181
			.saturating_add(Weight::from_parts(1_236_346, 0).saturating_mul(i.into()))
			.saturating_add(T::DbWeight::get().writes(1))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(i.into())))
	}
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(93), added: 2568, mode: `MaxEncodedLen`)
	fn fix_sufficients() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `104`
		//  Estimated: `3558`
		// Minimum execution time: 7_251_000 picoseconds.
		Weight::from_parts(7_996_000, 0)
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
		//  Measured:  `379`
		//  Estimated: `3844`
		// Minimum execution time: 51_827_000 picoseconds.
		Weight::from_parts(53_174_000, 0)
			.saturating_add(Weight::from_parts(0, 3844))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn on_initialize() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 92_000 picoseconds.
		Weight::from_parts(122_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	/// Storage: `Identity::Identities` (r:1 w:0)
	/// Proof: `Identity::Identities` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn do_revoke_identity_noop() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `327`
		//  Estimated: `3792`
		// Minimum execution time: 4_974_000 picoseconds.
		Weight::from_parts(5_310_000, 0)
			.saturating_add(Weight::from_parts(0, 3792))
			.saturating_add(T::DbWeight::get().reads(1))
	}
	/// Storage: `Identity::Identities` (r:1 w:1)
	/// Proof: `Identity::Identities` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Identity::IdentityChangeSchedule` (r:2 w:1)
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
		//  Measured:  `1840`
		//  Estimated: `15205`
		// Minimum execution time: 104_830_000 picoseconds.
		Weight::from_parts(107_034_000, 0)
			.saturating_add(Weight::from_parts(0, 15205))
			.saturating_add(T::DbWeight::get().reads(18))
			.saturating_add(T::DbWeight::get().writes(21))
	}
	/// Storage: `Identity::Identities` (r:1 w:0)
	/// Proof: `Identity::Identities` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn do_remove_identity_noop() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `327`
		//  Estimated: `3792`
		// Minimum execution time: 4_842_000 picoseconds.
		Weight::from_parts(5_288_000, 0)
			.saturating_add(Weight::from_parts(0, 3792))
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
	/// Storage: `Certification::StorageIdtyCertMeta` (r:5 w:5)
	/// Proof: `Certification::StorageIdtyCertMeta` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Identity::IdentityIndexOf` (r:0 w:1)
	/// Proof: `Identity::IdentityIndexOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Quota::IdtyQuota` (r:0 w:1)
	/// Proof: `Quota::IdtyQuota` (`max_values`: None, `max_size`: Some(24), added: 2499, mode: `MaxEncodedLen`)
	/// Storage: `Session::KeyOwner` (r:0 w:4)
	/// Proof: `Session::KeyOwner` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn do_remove_identity() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2289`
		//  Estimated: `15654`
		// Minimum execution time: 142_003_000 picoseconds.
		Weight::from_parts(149_704_000, 0)
			.saturating_add(Weight::from_parts(0, 15654))
			.saturating_add(T::DbWeight::get().reads(23))
			.saturating_add(T::DbWeight::get().writes(29))
	}
	/// Storage: `Membership::Membership` (r:1 w:1)
	/// Proof: `Membership::Membership` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Membership::CounterForMembership` (r:1 w:1)
	/// Proof: `Membership::CounterForMembership` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `Membership::MembershipsExpireOn` (r:1 w:1)
	/// Proof: `Membership::MembershipsExpireOn` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Identity::Identities` (r:1 w:1)
	/// Proof: `Identity::Identities` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Identity::IdentityChangeSchedule` (r:2 w:1)
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
	/// Storage: `Certification::StorageIdtyCertMeta` (r:5 w:5)
	/// Proof: `Certification::StorageIdtyCertMeta` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Quota::IdtyQuota` (r:0 w:1)
	/// Proof: `Quota::IdtyQuota` (`max_values`: None, `max_size`: Some(24), added: 2499, mode: `MaxEncodedLen`)
	/// Storage: `Session::KeyOwner` (r:0 w:4)
	/// Proof: `Session::KeyOwner` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn do_remove_identity_handler() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2282`
		//  Estimated: `15647`
		// Minimum execution time: 132_078_000 picoseconds.
		Weight::from_parts(137_709_000, 0)
			.saturating_add(Weight::from_parts(0, 15647))
			.saturating_add(T::DbWeight::get().reads(24))
			.saturating_add(T::DbWeight::get().writes(27))
	}
	/// Storage: `Identity::Identities` (r:1 w:1)
	/// Proof: `Identity::Identities` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Identity::IdentityChangeSchedule` (r:2 w:2)
	/// Proof: `Identity::IdentityChangeSchedule` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn membership_removed() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `407`
		//  Estimated: `6347`
		// Minimum execution time: 15_678_000 picoseconds.
		Weight::from_parts(16_855_000, 0)
			.saturating_add(Weight::from_parts(0, 6347))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: `Identity::IdentityChangeSchedule` (r:1 w:0)
	/// Proof: `Identity::IdentityChangeSchedule` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn prune_identities_noop() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `108`
		//  Estimated: `3573`
		// Minimum execution time: 2_673_000 picoseconds.
		Weight::from_parts(2_986_000, 0)
			.saturating_add(Weight::from_parts(0, 3573))
			.saturating_add(T::DbWeight::get().reads(1))
	}
	/// Storage: `Identity::IdentityChangeSchedule` (r:1 w:1)
	/// Proof: `Identity::IdentityChangeSchedule` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Identity::Identities` (r:1 w:0)
	/// Proof: `Identity::Identities` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn prune_identities_none() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `325`
		//  Estimated: `3790`
		// Minimum execution time: 7_520_000 picoseconds.
		Weight::from_parts(7_977_000, 0)
			.saturating_add(Weight::from_parts(0, 3790))
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
		//  Measured:  `914`
		//  Estimated: `4379`
		// Minimum execution time: 37_327_000 picoseconds.
		Weight::from_parts(38_438_000, 0)
			.saturating_add(Weight::from_parts(0, 4379))
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(7))
	}
	/// Storage: `Identity::IdentitiesNames` (r:1 w:0)
	/// Proof: `Identity::IdentitiesNames` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Identity::Identities` (r:1 w:1)
	/// Proof: `Identity::Identities` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Identity::IdentityChangeSchedule` (r:2 w:2)
	/// Proof: `Identity::IdentityChangeSchedule` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Membership::Membership` (r:1 w:1)
	/// Proof: `Membership::Membership` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Quota::IdtyQuota` (r:0 w:1)
	/// Proof: `Quota::IdtyQuota` (`max_values`: None, `max_size`: Some(24), added: 2499, mode: `MaxEncodedLen`)
	fn revoke_identity_legacy() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `823`
		//  Estimated: `6763`
		// Minimum execution time: 132_003_000 picoseconds.
		Weight::from_parts(136_197_000, 0)
			.saturating_add(Weight::from_parts(0, 6763))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(5))
	}
}
