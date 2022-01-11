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

use crate::entities::IdtyRight;
use frame_support::instances::Instance1;
use frame_support::weights::Weight;
use pallet_identity::traits::IdtyEvent;

pub struct OnIdtyChangeHandler<Runtime>(core::marker::PhantomData<Runtime>);
impl<
        IdtyIndex,
        Runtime: pallet_identity::Config<IdtyIndex = IdtyIndex>
            + pallet_certification::Config<Instance1, IdtyIndex = IdtyIndex>,
    > pallet_identity::traits::OnIdtyChange<Runtime> for OnIdtyChangeHandler<Runtime>
{
    fn on_idty_change(idty_index: IdtyIndex, idty_event: IdtyEvent<Runtime>) -> Weight {
        let total_weight = 0;
        match idty_event {
            IdtyEvent::Created { creator } => {
                // totad_weight += StrongCert::WeightInfo::add_cert();
                let _ = <pallet_certification::Pallet<Runtime, Instance1>>::add_cert(
                    frame_system::Origin::<Runtime>::Root.into(),
                    creator,
                    idty_index,
                );
            }
            IdtyEvent::Confirmed => {}
            IdtyEvent::Validated => {}
            IdtyEvent::Expired => {}
            IdtyEvent::Removed => {}
        };
        total_weight
    }
}

pub struct OnRightKeyChangeHandler<Runtime>(core::marker::PhantomData<Runtime>);
impl<
        IdtyIndex,
        Runtime: pallet_identity::Config<IdtyIndex = IdtyIndex, IdtyRight = IdtyRight>
            + pallet_ud_accounts_storage::Config,
    > pallet_identity::traits::OnRightKeyChange<Runtime> for OnRightKeyChangeHandler<Runtime>
{
    fn on_right_key_change(
        _idty_index: IdtyIndex,
        right: Runtime::IdtyRight,
        old_key_opt: Option<Runtime::AccountId>,
        new_key_opt: Option<Runtime::AccountId>,
    ) {
        match right {
            IdtyRight::Ud => <pallet_ud_accounts_storage::Pallet<Runtime>>::replace_account(
                old_key_opt,
                new_key_opt,
            ),
            IdtyRight::CreateIdty => 0,
            IdtyRight::LightCert => 0,
            IdtyRight::StrongCert => 0,
        };
    }
}

pub struct OnNewStrongCertHandler<
    Runtime,
    const MIN_STRONG_CERT_FOR_UD: u32,
    const MIN_STRONG_CERT_FOR_STRONG_CERT: u32,
>(core::marker::PhantomData<Runtime>);
impl<
        IdtyIndex,
        Runtime: pallet_identity::Config<IdtyIndex = IdtyIndex, IdtyRight = IdtyRight>,
        const MIN_STRONG_CERT_FOR_UD: u32,
        const MIN_STRONG_CERT_FOR_STRONG_CERT: u32,
    > pallet_certification::traits::OnNewcert<IdtyIndex>
    for OnNewStrongCertHandler<Runtime, MIN_STRONG_CERT_FOR_UD, MIN_STRONG_CERT_FOR_STRONG_CERT>
{
    fn on_new_cert(
        _issuer: IdtyIndex,
        _issuer_issued_count: u8,
        receiver: IdtyIndex,
        receiver_received_count: u32,
    ) -> frame_support::dispatch::Weight {
        let total_weight = 0;
        if receiver_received_count == MIN_STRONG_CERT_FOR_UD {
            // total_weight += Identity::WeightInfo::add_right();
            let _ = <pallet_identity::Pallet<Runtime>>::validate_identity_and_add_rights(
                frame_system::Origin::<Runtime>::Root.into(),
                receiver,
                sp_std::vec![IdtyRight::Ud],
            );
        } else if receiver_received_count == MIN_STRONG_CERT_FOR_STRONG_CERT {
            // total_weight += Identity::WeightInfo::add_right();
            let _ = <pallet_identity::Pallet<Runtime>>::add_right(
                frame_system::Origin::<Runtime>::Root.into(),
                receiver,
                IdtyRight::StrongCert,
            );
        }
        total_weight
    }
}

pub struct OnRemovedStrongCertHandler<Runtime, const MIN_STRONG_CERT_FOR_UD: u32>(
    core::marker::PhantomData<Runtime>,
);
impl<
        IdtyIndex,
        Runtime: pallet_identity::Config<IdtyIndex = IdtyIndex, IdtyRight = IdtyRight>,
        const MIN_STRONG_CERT_FOR_UD: u32,
    > pallet_certification::traits::OnRemovedCert<IdtyIndex>
    for OnRemovedStrongCertHandler<Runtime, MIN_STRONG_CERT_FOR_UD>
{
    fn on_removed_cert(
        _issuer: IdtyIndex,
        _issuer_issued_count: u8,
        receiver: IdtyIndex,
        receiver_received_count: u32,
        _expiration: bool,
    ) -> frame_support::dispatch::Weight {
        let total_weight = 0;
        if receiver_received_count < MIN_STRONG_CERT_FOR_UD {
            // total_weight += Identity::WeightInfo::del_right();
            let _ = <pallet_identity::Pallet<Runtime>>::del_right(
                frame_system::Origin::<Runtime>::Root.into(),
                receiver,
                IdtyRight::Ud,
            );
        }
        total_weight
    }
}
