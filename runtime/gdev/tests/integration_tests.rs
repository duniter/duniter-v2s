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

mod common;

use common::*;
use frame_support::traits::{PalletInfo, StorageInfo, StorageInfoTrait};
//use frame_support::{assert_err, assert_ok};
use frame_support::assert_ok;
use frame_support::{StorageHasher, Twox128};
use gdev_runtime::*;
use sp_core::crypto::Ss58Codec;

#[test]
fn verify_pallet_prefixes() {
    let prefix = |pallet_name, storage_name| {
        let mut res = [0u8; 32];
        res[0..16].copy_from_slice(&Twox128::hash(pallet_name));
        res[16..32].copy_from_slice(&Twox128::hash(storage_name));
        res.to_vec()
    };
    assert_eq!(
        <Timestamp as StorageInfoTrait>::storage_info(),
        vec![
            StorageInfo {
                pallet_name: b"Timestamp".to_vec(),
                storage_name: b"Now".to_vec(),
                prefix: prefix(b"Timestamp", b"Now"),
                max_values: Some(1),
                max_size: Some(8),
            },
            StorageInfo {
                pallet_name: b"Timestamp".to_vec(),
                storage_name: b"DidUpdate".to_vec(),
                prefix: prefix(b"Timestamp", b"DidUpdate"),
                max_values: Some(1),
                max_size: Some(1),
            }
        ]
    );
}

#[test]
fn verify_pallet_indices() {
    fn is_pallet_index<P: 'static>(index: usize) {
        assert_eq!(
            <Runtime as frame_system::Config>::PalletInfo::index::<P>(),
            Some(index)
        );
    }
    is_pallet_index::<System>(0);
}

#[test]
fn verify_proxy_type_indices() {
    assert_eq!(ProxyType::Any as u8, 0);
}

#[test]
fn test_genesis_build() {
    ExtBuilder::new(1, 3, 4).build().execute_with(|| {
        run_to_block(2);
    });
}

#[test]
fn test_remove_identity() {
    ExtBuilder::new(1, 3, 4).build().execute_with(|| {
        run_to_block(2);

        assert_ok!(Identity::remove_identity(
            frame_system::RawOrigin::Root.into(),
            4,
            None
        ));
        let events = System::events();
        assert_eq!(events.len(), 3);
        assert_eq!(
            System::events()[0].event,
            Event::Membership(pallet_membership::Event::MembershipRevoked(4))
        );
        /*println!(
            "{}",
            get_account_id_from_seed::<sp_core::sr25519::Public>("Charlie")
        );*/
        assert_eq!(
            System::events()[1].event,
            Event::System(frame_system::Event::KilledAccount {
                account: AccountId::from_ss58check(DAVE).unwrap()
            })
        );
        assert_eq!(
            System::events()[2].event,
            Event::Identity(pallet_identity::Event::IdtyRemoved { idty_index: 4 })
        );
        //println!("{:#?}", events);
    });
}

#[test]
fn test_remove_smith_identity() {
    ExtBuilder::new(1, 3, 4).build().execute_with(|| {
        run_to_block(2);

        assert_ok!(Identity::remove_identity(
            frame_system::RawOrigin::Root.into(),
            3,
            None
        ));
        let events = System::events();
        assert_eq!(events.len(), 4);
        assert_eq!(
            System::events()[0].event,
            Event::SmithsMembership(pallet_membership::Event::MembershipRevoked(3))
        );
        assert_eq!(
            System::events()[1].event,
            Event::AuthorityMembers(pallet_authority_members::Event::MemberRemoved(3))
        );
        assert_eq!(
            System::events()[2].event,
            Event::Membership(pallet_membership::Event::MembershipRevoked(3))
        );
        assert_eq!(
            System::events()[3].event,
            Event::Identity(pallet_identity::Event::IdtyRemoved { idty_index: 3 })
        );
        //println!("{:#?}", events);
    });
}
