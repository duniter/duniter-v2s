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

use super::{AccountId, IdtyIndex, entities::*};
use frame_support::{
    pallet_prelude::Weight,
    traits::{Imbalance, UnfilteredDispatchable},
};
use frame_system::pallet_prelude::BlockNumberFor;
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
impl<Runtime: pallet_duniter_wot::Config> pallet_identity::traits::OnNewIdty<Runtime>
    for OnNewIdtyHandler<Runtime>
{
    fn on_created(idty_index: &IdtyIndex, creator: &IdtyIndex) {
        pallet_duniter_wot::Pallet::<Runtime>::on_created(idty_index, creator);
    }
}

/// Runtime handler for OnRemoveIdty, calling all implementations of
/// OnRemoveIdty and implementing logic at the runtime level.
pub struct OnRemoveIdtyHandler<Runtime>(core::marker::PhantomData<Runtime>);
impl<Runtime: pallet_duniter_wot::Config + pallet_duniter_account::Config>
    pallet_identity::traits::OnRemoveIdty<Runtime> for OnRemoveIdtyHandler<Runtime>
{
    fn on_removed(idty_index: &IdtyIndex) -> Weight {
        pallet_duniter_wot::Pallet::<Runtime>::on_removed(idty_index)
    }

    fn on_revoked(idty_index: &IdtyIndex) -> Weight {
        pallet_duniter_wot::Pallet::<Runtime>::on_revoked(idty_index).saturating_add(
            pallet_duniter_account::Pallet::<Runtime>::on_revoked(idty_index),
        )
    }
}

/// Runtime handler for OnNewMembership, calling all implementations of
/// OnNewMembership and implementing logic at the runtime level.
pub struct OnNewMembershipHandler<Runtime>(core::marker::PhantomData<Runtime>);
impl<
    Runtime: frame_system::Config<AccountId = AccountId>
        + pallet_identity::Config<IdtyData = IdtyData, IdtyIndex = IdtyIndex>
        + pallet_duniter_wot::Config
        + pallet_universal_dividend::Config
        + pallet_quota::Config,
> sp_membership::traits::OnNewMembership<IdtyIndex> for OnNewMembershipHandler<Runtime>
{
    fn on_created(idty_index: &IdtyIndex) {
        // duniter-wot related actions
        pallet_duniter_wot::Pallet::<Runtime>::on_created(idty_index);

        pallet_quota::Pallet::<Runtime>::on_created(idty_index);

        // When main membership is acquired, it starts getting right to UD.
        pallet_identity::Identities::<Runtime>::mutate_exists(idty_index, |idty_val_opt| {
            if let Some(idty_val) = idty_val_opt {
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
///
/// OnRemoveMembership and implementing logic at the runtime level.
/// As the weight accounting is not trivial in this handler, the weight is
/// done at the handler level.
pub struct OnRemoveMembershipHandler<Runtime>(core::marker::PhantomData<Runtime>);
impl<
    Runtime: frame_system::Config
        + pallet_identity::Config<IdtyData = IdtyData, IdtyIndex = IdtyIndex>
        + pallet_smith_members::Config<IdtyIndex = IdtyIndex>
        + pallet_duniter_wot::Config
        + pallet_quota::Config
        + pallet_universal_dividend::Config,
> sp_membership::traits::OnRemoveMembership<IdtyIndex> for OnRemoveMembershipHandler<Runtime>
{
    fn on_removed(idty_index: &IdtyIndex) -> Weight {
        // duniter-wot related actions
        let mut weight = pallet_duniter_wot::Pallet::<Runtime>::on_removed(idty_index);

        // When membership is removed:
        // - call on_removed_member handler which auto claims UD;
        // - set the first_eligible_ud to None so the identity cannot claim UD anymore.
        pallet_identity::Identities::<Runtime>::mutate(idty_index, |maybe_idty_value| {
            if let Some(idty_value) = maybe_idty_value
                && let Some(first_ud_index) = idty_value.data.first_eligible_ud.0.take()
            {
                weight += pallet_universal_dividend::Pallet::<Runtime>::on_removed_member(
                    first_ud_index.into(),
                    &idty_value.owner_key,
                );
            }
        });
        weight.saturating_add(pallet_quota::Pallet::<Runtime>::on_removed(idty_index));
        weight.saturating_add(Runtime::DbWeight::get().reads_writes(1, 1));

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
            println!("faid to remove member: {e:?}")
        }
    }
}

/// Runtime handler OwnerKeyChangePermission.
pub struct KeyChangeHandler<Runtime, ReportLongevity>(
    core::marker::PhantomData<(Runtime, ReportLongevity)>,
);
impl<
    Runtime: frame_system::Config<AccountId = AccountId>
        + pallet_identity::Config<IdtyIndex = IdtyIndex>
        + pallet_authority_members::Config<MemberId = IdtyIndex>
        + pallet_smith_members::Config<IdtyIndex = IdtyIndex>,
    ReportLongevity: Get<BlockNumberFor<Runtime>>,
> pallet_identity::traits::KeyChange<Runtime> for KeyChangeHandler<Runtime, ReportLongevity>
{
    /// Handles the event when an identity's owner key is changed.
    ///
    /// # Errors
    /// * Returns `OwnerKeyInBound` if the smith was a validator and the bond period is not finished, meaning it can still be punished for past actions.
    /// * Returns `OwnerKeyUsedAsValidator` if the owner key is currently used as a validator.
    ///
    /// # Behavior
    /// * If the smith is online, the operation is rejected.
    /// * If the smith was a validator and is still within the bond period, the operation is rejected. It means they can still be punished for past actions.
    /// * If the smith is neither online nor within the bond period, the owner key is changed successfully and the change is reflected in the validator member data if available.
    fn on_changed(
        idty_index: IdtyIndex,
        account_id: AccountId,
    ) -> Result<(), sp_runtime::DispatchError> {
        if let Some(smith) = pallet_smith_members::Pallet::<Runtime>::smiths(&idty_index) {
            // last_online is None for both online validators and smiths who have never been online
            if let Some(last_online) = smith.last_online {
                if last_online + ReportLongevity::get()
                    > frame_system::pallet::Pallet::<Runtime>::block_number()
                {
                    return Err(pallet_identity::Error::<Runtime>::OwnerKeyInBound.into());
                }
            } else if pallet_authority_members::Pallet::<Runtime>::online().contains(&idty_index) {
                return Err(pallet_identity::Error::<Runtime>::OwnerKeyUsedAsValidator.into());
            }
            // New or future smiths who have not yet set keys are not authority members.
            // In that case, only the identity owner key changes and authority data is untouched.
            if pallet_authority_members::Members::<Runtime>::contains_key(idty_index) {
                pallet_authority_members::Pallet::<Runtime>::change_owner_key(
                    idty_index, account_id,
                )
                .map_err(|e| e.error)?;
            }
        }
        Ok(())
    }
}

/// Runtime handler for managing fee handling by transferring unbalanced amounts to a treasury account.
pub struct HandleFees<TreasuryAccount, Balances>(
    frame_support::pallet_prelude::PhantomData<(TreasuryAccount, Balances)>,
);
type CreditOf<Balances> = frame_support::traits::tokens::fungible::Credit<AccountId, Balances>;
impl<TreasuryAccount, Balances> frame_support::traits::OnUnbalanced<CreditOf<Balances>>
    for HandleFees<TreasuryAccount, Balances>
where
    TreasuryAccount: Get<AccountId>,
    Balances: frame_support::traits::fungible::Balanced<AccountId>,
{
    fn on_nonzero_unbalanced(amount: CreditOf<Balances>) {
        // fee is moved to treasury
        let _ = Balances::deposit(
            &TreasuryAccount::get(),
            amount.peek(),
            frame_support::traits::tokens::Precision::Exact,
        );
    }
}
