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

use super::entities::*;
use super::{AccountId, IdtyIndex};
use frame_support::pallet_prelude::Weight;
use frame_support::traits::UnfilteredDispatchable;
use pallet_smith_members::SmithRemovalReason;

// new session handler
pub struct OnNewSessionHandler<Runtime>(core::marker::PhantomData<Runtime>);
impl<Runtime> pallet_authority_members::traits::OnNewSession for OnNewSessionHandler<Runtime>
where
    Runtime: pallet_provide_randomness::Config + pallet_smith_members::Config,
{
    fn on_new_session(index: sp_staking::SessionIndex) {
        pallet_provide_randomness::Pallet::<Runtime>::on_new_epoch();
        pallet_smith_members::Pallet::<Runtime>::on_new_session(index);
    }
}

// membership event runtime handler
pub struct OnMembershipEventHandler<Inner, Runtime>(core::marker::PhantomData<(Inner, Runtime)>);
impl<
        Inner: sp_membership::traits::OnEvent<IdtyIndex>,
        Runtime: frame_system::Config<AccountId = AccountId>
            + pallet_identity::Config<IdtyData = IdtyData, IdtyIndex = IdtyIndex>
            + pallet_membership::Config
            + pallet_smith_members::Config<IdtyIndex = IdtyIndex>
            + pallet_universal_dividend::Config,
    > sp_membership::traits::OnEvent<IdtyIndex> for OnMembershipEventHandler<Inner, Runtime>
{
    fn on_event(membership_event: &sp_membership::Event<IdtyIndex>) {
        (match membership_event {
            // when membership is removed, call on_removed_member handler which auto claims UD
            sp_membership::Event::MembershipRemoved(idty_index) => {
                if let Some(idty_value) = pallet_identity::Identities::<Runtime>::get(idty_index) {
                    if let Some(first_ud_index) = idty_value.data.first_eligible_ud.into() {
                        pallet_universal_dividend::Pallet::<Runtime>::on_removed_member(
                            first_ud_index,
                            &idty_value.owner_key,
                        );
                    }
                }
                pallet_smith_members::Pallet::<Runtime>::on_removed_wot_member(*idty_index);
            }
            // when main membership is acquired, it starts getting right to UD
            sp_membership::Event::MembershipAdded(idty_index) => {
                pallet_identity::Identities::<Runtime>::mutate_exists(idty_index, |idty_val_opt| {
                    if let Some(ref mut idty_val) = idty_val_opt {
                        idty_val.data = IdtyData {
                            first_eligible_ud:
                                pallet_universal_dividend::Pallet::<Runtime>::init_first_eligible_ud(
                                ),
                        };
                    }
                });
            }
            // in other case, ther is nothing to do
            sp_membership::Event::MembershipRenewed(_) => (),
        });
        Inner::on_event(membership_event)
    }
}

// spend treasury handler
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

pub struct OnSmithDeletedHandler<Runtime>(core::marker::PhantomData<Runtime>);
impl<Runtime> pallet_smith_members::traits::OnSmithDelete<Runtime::MemberId>
    for OnSmithDeletedHandler<Runtime>
where
    Runtime: pallet_authority_members::Config,
{
    fn on_smith_delete(idty_index: Runtime::MemberId, _reason: SmithRemovalReason) {
        let call = pallet_authority_members::Call::<Runtime>::remove_member {
            member_id: idty_index,
        };
        if let Err(e) = call.dispatch_bypass_filter(frame_system::Origin::<Runtime>::Root.into()) {
            sp_std::if_std! {
                println!("faid to remove member: {:?}", e)
            }
        }
    }
}
