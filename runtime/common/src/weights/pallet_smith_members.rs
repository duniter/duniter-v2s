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

//! Autogenerated weights for `pallet_smith_members`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2024-02-04, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
// --output=./runtime/common/src/weights/

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_smith_members`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_smith_members::WeightInfo for WeightInfo<T> {
	/// Storage: `Identity::IdentityIndexOf` (r:1 w:0)
	/// Proof: `Identity::IdentityIndexOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SmithMembers::Smiths` (r:2 w:1)
	/// Proof: `SmithMembers::Smiths` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Membership::Membership` (r:1 w:0)
	/// Proof: `Membership::Membership` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SmithMembers::CurrentSession` (r:1 w:0)
	/// Proof: `SmithMembers::CurrentSession` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Parameters::ParametersStorage` (r:1 w:0)
	/// Proof: `Parameters::ParametersStorage` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SmithMembers::ExpiresOn` (r:1 w:1)
	/// Proof: `SmithMembers::ExpiresOn` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn invite_smith() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `765`
		//  Estimated: `6705`
		// Minimum execution time: 24_603_000 picoseconds.
		Weight::from_parts(26_004_000, 0)
			.saturating_add(Weight::from_parts(0, 6705))
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: `Identity::IdentityIndexOf` (r:1 w:0)
	/// Proof: `Identity::IdentityIndexOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SmithMembers::Smiths` (r:1 w:1)
	/// Proof: `SmithMembers::Smiths` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn accept_invitation() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `496`
		//  Estimated: `3961`
		// Minimum execution time: 13_956_000 picoseconds.
		Weight::from_parts(14_691_000, 0)
			.saturating_add(Weight::from_parts(0, 3961))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Identity::IdentityIndexOf` (r:1 w:0)
	/// Proof: `Identity::IdentityIndexOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SmithMembers::Smiths` (r:2 w:2)
	/// Proof: `SmithMembers::Smiths` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Parameters::ParametersStorage` (r:1 w:0)
	/// Proof: `Parameters::ParametersStorage` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SmithMembers::CurrentSession` (r:1 w:0)
	/// Proof: `SmithMembers::CurrentSession` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SmithMembers::ExpiresOn` (r:1 w:1)
	/// Proof: `SmithMembers::ExpiresOn` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn certify_smith() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `626`
		//  Estimated: `6566`
		// Minimum execution time: 25_044_000 picoseconds.
		Weight::from_parts(25_893_000, 0)
			.saturating_add(Weight::from_parts(0, 6566))
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(3))
	}
}