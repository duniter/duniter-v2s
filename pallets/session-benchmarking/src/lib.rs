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

//! Benchmarks for the Session Pallet.
// This is separated into its own crate due to cyclic dependency issues.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg(feature = "runtime-benchmarks")]
use parity_scale_codec::Decode;
use sp_std::{prelude::*, vec};

use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
use pallet_session::*;

pub struct Pallet<T: Config>(pallet_session::Pallet<T>);
pub trait Config: pallet_session::Config {}

benchmarks! {
    set_keys {
        let caller: T::AccountId = whitelisted_caller();
        frame_system::Pallet::<T>::inc_providers(&caller);
        let keys = T::Keys::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes()).unwrap();
        let proof: Vec<u8> = vec![0,1,2,3];
    }: _(RawOrigin::Signed(caller), keys, proof)

    purge_keys {
        let caller: T::AccountId = whitelisted_caller();
        frame_system::Pallet::<T>::inc_providers(&caller);
        let keys = T::Keys::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes()).unwrap();
        let proof: Vec<u8> = vec![0,1,2,3];
        let _t = pallet_session::Pallet::<T>::set_keys(RawOrigin::Signed(caller.clone()).into(), keys, proof);
    }: _(RawOrigin::Signed(caller))
}
