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

//! Autogenerated weights for `pallet_authority_members`
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
// --output=./runtime/gdev/src/weights/

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_authority_members`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_authority_members::WeightInfo for WeightInfo<T> {
	/// Storage: `Identity::IdentityIndexOf` (r:1 w:0)
	/// Proof: `Identity::IdentityIndexOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SmithMembers::Smiths` (r:1 w:0)
	/// Proof: `SmithMembers::Smiths` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `AuthorityMembers::Members` (r:1 w:0)
	/// Proof: `AuthorityMembers::Members` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `AuthorityMembers::OutgoingAuthorities` (r:1 w:1)
	/// Proof: `AuthorityMembers::OutgoingAuthorities` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AuthorityMembers::IncomingAuthorities` (r:1 w:0)
	/// Proof: `AuthorityMembers::IncomingAuthorities` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AuthorityMembers::OnlineAuthorities` (r:1 w:0)
	/// Proof: `AuthorityMembers::OnlineAuthorities` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn go_offline() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `933`
		//  Estimated: `4398`
		// Minimum execution time: 17_918_000 picoseconds.
		Weight::from_parts(18_513_000, 0)
			.saturating_add(Weight::from_parts(0, 4398))
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Identity::IdentityIndexOf` (r:1 w:0)
	/// Proof: `Identity::IdentityIndexOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SmithMembers::Smiths` (r:1 w:0)
	/// Proof: `SmithMembers::Smiths` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `AuthorityMembers::Blacklist` (r:1 w:0)
	/// Proof: `AuthorityMembers::Blacklist` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AuthorityMembers::Members` (r:1 w:0)
	/// Proof: `AuthorityMembers::Members` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Session::NextKeys` (r:1 w:0)
	/// Proof: `Session::NextKeys` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `AuthorityMembers::IncomingAuthorities` (r:1 w:1)
	/// Proof: `AuthorityMembers::IncomingAuthorities` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AuthorityMembers::OutgoingAuthorities` (r:1 w:0)
	/// Proof: `AuthorityMembers::OutgoingAuthorities` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AuthorityMembers::OnlineAuthorities` (r:1 w:0)
	/// Proof: `AuthorityMembers::OnlineAuthorities` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn go_online() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1390`
		//  Estimated: `4855`
		// Minimum execution time: 24_006_000 picoseconds.
		Weight::from_parts(25_000_000, 0)
			.saturating_add(Weight::from_parts(0, 4855))
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Identity::IdentityIndexOf` (r:1 w:0)
	/// Proof: `Identity::IdentityIndexOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SmithMembers::Smiths` (r:1 w:0)
	/// Proof: `SmithMembers::Smiths` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:0)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(93), added: 2568, mode: `MaxEncodedLen`)
	/// Storage: `Session::NextKeys` (r:1 w:1)
	/// Proof: `Session::NextKeys` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Session::KeyOwner` (r:4 w:0)
	/// Proof: `Session::KeyOwner` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `AuthorityMembers::Members` (r:0 w:1)
	/// Proof: `AuthorityMembers::Members` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn set_session_keys() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1839`
		//  Estimated: `12729`
		// Minimum execution time: 32_731_000 picoseconds.
		Weight::from_parts(34_093_000, 0)
			.saturating_add(Weight::from_parts(0, 12729))
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().writes(2))
	}
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
	/// Storage: `Session::KeyOwner` (r:0 w:4)
	/// Proof: `Session::KeyOwner` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn remove_member() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `848`
		//  Estimated: `4313`
		// Minimum execution time: 31_052_000 picoseconds.
		Weight::from_parts(32_171_000, 0)
			.saturating_add(Weight::from_parts(0, 4313))
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(10))
	}
	/// Storage: `AuthorityMembers::Blacklist` (r:1 w:1)
	/// Proof: `AuthorityMembers::Blacklist` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn remove_member_from_blacklist() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `199`
		//  Estimated: `1684`
		// Minimum execution time: 6_659_000 picoseconds.
		Weight::from_parts(7_014_000, 0)
			.saturating_add(Weight::from_parts(0, 1684))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}