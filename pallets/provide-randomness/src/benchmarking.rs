// Copyright 2021-2023 Axiom-Team
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

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::ensure;
use frame_support::pallet_prelude::IsType;
use frame_support::sp_runtime::{traits::One, Saturating};
use frame_support::traits::{Currency, Get, OnInitialize};
use frame_system::RawOrigin;
use sp_core::H256;

use crate::Pallet;

fn assert_has_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    frame_system::Pallet::<T>::assert_has_event(generic_event.into());
}

fn add_requests_next_block<T: Config>(i: u32) -> Result<(), &'static str> {
    for _ in 0..i {
        let salt: H256 = H256([0; 32]);
        let request_id = RequestIdProvider::<T>::mutate(|next_request_id| {
            core::mem::replace(next_request_id, next_request_id.saturating_add(1))
        });
        RequestsIds::<T>::insert(request_id, ());
        RequestsReadyAtNextBlock::<T>::append(Request { request_id, salt });
    }
    Ok(())
}

benchmarks! {
    where_clause { where
        T: pallet_balances::Config,
        T::Balance: From<u64>,
        <T::Currency as Currency<T::AccountId>>::Balance: IsType<T::Balance>
    }
    request {
        // Get account
        let caller: T::AccountId = whitelisted_caller();
        let caller_origin: <T as frame_system::Config>::RuntimeOrigin =
            RawOrigin::Signed(caller.clone()).into();

        // Provide deposit
        let existential_deposit = T::ExistentialDeposit::get();
        let balance = existential_deposit.saturating_mul((200).into());
        let _ = T::Currency::make_free_balance_be(&caller, balance.into());

        // Set randomness parameters
        let random = RandomnessType::RandomnessFromOneEpochAgo;
        let salt: H256 = H256([1; 32]);
    }: _<T::RuntimeOrigin>(caller_origin.clone(), random, salt.clone())
    verify {
        let request_id = RequestIdProvider::<T>::get() - 1;
        assert_has_event::<T>(Event::RequestedRandomness {
              request_id: request_id, salt: salt, r#type: random }.into() );
    }

    // Complexity depends on number of requests in RequestsReadyAtNextBlock and
    // in the RequestsReadyAtEpoch(at current epoch) and the sum of the two are bounded by MaxRequests.
    // The complexity is reduced to the number of elements in RequestsIds since the processing
    // of the two lists is quasi-identical.
    on_initialize {
        let i in 1 .. T::MaxRequests::get() => add_requests_next_block::<T>(i)?;
        ensure!(RequestsIds::<T>::count() == i, "List not filled properly.");
    }: { Pallet::<T>::on_initialize(T::BlockNumber::one()); }
    verify {
        ensure!(RequestsIds::<T>::count() == 0, "List not processed.");
    }
}
