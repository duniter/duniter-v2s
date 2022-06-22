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
use frame_support::traits::{Get, OnInitialize};
use frame_system::RawOrigin;
use pallet_balances::Pallet as Balances;
use sp_runtime::traits::Bounded;

use crate::Pallet;

const BLOCK_NUMBER: u32 = 2;
const ED_MULTIPLIER: u32 = 10;
const SEED: u32 = 0;

/*fn fill_storage<T: Config>(
    members_count: u32,
) -> Result<(), &'static str> {
    Ok(())
}*/

benchmarks! {
    // TODO add on_initialize_ud_created and on_initialize_ud_reevalued (after manual UD impl)
    on_initialize {
        //let n in 1 .. 10_000;
        //fill_storage::<T>(n)?;
    }: { Pallet::<T>::on_initialize(BLOCK_NUMBER.into()); }
    verify {
    }
    // Benchmark `transfer_ud` extrinsic with the worst possible conditions:
    // * Transfer will kill the sender account.
    // * Transfer will create the recipient account.
    where_clause { where T: pallet_balances::Config, T::Balance: From<u64>, <T::Currency as Currency<T::AccountId>>::Balance: IsType<T::Balance> }
    transfer_ud {
        let existential_deposit = T::ExistentialDeposit::get();
        let caller = whitelisted_caller();

        // Give some multiple of the existential deposit
        let balance = existential_deposit.saturating_mul(ED_MULTIPLIER.into());
        let _ = T::Currency::make_free_balance_be(&caller, balance.into());

        // Transfer `e - 1` existential deposits + 1 unit, which guarantees to create one account,
        // and reap this user.
        let recipient: T::AccountId = account("recipient", 0, SEED);
        let recipient_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(recipient.clone());
        let transfer_amount = existential_deposit.saturating_mul((ED_MULTIPLIER - 1).into()) + 1u32.into();
        let transfer_amount_ud = transfer_amount.saturating_mul(1_000.into()) / Pallet::<T>::current_ud().into();
    }: _(RawOrigin::Signed(caller.clone()), recipient_lookup, transfer_amount_ud.into())
    verify {
        assert_eq!(Balances::<T>::free_balance(&caller), Zero::zero());
        assert_eq!(Balances::<T>::free_balance(&recipient), transfer_amount);
    }

    // Benchmark `transfer_ud_keep_alive` with the worst possible condition:
    // * The recipient account is created.
    where_clause { where T: pallet_balances::Config, T::Balance: From<u64>, <T::Currency as Currency<T::AccountId>>::Balance: IsType<T::Balance> }
    transfer_ud_keep_alive {
        let caller = whitelisted_caller();
        let recipient: T::AccountId = account("recipient", 0, SEED);
        let recipient_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(recipient.clone());

        // Give the sender account max funds, thus a transfer will not kill account.
        let _ = T::Currency::make_free_balance_be(&caller, <T::Currency as Currency<T::AccountId>>::Balance::max_value());
        let existential_deposit = T::ExistentialDeposit::get();
        let transfer_amount = existential_deposit.saturating_mul(ED_MULTIPLIER.into());
        let transfer_amount_ud = transfer_amount.saturating_mul(1_000.into()) / Pallet::<T>::current_ud().into();
    }: _(RawOrigin::Signed(caller.clone()), recipient_lookup, transfer_amount_ud.into())
    verify {
        assert!(!Balances::<T>::free_balance(&caller).is_zero());
        assert_eq!(Balances::<T>::free_balance(&recipient), transfer_amount);
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::new_test_ext(crate::mock::UniversalDividendConfig {
            first_reeval: 8,
            first_ud: 1_000,
            initial_monetary_mass: 0,
        }),
        crate::mock::Test
    );
}
