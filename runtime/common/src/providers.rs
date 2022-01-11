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

use crate::entities::IdtyData;
use crate::{AccountId, BlockNumber, IdtyIndex};
use frame_support::traits::Get;
use sp_std::vec::Vec;

pub struct IdtyDataProvider<Runtime, const IDTY_CREATE_PERIOD: BlockNumber>(
    core::marker::PhantomData<Runtime>,
);
impl<Runtime, const IDTY_CREATE_PERIOD: BlockNumber>
    pallet_identity::traits::ProvideIdtyData<Runtime>
    for IdtyDataProvider<Runtime, IDTY_CREATE_PERIOD>
where
    Runtime: frame_system::Config<AccountId = AccountId, BlockNumber = BlockNumber>
        + pallet_identity::Config<IdtyData = IdtyData, IdtyIndex = IdtyIndex>,
{
    fn provide_identity_data(
        creator: IdtyIndex,
        _idty_name: &pallet_identity::IdtyName,
        _idty_owner_key: &AccountId,
    ) -> IdtyData {
        let block_number = frame_system::Pallet::<Runtime>::block_number();
        let creator_idty_data = IdtyData {
            can_create_on: block_number + IDTY_CREATE_PERIOD,
        };
        pallet_identity::Pallet::<Runtime>::set_idty_data(creator, creator_idty_data);
        Default::default()
    }
}

pub struct UdAccountsProvider<Runtime>(core::marker::PhantomData<Runtime>);
impl<Runtime: pallet_ud_accounts_storage::Config> Get<u64> for UdAccountsProvider<Runtime> {
    fn get() -> u64 {
        <pallet_ud_accounts_storage::Pallet<Runtime>>::ud_accounts_count()
    }
}
impl<Runtime: frame_system::Config<AccountId = AccountId> + pallet_ud_accounts_storage::Config>
    Get<Vec<AccountId>> for UdAccountsProvider<Runtime>
{
    fn get() -> Vec<AccountId> {
        <pallet_ud_accounts_storage::Pallet<Runtime>>::account_list()
    }
}
