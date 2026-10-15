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
use gtest_runtime::{opaque::SessionKeys, *};
use pallet_authority_members::OnNewSession;
use pallet_smith_members::SmithMeta;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::{AuthorityId as BabeId, Slot, VrfInput, VrfProof};
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{Encode, H256, Pair, Public, crypto::IsWrappedBy, sr25519};
use sp_keyring::sr25519::Keyring;
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
    certifiers: usize,
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
            certifiers: 0,
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
                .or_insert(GenesisAccountData {
                    ..Default::default()
                })
                .balance = balance;
        }
        self
    }

    /// Add a valid six-member WoT without occupying the named scenario accounts.
    /// Reserved fixture indexes keep the candidate's expected index stable.
    pub fn with_certifiers(mut self, count: usize) -> Self {
        self.certifiers = count;
        self
    }

    pub fn build(self) -> sp_io::TestExternalities {
        let original_count = self.initial_identities.len();
        let mut config = RuntimeGenesisConfig::default();
        config.account.accounts = self.initial_accounts;
        config.account.treasury_balance = ExistentialDeposit::get();
        for (i, (name, owner_key)) in self.initial_identities.iter().enumerate() {
            config.identity.identities.push(GenesisIdty {
                index: i as u32 + 1,
                name: name.clone(),
                value: IdtyValue {
                    data: IdtyData {
                        first_eligible_ud: pallet_universal_dividend::FirstEligibleUd::min(),
                    },
                    next_creatable_identity_on: 0,
                    old_owner_key: None,
                    owner_key: owner_key.clone(),
                    next_scheduled: 0,
                    status: IdtyStatus::Member,
                },
            });
        }
        if self.certifiers > 0 {
            for i in 0..self.certifiers.saturating_sub(original_count) {
                let index = 100 + i as u32;
                let name = format!("Support{i}");
                let owner_key = get_account_id_from_seed::<sr25519::Public>(&name);
                config.account.accounts.insert(
                    owner_key.clone(),
                    GenesisAccountData {
                        balance: 0,
                        idty_id: Some(index),
                    },
                );
                config.identity.identities.push(GenesisIdty {
                    index,
                    name: IdtyName::from(name.as_str()),
                    value: IdtyValue {
                        data: IdtyData {
                            first_eligible_ud: pallet_universal_dividend::FirstEligibleUd::min(),
                        },
                        next_creatable_identity_on: 0,
                        old_owner_key: None,
                        owner_key,
                        next_scheduled: 0,
                        status: IdtyStatus::Member,
                    },
                });
            }
        }
        let indices: Vec<u32> = config.identity.identities.iter().map(|i| i.index).collect();
        config.quota.identities = indices.clone();
        config.membership.memberships = indices
            .iter()
            .map(|i| {
                (
                    *i,
                    MembershipData {
                        expire_on: if *i >= 100 {
                            u32::MAX
                        } else {
                            MembershipPeriod::get()
                        },
                    },
                )
            })
            .collect();
        config.certification.certs_by_receiver = indices
            .iter()
            .map(|receiver| {
                (
                    *receiver,
                    indices
                        .iter()
                        .filter(|issuer| *issuer != receiver)
                        .map(|issuer| (*issuer, Some(ValidityPeriod::get())))
                        .collect(),
                )
            })
            .collect();
        config.certification.apply_cert_period_at_genesis = false;
        config.authority_members.initial_authorities = self
            .initial_smiths
            .iter()
            .enumerate()
            .map(|(i, keys)| {
                (
                    i as u32 + 1,
                    (keys.0.clone(), i < self.initial_authorities_len),
                )
            })
            .collect();
        config.session.keys = self
            .initial_smiths
            .iter()
            .map(|keys| {
                (
                    keys.0.clone(),
                    keys.0.clone(),
                    session_keys(
                        keys.1.clone(),
                        keys.2.clone(),
                        keys.3.clone(),
                        keys.4.clone(),
                    ),
                )
            })
            .collect();
        let mut smiths: Vec<u32> = (1..=self.initial_smiths.len() as u32).collect();
        if self.certifiers > 0 && self.initial_smiths.len() < 4 {
            smiths.push(100);
        }
        config.smith_members.initial_smiths = smiths
            .iter()
            .map(|i| {
                (
                    *i,
                    (true, smiths.iter().copied().filter(|j| j != i).collect()),
                )
            })
            .collect();
        let mass = config.account.accounts.values().map(|a| a.balance).sum();
        config.balances.total_issuance = mass;
        config.universal_dividend.first_ud = Some(24_000);
        config.universal_dividend.first_reeval = Some(UdReevalPeriod::get());
        config.universal_dividend.initial_monetary_mass = mass;
        config.universal_dividend.ud = 1_000;
        let mut storage = frame_system::GenesisConfig::<Runtime>::default()
            .build_storage()
            .unwrap();
        config
            .authority_members
            .assimilate_storage(&mut storage)
            .unwrap();
        config.account.assimilate_storage(&mut storage).unwrap();
        config.balances.assimilate_storage(&mut storage).unwrap();
        config.session.assimilate_storage(&mut storage).unwrap();
        config.identity.assimilate_storage(&mut storage).unwrap();
        config.quota.assimilate_storage(&mut storage).unwrap();
        config.membership.assimilate_storage(&mut storage).unwrap();
        config
            .certification
            .assimilate_storage(&mut storage)
            .unwrap();
        config
            .smith_members
            .assimilate_storage(&mut storage)
            .unwrap();
        config
            .universal_dividend
            .assimilate_storage(&mut storage)
            .unwrap();
        let mut ext = sp_io::TestExternalities::new(storage);
        ext.execute_with(|| {
            let mut next_index_key = sp_core::twox_128(b"Identity").to_vec();
            next_index_key.extend(sp_core::twox_128(b"NextIdtyIndex"));
            frame_support::storage::unhashed::put(&next_index_key, &(original_count as u32 + 1));
            System::set_block_number(1);
        });
        ext
    }
}

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{seed}"), None)
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
    signer: Keyring,
    tip: Balance,
    nonce: u32,
) -> gtest_runtime::UncheckedExtrinsic {
    let tx_ext: gtest_runtime::TxExtension = (
        frame_system::CheckNonZeroSender::<gtest_runtime::Runtime>::new(),
        frame_system::CheckSpecVersion::<gtest_runtime::Runtime>::new(),
        frame_system::CheckTxVersion::<gtest_runtime::Runtime>::new(),
        frame_system::CheckGenesis::<gtest_runtime::Runtime>::new(),
        frame_system::CheckEra::<gtest_runtime::Runtime>::from(sp_runtime::generic::Era::mortal(
            era, block,
        )),
        pallet_oneshot_account::CheckNonce::<gtest_runtime::Runtime>::from(
            frame_system::CheckNonce::<gtest_runtime::Runtime>::from(nonce),
        ),
        frame_system::CheckWeight::<gtest_runtime::Runtime>::new(),
        pallet_transaction_payment::ChargeTransactionPayment::<gtest_runtime::Runtime>::from(tip),
        frame_metadata_hash_extension::CheckMetadataHash::<gtest_runtime::Runtime>::new(false),
    );
    let payload = SignedPayload::new(call.clone(), tx_ext.clone()).unwrap();
    let origin = signer;
    let sig = payload.using_encoded(|payload| origin.pair().sign(payload));

    gtest_runtime::UncheckedExtrinsic::new_signed(
        call,
        origin.to_account_id().into(),
        sig.into(),
        tx_ext,
    )
}

/// Certify a candidate with distinct eligible members under the runtime threshold.
pub fn complete_certifications(receiver: u32) {
    let needed = WotMinCertForMembership::get() as usize;
    let members: Vec<_> = pallet_identity::Identities::<Runtime>::iter()
        .filter(|(index, value)| *index != receiver && value.status == IdtyStatus::Member)
        .map(|(index, value)| (index, value.owner_key))
        .collect();
    for (index, owner) in members {
        if Certification::certs_by_receiver(receiver).len() >= needed {
            return;
        }
        if Certification::certs_by_receiver(receiver)
            .iter()
            .any(|(issuer, _)| *issuer == index)
        {
            continue;
        }
        let next =
            pallet_certification::StorageIdtyCertMeta::<Runtime>::get(index).next_issuable_on;
        run_to_block(next.max(System::block_number()));
        frame_support::assert_ok!(Certification::add_cert(
            RuntimeOrigin::signed(owner),
            receiver
        ));
    }
    assert!(
        Certification::certs_by_receiver(receiver).len() >= needed,
        "not enough available certifiers in fixture"
    );
}

pub fn run_to_next_ud() {
    let next = UniversalDividend::next_ud().expect("UD schedule must be initialized");
    run_to_block(next.div_ceil(BLOCK_TIME) as u32);
}
