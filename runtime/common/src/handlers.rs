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

use super::{entities::*, AccountId, IdtyIndex};
use frame_support::{pallet_prelude::Weight, traits::UnfilteredDispatchable};
use pallet_smith_members::SmithRemovalReason;
use sp_core::Get;

/// OnNewSession handler for the runtime calling all the implementation
/// of OnNewSession
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

/// Runtime handler for OnNewIdty, calling all implementations of
/// OnNewIdty and implementing logic at the runtime level.
pub struct OnNewIdtyHandler<Runtime>(core::marker::PhantomData<Runtime>);
impl<Runtime: pallet_duniter_wot::Config + pallet_quota::Config>
    pallet_identity::traits::OnNewIdty<Runtime> for OnNewIdtyHandler<Runtime>
{
    fn on_created(idty_index: &IdtyIndex, creator: &IdtyIndex) {
        pallet_duniter_wot::Pallet::<Runtime>::on_created(idty_index, creator);
        pallet_quota::Pallet::<Runtime>::on_created(idty_index, creator);
    }
}

/// Runtime handler for OnRemoveIdty, calling all implementations of
/// OnRemoveIdty and implementing logic at the runtime level.
pub struct OnRemoveIdtyHandler<Runtime>(core::marker::PhantomData<Runtime>);
impl<Runtime: pallet_duniter_wot::Config + pallet_quota::Config>
    pallet_identity::traits::OnRemoveIdty<Runtime> for OnRemoveIdtyHandler<Runtime>
{
    fn on_removed(idty_index: &IdtyIndex) -> Weight {
        let mut weight = pallet_duniter_wot::Pallet::<Runtime>::on_removed(idty_index);
        weight += pallet_quota::Pallet::<Runtime>::on_removed(idty_index);
        weight
    }

    fn on_revoked(idty_index: &IdtyIndex) -> Weight {
        let mut weight = pallet_duniter_wot::Pallet::<Runtime>::on_revoked(idty_index);
        weight += pallet_quota::Pallet::<Runtime>::on_revoked(idty_index);
        weight
    }
}

/// Runtime handler for OnNewMembership, calling all implementations of
/// OnNewMembership and implementing logic at the runtime level.
pub struct OnNewMembershipHandler<Runtime>(core::marker::PhantomData<Runtime>);
impl<
        Runtime: frame_system::Config<AccountId = AccountId>
            + pallet_identity::Config<IdtyData = IdtyData, IdtyIndex = IdtyIndex>
            + pallet_duniter_wot::Config
            + pallet_universal_dividend::Config,
    > sp_membership::traits::OnNewMembership<IdtyIndex> for OnNewMembershipHandler<Runtime>
{
    fn on_created(idty_index: &IdtyIndex) {
        // duniter-wot related actions
        pallet_duniter_wot::Pallet::<Runtime>::on_created(idty_index);

        // When main membership is acquired, it starts getting right to UD.
        pallet_identity::Identities::<Runtime>::mutate_exists(idty_index, |idty_val_opt| {
            if let Some(ref mut idty_val) = idty_val_opt {
                idty_val.data = IdtyData {
                    first_eligible_ud:
                        pallet_universal_dividend::Pallet::<Runtime>::init_first_eligible_ud(),
                };
            }
        });
    }

    fn on_renewed(_idty_index: &IdtyIndex) {}
}

/// Runtime handler for OnRemoveMembership, calling all implementations of
/// OnRemoveMembership and implementing logic at the runtime level.
/// As the weight accounting is not trivial in this handler, the weight is
/// done at the handler level.
pub struct OnRemoveMembershipHandler<Runtime>(core::marker::PhantomData<Runtime>);
impl<
        Runtime: frame_system::Config<AccountId = AccountId>
            + pallet_identity::Config<IdtyData = IdtyData, IdtyIndex = IdtyIndex>
            + pallet_smith_members::Config<IdtyIndex = IdtyIndex>
            + pallet_duniter_wot::Config
            + pallet_universal_dividend::Config,
    > sp_membership::traits::OnRemoveMembership<IdtyIndex> for OnRemoveMembershipHandler<Runtime>
{
    fn on_removed(idty_index: &IdtyIndex) -> Weight {
        // duniter-wot related actions
        let mut weight = pallet_duniter_wot::Pallet::<Runtime>::on_removed(idty_index);

        let mut add_db_reads_writes = |reads, writes| {
            weight += Runtime::DbWeight::get().reads_writes(reads, writes);
        };

        // When membership is removed, call on_removed_member handler which auto claims UD.
        if let Some(idty_value) = pallet_identity::Identities::<Runtime>::get(idty_index) {
            add_db_reads_writes(1, 0);
            if let Some(first_ud_index) = idty_value.data.first_eligible_ud.into() {
                add_db_reads_writes(1, 0);
                weight += pallet_universal_dividend::Pallet::<Runtime>::on_removed_member(
                    first_ud_index,
                    &idty_value.owner_key,
                );
            }
        }

        // When membership is removed, also remove from smith member.
        weight.saturating_add(
            pallet_smith_members::Pallet::<Runtime>::on_removed_wot_member(*idty_index),
        )
    }
}

/// Runtime handler for TreasurySpendFunds.
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

/// Runtime handler for OnSmithDelete.
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
            #[cfg(feature = "std")]
            println!("faid to remove member: {:?}", e)
        }
    }
}

/// Runtime handler OwnerKeyChangePermission.
pub struct OwnerKeyChangePermissionHandler<Runtime>(core::marker::PhantomData<Runtime>);
impl<
        Runtime: frame_system::Config
            + pallet_identity::Config<IdtyIndex = IdtyIndex>
            + pallet_authority_members::Config<MemberId = IdtyIndex>,
    > pallet_identity::traits::CheckKeyChangeAllowed<Runtime>
    for OwnerKeyChangePermissionHandler<Runtime>
{
    fn check_allowed(idty_index: &IdtyIndex) -> bool {
        !pallet_authority_members::Pallet::<Runtime>::online().contains(idty_index)
    }
}
