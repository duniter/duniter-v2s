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
use frame_support::traits::Get;
use frame_support::Parameter;
use sp_runtime::traits::IsMember;

pub struct OnNewSessionHandler<Runtime>(core::marker::PhantomData<Runtime>);
impl<Runtime> pallet_authority_members::traits::OnNewSession for OnNewSessionHandler<Runtime>
where
    Runtime: pallet_provide_randomness::Config,
{
    fn on_new_session(_index: sp_staking::SessionIndex) -> Weight {
        pallet_provide_randomness::Pallet::<Runtime>::on_new_epoch();
        0
    }
}

pub struct OnMembershipEventHandler<Inner, Runtime>(core::marker::PhantomData<(Inner, Runtime)>);

type MembershipMetaData = pallet_duniter_wot::MembershipMetaData<AccountId>;

impl<
        Inner: sp_membership::traits::OnEvent<IdtyIndex, MembershipMetaData>,
        Runtime: frame_system::Config<AccountId = AccountId>
            + pallet_identity::Config<IdtyData = IdtyData, IdtyIndex = IdtyIndex>
            + pallet_membership::Config<Instance1, MetaData = MembershipMetaData>
            + pallet_universal_dividend::Config,
    > sp_membership::traits::OnEvent<IdtyIndex, MembershipMetaData>
    for OnMembershipEventHandler<Inner, Runtime>
{
    fn on_event(membership_event: &sp_membership::Event<IdtyIndex, MembershipMetaData>) -> Weight {
        (match membership_event {
            sp_membership::Event::MembershipAcquired(idty_index, _owner_key) => {
                pallet_identity::Identities::<Runtime>::mutate_exists(idty_index, |idty_val_opt| {
                    if let Some(ref mut idty_val) = idty_val_opt {
                        idty_val.data = IdtyData {
                            first_eligible_ud:
                                pallet_universal_dividend::Pallet::<Runtime>::init_first_eligible_ud(
                                ),
                        }
                    }
                });
                Runtime::DbWeight::get().reads_writes(1, 1)
            }
            sp_membership::Event::MembershipRevoked(idty_index) => {
                if let Some(idty_value) = pallet_identity::Identities::<Runtime>::get(idty_index) {
                    if let Some(first_ud_index) = idty_value.data.first_eligible_ud.into() {
                        pallet_universal_dividend::Pallet::<Runtime>::on_removed_member(
                            first_ud_index,
                            &idty_value.owner_key,
                        )
                    } else {
                        Runtime::DbWeight::get().reads(1)
                    }
                } else {
                    Runtime::DbWeight::get().reads(1)
                }
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
                        println!("fail to set session keys: {:?}", e)
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
                        println!("faid to remove member: {:?}", e)
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
                println!("fail to revoke membership: {:?}", e)
            }
        }
        0
    }
}

pub struct RemoveIdentityConsumersImpl<Runtime>(core::marker::PhantomData<Runtime>);
impl<Runtime> pallet_identity::traits::RemoveIdentityConsumers<IdtyIndex>
    for RemoveIdentityConsumersImpl<Runtime>
where
    Runtime: pallet_identity::Config<IdtyIndex = IdtyIndex>
        + pallet_authority_members::Config<MemberId = IdtyIndex>
        + pallet_membership::Config<Instance1, IdtyId = IdtyIndex>
        + pallet_membership::Config<Instance2, IdtyId = IdtyIndex>,
{
    fn remove_idty_consumers(idty_index: IdtyIndex) -> Weight {
        // Remove smith member
        if pallet_membership::Pallet::<Runtime, Instance2>::is_member(&idty_index) {
            if let Err(e) = pallet_membership::Pallet::<Runtime, Instance2>::revoke_membership(
                frame_system::RawOrigin::Root.into(),
                Some(idty_index),
            ) {
                log::error!(
                    target: "runtime::common",
                    "Logic error: fail to revoke smith membership in remove_idty_consumers(): {:?}",
                    e
                );
            }
        }
        // Remove "classic" member
        if let Err(e) = pallet_membership::Pallet::<Runtime, Instance1>::revoke_membership(
            frame_system::RawOrigin::Root.into(),
            Some(idty_index),
        ) {
            log::error!(
                target: "runtime::common",
                "Logic error: fail to revoke membership in remove_idty_consumers(): {:?}",
                e
            );
        }

        0
    }
}

pub struct TreasurySpendFunds<Runtime>(core::marker::PhantomData<Runtime>);
impl<Runtime> pallet_treasury::SpendFunds<Runtime> for TreasurySpendFunds<Runtime>
where
    Runtime: pallet_treasury::Config,
{
    fn spend_funds(
        _budget_remaining: &mut pallet_treasury::BalanceOf<Runtime>,
        _imbalance: &mut pallet_treasury::PositiveImbalanceOf<Runtime>,
        _total_weight: &mut Weight,
        missed_any: &mut bool,
    ) {
        *missed_any = true;
    }
}
