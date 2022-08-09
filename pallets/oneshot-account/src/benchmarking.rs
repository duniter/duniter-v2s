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

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::pallet_prelude::IsType;
use frame_support::traits::Get;
use frame_system::RawOrigin;
use pallet_balances::Pallet as Balances;

use crate::Pallet;

const SEED: u32 = 0;

benchmarks! {
    where_clause { where
        T: pallet_balances::Config,
        T::Balance: From<u64>,
        <T::Currency as Currency<T::AccountId>>::Balance: IsType<T::Balance>
    }
    create_oneshot_account {
        let existential_deposit = T::ExistentialDeposit::get();
        let caller = whitelisted_caller();

        // Give some multiple of the existential deposit
        let balance = existential_deposit.saturating_mul((2).into());
        let _ = T::Currency::make_free_balance_be(&caller, balance.into());

        let recipient: T::AccountId = account("recipient", 0, SEED);
        let recipient_lookup: <T::Lookup as StaticLookup>::Source =
            T::Lookup::unlookup(recipient.clone());
        let transfer_amount = existential_deposit;
    }: _(RawOrigin::Signed(caller.clone()), recipient_lookup, transfer_amount.into())
    verify {
        assert_eq!(Balances::<T>::free_balance(&caller), transfer_amount);
        assert_eq!(OneshotAccounts::<T>::get(&recipient), Some(transfer_amount.into()));
    }
    where_clause { where
        T: pallet_balances::Config,
        T::Balance: From<u64>,
        <T::Currency as Currency<T::AccountId>>::Balance: IsType<T::Balance>+From<T::Balance>
    }
    consume_oneshot_account {
        let existential_deposit = T::ExistentialDeposit::get();
        let caller: T::AccountId = whitelisted_caller();

        // Give some multiple of the existential deposit
        let balance = existential_deposit.saturating_mul((2).into());
        OneshotAccounts::<T>::insert(
            caller.clone(),
            Into::<<T::Currency as Currency<T::AccountId>>::Balance>::into(balance)
        );

        // Deposit into a normal account is more expensive than into a oneshot account
        // so we create the recipient account with an existential deposit.
        let recipient: T::AccountId = account("recipient", 0, SEED);
        let recipient_lookup: <T::Lookup as StaticLookup>::Source =
            T::Lookup::unlookup(recipient.clone());
        let _ = T::Currency::make_free_balance_be(&recipient, existential_deposit.into());
    }: _(
        RawOrigin::Signed(caller.clone()),
        T::BlockNumber::zero(),
        Account::<<T::Lookup as StaticLookup>::Source>::Normal(recipient_lookup)
    )
    verify {
        assert_eq!(OneshotAccounts::<T>::get(&caller), None);
        assert_eq!(
            Balances::<T>::free_balance(&recipient),
            existential_deposit.saturating_mul((3).into())
        );
    }
    where_clause { where
        T: pallet_balances::Config,
        T::Balance: From<u64>,
        <T::Currency as Currency<T::AccountId>>::Balance: IsType<T::Balance>+From<T::Balance>
    }
    consume_oneshot_account_with_remaining {
        let existential_deposit = T::ExistentialDeposit::get();
        let caller: T::AccountId = whitelisted_caller();

        // Give some multiple of the existential deposit
        let balance = existential_deposit.saturating_mul((2).into());
        OneshotAccounts::<T>::insert(
            caller.clone(),
            Into::<<T::Currency as Currency<T::AccountId>>::Balance>::into(balance)
        );

        // Deposit into a normal account is more expensive than into a oneshot account
        // so we create the recipient accounts with an existential deposits.
        let recipient1: T::AccountId = account("recipient1", 0, SEED);
        let recipient1_lookup: <T::Lookup as StaticLookup>::Source =
            T::Lookup::unlookup(recipient1.clone());
        let _ = T::Currency::make_free_balance_be(&recipient1, existential_deposit.into());
        let recipient2: T::AccountId = account("recipient2", 1, SEED);
        let recipient2_lookup: <T::Lookup as StaticLookup>::Source =
            T::Lookup::unlookup(recipient2.clone());
        let _ = T::Currency::make_free_balance_be(&recipient2, existential_deposit.into());
    }: _(
        RawOrigin::Signed(caller.clone()),
        T::BlockNumber::zero(),
        Account::<<T::Lookup as StaticLookup>::Source>::Normal(recipient1_lookup),
        Account::<<T::Lookup as StaticLookup>::Source>::Normal(recipient2_lookup),
        existential_deposit.into()
    )
    verify {
        assert_eq!(OneshotAccounts::<T>::get(&caller), None);
        assert_eq!(
            Balances::<T>::free_balance(&recipient1),
            existential_deposit.saturating_mul((2).into())
        );
        assert_eq!(
            Balances::<T>::free_balance(&recipient2),
            existential_deposit.saturating_mul((2).into())
        );
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::new_test_ext(),
        crate::mock::Test
    );
}
