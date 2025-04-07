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

use common_runtime::{constants::*, *};
use frame_support::traits::{OnFinalize, OnInitialize};
use gdev_runtime::{opaque::SessionKeys, *};
use pallet_authority_members::OnNewSession;
use pallet_smith_members::SmithMeta;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::{AuthorityId as BabeId, Slot, VrfInput, VrfProof};
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{crypto::IsWrappedBy, sr25519, Encode, Pair, Public, H256};
use sp_keyring::AccountKeyring;
use sp_membership::MembershipData;
use sp_runtime::{
    generic::SignedPayload,
    testing::{Digest, DigestItem},
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
    // endowed accounts with balances
    initial_accounts: BTreeMap<AccountId, GenesisAccountData<Balance, u32>>,
    initial_authorities_len: usize,
    initial_identities: BTreeMap<IdtyName, AccountId>,
    initial_smiths: Vec<AuthorityKeys>,
    parameters: GenesisParameters<u32, u32, Balance, u32>,
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
            parameters: GenesisParameters {
                babe_epoch_duration: 25,
                cert_period: 15,
                cert_max_by_issuer: 10,
                cert_min_received_cert_to_issue_cert: 2,
                cert_validity_period: 10_000,
                distance_evaluation_period: 7,
                idty_confirm_period: 40,
                idty_creation_period: 50,
                membership_period: 100,
                membership_renewal_period: 10,
                ud_creation_period: 60_000,
                ud_reeval_period: 60_000 * 20,
                smith_cert_max_by_issuer: 8,
                smith_wot_min_cert_for_membership: 2,
                smith_inactivity_max_duration: 48,
                wot_first_cert_issuable_on: 20,
                wot_min_cert_for_create_idty_right: 2,
                wot_min_cert_for_membership: 2,
            },
        }
    }

    pub fn with_initial_balances(mut self, initial_balances: Vec<(AccountId, Balance)>) -> Self {
        for (account_id, balance) in initial_balances {
            self.initial_accounts
                .entry(account_id.clone())
                .or_insert(GenesisAccountData {
                    ..Default::default()
                })
                .balance = balance;
        }
        self
    }

    /*pub fn with_parameters(mut self, parameters: GenesisParameters<u32, u32, Balance>) -> Self {
        self.parameters = parameters;
        self
    }*/

    pub fn change_parameters<
        F: Fn(&mut pallet_duniter_test_parameters::Parameters<u32, u32, Balance, u32>),
    >(
        mut self,
        f: F,
    ) -> Self {
        f(&mut self.parameters);
        self
    }

    pub fn build(self) -> sp_io::TestExternalities {
        let Self {
            initial_accounts,
            initial_authorities_len,
            initial_identities,
            initial_smiths,
            parameters,
        } = self;

        let mut t = frame_system::GenesisConfig::<Runtime>::default()
            .build_storage()
            .unwrap();

        // compute total monetary mass
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

        /*pallet_babe::GenesisConfig {
            authorities: Vec::with_capacity(0),
            epoch_config: Some(BABE_GENESIS_EPOCH_CONFIG),
        }
        .assimilate_storage(&mut t)
        .unwrap();*/

        pallet_duniter_account::GenesisConfig::<Runtime> {
            accounts: initial_accounts.clone(),
            treasury_balance: <Runtime as pallet_balances::Config>::ExistentialDeposit::get(),
        }
        .assimilate_storage(&mut t)
        .unwrap();

        // Necessary to initialize TotalIssuance
        pallet_balances::GenesisConfig::<Runtime> {
            total_issuance: monetary_mass,
        }
        .assimilate_storage(&mut t)
        .unwrap();

        pallet_duniter_test_parameters::GenesisConfig::<Runtime> {
            parameters: parameters.clone(),
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
                            expire_on: parameters.membership_period,
                        },
                    )
                })
                .collect(),
        }
        .assimilate_storage(&mut t)
        .unwrap();

        pallet_certification::GenesisConfig::<Runtime> {
            certs_by_receiver: clique_wot(
                initial_identities.len(),
                parameters.cert_validity_period,
            ),
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

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an authority keys.
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
    // force epoch change based on session
    let session = Session::current_index() as u64;
    pallet_babe::EpochIndex::<Runtime>::put(session);
}

pub fn run_to_block(n: u32) {
    while System::block_number() < n {
        // ---
        // Finalize the previous block
        Babe::on_finalize(System::block_number());
        //Timestamp::on_finalize(System::block_number());
        Distance::on_finalize(System::block_number());
        TransactionPayment::on_finalize(System::block_number());
        Authorship::on_finalize(System::block_number());
        Grandpa::on_finalize(System::block_number());

        // ---
        // Set the new block number and author
        System::reset_events();
        System::set_block_number(System::block_number() + 1);
        // Reset the block weight
        System::set_block_consumed_resources(Weight::zero(), 0_usize);

        // Current slot is not incremented by BABE
        pallet_babe::CurrentSlot::<Runtime>::put(pallet_babe::CurrentSlot::<Runtime>::get() + 1);

        // Initialize the new block
        Account::on_initialize(System::block_number());
        Scheduler::on_initialize(System::block_number());
        Session::on_initialize(System::block_number());
        mock_babe_initialize(System::block_number());
        Authorship::on_initialize(System::block_number());

        /*if session_before != session_after {
            Distance::on_new_session(session_after);
        } else {
            use pallet_session::ShouldEndSession;
            assert!(!Babe::should_end_session(System::block_number()));
        }*/

        // ---
        UniversalDividend::on_initialize(System::block_number());
        Wot::on_initialize(System::block_number());
        Identity::on_initialize(System::block_number());
        Membership::on_initialize(System::block_number());
        Certification::on_initialize(System::block_number());

        Timestamp::set_timestamp(System::block_number() as u64 * BLOCK_TIME);
        Distance::on_initialize(System::block_number());
    }
}

/*pub fn make_primary_pre_digest(
    authority_index: sp_consensus_babe::AuthorityIndex,
    slot: sp_consensus_babe::Slot,
    vrf_output: VRFOutput,
    vrf_proof: VRFProof,
) -> Digest {
    let digest_data = sp_consensus_babe::digests::PreDigest::Primary(
        sp_consensus_babe::digests::PrimaryPreDigest {
            authority_index,
            slot,
            vrf_output,
            vrf_proof,
        },
    );
    let log = DigestItem::PreRuntime(sp_consensus_babe::BABE_ENGINE_ID, digest_data.encode());
    Digest { logs: vec![log] }
}

pub fn make_vrf_output(
    slot: Slot,
    pair: &sp_consensus_babe::AuthorityPair,
) -> (VRFOutput, VRFProof, [u8; 32]) {
    let pair = sp_core::sr25519::Pair::from_ref(pair).as_ref();
    let transcript = sp_consensus_babe::make_transcript(&Babe::randomness(), slot, 0);
    let vrf_inout = pair.vrf_sign(transcript);
    let vrf_randomness: sp_consensus_vrf::schnorrkel::Randomness = vrf_inout
        .0
        .make_bytes::<[u8; 32]>(&sp_consensus_babe::BABE_VRF_INOUT_CONTEXT);
    let vrf_output = VRFOutput(vrf_inout.0.to_output());
    let vrf_proof = VRFProof(vrf_inout.1);

    (vrf_output, vrf_proof, vrf_randomness)
}

pub fn make_secondary_vrf_pre_digest(
    authority_index: sp_consensus_babe::AuthorityIndex,
    slot: sp_consensus_babe::Slot,
    vrf_output: VRFOutput,
    vrf_proof: VRFProof,
) -> Digest {
    let digest_data = sp_consensus_babe::digests::PreDigest::SecondaryVRF(
        sp_consensus_babe::digests::SecondaryVRFPreDigest {
            authority_index,
            slot,
            vrf_output,
            vrf_proof,
        },
    );
    let log = DigestItem::PreRuntime(sp_consensus_babe::BABE_ENGINE_ID, digest_data.encode());
    Digest { logs: vec![log] }
}*/

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

/// get extrinsic for given call
pub fn get_unchecked_extrinsic(
    call: RuntimeCall,
    era: u64,
    block: u64,
    signer: AccountKeyring,
    tip: Balance,
    nonce: u32,
) -> gdev_runtime::UncheckedExtrinsic {
    let tx_ext: gdev_runtime::TxExtension = (
        frame_system::CheckNonZeroSender::<gdev_runtime::Runtime>::new(),
        frame_system::CheckSpecVersion::<gdev_runtime::Runtime>::new(),
        frame_system::CheckTxVersion::<gdev_runtime::Runtime>::new(),
        frame_system::CheckGenesis::<gdev_runtime::Runtime>::new(),
        frame_system::CheckEra::<gdev_runtime::Runtime>::from(sp_runtime::generic::Era::mortal(
            era, block,
        )),
        pallet_oneshot_account::CheckNonce::<gdev_runtime::Runtime>::from(
            frame_system::CheckNonce::<gdev_runtime::Runtime>::from(nonce),
        ),
        frame_system::CheckWeight::<gdev_runtime::Runtime>::new(),
        pallet_transaction_payment::ChargeTransactionPayment::<gdev_runtime::Runtime>::from(tip),
        frame_metadata_hash_extension::CheckMetadataHash::<gdev_runtime::Runtime>::new(false),
    );
    let payload = SignedPayload::new(call.clone(), tx_ext.clone()).unwrap();
    let origin = signer;
    let sig = payload.using_encoded(|payload| origin.pair().sign(payload));

    gdev_runtime::UncheckedExtrinsic::new_signed(
        call,
        origin.to_account_id().into(),
        sig.into(),
        tx_ext,
    )
}
