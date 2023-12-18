// Copyright 2021-2023 Axiom-Team
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

#![cfg(feature = "runtime-benchmarks")]

use super::*;

//use codec::Encode;
use codec::Encode;
use frame_benchmarking::{account, benchmarks};
use frame_support::traits::OnInitialize;
use frame_system::pallet_prelude::BlockNumberFor;
use frame_system::RawOrigin;
use sp_core::Get;
use sp_io::crypto::{sr25519_generate, sr25519_sign};
use sp_runtime::{AccountId32, MultiSigner};

use crate::Pallet;

const SEED: u32 = 1;

fn assert_has_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    frame_system::Pallet::<T>::assert_has_event(generic_event.into());
}

struct Account<T: Config> {
    key: T::AccountId,
    index: T::IdtyIndex,
    origin: <T as frame_system::Config>::RuntimeOrigin,
    // name: IdtyName,
}

// Create and confirm one account using Alice authorized account.
// key, origin, name and index are returned.
// Alice next_creatable_identity_on is reinitialized at the end so several account can be
// created in a row.
fn create_one_identity<T: Config>(owner_key: T::AccountId) -> Result<Account<T>, &'static str> {
    // get Alice account to create identity
    let caller: T::AccountId = Identities::<T>::get(T::IdtyIndex::from(1u32))
        .unwrap()
        .owner_key;
    let caller_origin: <T as frame_system::Config>::RuntimeOrigin =
        RawOrigin::Signed(caller.clone()).into();
    let owner_key_origin: <T as frame_system::Config>::RuntimeOrigin =
        RawOrigin::Signed(owner_key.clone()).into();
    Pallet::<T>::create_identity(caller_origin.clone(), owner_key.clone())?;
    let name = IdtyName("new_identity".into());
    Pallet::<T>::confirm_identity(owner_key_origin.clone(), name.clone())?;
    let idty_index = IdentityIndexOf::<T>::get(&owner_key).unwrap();
    // make identity member
    <Identities<T>>::mutate_exists(T::IdtyIndex::from(idty_index), |idty_val_opt| {
        if let Some(ref mut idty_val) = idty_val_opt {
            idty_val.status = IdtyStatus::Member;
        }
    });
    // Reset next_creatable_identity_on to add more identities with Alice
    <Identities<T>>::mutate_exists(T::IdtyIndex::from(1u32), |idty_val_opt| {
        if let Some(ref mut idty_val) = idty_val_opt {
            idty_val.next_creatable_identity_on = T::BlockNumber::zero();
        }
    });
    Ok(Account {
        key: owner_key,
        index: idty_index,
        origin: owner_key_origin,
        // name: name,
    })
}

// Create a dummy identity bypassing all the checks.
fn create_dummy_identity<T: Config>(i: u32) -> Result<(), &'static str> {
    let idty_index: T::IdtyIndex = i.into();
    let owner_key: T::AccountId = account("Bob", i, SEED);
    let next_scheduled = T::BlockNumber::zero();
    let value = IdtyValue {
        data: Default::default(),
        next_creatable_identity_on: T::BlockNumber::zero(),
        old_owner_key: None,
        owner_key: owner_key.clone(),
        next_scheduled,
        status: IdtyStatus::Unvalidated,
    };
    let name = i.to_le_bytes();
    let idty_name = IdtyName(name.into());
    <Identities<T>>::insert(idty_index, value);
    IdentityChangeSchedule::<T>::append(next_scheduled, idty_index);
    IdentityIndexOf::<T>::insert(owner_key.clone(), idty_index);
    <IdentitiesNames<T>>::insert(idty_name.clone(), idty_index);
    Ok(())
}

// Add `i` dummy identities.
fn create_identities<T: Config>(i: u32) -> Result<(), &'static str> {
    let identities_count = Pallet::<T>::identities_count();
    for j in 0..i {
        create_dummy_identity::<T>(j + identities_count + 1)?;
    }
    assert!(
        identities_count + i == Pallet::<T>::identities_count(),
        "Identities not created"
    );
    Ok(())
}

benchmarks! {
    where_clause {
        where
            T::Signature: From<sp_core::sr25519::Signature>,
            T::Signature: From<sp_core::sr25519::Signature>,
            T::AccountId: From<AccountId32>,
            T::IdtyIndex: From<u32>,
    }

    // create identity
    create_identity {
        let caller: T::AccountId  = Identities::<T>::get(T::IdtyIndex::one()).unwrap().owner_key; // Alice
        let caller_origin: <T as frame_system::Config>::RuntimeOrigin = RawOrigin::Signed(caller.clone()).into();
        let owner_key: T::AccountId = account("new_identity", 2, SEED);
    }: _<T::RuntimeOrigin>(caller_origin.clone(), owner_key.clone())
    verify {
        let idty_index = IdentityIndexOf::<T>::get(&owner_key);
        assert!(idty_index.is_some(), "Identity not added");
        assert_has_event::<T>(Event::<T>::IdtyCreated { idty_index: idty_index.unwrap(), owner_key: owner_key }.into());
    }

    // confirm identity
    confirm_identity {
        let caller: T::AccountId  = Identities::<T>::get(T::IdtyIndex::one()).unwrap().owner_key;
        let caller_origin: <T as frame_system::Config>::RuntimeOrigin = RawOrigin::Signed(caller.clone()).into();
        let owner_key: T::AccountId = account("new_identity", 2, SEED);
        let owner_key_origin: <T as frame_system::Config>::RuntimeOrigin = RawOrigin::Signed(owner_key.clone()).into();
        Pallet::<T>::create_identity(caller_origin.clone(), owner_key.clone())?;
    }: _<T::RuntimeOrigin>(owner_key_origin.clone(), IdtyName("new_identity".into()))
    verify {
        let idty_index = IdentityIndexOf::<T>::get(&owner_key);
        assert_has_event::<T>(Event::<T>::IdtyConfirmed { idty_index: idty_index.unwrap(), owner_key: owner_key, name: IdtyName("new_identity".into()) }.into());
    }

    // change owner key
    change_owner_key {
        let old_key: T::AccountId = account("new_identity", 2, SEED);
        let account: Account<T> = create_one_identity(old_key.clone())?;

        // Change key a first time to add an old-old key
        let genesis_hash = frame_system::Pallet::<T>::block_hash(T::BlockNumber::zero());
        let new_key_payload = IdtyIndexAccountIdPayload {
            genesis_hash: &genesis_hash,
            idty_index: account.index,
            old_owner_key: &account.key,
        };
        let message = (NEW_OWNER_KEY_PAYLOAD_PREFIX, new_key_payload).encode();
        let caller_public = sr25519_generate(0.into(), None);
        let caller: T::AccountId = MultiSigner::Sr25519(caller_public.clone()).into_account().into();
        let signature = sr25519_sign(0.into(), &caller_public, &message).unwrap().into();
        Pallet::<T>::change_owner_key(account.origin.clone(), caller.clone(), signature)?;

        // Change key a second time to benchmark
        //  The sufficients for the old_old key will drop to 0 during benchmark
        let caller_origin: <T as frame_system::Config>::RuntimeOrigin = RawOrigin::Signed(caller.clone()).into();
        let genesis_hash = frame_system::Pallet::<T>::block_hash(T::BlockNumber::zero());
        let new_key_payload = IdtyIndexAccountIdPayload {
            genesis_hash: &genesis_hash,
            idty_index: account.index,
            old_owner_key: &caller_public,
        };
        let message = (NEW_OWNER_KEY_PAYLOAD_PREFIX, new_key_payload).encode();
        let caller_public = sr25519_generate(0.into(), None);
        let caller: T::AccountId = MultiSigner::Sr25519(caller_public.clone()).into_account().into();
        let signature = sr25519_sign(0.into(), &caller_public, &message).unwrap().into();
        <frame_system::Pallet<T>>::set_block_number(<frame_system::Pallet<T>>::block_number() + T::ChangeOwnerKeyPeriod::get());
    }: _<T::RuntimeOrigin>(caller_origin.clone(), caller.clone(), signature)
    verify {
        assert_has_event::<T>(Event::<T>::IdtyChangedOwnerKey { idty_index: account.index, new_owner_key: caller.clone() }.into());
        assert!(IdentityIndexOf::<T>::get(&caller).unwrap() == account.index, "Owner key not changed");
    }

    // revoke identity
    revoke_identity {
        let old_key: T::AccountId = account("new_identity", 2, SEED);
        let account: Account<T> = create_one_identity(old_key.clone())?;

        // Change key
        //  The sufficients for the old key will drop to 0 during benchmark (not for revoke, only for remove)
        let genesis_hash = frame_system::Pallet::<T>::block_hash(T::BlockNumber::zero());
        let new_key_payload = IdtyIndexAccountIdPayload {
            genesis_hash: &genesis_hash,
            idty_index: account.index,
            old_owner_key: &account.key,
        };
        let message = (NEW_OWNER_KEY_PAYLOAD_PREFIX, new_key_payload).encode();
        let caller_public = sr25519_generate(0.into(), None);
        let caller: T::AccountId = MultiSigner::Sr25519(caller_public.clone()).into_account().into();
        let signature = sr25519_sign(0.into(), &caller_public, &message).unwrap().into();
        Pallet::<T>::change_owner_key(account.origin.clone(), caller.clone(), signature)?;

        let genesis_hash = frame_system::Pallet::<T>::block_hash(T::BlockNumber::zero());
        let revocation_payload = RevocationPayload {
            genesis_hash: &genesis_hash,
            idty_index: account.index.clone(),
        };
        let message = (REVOCATION_PAYLOAD_PREFIX, revocation_payload).encode();
        let signature = sr25519_sign(0.into(), &caller_public, &message).unwrap().into();
    }: _<T::RuntimeOrigin>(account.origin.clone(), account.index.clone().into(), caller.clone(), signature)
    verify {
        assert_has_event::<T>(Event::<T>::IdtyRevoked { idty_index: account.index, reason: RevocationReason::User }.into());
        // revocation does not mean deletion anymore
        // assert!(IdentityIndexOf::<T>::get(&account.key).is_none(), "Identity not revoked");
    }

    // The complexity depends on the number of identities to prune
    prune_item_identities_names {
        // Populate identities
        let identities_count = Pallet::<T>::identities_count();
        let i in 1 .. 1000 => create_identities::<T>(i)?;

        let mut names = Vec::<IdtyName>::new();
        for k in 1..i {
            let name: IdtyName = IdtyName((k + identities_count).to_le_bytes().into());
            assert!(IdentitiesNames::<T>::contains_key(&name), "Name not existing");
            names.push(name);
        }
    }: _<T::RuntimeOrigin>(RawOrigin::Root.into(), names.clone())
    verify {
        for name in names {
            assert!(!IdentitiesNames::<T>::contains_key(&name), "Name existing");
        }
    }

    // fix sufficients identity
    fix_sufficients {
        let new_identity: T::AccountId = account("Bob", 2, SEED);
        let account: Account<T> = create_one_identity(new_identity)?;
        let sufficient = frame_system::Pallet::<T>::sufficients(&account.key);
    }: _<T::RuntimeOrigin>(RawOrigin::Root.into(), account.key.clone(), true)
    verify {
        assert!(sufficient < frame_system::Pallet::<T>::sufficients(&account.key), "Sufficient not incremented");
    }

    // link account
    link_account {
        let alice_origin = RawOrigin::Signed(Identities::<T>::get(T::IdtyIndex::one()).unwrap().owner_key);
        let bob_public = sr25519_generate(0.into(), None);
        let bob: T::AccountId = MultiSigner::Sr25519(bob_public).into_account().into();
        frame_system::Pallet::<T>::inc_providers(&bob);
        let genesis_hash = frame_system::Pallet::<T>::block_hash(T::BlockNumber::zero());
        let payload = (
            LINK_IDTY_PAYLOAD_PREFIX, genesis_hash, T::IdtyIndex::one(), bob.clone(),
        ).encode();
        let signature = sr25519_sign(0.into(), &bob_public, &payload).unwrap().into();
    }: _<T::RuntimeOrigin>(alice_origin.into(), bob, signature)

    // Base weight of an empty initialize
    on_initialize {
    }: {Pallet::<T>::on_initialize(BlockNumberFor::<T>::zero());}

    // --- do revoke identity
    do_revoke_identity_noop {
        let idty_index: T::IdtyIndex = 0u32.into();
        assert!(Identities::<T>::get(idty_index).is_none());
    }: {Pallet::<T>::do_revoke_identity(idty_index, RevocationReason::Root);}
    do_revoke_identity {
        let idty_index: T::IdtyIndex = 1u32.into();
        let new_identity: T::AccountId = account("Bob", 2, SEED);
        assert!(Identities::<T>::get(idty_index).is_some());
        Identities::<T>::mutate( idty_index, |id| {
            if let Some(id) = id {
                id.old_owner_key = Some((new_identity, BlockNumberFor::<T>::zero()));
            }
        });
        assert!(Identities::<T>::get(idty_index).unwrap().old_owner_key.is_some());
    }: {Pallet::<T>::do_revoke_identity(idty_index, RevocationReason::Root);}
    verify {
        assert_has_event::<T>(Event::<T>::IdtyRevoked { idty_index, reason: RevocationReason::Root }.into());
    }

    // --- do remove identity
    do_remove_identity_noop {
        let idty_index: T::IdtyIndex = 0u32.into();
        assert!(Identities::<T>::get(idty_index).is_none());
    }: {Pallet::<T>::do_remove_identity(idty_index, RemovalReason::Revoked);}
    do_remove_identity {
        let idty_index: T::IdtyIndex = 1u32.into();
        let new_identity: T::AccountId = account("Bob", 2, SEED);
        assert!(Identities::<T>::get(idty_index).is_some());
        Identities::<T>::mutate( idty_index, |id| {
            if let Some(id) = id {
                id.old_owner_key = Some((new_identity, BlockNumberFor::<T>::zero()));
            }
        });
        assert!(Identities::<T>::get(idty_index).unwrap().old_owner_key.is_some());
    }: {Pallet::<T>::do_remove_identity(idty_index, RemovalReason::Revoked);}
    verify {
        assert_has_event::<T>(Event::<T>::IdtyRemoved { idty_index, reason: RemovalReason::Revoked }.into());
    }

    // --- prune identities
    prune_identities_noop {
        assert!(IdentityChangeSchedule::<T>::try_get(T::BlockNumber::zero()).is_err());
    }: {Pallet::<T>::prune_identities(T::BlockNumber::zero());}

    prune_identities_none {
        let idty_index: T::IdtyIndex = 100u32.into();
        IdentityChangeSchedule::<T>::append(T::BlockNumber::zero(), idty_index);
        assert!(IdentityChangeSchedule::<T>::try_get(T::BlockNumber::zero()).is_ok());
        assert!(<Identities<T>>::try_get(idty_index).is_err());
    }: {Pallet::<T>::prune_identities(T::BlockNumber::zero());}

    prune_identities_err {
        let idty_index: T::IdtyIndex = 100u32.into();
        create_dummy_identity::<T>(100u32)?;
        IdentityChangeSchedule::<T>::append(T::BlockNumber::zero(), idty_index);
    }: {Pallet::<T>::prune_identities(T::BlockNumber::zero());}

    impl_benchmark_test_suite!(
        Pallet,
        // Create genesis identity Alice to test benchmark in mock
        crate::mock::new_test_ext(crate::mock::IdentityConfig{ identities: vec![
            GenesisIdty {
                index: 1,
                name: IdtyName::from("Alice"),
                value: IdtyValue {
                    data: (),
                    next_creatable_identity_on: 0,
                    old_owner_key: None,
                    owner_key: account("Alice", 1, SEED),
                    next_scheduled: 0,
                    status: crate::IdtyStatus::Member,
                },
            },
            GenesisIdty {
                index: 2,
                name: IdtyName::from("Bob"),
                value: IdtyValue {
                    data: (),
                    next_creatable_identity_on: 0,
                    old_owner_key: None,
                    owner_key: account("Bob", 1, SEED),
                    next_scheduled: 0,
                    status: crate::IdtyStatus::Unconfirmed,
                },
            },
        ]}),
        crate::mock::Test,
    );
}
