// Copyright 2021 Axiom-Team
//
// This file is part of Substrate-Libre-Currency.
//
// Substrate-Libre-Currency is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License.
//
// Substrate-Libre-Currency is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with Substrate-Libre-Currency. If not, see <https://www.gnu.org/licenses/>.

use super::{
    AccountId, Identity, IdtyIndex, IdtyRight, Origin, Runtime, StrongCert, UdAccountsStorage,
    Weight,
};
use pallet_identity::traits::IdtyEvent;

const MIN_STRONG_CERT_FOR_UD: u32 = 2;
const MIN_STRONG_CERT_FOR_STRONG_CERT: u32 = 3;

pub struct OnIdtyChangeHandler;
impl pallet_identity::traits::OnIdtyChange<Runtime> for OnIdtyChangeHandler {
    fn on_idty_change(idty_index: IdtyIndex, idty_event: IdtyEvent<Runtime>) -> Weight {
        let total_weight = 0;
        match idty_event {
            IdtyEvent::Created { creator } => {
                // totad_weight += StrongCert::WeightInfo::add_cert();
                let _ = StrongCert::add_cert(Origin::root(), creator, idty_index);
            }
            IdtyEvent::Confirmed => {}
            IdtyEvent::Validated => {}
            IdtyEvent::Expired => {}
            IdtyEvent::Removed => {}
        };
        total_weight
    }
}

pub struct OnRightKeyChangeHandler;
impl pallet_identity::traits::OnRightKeyChange<Runtime> for OnRightKeyChangeHandler {
    fn on_right_key_change(
        _idty_index: IdtyIndex,
        right: IdtyRight,
        old_key_opt: Option<AccountId>,
        new_key_opt: Option<AccountId>,
    ) {
        match right {
            IdtyRight::Ud => UdAccountsStorage::replace_account(old_key_opt, new_key_opt),
            IdtyRight::CreateIdty => 0,
            IdtyRight::LightCert => 0,
            IdtyRight::StrongCert => 0,
        };
    }
}

pub struct OnNewStrongCertHandler;
impl pallet_certification::traits::OnNewcert<IdtyIndex> for OnNewStrongCertHandler {
    fn on_new_cert(
        _issuer: IdtyIndex,
        _issuer_issued_count: u8,
        receiver: IdtyIndex,
        receiver_received_count: u32,
    ) -> frame_support::dispatch::Weight {
        let total_weight = 0;
        match receiver_received_count {
            MIN_STRONG_CERT_FOR_UD => {
                // total_weight += Identity::WeightInfo::add_right();
                let _ = Identity::validate_identity_and_add_rights(
                    Origin::root(),
                    receiver,
                    sp_std::vec![IdtyRight::Ud],
                );
            }
            MIN_STRONG_CERT_FOR_STRONG_CERT => {
                // total_weight += Identity::WeightInfo::add_right();
                let _ = Identity::add_right(Origin::root(), receiver, IdtyRight::StrongCert);
            }
            _ => {}
        }
        total_weight
    }
}

pub struct OnRemovedStrongCertHandler;
impl pallet_certification::traits::OnRemovedCert<IdtyIndex> for OnRemovedStrongCertHandler {
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
            let _ = Identity::del_right(Origin::root(), receiver, IdtyRight::Ud);
        }
        total_weight
    }
}
