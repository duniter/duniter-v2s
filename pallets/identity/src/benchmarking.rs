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

use codec::Encode;
use frame_benchmarking::{account, v2::*};
use frame_support::traits::OnInitialize;
use frame_system::{RawOrigin, pallet_prelude::BlockNumberFor};
use sp_core::{Get, crypto::Ss58Codec};
use sp_io::crypto::{sr25519_generate, sr25519_sign};
use sp_runtime::{AccountId32, MultiSigner};

use crate::Pallet;

#[benchmarks(
        where
            T::Signature: From<sp_core::sr25519::Signature>,
            T::AccountId: From<AccountId32>,
            T::IdtyIndex: From<u32>,
)]
mod benchmarks {
    use super::*;

    fn assert_has_event<T: Config>(generic_event: T::RuntimeEvent) {
        frame_system::Pallet::<T>::assert_has_event(generic_event);
    }

    struct Account<T: Config> {
        key: T::AccountId,
        index: T::IdtyIndex,
        origin: <T as frame_system::Config>::RuntimeOrigin,
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
        T::CheckAccountWorthiness::set_worthy(&owner_key);
        Pallet::<T>::create_identity(caller_origin.clone(), owner_key.clone())?;
        let name = IdtyName("new_identity".into());
        Pallet::<T>::confirm_identity(owner_key_origin.clone(), name.clone())?;
        let idty_index = IdentityIndexOf::<T>::get(&owner_key).unwrap();
        // make identity member
        <Identities<T>>::mutate_exists(idty_index, |idty_val_opt| {
            if let Some(idty_val) = idty_val_opt {
                idty_val.status = IdtyStatus::Member;
            }
        });
        // Reset next_creatable_identity_on to add more identities with Alice
        <Identities<T>>::mutate_exists(T::IdtyIndex::from(1u32), |idty_val_opt| {
            if let Some(idty_val) = idty_val_opt {
                idty_val.next_creatable_identity_on = BlockNumberFor::<T>::zero();
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
        let owner_key: T::AccountId = account("Bob", i, 1);
        let next_scheduled = BlockNumberFor::<T>::zero();
        let value = IdtyValue {
            data: Default::default(),
            next_creatable_identity_on: BlockNumberFor::<T>::zero(),
            old_owner_key: None,
            owner_key: owner_key.clone(),
            next_scheduled,
            status: IdtyStatus::Unvalidated,
        };
        let name = i.to_le_bytes();
        let idty_name = IdtyName(name.into());
        frame_system::Pallet::<T>::inc_sufficients(&owner_key);
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

    #[benchmark]
    fn create_identity() {
        let caller: T::AccountId = Identities::<T>::get(T::IdtyIndex::one()).unwrap().owner_key; // Alice
        let owner_key: T::AccountId = account("new_identity", 2, 1);
        T::CheckAccountWorthiness::set_worthy(&owner_key);

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), owner_key.clone());

        let idty_index = IdentityIndexOf::<T>::get(&owner_key);
        assert!(idty_index.is_some(), "Identity not added");
        assert_has_event::<T>(
            Event::<T>::IdtyCreated {
                idty_index: idty_index.unwrap(),
                owner_key,
            }
            .into(),
        );
    }

    #[benchmark]
    fn confirm_identity() -> Result<(), BenchmarkError> {
        let caller: T::AccountId = Identities::<T>::get(T::IdtyIndex::one()).unwrap().owner_key;
        let caller_origin: <T as frame_system::Config>::RuntimeOrigin =
            RawOrigin::Signed(caller.clone()).into();
        let owner_key: T::AccountId = account("new_identity", 2, 1);
        T::CheckAccountWorthiness::set_worthy(&owner_key);
        Pallet::<T>::create_identity(caller_origin.clone(), owner_key.clone())?;

        #[extrinsic_call]
        _(
            RawOrigin::Signed(owner_key.clone()),
            IdtyName("new_identity".into()),
        );

        let idty_index = IdentityIndexOf::<T>::get(&owner_key);
        assert_has_event::<T>(
            Event::<T>::IdtyConfirmed {
                idty_index: idty_index.unwrap(),
                name: IdtyName("new_identity".into()),
            }
            .into(),
        );
        Ok(())
    }

    #[benchmark]
    fn change_owner_key() -> Result<(), BenchmarkError> {
        let old_key: T::AccountId = account("new_identity", 2, 1);
        let account: Account<T> = create_one_identity(old_key.clone())?;

        // Change key a first time to add an old-old key
        let genesis_hash = frame_system::Pallet::<T>::block_hash(BlockNumberFor::<T>::zero());
        let new_key_payload = IdtyIndexAccountIdPayload {
            genesis_hash: &genesis_hash,
            idty_index: account.index,
            old_owner_key: &account.key,
        };
        let message = (NEW_OWNER_KEY_PAYLOAD_PREFIX, new_key_payload).encode();
        let caller_public = sr25519_generate(0.into(), None);
        let caller: T::AccountId = MultiSigner::Sr25519(caller_public).into_account().into();
        let signature = sr25519_sign(0.into(), &caller_public, &message)
            .unwrap()
            .into();
        Pallet::<T>::change_owner_key(account.origin.clone(), caller.clone(), signature)?;

        // Change key a second time to benchmark
        //  The sufficients for the old_old key will drop to 0 during benchmark
        let caller_origin = RawOrigin::Signed(caller.clone());
        let genesis_hash = frame_system::Pallet::<T>::block_hash(BlockNumberFor::<T>::zero());
        let new_key_payload = IdtyIndexAccountIdPayload {
            genesis_hash: &genesis_hash,
            idty_index: account.index,
            old_owner_key: &caller_public,
        };
        let message = (NEW_OWNER_KEY_PAYLOAD_PREFIX, new_key_payload).encode();
        let caller_public = sr25519_generate(0.into(), None);
        let caller: T::AccountId = MultiSigner::Sr25519(caller_public).into_account().into();
        let signature = sr25519_sign(0.into(), &caller_public, &message)
            .unwrap()
            .into();
        <frame_system::Pallet<T>>::set_block_number(
            <frame_system::Pallet<T>>::block_number() + T::ChangeOwnerKeyPeriod::get(),
        );

        #[extrinsic_call]
        _(caller_origin, caller.clone(), signature);

        assert_has_event::<T>(
            Event::<T>::IdtyChangedOwnerKey {
                idty_index: account.index,
                new_owner_key: caller.clone(),
            }
            .into(),
        );
        assert!(
            IdentityIndexOf::<T>::get(&caller).unwrap() == account.index,
            "Owner key not changed"
        );
        Ok(())
    }

    #[benchmark]
    fn revoke_identity() -> Result<(), BenchmarkError> {
        let old_key: T::AccountId = account("new_identity", 2, 1);
        let account: Account<T> = create_one_identity(old_key.clone())?;

        // Change key
        //  The sufficients for the old key will drop to 0 during benchmark (not for revoke, only for remove)
        let genesis_hash = frame_system::Pallet::<T>::block_hash(BlockNumberFor::<T>::zero());
        let new_key_payload = IdtyIndexAccountIdPayload {
            genesis_hash: &genesis_hash,
            idty_index: account.index,
            old_owner_key: &account.key,
        };
        let message = (NEW_OWNER_KEY_PAYLOAD_PREFIX, new_key_payload).encode();
        let caller_public = sr25519_generate(0.into(), None);
        let caller: T::AccountId = MultiSigner::Sr25519(caller_public).into_account().into();
        let signature = sr25519_sign(0.into(), &caller_public, &message)
            .unwrap()
            .into();
        Pallet::<T>::change_owner_key(account.origin.clone(), caller.clone(), signature)?;

        let genesis_hash = frame_system::Pallet::<T>::block_hash(BlockNumberFor::<T>::zero());
        let revocation_payload = RevocationPayload {
            genesis_hash: &genesis_hash,
            idty_index: account.index,
        };
        let message = (REVOCATION_PAYLOAD_PREFIX, revocation_payload).encode();
        let signature = sr25519_sign(0.into(), &caller_public, &message)
            .unwrap()
            .into();

        #[extrinsic_call]
        _(
            RawOrigin::Signed(account.key),
            account.index,
            caller.clone(),
            signature,
        );

        assert_has_event::<T>(
            Event::<T>::IdtyRevoked {
                idty_index: account.index,
                reason: RevocationReason::User,
            }
            .into(),
        );
        Ok(())
    }

    #[benchmark]
    fn prune_item_identities_names(i: Linear<2, 1_000>) -> Result<(), BenchmarkError> {
        // The complexity depends on the number of identities to prune
        // Populate identities
        let identities_count = Pallet::<T>::identities_count();
        create_identities::<T>(i)?;

        let mut names = Vec::<IdtyName>::new();
        // Keep names count aligned with benchmark component `i`.
        for k in 1..=i {
            let name: IdtyName = IdtyName((k + identities_count).to_le_bytes().into());
            assert!(
                IdentitiesNames::<T>::contains_key(&name),
                "Name not existing"
            );
            names.push(name);
        }

        #[extrinsic_call]
        _(RawOrigin::Root, names.clone());

        for name in names {
            assert!(!IdentitiesNames::<T>::contains_key(&name), "Name existing");
        }
        Ok(())
    }

    #[benchmark]
    fn fix_sufficients() -> Result<(), BenchmarkError> {
        let new_identity: T::AccountId = account("Bob", 2, 1);
        let account: Account<T> = create_one_identity(new_identity)?;
        let sufficient = frame_system::Pallet::<T>::sufficients(&account.key);

        #[extrinsic_call]
        _(RawOrigin::Root, account.key.clone(), true);

        assert!(
            sufficient < frame_system::Pallet::<T>::sufficients(&account.key),
            "Sufficient not incremented"
        );
        Ok(())
    }

    #[benchmark]
    fn link_account() -> Result<(), BenchmarkError> {
        let alice_origin =
            RawOrigin::Signed(Identities::<T>::get(T::IdtyIndex::one()).unwrap().owner_key);
        let bob_public = sr25519_generate(0.into(), None);
        let bob: T::AccountId = MultiSigner::Sr25519(bob_public).into_account().into();
        frame_system::Pallet::<T>::inc_providers(&bob);
        let genesis_hash = frame_system::Pallet::<T>::block_hash(BlockNumberFor::<T>::zero());
        let payload = (
            LINK_IDTY_PAYLOAD_PREFIX,
            genesis_hash,
            T::IdtyIndex::one(),
            bob.clone(),
        )
            .encode();
        let signature = sr25519_sign(0.into(), &bob_public, &payload)
            .unwrap()
            .into();

        #[extrinsic_call]
        _(alice_origin, bob, signature);

        Ok(())
    }

    #[benchmark]
    fn on_initialize() {
        // Base weight of an empty initialize
        #[block]
        {
            Pallet::<T>::on_initialize(BlockNumberFor::<T>::zero());
        }
    }

    #[benchmark]
    fn do_revoke_identity_noop() {
        let idty_index: T::IdtyIndex = 0u32.into();
        assert!(Identities::<T>::get(idty_index).is_none());

        #[block]
        {
            Pallet::<T>::do_revoke_identity(idty_index, RevocationReason::Root);
        }
    }

    #[benchmark]
    fn do_revoke_identity() {
        let idty_index: T::IdtyIndex = 1u32.into();
        let new_identity: T::AccountId = account("Bob", 2, 1);
        assert!(Identities::<T>::get(idty_index).is_some());
        Identities::<T>::mutate(idty_index, |id| {
            if let Some(id) = id {
                id.old_owner_key = Some((new_identity, BlockNumberFor::<T>::zero()));
            }
        });
        assert!(
            Identities::<T>::get(idty_index)
                .unwrap()
                .old_owner_key
                .is_some()
        );

        #[block]
        {
            Pallet::<T>::do_revoke_identity(idty_index, RevocationReason::Root);
        }

        assert_has_event::<T>(
            Event::<T>::IdtyRevoked {
                idty_index,
                reason: RevocationReason::Root,
            }
            .into(),
        );
    }

    #[benchmark]
    fn do_remove_identity_noop() {
        let idty_index: T::IdtyIndex = 0u32.into();
        assert!(Identities::<T>::get(idty_index).is_none());

        #[block]
        {
            Pallet::<T>::do_remove_identity(idty_index, RemovalReason::Revoked);
        }
    }

    #[benchmark]
    fn do_remove_identity() {
        let idty_index: T::IdtyIndex = 1u32.into();
        let new_identity: T::AccountId = account("Bob", 2, 1);
        assert!(Identities::<T>::get(idty_index).is_some());
        frame_system::Pallet::<T>::inc_sufficients(&new_identity);
        Identities::<T>::mutate(idty_index, |id| {
            if let Some(id) = id {
                id.old_owner_key = Some((new_identity, BlockNumberFor::<T>::zero()));
            }
        });
        assert!(
            Identities::<T>::get(idty_index)
                .unwrap()
                .old_owner_key
                .is_some()
        );

        #[block]
        {
            Pallet::<T>::do_remove_identity(idty_index, RemovalReason::Revoked);
        }

        assert_has_event::<T>(
            Event::<T>::IdtyRemoved {
                idty_index,
                reason: RemovalReason::Revoked,
            }
            .into(),
        );
    }

    #[benchmark]
    fn do_remove_identity_handler() {
        let idty_index: T::IdtyIndex = 1u32.into();
        let new_identity: T::AccountId = account("Bob", 2, 1);
        assert!(Identities::<T>::get(idty_index).is_some());
        frame_system::Pallet::<T>::inc_sufficients(&new_identity);
        Identities::<T>::mutate(idty_index, |id| {
            if let Some(id) = id {
                id.old_owner_key = Some((new_identity, BlockNumberFor::<T>::zero()));
            }
        });
        assert!(
            Identities::<T>::get(idty_index)
                .unwrap()
                .old_owner_key
                .is_some()
        );

        #[block]
        {
            T::OnRemoveIdty::on_removed(&idty_index);
        }
    }

    #[benchmark]
    fn membership_removed() -> Result<(), BenchmarkError> {
        let key: T::AccountId = account("new_identity", 2, 1);
        let account: Account<T> = create_one_identity(key)?;
        assert_eq!(
            Identities::<T>::get(account.index).unwrap().status,
            IdtyStatus::Member
        );

        #[block]
        {
            Pallet::<T>::membership_removed(account.index);
        }

        assert_eq!(
            Identities::<T>::get(account.index).unwrap().status,
            IdtyStatus::NotMember
        );
        Ok(())
    }

    #[benchmark]
    fn prune_identities_noop() {
        assert!(IdentityChangeSchedule::<T>::try_get(BlockNumberFor::<T>::zero()).is_err());

        #[block]
        {
            Pallet::<T>::prune_identities(BlockNumberFor::<T>::zero());
        }
    }

    #[benchmark]
    fn prune_identities_none() {
        let idty_index: T::IdtyIndex = 100u32.into();
        IdentityChangeSchedule::<T>::append(BlockNumberFor::<T>::zero(), idty_index);
        assert!(IdentityChangeSchedule::<T>::try_get(BlockNumberFor::<T>::zero()).is_ok());
        assert!(<Identities<T>>::try_get(idty_index).is_err());

        #[block]
        {
            Pallet::<T>::prune_identities(BlockNumberFor::<T>::zero());
        }
    }

    #[benchmark]
    fn prune_identities_err() -> Result<(), BenchmarkError> {
        let idty_index: T::IdtyIndex = 100u32.into();
        create_dummy_identity::<T>(100u32)?;
        IdentityChangeSchedule::<T>::append(BlockNumberFor::<T>::zero(), idty_index);

        #[block]
        {
            Pallet::<T>::prune_identities(BlockNumberFor::<T>::zero());
        }

        Ok(())
    }

    #[benchmark]
    fn revoke_identity_legacy() -> Result<(), BenchmarkError> {
        let caller_index = T::IdtyIndex::from(1u32);
        let caller: T::AccountId = Identities::<T>::get(caller_index).unwrap().owner_key;

        let idty_index: T::IdtyIndex = 102.into();
        let owner_key: T::AccountId =
            AccountId32::from_ss58check("5H2nLXGku46iztpqdRwsCAiP6vHZbShhKmSV4yyufQgEUFvV")
                .unwrap()
                .into();
        let next_scheduled = BlockNumberFor::<T>::zero();
        let value = IdtyValue {
            data: Default::default(),
            next_creatable_identity_on: BlockNumberFor::<T>::zero(),
            old_owner_key: None,
            owner_key: owner_key.clone(),
            next_scheduled,
            status: IdtyStatus::Member,
        };
        let name = "Charlie";
        let idty_name = IdtyName(name.into());
        frame_system::Pallet::<T>::inc_sufficients(&owner_key);
        <Identities<T>>::insert(idty_index, value);
        IdentityChangeSchedule::<T>::append(next_scheduled, idty_index);
        IdentityIndexOf::<T>::insert(owner_key.clone(), idty_index);
        <IdentitiesNames<T>>::insert(idty_name.clone(), idty_index);

        let document = r"Version: 10
Type: Revocation
Currency: g1
Issuer: Fnf2xaxYdQpB4kU45DMLQ9Ey4bd6DtoebKJajRkLBUXm
IdtyUniqueID: Charlie
IdtyTimestamp: 42-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
IdtySignature: 7KUagcMiQw05rwbkBsRrnNqPRHu/Y5ukCLoAEpb/1tXAQsSNf2gRi1h5PWIGs9y/vHnFXvF5epKsOjA6X75vDg==
CfiG4xhcWS+/DgxY0xFIyOA9TVr4Im3XEXcCApNgXC+Ns9jy2yrNoC3NF8MCD63cZ8QTRfrr4Iv6n3leYCCcDQ==
";

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), document.into());

        assert_has_event::<T>(
            Event::<T>::IdtyRevoked {
                idty_index,
                reason: RevocationReason::User,
            }
            .into(),
        );
        Ok(())
    }

    impl_benchmark_test_suite!(
        Pallet,
        // Create genesis identity Alice to test benchmark in mock
        crate::mock::new_test_ext(crate::mock::IdentityConfig {
            identities: vec![
                crate::GenesisIdty {
                    index: 1,
                    name: crate::IdtyName::from("Alice"),
                    value: crate::IdtyValue {
                        data: (),
                        next_creatable_identity_on: 0,
                        old_owner_key: None,
                        owner_key: frame_benchmarking::account("Alice", 1, 1),
                        next_scheduled: 0,
                        status: crate::IdtyStatus::Member,
                    },
                },
                crate::GenesisIdty {
                    index: 2,
                    name: crate::IdtyName::from("Bob"),
                    value: crate::IdtyValue {
                        data: (),
                        next_creatable_identity_on: 0,
                        old_owner_key: None,
                        owner_key: frame_benchmarking::account("Bob", 1, 1),
                        next_scheduled: 0,
                        status: crate::IdtyStatus::Unconfirmed,
                    },
                },
            ]
        }),
        crate::mock::Test,
    );
}
