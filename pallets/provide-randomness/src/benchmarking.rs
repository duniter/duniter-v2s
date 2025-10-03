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
#![allow(clippy::multiple_bound_locations)]

use super::*;

use frame_benchmarking::{v2::*, whitelisted_caller};
use frame_support::{
    ensure,
    pallet_prelude::IsType,
    sp_runtime::{Saturating, traits::One},
    traits::{Get, OnInitialize, fungible::Mutate},
};
use frame_system::{RawOrigin, pallet_prelude::BlockNumberFor};
use sp_core::H256;

use crate::Pallet;

#[benchmarks(
        where
        T: pallet_balances::Config,
        T::Balance: From<u64>,
        BalanceOf<T>: IsType<T::Balance>,
        BlockNumberFor<T>: From<u32>,
)]
mod benchmarks {
    use super::*;

    fn assert_has_event<T: Config>(generic_event: <T as frame_system::Config>::RuntimeEvent) {
        frame_system::Pallet::<T>::assert_has_event(generic_event);
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

    #[benchmark]
    fn request() {
        // Get account
        let caller: T::AccountId = whitelisted_caller();

        // Provide deposit
        let existential_deposit = T::ExistentialDeposit::get();
        let balance = existential_deposit.saturating_mul((200).into());
        let _ = T::Currency::set_balance(&caller, balance.into());

        // Set randomness parameters
        let random = RandomnessType::RandomnessFromOneEpochAgo;
        let salt: H256 = H256([1; 32]);

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), random, salt);

        let request_id = RequestIdProvider::<T>::get() - 1;
        assert_has_event::<T>(
            Event::RequestedRandomness {
                request_id,
                salt,
                r#type: random,
            }
            .into(),
        );
    }

    #[benchmark]
    fn on_initialize(i: Linear<1, { T::MaxRequests::get() }>) -> Result<(), BenchmarkError> {
        add_requests_next_block::<T>(i)?;
        ensure!(RequestsIds::<T>::count() == i, "List not filled properly.");
        ensure!(
            RequestsReadyAtNextBlock::<T>::get().len() == i as usize,
            "List not filled properly."
        );
        let next_epoch_hook_in = NexEpochHookIn::<T>::mutate(|next_in| {
            core::mem::replace(next_in, next_in.saturating_sub(1))
        });
        ensure!(next_epoch_hook_in != 1, "Will be next epoch.");

        #[block]
        {
            Pallet::<T>::on_initialize(BlockNumberFor::<T>::one());
        }

        ensure!(RequestsIds::<T>::count() == 0, "List not processed.");
        ensure!(
            RequestsReadyAtNextBlock::<T>::get().is_empty(),
            "List not processed."
        );
        Ok(())
    }

    #[benchmark]
    fn on_initialize_epoch(i: Linear<1, { T::MaxRequests::get() }>) -> Result<(), BenchmarkError> {
        add_requests_next_epoch::<T>(i)?;
        ensure!(
            RequestsReadyAtNextBlock::<T>::get().is_empty(),
            "List not filled properly."
        );
        ensure!(RequestsIds::<T>::count() == i, "List not filled properly.");
        ensure!(
            RequestsReadyAtEpoch::<T>::get(T::GetCurrentEpochIndex::get()).len() == i as usize,
            "List not filled properly."
        );
        NexEpochHookIn::<T>::mutate(|next_in| core::mem::replace(next_in, 1));

        #[block]
        {
            Pallet::<T>::on_initialize(1.into());
        }

        ensure!(RequestsIds::<T>::count() == 0, "List not processed.");
        ensure!(
            RequestsReadyAtEpoch::<T>::get(T::GetCurrentEpochIndex::get()).is_empty(),
            "List not processed properly."
        );
        Ok(())
    }
}
