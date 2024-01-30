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

use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::sp_runtime::{traits::One, Saturating};
use frame_support::traits::{Currency, Get};
use pallet_provide_randomness::OnFilledRandomness;

use crate::Pallet;

fn assert_has_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    frame_system::Pallet::<T>::assert_has_event(generic_event.into());
}

fn create_pending_accounts<T: Config>(
    i: u32,
    is_balance: bool,
    is_sufficient: bool,
) -> Result<(), &'static str> {
    for _ in 0..i {
        let caller: T::AccountId = whitelisted_caller();
        if is_balance {
            let existential_deposit = T::ExistentialDeposit::get();
            let balance = existential_deposit.saturating_mul((200u32).into());
            let _ = <pallet_balances::Pallet<T> as Currency<T::AccountId>>::make_free_balance_be(
                &caller, balance,
            );
        } else {
            assert!(
                frame_system::Pallet::<T>::get(&caller).free
                    < T::NewAccountPrice::get() + T::ExistentialDeposit::get()
            );
        }
        if is_sufficient {
            frame_system::Pallet::<T>::inc_sufficients(&caller);
        } else {
            assert!(frame_system::Pallet::<T>::sufficients(&caller) == 0);
        }
        PendingNewAccounts::<T>::insert(caller, ());
    }
    Ok(())
}

benchmarks! {
    unlink_identity {
        let account = account("Alice", 1, 1);
        let origin = frame_system::RawOrigin::Signed(account);
    }: _<T::RuntimeOrigin>(origin.into())
    on_initialize_sufficient  {
        let i in 0 .. T::MaxNewAccountsPerBlock::get() => create_pending_accounts::<T>(i, false, true)?;
    }: { Pallet::<T>::on_initialize(BlockNumberFor::<T>::one()); }
    on_initialize_with_balance {
        let i in 0 .. T::MaxNewAccountsPerBlock::get() => create_pending_accounts::<T>(i, true, false)?;
    }: { Pallet::<T>::on_initialize(BlockNumberFor::<T>::one()); }
    on_initialize_no_balance {
        let i in 0 .. T::MaxNewAccountsPerBlock::get() => create_pending_accounts::<T>(i, false, false)?;
    }: { Pallet::<T>::on_initialize(BlockNumberFor::<T>::one()); }
    on_filled_randomness_pending {
        let caller: T::AccountId = whitelisted_caller();
        let randomness = H256(T::AccountIdToSalt::convert(caller.clone()));
        let request_id = pallet_provide_randomness::Pallet::<T>::force_request(pallet_provide_randomness::RandomnessType::RandomnessFromTwoEpochsAgo, randomness);
        PendingRandomIdAssignments::<T>::insert(request_id, caller.clone());
    }: { Pallet::<T>::on_filled_randomness(request_id,  randomness); }
    verify {
        assert_has_event::<T>(Event::<T>::RandomIdAssigned { who: caller, random_id: randomness }.into());
    }
    on_filled_randomness_no_pending {
        let caller: T::AccountId = whitelisted_caller();
        let randomness = H256(T::AccountIdToSalt::convert(caller.clone()));
        let request_id = pallet_provide_randomness::Pallet::<T>::force_request(pallet_provide_randomness::RandomnessType::RandomnessFromTwoEpochsAgo, randomness);
        assert!(!PendingRandomIdAssignments::<T>::contains_key(request_id));
    }: { Pallet::<T>::on_filled_randomness(request_id,  randomness); }
}
