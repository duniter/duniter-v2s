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

use super::entities::*;
use super::{AccountId, IdtyIndex};
use frame_support::dispatch::UnfilteredDispatchable;
use frame_support::instances::{Instance1, Instance2};
use frame_support::pallet_prelude::Weight;
use frame_support::Parameter;

pub struct OnMembershipEventHandler<Inner, Runtime>(core::marker::PhantomData<(Inner, Runtime)>);

type MembershipMetaData = pallet_duniter_wot::MembershipMetaData<AccountId>;

impl<
        Inner: sp_membership::traits::OnEvent<IdtyIndex, MembershipMetaData>,
        Runtime: frame_system::Config<AccountId = AccountId>
            + pallet_identity::Config<IdtyIndex = IdtyIndex>
            + pallet_membership::Config<Instance1, MetaData = MembershipMetaData>
            + pallet_ud_accounts_storage::Config,
    > sp_membership::traits::OnEvent<IdtyIndex, MembershipMetaData>
    for OnMembershipEventHandler<Inner, Runtime>
{
    fn on_event(membership_event: &sp_membership::Event<IdtyIndex, MembershipMetaData>) -> Weight {
        (match membership_event {
            sp_membership::Event::MembershipAcquired(idty_index, owner_key) => {
                pallet_ud_accounts_storage::Pallet::<Runtime>::replace_account(
                    None,
                    Some(owner_key.0.clone()),
                    *idty_index,
                )
            }
            sp_membership::Event::MembershipRevoked(idty_index) => {
                pallet_ud_accounts_storage::Pallet::<Runtime>::remove_account(*idty_index)
            }
            _ => 0,
        }) + Inner::on_event(membership_event)
    }
}

pub struct OnSmithMembershipEventHandler<Inner, Runtime>(
    core::marker::PhantomData<(Inner, Runtime)>,
);

impl<
        IdtyIndex: Copy + Parameter,
        SessionKeysWrapper: Clone,
        Inner: sp_membership::traits::OnEvent<IdtyIndex, SmithsMembershipMetaData<SessionKeysWrapper>>,
        Runtime: frame_system::Config<AccountId = AccountId>
            + pallet_identity::Config<IdtyIndex = IdtyIndex>
            + pallet_authority_members::Config<KeysWrapper = SessionKeysWrapper, MemberId = IdtyIndex>
            + pallet_membership::Config<
                Instance2,
                MetaData = SmithsMembershipMetaData<SessionKeysWrapper>,
            >,
    > sp_membership::traits::OnEvent<IdtyIndex, SmithsMembershipMetaData<SessionKeysWrapper>>
    for OnSmithMembershipEventHandler<Inner, Runtime>
{
    fn on_event(
        membership_event: &sp_membership::Event<
            IdtyIndex,
            SmithsMembershipMetaData<SessionKeysWrapper>,
        >,
    ) -> Weight {
        (match membership_event {
            sp_membership::Event::MembershipAcquired(
                _idty_index,
                SmithsMembershipMetaData {
                    owner_key,
                    session_keys,
                    ..
                },
            ) => {
                let call = pallet_authority_members::Call::<Runtime>::set_session_keys {
                    keys: session_keys.clone(),
                };
                if let Err(e) = call.dispatch_bypass_filter(
                    frame_system::Origin::<Runtime>::Signed(owner_key.clone()).into(),
                ) {
                    sp_std::if_std! {
                        println!("{:?}", e)
                    }
                }
                0
            }
            sp_membership::Event::MembershipRevoked(idty_index) => {
                let call = pallet_authority_members::Call::<Runtime>::remove_member {
                    member_id: *idty_index,
                };
                if let Err(e) =
                    call.dispatch_bypass_filter(frame_system::Origin::<Runtime>::Root.into())
                {
                    sp_std::if_std! {
                        println!("{:?}", e)
                    }
                }
                0
            }
            _ => 0,
        }) + Inner::on_event(membership_event)
    }
}

pub struct OnRemovedAuthorityMemberHandler<Runtime>(core::marker::PhantomData<Runtime>);
impl<Runtime> pallet_authority_members::traits::OnRemovedMember<IdtyIndex>
    for OnRemovedAuthorityMemberHandler<Runtime>
where
    Runtime: frame_system::Config + pallet_membership::Config<Instance2, IdtyId = IdtyIndex>,
{
    fn on_removed_member(idty_index: IdtyIndex) -> Weight {
        if let Err(e) = pallet_membership::Pallet::<Runtime, Instance2>::revoke_membership(
            frame_system::RawOrigin::Root.into(),
            Some(idty_index),
        ) {
            sp_std::if_std! {
                println!("{:?}", e)
            }
        }
        0
    }
}
