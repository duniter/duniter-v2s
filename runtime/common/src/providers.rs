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

use crate::{AccountId, IdtyIndex};
use core::marker::PhantomData;
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

#[allow(clippy::type_complexity)]
pub struct IdtyDataIter<T: pallet_identity::Config>(
    Box<dyn Iterator<Item = pallet_identity::IdtyValue<T::BlockNumber, T::AccountId, T::IdtyData>>>,
    PhantomData<T>,
);

impl<T: pallet_identity::Config> From<Option<Vec<u8>>> for IdtyDataIter<T> {
    fn from(maybe_key: Option<Vec<u8>>) -> Self {
        let mut iter = pallet_identity::Identities::<T>::iter_values();
        if let Some(key) = maybe_key {
            iter.set_last_raw_key(key);
        }
        Self(Box::new(iter), PhantomData)
    }
}

impl<T: pallet_identity::Config> Iterator for IdtyDataIter<T> {
    type Item = (T::AccountId, T::IdtyData);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(pallet_identity::IdtyValue {
            owner_key, data, ..
        }) = self.0.next()
        {
            Some((owner_key, data))
        } else {
            None
        }
    }
}
