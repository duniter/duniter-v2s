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

// Shared tests run against each runtime's actual ProxyType and pallet configuration.

use crate::*;
use frame_support::{
    assert_noop, assert_ok,
    traits::{Currency, InstanceFilter},
};

const PROXY_TYPES: [ProxyType; 4] = [
    ProxyType::AlmostAny,
    ProxyType::TransferOnly,
    ProxyType::CancelProxy,
    ProxyType::TechnicalCommitteePropose,
];

fn externalities() -> sp_io::TestExternalities {
    let storage = frame_system::GenesisConfig::<Runtime>::default()
        .build_storage()
        .unwrap();
    sp_io::TestExternalities::new(storage)
}

fn proxy_call(pure: &AccountId, call: RuntimeCall) -> RuntimeCall {
    RuntimeCall::Proxy(pallet_proxy::Call::proxy {
        real: pure.clone().into(),
        force_proxy_type: Some(ProxyType::AlmostAny),
        call: Box::new(call),
    })
}

fn batch(calls: Vec<RuntimeCall>) -> RuntimeCall {
    RuntimeCall::Utility(pallet_utility::Call::batch_all { calls })
}

fn add(delegate: &AccountId, proxy_type: ProxyType) -> RuntimeCall {
    RuntimeCall::Proxy(pallet_proxy::Call::add_proxy {
        delegate: delegate.clone().into(),
        proxy_type,
        delay: 0,
    })
}

fn remove(delegate: &AccountId, proxy_type: ProxyType) -> RuntimeCall {
    RuntimeCall::Proxy(pallet_proxy::Call::remove_proxy {
        delegate: delegate.clone().into(),
        proxy_type,
        delay: 0,
    })
}

fn assert_proxy_result(result: Result<(), sp_runtime::DispatchError>) {
    System::assert_has_event(RuntimeEvent::Proxy(pallet_proxy::Event::ProxyExecuted {
        result,
    }));
}

#[test]
fn proxy_permission_relation() {
    let expected = [
        [true, true, true, true],
        [false, true, false, false],
        [false, false, true, false],
        [false, false, false, true],
    ];
    for (row, proxy_type) in PROXY_TYPES.iter().enumerate() {
        for (column, other) in PROXY_TYPES.iter().enumerate() {
            assert_eq!(proxy_type.is_superset(other), expected[row][column]);
        }
    }
}

#[test]
fn pure_proxy_can_change_multisig_and_return_creator_deposit() {
    externalities().execute_with(|| {
        System::set_block_number(1);
        System::set_extrinsic_index(0);
        let creator = AccountId::from([1; 32]);
        let bob = AccountId::from([2; 32]);
        let charlie = AccountId::from([3; 32]);
        let dave = AccountId::from([4; 32]);
        let old_group = Multisig::multi_account_id(&[bob.clone(), charlie.clone()], 1);
        let new_group = Multisig::multi_account_id(&[charlie.clone(), dave.clone()], 1);
        let _ = Balances::make_free_balance_be(&creator, 1_000_000);
        assert_ok!(Proxy::create_pure(
            RuntimeOrigin::signed(creator.clone()),
            ProxyType::AlmostAny,
            0,
            0
        ));
        let pure = Proxy::pure_account(&creator, &ProxyType::AlmostAny, 0, None);
        let deposit = Balances::reserved_balance(&creator);
        assert!(deposit > 0);
        let _ = Balances::make_free_balance_be(&pure, 10_000);

        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(creator.clone()),
            pure.clone().into(),
            Some(ProxyType::AlmostAny),
            Box::new(batch(vec![
                add(&old_group, ProxyType::AlmostAny),
                remove(&creator, ProxyType::AlmostAny),
            ])),
        ));
        assert_proxy_result(Ok(()));
        assert_eq!(Proxy::proxies(pure.clone()).0.len(), 1);
        assert_eq!(Proxy::proxies(pure.clone()).0[0].delegate, old_group);
        assert_eq!(Balances::reserved_balance(&pure), 0);
        assert_eq!(Balances::reserved_balance(&creator), deposit);

        System::reset_events();
        assert_ok!(Multisig::as_multi_threshold_1(
            RuntimeOrigin::signed(bob),
            vec![charlie.clone()],
            Box::new(proxy_call(
                &pure,
                batch(vec![
                    add(&new_group, ProxyType::AlmostAny),
                    remove(&old_group, ProxyType::AlmostAny),
                ])
            )),
        ));
        assert_proxy_result(Ok(()));
        assert_eq!(Proxy::proxies(pure.clone()).0.len(), 1);
        assert_eq!(Proxy::proxies(pure.clone()).0[0].delegate, new_group);
        assert_eq!(Balances::reserved_balance(&pure), 0);
        assert_noop!(
            Proxy::proxy(
                RuntimeOrigin::signed(old_group),
                pure.clone().into(),
                Some(ProxyType::AlmostAny),
                Box::new(RuntimeCall::System(frame_system::Call::remark {
                    remark: vec![]
                })),
            ),
            pallet_proxy::Error::<Runtime>::NotProxy
        );

        System::reset_events();
        assert_ok!(Multisig::as_multi_threshold_1(
            RuntimeOrigin::signed(charlie),
            vec![dave],
            Box::new(proxy_call(
                &pure,
                batch(vec![
                    RuntimeCall::Balances(pallet_balances::Call::transfer_all {
                        dest: creator.clone().into(),
                        keep_alive: false,
                    }),
                    RuntimeCall::Proxy(pallet_proxy::Call::kill_pure {
                        spawner: creator.clone().into(),
                        proxy_type: ProxyType::AlmostAny,
                        index: 0,
                        height: 1,
                        ext_index: 0,
                    }),
                ])
            )),
        ));
        assert_proxy_result(Ok(()));
        assert!(Proxy::proxies(pure.clone()).0.is_empty());
        assert_eq!(Balances::reserved_balance(&creator), 0);
        assert_eq!(Balances::free_balance(&creator), 1_010_000);
        assert_eq!(Balances::free_balance(&pure), 0);
    });
}

#[test]
fn restricted_proxies_cannot_manage_delegations() {
    for restricted in &PROXY_TYPES[1..] {
        externalities().execute_with(|| {
            System::set_block_number(1);
            let owner = AccountId::from([1; 32]);
            let delegate = AccountId::from([2; 32]);
            let target = AccountId::from([3; 32]);
            let _ = Balances::make_free_balance_be(&owner, 1_000_000);
            assert_ok!(Proxy::add_proxy(
                RuntimeOrigin::signed(owner.clone()),
                delegate.clone().into(),
                *restricted,
                0,
            ));
            for target_type in PROXY_TYPES {
                assert_ok!(Proxy::add_proxy(
                    RuntimeOrigin::signed(owner.clone()),
                    target.clone().into(),
                    target_type,
                    0,
                ));
                let before = Proxy::proxies(owner.clone());
                // Equality in is_superset must not bypass the type's call filter.
                for call in [
                    add(&target, target_type),
                    remove(&target, target_type),
                    batch(vec![add(&target, target_type)]),
                    batch(vec![remove(&target, target_type)]),
                    RuntimeCall::Proxy(pallet_proxy::Call::remove_proxies {}),
                    RuntimeCall::Proxy(pallet_proxy::Call::kill_pure {
                        spawner: owner.clone().into(),
                        proxy_type: *restricted,
                        index: 0,
                        height: 1,
                        ext_index: 0,
                    }),
                ] {
                    System::reset_events();
                    assert_ok!(Proxy::proxy(
                        RuntimeOrigin::signed(delegate.clone()),
                        owner.clone().into(),
                        Some(*restricted),
                        Box::new(call),
                    ));
                    assert_proxy_result(Err(frame_system::Error::<Runtime>::CallFiltered.into()));
                    assert_eq!(Proxy::proxies(owner.clone()), before);
                }
            }
        });
    }
}
