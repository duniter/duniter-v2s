// Copyright 2021 Axiom-Team
//
// This file is part of Substrate-Libre-Currency.
//
// Substrate-Libre-Currency is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, version 3 of the License.
//
// Substrate-Libre-Currency is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with Substrate-Libre-Currency. If not, see <https://www.gnu.org/licenses/>.

use crate::{entities::IdtyData, AccountId, IdtyIndex};
use core::marker::PhantomData;
use pallet_universal_dividend::FirstEligibleUd;
use sp_std::boxed::Box;
use sp_std::vec::Vec;

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

#[allow(clippy::type_complexity)]
pub struct UdMembersStorageIter<T: pallet_identity::Config>(
    Box<dyn Iterator<Item = pallet_identity::IdtyValue<T::BlockNumber, T::AccountId, T::IdtyData>>>,
    PhantomData<T>,
);

impl<T: pallet_identity::Config> From<Option<Vec<u8>>> for UdMembersStorageIter<T> {
    fn from(maybe_key: Option<Vec<u8>>) -> Self {
        let mut iter = pallet_identity::Identities::<T>::iter_values();
        if let Some(key) = maybe_key {
            iter.set_last_raw_key(key);
        }
        Self(Box::new(iter), PhantomData)
    }
}

impl<T> Iterator for UdMembersStorageIter<T>
where
    T: pallet_identity::Config,
    T::IdtyData: Into<FirstEligibleUd>,
{
    type Item = (T::AccountId, FirstEligibleUd);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(pallet_identity::IdtyValue {
            owner_key, data, ..
        }) = self.0.next()
        {
            Some((owner_key, data.into()))
        } else {
            None
        }
    }
}
