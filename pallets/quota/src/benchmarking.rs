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
use frame_benchmarking::{account, benchmarks};

// FIXME this is a naïve implementation of benchmarks:
// - without properly prepare data
// - without "verify" blocks
// - without thinking about worst case scenario
// - without writing complexity in the term of refund queue length
// It's there as a seed for benchmark implementation and to use WeightInfo where needed.

benchmarks! {
    where_clause {
        where
            IdtyId<T>: From<u32>,
            BalanceOf<T>: From<u64>,
    }
    queue_refund {
        let account: T::AccountId = account("Alice", 1, 1);
        let refund = Refund {
            account,
            identity: 1u32.into(),
            amount: 10u64.into(),
        };
    }: { Pallet::<T>::queue_refund(refund) }
    spend_quota {
        let idty_id = 1u32;
        let amount = 1u64;
    }: { Pallet::<T>::spend_quota(idty_id.into(), amount.into()) }
    try_refund {
        let account: T::AccountId = account("Alice", 1, 1);
        let refund = Refund {
            account,
            identity: 1u32.into(),
            amount: 10u64.into(),
        };
    }: { Pallet::<T>::try_refund(refund) }
    do_refund {
        let account: T::AccountId = account("Alice", 1, 1);
        let refund = Refund {
            account,
            identity: 1u32.into(),
            amount: 10u64.into(),
        };
        let amount = 5u64.into();
    }: { Pallet::<T>::do_refund(refund, amount) }
}
