// Copyright 2021 Axiom-Team
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

use crate::{entities::IdtyData, AccountId, IdtyIndex};
use core::marker::PhantomData;
use pallet_universal_dividend::FirstEligibleUd;

pub struct IdentityAccountIdProvider<Runtime>(PhantomData<Runtime>);

impl<
        Runtime: frame_system::Config<AccountId = AccountId>
            + pallet_identity::Config<IdtyIndex = IdtyIndex>,
    > sp_runtime::traits::Convert<IdtyIndex, Option<AccountId>>
    for IdentityAccountIdProvider<Runtime>
{
    fn convert(idty_index: IdtyIndex) -> Option<AccountId> {
        pallet_identity::Pallet::<Runtime>::identity(idty_index).map(|idty| idty.owner_key)
    }
}

pub struct IdentityIndexOf<T: pallet_identity::Config>(PhantomData<T>);

impl<T: pallet_identity::Config> sp_runtime::traits::Convert<T::AccountId, Option<T::IdtyIndex>>
    for IdentityIndexOf<T>
{
    fn convert(account_id: T::AccountId) -> Option<T::IdtyIndex> {
        pallet_identity::Pallet::<T>::identity_index_of(account_id)
    }
}

pub struct UdMembersStorage<T: pallet_identity::Config>(PhantomData<T>);

impl<T> frame_support::traits::StoredMap<AccountId, FirstEligibleUd> for UdMembersStorage<T>
where
    T: frame_system::Config<AccountId = AccountId>,
    T: pallet_identity::Config<IdtyData = IdtyData>,
{
    fn get(key: &T::AccountId) -> FirstEligibleUd {
        pallet_identity::Pallet::<T>::get(key).first_eligible_ud
    }
    fn try_mutate_exists<R, E: From<sp_runtime::DispatchError>>(
        key: &T::AccountId,
        f: impl FnOnce(&mut Option<FirstEligibleUd>) -> Result<R, E>,
    ) -> Result<R, E> {
        pallet_identity::Pallet::<T>::try_mutate_exists(key, |maybe_idty_data| {
            if let Some(ref mut idty_data) = maybe_idty_data {
                let mut maybe_first_eligible_ud = Some(idty_data.first_eligible_ud);
                let result = f(&mut maybe_first_eligible_ud)?;
                if let Some(first_eligible_ud) = maybe_first_eligible_ud {
                    idty_data.first_eligible_ud = first_eligible_ud;
                }
                Ok(result)
            } else {
                f(&mut None)
            }
        })
    }
}

pub struct IsWoTMemberProvider<T>(PhantomData<T>);
impl<T: pallet_smith_members::Config>
    sp_runtime::traits::IsMember<<T as pallet_membership::Config>::IdtyId>
    for IsWoTMemberProvider<T>
where
    T: pallet_distance::Config + pallet_membership::Config,
{
    fn is_member(idty_id: &T::IdtyId) -> bool {
        pallet_membership::Pallet::<T>::is_member(idty_id)
    }
}

#[cfg(feature = "runtime-benchmarks")]
pub struct BenchmarkSetupHandler<T>(PhantomData<T>);

// Macro implementing the BenchmarkSetupHandler trait for pallets requiring identity preparation for benchmarks.
#[cfg(feature = "runtime-benchmarks")]
macro_rules! impl_benchmark_setup_handler {
    ($t:ty) => {
        impl<T> $t for BenchmarkSetupHandler<T>
        where
            T: pallet_distance::Config,
            T: pallet_certification::Config,
            <T as pallet_certification::Config>::IdtyIndex: From<u32>,
        {
            fn force_valid_distance_status(idty_id: &IdtyIndex) -> () {
                let _ = pallet_distance::Pallet::<T>::do_valid_distance_status(*idty_id);
            }
            fn add_cert(issuer: &IdtyIndex, receiver: &IdtyIndex) {
                let _ = pallet_certification::Pallet::<T>::do_add_cert_checked(
                    (*issuer).into(),
                    (*receiver).into(),
                    false,
                );
            }
        }
    };
}

#[cfg(feature = "runtime-benchmarks")]
impl_benchmark_setup_handler!(pallet_membership::SetupBenchmark<<T as pallet_identity::Config>::IdtyIndex, T::AccountId>);
