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

#![allow(dead_code, unused_imports)]

use codec::Encode;
use common_runtime::{constants::*, *};
use frame_support::traits::{OnFinalize, OnIdle, OnInitialize};
use gtest_runtime::{opaque::SessionKeys, *};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{Pair, Public, sr25519};
use sp_keyring::sr25519::Keyring;
use sp_membership::MembershipData;
use sp_runtime::{
    generic::SignedPayload,
    traits::{ExtrinsicLike, IdentifyAccount, Verify},
};
use std::collections::BTreeMap;

pub type AccountPublic = <Signature as Verify>::Signer;

pub type AuthorityKeys = (
    AccountId,
    BabeId,
    GrandpaId,
    ImOnlineId,
    AuthorityDiscoveryId,
);

pub const BLOCK_TIME: u64 = 6_000;
pub const NAMES: [&str; 6] = ["Alice", "Bob", "Charlie", "Dave", "Eve", "Ferdie"];

pub struct ExtBuilder {
    initial_accounts: BTreeMap<AccountId, GenesisAccountData<Balance, u32>>,
    initial_authorities_len: usize,
    initial_identities: BTreeMap<IdtyName, AccountId>,
    initial_smiths: Vec<AuthorityKeys>,
}

impl ExtBuilder {
    pub fn new(
        initial_authorities_len: usize,
        initial_smiths_len: usize,
        initial_identities_len: usize,
    ) -> Self {
        assert!(initial_identities_len <= 6);
        assert!(initial_smiths_len <= initial_identities_len);
        assert!(initial_authorities_len <= initial_smiths_len);

        let initial_accounts = (0..initial_identities_len)
            .map(|i| {
                (
                    get_account_id_from_seed::<sr25519::Public>(NAMES[i]),
                    GenesisAccountData {
                        balance: 0,
                        idty_id: Some(i as u32 + 1),
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();
        let initial_identities = (0..initial_identities_len)
            .map(|i| {
                (
                    IdtyName::from(NAMES[i]),
                    get_account_id_from_seed::<sr25519::Public>(NAMES[i]),
                )
            })
            .collect::<BTreeMap<IdtyName, AccountId>>();
        let initial_smiths = (0..initial_smiths_len)
            .map(|i| get_authority_keys_from_seed(NAMES[i]))
            .collect::<Vec<AuthorityKeys>>();

        Self {
            initial_accounts,
            initial_authorities_len,
            initial_identities,
            initial_smiths,
        }
    }

    pub fn with_initial_balances(mut self, initial_balances: Vec<(AccountId, Balance)>) -> Self {
        for (account_id, balance) in initial_balances {
            self.initial_accounts
                .entry(account_id.clone())
                .or_insert(GenesisAccountData::default())
                .balance = balance;
        }
        self
    }

    pub fn build(self) -> sp_io::TestExternalities {
        let Self {
            initial_accounts,
            initial_authorities_len,
            initial_identities,
            initial_smiths,
        } = self;

        let mut t = frame_system::GenesisConfig::<Runtime>::default()
            .build_storage()
            .unwrap();

        let monetary_mass = initial_accounts
            .values()
            .map(|balance| balance.balance)
            .sum();

        pallet_authority_members::GenesisConfig::<Runtime> {
            initial_authorities: initial_smiths
                .iter()
                .enumerate()
                .map(|(i, keys)| (i as u32 + 1, (keys.0.clone(), i < initial_authorities_len)))
                .collect(),
        }
        .assimilate_storage(&mut t)
        .unwrap();

        pallet_duniter_account::GenesisConfig::<Runtime> {
            accounts: initial_accounts.clone(),
            treasury_balance: <Runtime as pallet_balances::Config>::ExistentialDeposit::get(),
        }
        .assimilate_storage(&mut t)
        .unwrap();

        // Necessary to initialize TotalIssuance.
        pallet_balances::GenesisConfig::<Runtime> {
            total_issuance: monetary_mass,
        }
        .assimilate_storage(&mut t)
        .unwrap();

        pallet_session::GenesisConfig::<Runtime> {
            keys: initial_smiths
                .iter()
                .map(|x| {
                    (
                        x.0.clone(),
                        x.0.clone(),
                        session_keys(x.1.clone(), x.2.clone(), x.3.clone(), x.4.clone()),
                    )
                })
                .collect::<Vec<_>>(),
            non_authority_keys: Vec::new(),
        }
        .assimilate_storage(&mut t)
        .unwrap();

        pallet_identity::GenesisConfig::<Runtime> {
            identities: initial_identities
                .iter()
                .enumerate()
                .map(|(i, (name, owner_key))| GenesisIdty {
                    index: i as u32 + 1,
                    name: name.clone(),
                    value: IdtyValue {
                        data: IdtyData {
                            first_eligible_ud: pallet_universal_dividend::FirstEligibleUd::min(),
                        },
                        next_creatable_identity_on: Default::default(),
                        owner_key: owner_key.clone(),
                        old_owner_key: None,
                        next_scheduled: 0,
                        status: IdtyStatus::Member,
                    },
                })
                .collect(),
        }
        .assimilate_storage(&mut t)
        .unwrap();

        pallet_quota::GenesisConfig::<Runtime> {
            identities: initial_identities
                .iter()
                .enumerate()
                .map(|(i, _)| i as u32 + 1)
                .collect(),
        }
        .assimilate_storage(&mut t)
        .unwrap();

        pallet_membership::GenesisConfig::<Runtime> {
            memberships: (1..=initial_identities.len())
                .map(|i| {
                    (
                        i as u32,
                        MembershipData {
                            expire_on: MembershipPeriod::get(),
                        },
                    )
                })
                .collect(),
        }
        .assimilate_storage(&mut t)
        .unwrap();

        pallet_certification::GenesisConfig::<Runtime> {
            certs_by_receiver: clique_wot(initial_identities.len(), ValidityPeriod::get()),
            apply_cert_period_at_genesis: false,
        }
        .assimilate_storage(&mut t)
        .unwrap();

        pallet_smith_members::GenesisConfig::<Runtime> {
            initial_smiths: clique_smith_wot(initial_smiths.len()),
        }
        .assimilate_storage(&mut t)
        .unwrap();

        pallet_universal_dividend::GenesisConfig::<Runtime> {
            first_reeval: Some(600_000),
            first_ud: Some(24_000),
            initial_monetary_mass: monetary_mass,
            ud: 1_000,
        }
        .assimilate_storage(&mut t)
        .unwrap();

        let mut ext = sp_io::TestExternalities::new(t);
        ext.execute_with(|| {
            System::set_block_number(1);
        });
        ext
    }
}

pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn get_authority_keys_from_seed(s: &str) -> AuthorityKeys {
    (
        get_account_id_from_seed::<sr25519::Public>(s),
        get_from_seed::<BabeId>(s),
        get_from_seed::<GrandpaId>(s),
        get_from_seed::<ImOnlineId>(s),
        get_from_seed::<AuthorityDiscoveryId>(s),
    )
}

fn mock_babe_initialize(n: u32) {
    let slot: sp_consensus_babe::Slot = (n as u64).into();
    pallet_babe::CurrentSlot::<Runtime>::put(slot);
    let session = Session::current_index() as u64;
    pallet_babe::EpochIndex::<Runtime>::put(session);
}

pub fn run_to_block(n: u32) {
    while System::block_number() < n {
        Babe::on_finalize(System::block_number());
        Distance::on_finalize(System::block_number());
        TransactionPayment::on_finalize(System::block_number());
        Authorship::on_finalize(System::block_number());
        Grandpa::on_finalize(System::block_number());

        System::reset_events();
        System::set_block_number(System::block_number() + 1);
        System::set_block_consumed_resources(Weight::zero(), 0_usize);

        pallet_babe::CurrentSlot::<Runtime>::put(pallet_babe::CurrentSlot::<Runtime>::get() + 1);

        Account::on_initialize(System::block_number());
        Scheduler::on_initialize(System::block_number());
        Session::on_initialize(System::block_number());
        mock_babe_initialize(System::block_number());
        Authorship::on_initialize(System::block_number());

        UniversalDividend::on_initialize(System::block_number());
        Wot::on_initialize(System::block_number());
        Identity::on_initialize(System::block_number());
        Membership::on_initialize(System::block_number());
        Certification::on_initialize(System::block_number());

        Timestamp::set_timestamp(System::block_number() as u64 * BLOCK_TIME);
        Distance::on_initialize(System::block_number());

        // Process queued quota refunds at the end of each simulated block.
        Quota::on_idle(System::block_number(), Weight::from(1_000_000_000));
    }
}

/// Build an unchecked extrinsic for runtime integration tests.
pub fn get_unchecked_extrinsic(
    call: RuntimeCall,
    signer: Keyring,
    tip: Balance,
    nonce: u32,
) -> gtest_runtime::UncheckedExtrinsic {
    let tx_ext: gtest_runtime::TxExtension = (
        frame_system::CheckNonZeroSender::<gtest_runtime::Runtime>::new(),
        frame_system::CheckSpecVersion::<gtest_runtime::Runtime>::new(),
        frame_system::CheckTxVersion::<gtest_runtime::Runtime>::new(),
        frame_system::CheckGenesis::<gtest_runtime::Runtime>::new(),
        frame_system::CheckEra::<gtest_runtime::Runtime>::from(sp_runtime::generic::Era::Immortal),
        pallet_oneshot_account::CheckNonce::<gtest_runtime::Runtime>::from(
            frame_system::CheckNonce::<gtest_runtime::Runtime>::from(nonce),
        ),
        frame_system::CheckWeight::<gtest_runtime::Runtime>::new(),
        pallet_transaction_payment::ChargeTransactionPayment::<gtest_runtime::Runtime>::from(tip),
        frame_metadata_hash_extension::CheckMetadataHash::<gtest_runtime::Runtime>::new(false),
    );
    let payload = SignedPayload::new(call.clone(), tx_ext.clone()).unwrap();
    let sig = payload.using_encoded(|payload| signer.pair().sign(payload));

    gtest_runtime::UncheckedExtrinsic::new_signed(
        call,
        signer.to_account_id().into(),
        sig.into(),
        tx_ext,
    )
}

fn clique_wot(
    initial_identities_len: usize,
    cert_validity_period: common_runtime::BlockNumber,
) -> BTreeMap<IdtyIndex, BTreeMap<IdtyIndex, Option<common_runtime::BlockNumber>>> {
    let mut certs_by_issuer = BTreeMap::new();
    for i in 1..=initial_identities_len {
        certs_by_issuer.insert(
            i as IdtyIndex,
            (1..=initial_identities_len)
                .filter_map(|j| {
                    if i != j {
                        Some((j as IdtyIndex, Some(cert_validity_period)))
                    } else {
                        None
                    }
                })
                .collect(),
        );
    }
    certs_by_issuer
}

fn clique_smith_wot(initial_identities_len: usize) -> BTreeMap<IdtyIndex, (bool, Vec<IdtyIndex>)> {
    let mut certs_by_issuer = BTreeMap::new();
    for i in 1..=initial_identities_len {
        certs_by_issuer.insert(
            i as IdtyIndex,
            (
                true,
                (1..=initial_identities_len)
                    .filter_map(|j| if i != j { Some(j as IdtyIndex) } else { None })
                    .collect(),
            ),
        );
    }
    certs_by_issuer
}

fn session_keys(
    babe: BabeId,
    grandpa: GrandpaId,
    im_online: ImOnlineId,
    authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
    SessionKeys {
        babe,
        grandpa,
        im_online,
        authority_discovery,
    }
}
