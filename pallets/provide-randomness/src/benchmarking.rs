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

fn add_requests_next_epoch<T: Config>(i: u32) -> Result<(), &'static str> {
    for _ in 0..i {
        let salt: H256 = H256([0; 32]);
        let request_id = RequestIdProvider::<T>::mutate(|next_request_id| {
            core::mem::replace(next_request_id, next_request_id.saturating_add(1))
        });
        RequestsIds::<T>::insert(request_id, ());
        RequestsReadyAtEpoch::<T>::append(
            T::GetCurrentEpochIndex::get(),
            Request { request_id, salt },
        );
    }
    Ok(())
}

benchmarks! {
    where_clause { where
        T: pallet_balances::Config,
        T::Balance: From<u64>,
        <T::Currency as Currency<T::AccountId>>::Balance: IsType<T::Balance>,
        T::BlockNumber: From<u32>,
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
    on_initialize {
        let i in 1 .. T::MaxRequests::get() => add_requests_next_block::<T>(i)?;
        ensure!(RequestsIds::<T>::count() == i, "List not filled properly.");
        ensure!(RequestsReadyAtNextBlock::<T>::get().len() == i as usize, "List not filled properly.");
        let next_epoch_hook_in = NexEpochHookIn::<T>::mutate(|next_in| {
            core::mem::replace(next_in, next_in.saturating_sub(1))
        });
        ensure!(next_epoch_hook_in != 1, "Will be next epoch.");
    }: { Pallet::<T>::on_initialize(T::BlockNumber::one()); }
    verify {
        ensure!(RequestsIds::<T>::count() == 0, "List not processed.");
        ensure!(RequestsReadyAtNextBlock::<T>::get().len() == 0, "List not processed.");
    }
    on_initialize_epoch {
        let i in 1 .. T::MaxRequests::get() => add_requests_next_epoch::<T>(i)?;
        ensure!(RequestsReadyAtNextBlock::<T>::get().len() == 0, "List not filled properly.");
        ensure!(RequestsIds::<T>::count() == i, "List not filled properly.");
        ensure!(RequestsReadyAtEpoch::<T>::get(T::GetCurrentEpochIndex::get()).len() == i as usize, "List not filled properly.");
        let next_epoch_hook_in = NexEpochHookIn::<T>::mutate(|next_in| {
            core::mem::replace(next_in, 1)
        });
    }: { Pallet::<T>::on_initialize(1.into()); }
    verify {
        ensure!(RequestsIds::<T>::count() == 0, "List not processed.");
        ensure!(RequestsReadyAtEpoch::<T>::get(T::GetCurrentEpochIndex::get()).len() == 0, "List not processed properly.");
    }
}
