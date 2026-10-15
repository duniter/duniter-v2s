//! Compare production Wasm artifacts from before and after the G1 refactor.
#![cfg(feature = "g1")]

use std::{collections::BTreeMap, path::PathBuf};

use codec::{Decode, Encode};
use g1_runtime::{
    AccountId, Block, Header, Runtime, RuntimeCall, RuntimeGenesisConfig, TxExtension,
    UncheckedExtrinsic,
};
use sc_executor::WasmExecutor;
use sp_core::{
    Pair,
    traits::{CallContext, CodeExecutor, RuntimeCode, WrappedRuntimeCode},
};
use sp_io::TestExternalities;
use sp_keyring::sr25519::Keyring;
use sp_runtime::{
    ApplyExtrinsicResult, Digest, DigestItem,
    generic::{Era, SignedPayload},
    traits::{Block as BlockT, Header as HeaderT},
};

struct Wasm {
    code: Vec<u8>,
    hash: Vec<u8>,
    executor: WasmExecutor<sp_io::SubstrateHostFunctions>,
}

impl Wasm {
    fn load(variable: &str) -> Self {
        let code =
            std::fs::read(std::env::var(variable).expect(variable)).expect("read runtime Wasm");
        Self {
            hash: sp_core::blake2_256(&code).to_vec(),
            code,
            executor: WasmExecutor::builder().build(),
        }
    }

    fn call(&self, state: &mut TestExternalities, method: &str, data: &[u8]) -> Vec<u8> {
        let fetcher = WrappedRuntimeCode((&self.code[..]).into());
        let code = RuntimeCode {
            code_fetcher: &fetcher,
            heap_pages: None,
            hash: self.hash.clone(),
        };
        self.executor
            .call(&mut state.ext(), &code, method, data, CallContext::Onchain)
            .0
            .unwrap_or_else(|error| panic!("{method}: {error}"))
    }

    fn metadata(&self) -> Vec<u8> {
        let encoded = self.call(
            &mut TestExternalities::default(),
            "Metadata_metadata_at_version",
            &15u32.encode(),
        );
        Option::<Vec<u8>>::decode(&mut &encoded[..])
            .expect("decode metadata")
            .expect("metadata V15")
    }

    fn metadata_hash(&self, metadata: &[u8], symbol: &str) -> [u8; 32] {
        let metadata = frame_metadata::RuntimeMetadataPrefixed::decode(&mut &metadata[..])
            .expect("decode metadata")
            .1;
        merkleized_metadata::generate_metadata_digest(
            &metadata,
            merkleized_metadata::ExtraInfo {
                spec_version: g1_runtime::VERSION.spec_version,
                spec_name: "g1".into(),
                base58_prefix: g1_runtime::SS58Prefix::get(),
                decimals: 2,
                token_symbol: symbol.into(),
            },
        )
        .expect("compute metadata digest")
        .hash()
    }
}

type State = BTreeMap<Vec<u8>, Vec<u8>>;

fn snapshot(ext: &mut TestExternalities) -> State {
    ext.execute_with(|| {
        let mut result = BTreeMap::new();
        let mut key = Vec::new();
        while let Some(next) = sp_io::storage::next_key(&key) {
            result.insert(next.clone(), sp_io::storage::get(&next).unwrap().to_vec());
            key = next;
        }
        result
    })
}

fn restored(state: &State) -> TestExternalities {
    TestExternalities::new(sp_core::storage::Storage {
        top: state.clone(),
        children_default: Default::default(),
    })
}

fn assert_state_eq(left: &mut TestExternalities, right: &mut TestExternalities, where_: &str) {
    let left_root = left.execute_with(|| sp_io::storage::root(sp_runtime::StateVersion::V1));
    let right_root = right.execute_with(|| sp_io::storage::root(sp_runtime::StateVersion::V1));
    assert_eq!(
        left_root, right_root,
        "{where_}: complete storage roots differ"
    );
    let left = snapshot(left);
    let right = snapshot(right);
    let differences: Vec<_> = left
        .keys()
        .chain(right.keys())
        .filter(|key| left.get(*key) != right.get(*key))
        .map(hex::encode)
        .collect();
    assert!(
        differences.is_empty(),
        "{where_}: storage differs at {differences:?}"
    );
}

fn initial_state(before: &Wasm) -> State {
    let mut config = RuntimeGenesisConfig::default();
    let members = [
        Keyring::Alice,
        Keyring::Bob,
        Keyring::Charlie,
        Keyring::Dave,
        Keyring::Eve,
        Keyring::Ferdie,
    ];
    for (offset, key) in members.iter().enumerate() {
        let index = offset as u32 + 1;
        let account = key.to_account_id();
        config.account.accounts.insert(
            account.clone(),
            common_runtime::GenesisAccountData {
                balance: 1_000_000_000,
                idty_id: Some(index),
            },
        );
        config
            .identity
            .identities
            .push(common_runtime::GenesisIdty {
                index,
                name: common_runtime::IdtyName::from(format!("Member{index}").as_str()),
                value: common_runtime::IdtyValue {
                    data: common_runtime::entities::IdtyData {
                        first_eligible_ud:
                            g1_runtime::pallet_universal_dividend::FirstEligibleUd::min(),
                    },
                    next_creatable_identity_on: 0,
                    old_owner_key: None,
                    owner_key: account,
                    next_scheduled: 0,
                    status: common_runtime::IdtyStatus::Member,
                },
            });
        config.membership.memberships.insert(
            index,
            sp_membership::MembershipData {
                // Exercise expiry without removing the sole block author.
                expire_on: if index == 6 { 45 } else { u32::MAX },
            },
        );
        config.certification.certs_by_receiver.insert(
            index,
            (1..=6)
                .filter(|issuer| *issuer != index)
                .map(|issuer| (issuer, Some(if index == 5 { 40 } else { u32::MAX })))
                .collect(),
        );
    }
    config.certification.apply_cert_period_at_genesis = false;
    config.quota.identities = (1..=6).collect();
    let alice = Keyring::Alice.to_account_id();
    config
        .authority_members
        .initial_authorities
        .insert(1, (alice.clone(), true));
    config
        .smith_members
        .initial_smiths
        .insert(1, (true, vec![]));
    config.session.keys.push((
        alice.clone(),
        alice,
        g1_runtime::opaque::SessionKeys {
            babe: sp_consensus_babe::AuthorityId::from(Keyring::Alice.public()),
            grandpa: sp_core::ed25519::Pair::from_string("//Alice", None)
                .unwrap()
                .public()
                .into(),
            im_online: Keyring::Alice.public().into(),
            authority_discovery: Keyring::Alice.public().into(),
        },
    ));
    config.babe.epoch_config = g1_runtime::BABE_GENESIS_EPOCH_CONFIG;
    config.technical_committee.members = vec![Keyring::Alice.to_account_id()];
    config.account.treasury_balance = 1_000_000;
    let mass = 6_001_000_000;
    config.balances.total_issuance = mass;
    config.universal_dividend.initial_monetary_mass = mass;
    config.universal_dividend.ud = 1000;
    // Seed upcoming schedules in state. Production period constants stay untouched.
    config.universal_dividend.first_ud = Some(12_000);
    config.universal_dividend.first_reeval = Some(12_000);
    let mut config = serde_json::to_value(config).unwrap();
    config["sudo"] = serde_json::json!({"key": null});
    let mut state = TestExternalities::default();
    let result = before.call(
        &mut state,
        "GenesisBuilder_build_state",
        &serde_json::to_vec(&config).unwrap().encode(),
    );
    Result::<(), String>::decode(&mut &result[..])
        .unwrap()
        .expect("baseline genesis");
    snapshot(&mut state)
}

fn signed(
    state: &mut TestExternalities,
    signer: Keyring,
    call: RuntimeCall,
    hash: Option<[u8; 32]>,
) -> UncheckedExtrinsic {
    state.execute_with(|| {
        let extensions: TxExtension = (
            frame_system::CheckNonZeroSender::<Runtime>::new(),
            frame_system::CheckSpecVersion::<Runtime>::new(),
            frame_system::CheckTxVersion::<Runtime>::new(),
            frame_system::CheckGenesis::<Runtime>::new(),
            frame_system::CheckEra::<Runtime>::from(Era::Immortal),
            pallet_oneshot_account::CheckNonce::<Runtime>::from(
                frame_system::CheckNonce::<Runtime>::from(
                    frame_system::Pallet::<Runtime>::account_nonce(signer.to_account_id()),
                ),
            ),
            frame_system::CheckWeight::<Runtime>::new(),
            pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(0),
            hash.map_or_else(
                || frame_metadata_hash_extension::CheckMetadataHash::<Runtime>::new(false),
                frame_metadata_hash_extension::CheckMetadataHash::<Runtime>::new_with_custom_hash,
            ),
        );
        let payload =
            SignedPayload::new(call.clone(), extensions.clone()).expect("signing payload");
        let signature = payload.using_encoded(|bytes| signer.pair().sign(bytes));
        UncheckedExtrinsic::new_signed(
            call,
            signer.to_account_id().into(),
            signature.into(),
            extensions,
        )
    })
}

fn apply(
    wasm: &Wasm,
    state: &mut TestExternalities,
    xt: &UncheckedExtrinsic,
) -> ApplyExtrinsicResult {
    let result = wasm.call(state, "BlockBuilder_apply_extrinsic", &xt.encode());
    ApplyExtrinsicResult::decode(&mut &result[..]).expect("decode extrinsic result")
}

fn transfer() -> RuntimeCall {
    RuntimeCall::Balances(g1_runtime::BalancesCall::transfer_allow_death {
        dest: AccountId::new([42; 32]).into(),
        value: 10_000,
    })
}

#[test]
#[ignore = "run scripts/check_g1_refactor.sh with both production Wasm artifacts"]
fn g1_before_and_after_execute_the_same_state() {
    const {
        assert!(
            !cfg!(feature = "fast")
                && !cfg!(feature = "constant-fees")
                && !cfg!(feature = "runtime-benchmarks"),
            "use production runtime features"
        );
    }
    let before = Wasm::load("G1_BEFORE_WASM");
    let after = Wasm::load("G1_AFTER_WASM");
    let report = PathBuf::from(std::env::var("G1_COMPARISON_DIR").expect("G1_COMPARISON_DIR"));
    std::fs::create_dir_all(&report).unwrap();
    let old_metadata = before.metadata();
    let new_metadata = after.metadata();
    std::fs::write(report.join("before.scale"), &old_metadata).unwrap();
    std::fs::write(report.join("after.scale"), &new_metadata).unwrap();
    let mut old_version = sc_executor::RuntimeVersion::decode(
        &mut &before.call(&mut TestExternalities::default(), "Core_version", &[])[..],
    )
    .unwrap();
    let new_version = sc_executor::RuntimeVersion::decode(
        &mut &after.call(&mut TestExternalities::default(), "Core_version", &[])[..],
    )
    .unwrap();
    assert_eq!(old_version.impl_name, "duniter-g1");
    assert_eq!(new_version.impl_name, "duniter");
    old_version.impl_name = new_version.impl_name.clone();
    assert_eq!(
        old_version.encode(),
        new_version.encode(),
        "unexpected runtime version change"
    );
    let old_hash = before.metadata_hash(&old_metadata, "Ğ");
    let new_hash = after.metadata_hash(&new_metadata, "Ğ1");
    assert_ne!(old_hash, new_hash);
    let initial = initial_state(&before);
    let mut left = restored(&initial);
    let mut right = restored(&initial);
    assert_state_eq(&mut left, &mut right, "initial state");
    let mut parent = sp_core::H256::zero();
    let mut saw_ud = false;
    let mut saw_reevaluation = false;
    let mut saw_certification_expiry = false;
    let mut successful_calls = 0;
    let mut failed_dispatches = 0;
    // Cross an epoch boundary using production constants and real Wasm block execution.
    let blocks = g1_runtime::EpochDuration::get() as u32 + 2;
    for number in 1..=blocks {
        let parent_state = snapshot(&mut left);
        let digest = Digest {
            logs: vec![DigestItem::PreRuntime(
                sp_consensus_babe::BABE_ENGINE_ID,
                sp_consensus_babe::digests::PreDigest::SecondaryPlain(
                    sp_consensus_babe::digests::SecondaryPlainPreDigest {
                        authority_index: 0,
                        slot: (number as u64).into(),
                    },
                )
                .encode(),
            )],
        };
        let header = Header::new(
            number,
            Default::default(),
            Default::default(),
            parent,
            digest,
        );
        assert_eq!(
            before.call(&mut left, "Core_initialize_block", &header.encode()),
            after.call(&mut right, "Core_initialize_block", &header.encode())
        );
        let mut extrinsics = vec![UncheckedExtrinsic::new_bare(RuntimeCall::Timestamp(
            g1_runtime::TimestampCall::set {
                now: number as u64 * 6000,
            },
        ))];
        assert_eq!(apply(&before, &mut left, &extrinsics[0]), Ok(Ok(())));
        assert_eq!(apply(&after, &mut right, &extrinsics[0]), Ok(Ok(())));
        if number <= 8 || number == blocks {
            let call = match number {
                2 => RuntimeCall::System(g1_runtime::SystemCall::remark_with_event {
                    remark: b"G1 differential".to_vec(),
                }),
                3 => RuntimeCall::UniversalDividend(
                    g1_runtime::pallet_universal_dividend::Call::claim_uds {},
                ),
                4 => RuntimeCall::Balances(g1_runtime::BalancesCall::transfer_allow_death {
                    dest: AccountId::new([43; 32]).into(),
                    value: u64::MAX,
                }),
                _ => transfer(),
            };
            let xt = signed(&mut left, Keyring::Bob, call, None);
            let result = apply(&before, &mut left, &xt);
            assert_eq!(
                result,
                apply(&after, &mut right, &xt),
                "block {number}: dispatch result"
            );
            match result.expect("valid transaction") {
                Ok(()) => successful_calls += 1,
                Err(_) => failed_dispatches += 1,
            }
            extrinsics.push(xt);
        }
        // Exercise hardware-wallet metadata checks without normalizing their outcomes.
        if number == 1 {
            for (wasm, state, own_hash, wrong_symbol) in [
                (
                    &before,
                    &mut left,
                    old_hash,
                    before.metadata_hash(&old_metadata, "Ğ1"),
                ),
                (
                    &after,
                    &mut right,
                    new_hash,
                    after.metadata_hash(&new_metadata, "Ğ"),
                ),
            ] {
                let probe_state = snapshot(state);
                for rejected_hash in [
                    wrong_symbol,
                    if own_hash == old_hash {
                        new_hash
                    } else {
                        old_hash
                    },
                ] {
                    let mut probe = restored(&probe_state);
                    let xt = signed(&mut probe, Keyring::Bob, transfer(), Some(rejected_hash));
                    assert_eq!(
                        apply(wasm, &mut probe, &xt),
                        Err(sp_runtime::transaction_validity::InvalidTransaction::BadProof.into())
                    );
                }
            }
            // Each runtime accepts its own metadata digest. Different signatures produce
            // different extrinsic roots, so compare these isolated probes, then discard them.
            let mut old_probe = restored(&snapshot(&mut left));
            let mut new_probe = restored(&snapshot(&mut right));
            let old_xt = signed(&mut old_probe, Keyring::Bob, transfer(), Some(old_hash));
            let new_xt = signed(&mut new_probe, Keyring::Bob, transfer(), Some(new_hash));
            assert_eq!(apply(&before, &mut old_probe, &old_xt), Ok(Ok(())));
            assert_eq!(apply(&after, &mut new_probe, &new_xt), Ok(Ok(())));
        }
        for record in left.execute_with(g1_runtime::System::events) {
            match record.event {
                g1_runtime::RuntimeEvent::UniversalDividend(
                    g1_runtime::pallet_universal_dividend::Event::NewUdCreated { .. },
                ) => saw_ud = true,
                g1_runtime::RuntimeEvent::UniversalDividend(
                    g1_runtime::pallet_universal_dividend::Event::UdReevalued { .. },
                ) => saw_reevaluation = true,
                g1_runtime::RuntimeEvent::Certification(_) if number == 40 => {
                    saw_certification_expiry = true
                }
                _ => {}
            }
        }
        let old_header = before.call(&mut left, "BlockBuilder_finalize_block", &[]);
        let new_header = after.call(&mut right, "BlockBuilder_finalize_block", &[]);
        assert_eq!(old_header, new_header, "block {number}: finalized header");
        assert_state_eq(&mut left, &mut right, &format!("block {number}"));
        let header = Header::decode(&mut &old_header[..]).unwrap();
        let block = Block::new(header.clone(), extrinsics);
        // Import exactly the same encoded block on independent copies of its parent state.
        let mut old_import = restored(&parent_state);
        let mut new_import = restored(&parent_state);
        assert_eq!(
            before.call(&mut old_import, "Core_execute_block", &block.encode()),
            after.call(&mut new_import, "Core_execute_block", &block.encode())
        );
        assert_state_eq(&mut old_import, &mut new_import, "block import");
        assert_state_eq(&mut left, &mut old_import, "build versus import");
        parent = header.hash();
    }
    assert!(successful_calls >= 8 && failed_dispatches == 1);
    assert!(
        saw_ud && saw_reevaluation && saw_certification_expiry,
        "scheduled hooks must execute"
    );
    left.execute_with(|| {
        assert!(
            g1_runtime::Membership::membership(6).is_none(),
            "membership must expire"
        );
        assert_eq!(
            g1_runtime::Session::current_index(),
            1,
            "production epoch must advance"
        );
    });
    let summary = format!(
        "{blocks} production blocks built and imported on identical state; {successful_calls} successful signed calls; {failed_dispatches} failed dispatch; metadata hash accepts each expected symbol and rejects wrong symbols and cross-runtime hashes.\n"
    );
    std::fs::write(report.join("differential.txt"), &summary).unwrap();
    println!("{summary}");
}
