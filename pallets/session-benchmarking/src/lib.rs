// Copyright 2023 Axiom-Team
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

//! # Duniter Session Benchmarking Pallet
//!
//! This crate provides benchmarks specifically for the `pallet-session` within Duniter. Unlike traditional setups, this implementation is decoupled from the `staking-pallet`, which is not utilized in Duniter's architecture. Instead, session management functionalities are integrated into the `authority-members` pallet.
//!
//! ## Note
//!
//! This crate is separated from the main codebase due to cyclic dependency issues, focusing solely on session-related benchmarking independent of staking-related functionalities.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
use pallet_session::*;
use scale_info::prelude::vec::Vec;

pub struct Pallet<T: Config>(pallet_session::Pallet<T>);
pub trait Config: pallet_session::Config {
    /// Generate session keys together with a valid ownership proof for `owner`.
    ///
    /// `pallet_session::set_keys` now verifies the ownership proof, so the benchmark
    /// needs real keys and a matching proof. Producing them requires the concrete
    /// (non-opaque) `SessionKeys` type and a keystore, which are only available in the
    /// runtime; hence this is delegated here (mirroring `pallet-session-benchmarking`).
    fn generate_session_keys_and_proof(owner: Self::AccountId) -> (Self::Keys, Vec<u8>);
}

benchmarks! {
    set_keys {
        let caller: T::AccountId = whitelisted_caller();
        frame_system::Pallet::<T>::inc_providers(&caller);
        let (keys, proof) = T::generate_session_keys_and_proof(caller.clone());
    }: _(RawOrigin::Signed(caller), keys, proof)

    purge_keys {
        let caller: T::AccountId = whitelisted_caller();
        frame_system::Pallet::<T>::inc_providers(&caller);
        let (keys, proof) = T::generate_session_keys_and_proof(caller.clone());
        let _t = pallet_session::Pallet::<T>::set_keys(RawOrigin::Signed(caller.clone()).into(), keys, proof);
    }: _(RawOrigin::Signed(caller))
}
