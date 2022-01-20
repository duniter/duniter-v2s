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

use frame_support::pallet_prelude::Weight;
use frame_support::Parameter;

pub struct OnMembershipEventHandler<Inner, Runtime>(core::marker::PhantomData<(Inner, Runtime)>);

impl<
        IdtyIndex: Parameter,
        Inner: sp_membership::traits::OnEvent<IdtyIndex, ()>,
        Runtime: pallet_identity::Config<IdtyIndex = IdtyIndex> + pallet_ud_accounts_storage::Config,
    > sp_membership::traits::OnEvent<IdtyIndex, ()> for OnMembershipEventHandler<Inner, Runtime>
{
    fn on_event(membership_event: &sp_membership::Event<IdtyIndex>) -> Weight {
        (match membership_event {
            sp_membership::Event::<IdtyIndex>::MembershipAcquired(idty_index) => {
                if let Some(idty_value) = pallet_identity::Pallet::<Runtime>::identity(idty_index) {
                    <pallet_ud_accounts_storage::Pallet<Runtime>>::replace_account(
                        None,
                        Some(idty_value.owner_key),
                    )
                } else {
                    0
                }
            }
            sp_membership::Event::<IdtyIndex>::MembershipRevoked(idty_index) => {
                if let Some(idty_value) = pallet_identity::Pallet::<Runtime>::identity(idty_index) {
                    <pallet_ud_accounts_storage::Pallet<Runtime>>::replace_account(
                        Some(idty_value.owner_key),
                        None,
                    )
                } else {
                    0
                }
            }
            _ => 0,
        }) + Inner::on_event(membership_event)
    }
}
