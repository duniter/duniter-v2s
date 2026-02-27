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
use frame_benchmarking::{account, v2::*};
use frame_support::traits::fungible::Mutate;
use sp_runtime::traits::One;

fn assert_has_event<T: Config>(generic_event: <T as frame_system::Config>::RuntimeEvent) {
    frame_system::Pallet::<T>::assert_has_event(generic_event);
}

#[benchmarks(
        where
            IdtyId<T>: From<u32>,
            BalanceOf<T>: From<u64>,
            T::AccountId: From<[u8; 32]>,
)]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn queue_refund() {
        let account: T::AccountId = account("Alice", 1, 1);
        let dummy_refund = Refund {
            account: account.clone(),
            identity: 0u32.into(),
            amount: 20u64.into(),
        };
        let refund = Refund {
            account,
            identity: 1u32.into(),
            amount: 10u64.into(),
        };
        // Complexity is bound to MAX_QUEUD_REFUNDS where an insertion is O(n-1)
        for _ in 0..MAX_QUEUED_REFUNDS - 1 {
            Pallet::<T>::queue_refund(dummy_refund.clone())
        }

        #[block]
        {
            Pallet::<T>::queue_refund(refund.clone());
        }

        assert_eq!(RefundQueue::<T>::get().last(), Some(refund).as_ref());
        assert_eq!(RefundQueue::<T>::get().len() as u32, MAX_QUEUED_REFUNDS);
    }

    #[benchmark]
    fn spend_quota() {
        let idty_id: IdtyId<T> = 1u32.into();
        let amount = 2u64;
        let quota_amount = 10u64;
        IdtyQuota::<T>::insert(
            idty_id,
            Quota {
                last_use: BlockNumberFor::<T>::zero(),
                amount: quota_amount.into(),
            },
        );

        #[block]
        {
            Pallet::<T>::spend_quota(idty_id, amount.into());
        }

        let quota_growth =
            sp_runtime::Perbill::from_rational(BlockNumberFor::<T>::one(), T::ReloadRate::get())
                .mul_floor(T::MaxQuota::get());
        assert_eq!(
            IdtyQuota::<T>::get(idty_id).unwrap().amount,
            quota_growth + quota_amount.into() - amount.into()
        );
    }

    #[benchmark]
    fn try_refund() {
        let account: T::AccountId = account("Alice", 1, 1);
        let idty_id: IdtyId<T> = 1u32.into();
        IdtyQuota::<T>::insert(
            idty_id,
            Quota {
                last_use: BlockNumberFor::<T>::zero(),
                amount: 10u64.into(),
            },
        );
        let _ = CurrencyOf::<T>::set_balance(&T::RefundAccount::get(), u32::MAX.into());
        // The worst-case scenario is when the refund fails
        // and can only be triggered if the account is dead,
        // in this case by having no balance in the account.
        let refund = Refund {
            account: account.clone(),
            identity: 1u32.into(),
            amount: 10u64.into(),
        };

        #[block]
        {
            Pallet::<T>::try_refund(refund);
        }

        assert_has_event::<T>(Event::<T>::RefundFailed(account).into());
    }

    #[benchmark]
    fn do_refund() {
        let account: T::AccountId = account("Alice", 1, 1);
        let _ = CurrencyOf::<T>::set_balance(&T::RefundAccount::get(), u32::MAX.into());
        // Worst branch for do_refund: withdraw succeeds, then resolve_into_existing fails
        // because the destination account does not exist.
        let refund = Refund {
            account: account.clone(),
            identity: 1u32.into(),
            amount: 10u64.into(),
        };

        #[block]
        {
            Pallet::<T>::do_refund(refund, 10u64.into());
        }

        assert_has_event::<T>(Event::<T>::RefundFailed(account).into());
    }

    #[benchmark]
    fn on_process_refund_queue() {
        // The base weight consumed on processing refund queue when empty.
        assert_eq!(RefundQueue::<T>::get().len() as u32, 0);

        #[block]
        {
            Pallet::<T>::process_refund_queue(Weight::MAX);
        }
    }

    #[benchmark]
    fn on_process_refund_queue_elements(i: Linear<1, MAX_QUEUED_REFUNDS>) {
        // The weight consumed on processing refund queue with one element.
        // Can deduce the process_refund_queue overhead by subtracting try_refund weight.
        let account: T::AccountId = account("Alice", 1, 1);
        let idty_id: IdtyId<T> = 1u32.into();
        IdtyQuota::<T>::insert(
            idty_id,
            Quota {
                last_use: BlockNumberFor::<T>::zero(),
                amount: 10u64.into(),
            },
        );
        let _ = CurrencyOf::<T>::set_balance(&T::RefundAccount::get(), u32::MAX.into());
        // The worst-case scenario is when the refund fails
        // and can only be triggered if the account is dead,
        // in this case by having no balance in the account.
        let refund = Refund {
            account: account.clone(),
            identity: 1u32.into(),
            amount: 10u64.into(),
        };
        for _ in 0..i {
            Pallet::<T>::queue_refund(refund.clone());
        }
        assert_eq!(RefundQueue::<T>::get().len() as u32, i);

        #[block]
        {
            Pallet::<T>::process_refund_queue(Weight::MAX);
        }

        assert_eq!(RefundQueue::<T>::get().len() as u32, 0);
        assert_has_event::<T>(Event::<T>::RefundFailed(account).into());
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::new_test_ext(crate::mock::QuotaConfig {
            identities: vec![1, 2]
        }),
        crate::mock::Test
    );
}
