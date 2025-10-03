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
#![allow(clippy::multiple_bound_locations)]

use super::*;
use core::num::NonZeroU16;
use frame_benchmarking::{account, v2::*, whitelisted_caller};
use frame_support::{pallet_prelude::IsType, traits::StoredMap};
use frame_system::RawOrigin;
use pallet_balances::Pallet as Balances;

use crate::Pallet;

const ED_MULTIPLIER: u32 = 10;

#[benchmarks(
        where
        T: pallet_balances::Config, T::Balance: From<u64>,
        BalanceOf<T>: IsType<T::Balance>
)]
mod benchmarks {
    use super::*;

    fn assert_has_event<T: Config>(generic_event: <T as frame_system::Config>::RuntimeEvent) {
        frame_system::Pallet::<T>::assert_has_event(generic_event);
    }

    #[benchmark]
    fn claim_uds(i: Linear<1, { T::MaxPastReeval::get() }>) -> Result<(), BenchmarkError> {
        // Benchmark `transfer_ud` extrinsic with the worst possible conditions:
        // * Transfer will kill the sender account.
        // * Transfer will create the recipient account.
        let caller: T::AccountId = T::IdtyAttr::owner_key(1).unwrap();
        CurrentUdIndex::<T>::put(2054u16);
        T::MembersStorage::insert(
            &caller,
            FirstEligibleUd(Some(
                NonZeroU16::new(CurrentUdIndex::<T>::get() - i as u16).unwrap(),
            )),
        )?;
        let (_, uds_total) = compute_claim_uds::compute_claim_uds(
            CurrentUdIndex::<T>::get(),
            CurrentUdIndex::<T>::get() - i as u16,
            PastReevals::<T>::get().into_iter(),
        );

        #[extrinsic_call]
        _(RawOrigin::Signed(caller.clone()));

        assert_has_event::<T>(
            Event::<T>::UdsClaimed {
                count: i as u16,
                total: uds_total,
                who: caller,
            }
            .into(),
        );
        Ok(())
    }

    #[benchmark]
    fn transfer_ud() {
        let existential_deposit = T::ExistentialDeposit::get();
        let caller = whitelisted_caller();
        let balance = existential_deposit.saturating_mul(ED_MULTIPLIER.into());
        let _ = T::Currency::set_balance(&caller, balance.into());
        // Transfer `e - 1` existential deposits + 1 unit, which guarantees to create one account and reap this user.
        let recipient: T::AccountId = account("recipient", 0, 1);
        let recipient_lookup: <T::Lookup as StaticLookup>::Source =
            T::Lookup::unlookup(recipient.clone());
        let transfer_amount =
            existential_deposit.saturating_mul((ED_MULTIPLIER - 1).into()) + 1u32.into();
        let transfer_amount_ud =
            transfer_amount.saturating_mul(1_000.into()) / Pallet::<T>::current_ud().into();

        #[extrinsic_call]
        _(
            RawOrigin::Signed(caller.clone()),
            recipient_lookup,
            transfer_amount_ud.into(),
        );

        assert_eq!(Balances::<T>::free_balance(&caller), Zero::zero());
        assert_eq!(Balances::<T>::free_balance(&recipient), transfer_amount);
    }

    #[benchmark]
    fn transfer_ud_keep_alive() {
        // Benchmark `transfer_ud_keep_alive` with the worst possible condition:
        // * The recipient account is created.
        let caller = whitelisted_caller();
        let recipient: T::AccountId = account("recipient", 0, 1);
        let recipient_lookup: <T::Lookup as StaticLookup>::Source =
            T::Lookup::unlookup(recipient.clone());
        // Give the sender account max funds, thus a transfer will not kill account.
        let _ = T::Currency::set_balance(&caller, u32::MAX.into());
        let existential_deposit = T::ExistentialDeposit::get();
        let transfer_amount = existential_deposit.saturating_mul(ED_MULTIPLIER.into());
        let transfer_amount_ud =
            transfer_amount.saturating_mul(1_000.into()) / Pallet::<T>::current_ud().into();

        #[extrinsic_call]
        _(
            RawOrigin::Signed(caller.clone()),
            recipient_lookup,
            transfer_amount_ud.into(),
        );

        assert!(!Balances::<T>::free_balance(&caller).is_zero());
        assert_eq!(Balances::<T>::free_balance(&recipient), transfer_amount);
    }

    #[benchmark]
    fn on_removed_member(i: Linear<1, { T::MaxPastReeval::get() }>) -> Result<(), BenchmarkError> {
        let caller: T::AccountId = T::IdtyAttr::owner_key(1).unwrap();
        CurrentUdIndex::<T>::put(2054u16);
        T::MembersStorage::insert(
            &caller,
            FirstEligibleUd(Some(
                NonZeroU16::new(CurrentUdIndex::<T>::get() - i as u16).unwrap(),
            )),
        )?;
        let (_, uds_total) = compute_claim_uds::compute_claim_uds(
            CurrentUdIndex::<T>::get(),
            CurrentUdIndex::<T>::get() - i as u16,
            PastReevals::<T>::get().into_iter(),
        );

        #[block]
        {
            Pallet::<T>::on_removed_member(CurrentUdIndex::<T>::get() - i as u16, &caller);
        }

        if i != 0 {
            assert_has_event::<T>(
                Event::<T>::UdsAutoPaid {
                    count: i as u16,
                    total: uds_total,
                    who: caller,
                }
                .into(),
            );
        }
        Ok(())
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::new_test_ext(crate::mock::UniversalDividendConfig {
            first_reeval: Some(48_000),
            first_ud: Some(6_000),
            initial_monetary_mass: 0,
            initial_members: vec![1],
            ud: 10,
        }),
        crate::mock::Test
    );
}
