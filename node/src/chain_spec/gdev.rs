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

use super::*;
use common_runtime::constants::*;
use common_runtime::entities::IdtyData;
use common_runtime::*;
use gdev_runtime::{
    opaque::SessionKeys, AccountConfig, AccountId, AuthorityMembersConfig, BabeConfig, CertConfig,
    GenesisConfig, IdentityConfig, ImOnlineId, MembershipConfig, ParametersConfig, SessionConfig,
    SmithCertConfig, SmithMembershipConfig, SudoConfig, SystemConfig, TechnicalCommitteeConfig,
    UniversalDividendConfig, WASM_BINARY,
};
use sc_service::ChainType;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{blake2_256, sr25519, Encode, H256};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_membership::MembershipData;
use std::collections::BTreeMap;

pub type AuthorityKeys = (
    AccountId,
    GrandpaId,
    BabeId,
    ImOnlineId,
    AuthorityDiscoveryId,
);

pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

type GenesisParameters = gdev_runtime::GenesisParameters<u32, u32, u64>;

const TOKEN_DECIMALS: usize = 2;
const TOKEN_SYMBOL: &str = "ĞD";
// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Generate an authority keys.
pub fn get_authority_keys_from_seed(s: &str) -> AuthorityKeys {
    (
        get_account_id_from_seed::<sr25519::Public>(s),
        get_from_seed::<GrandpaId>(s),
        get_from_seed::<BabeId>(s),
        get_from_seed::<ImOnlineId>(s),
        get_from_seed::<AuthorityDiscoveryId>(s),
    )
}

/// Generate session keys
fn get_session_keys_from_seed(s: &str) -> SessionKeys {
    let authority_keys = get_authority_keys_from_seed(s);
    session_keys(
        authority_keys.1,
        authority_keys.2,
        authority_keys.3,
        authority_keys.4,
    )
}

/// get environment variable
fn get_env_u32(env_var_name: &'static str, default_value: u32) -> u32 {
    std::env::var(env_var_name)
        .map_or(Ok(default_value), |s| s.parse())
        .unwrap_or_else(|_| panic!("{} must be a number", env_var_name))
}

/// make session keys struct
fn session_keys(
    grandpa: GrandpaId,
    babe: BabeId,
    im_online: ImOnlineId,
    authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
    SessionKeys {
        grandpa,
        babe,
        im_online,
        authority_discovery,
    }
}

/// generate development chainspec with Alice validator
pub fn development_chain_spec() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    // custom genesis
    if std::env::var("DUNITER_GENESIS_CONFIG").is_ok() {
        super::gen_genesis_data::generate_genesis_data(
            |genesis_data| {
                ChainSpec::from_genesis(
                    // Name
                    "Development",
                    // ID
                    "gdev",
                    // chain type
                    sc_service::ChainType::Development,
                    // constructor
                    move || genesis_data_to_gdev_genesis_conf(genesis_data.clone(), wasm_binary),
                    // Bootnodes
                    vec![],
                    // Telemetry
                    None,
                    // Protocol ID
                    None,
                    //Fork ID
                    None,
                    // Properties
                    Some(
                        serde_json::json!({
                            "tokenDecimals": TOKEN_DECIMALS,
                            "tokenSymbol": TOKEN_SYMBOL,
                        })
                        .as_object()
                        .expect("must be a map")
                        .clone(),
                    ),
                    // Extensions
                    None,
                )
            },
            Some(get_session_keys_from_seed("Alice").encode()),
        )
    }
    // generated genesis
    else {
        Ok(ChainSpec::from_genesis(
            // Name
            "Development",
            // ID
            "gdev",
            // chain type
            ChainType::Development,
            // constructor
            move || {
                gen_genesis_for_local_chain(
                    wasm_binary,
                    // Initial authorities len
                    1,
                    // Initial smith members len
                    3,
                    // Inital identities len
                    4,
                    // Sudo account
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    true,
                )
            },
            // Bootnodes
            vec![],
            // Telemetry
            None,
            // Protocol ID
            None,
            //Fork ID
            None,
            // Properties
            Some(
                serde_json::json!({
                        "tokenDecimals": TOKEN_DECIMALS,
                        "tokenSymbol": TOKEN_SYMBOL,
                })
                .as_object()
                .expect("must be a map")
                .clone(),
            ),
            // Extensions
            None,
        ))
    }
}

/// generate live network chainspecs
pub fn gen_live_conf() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "wasm not available".to_string())?;

    super::gen_genesis_data::generate_genesis_data(
        |genesis_data| {
            ChainSpec::from_genesis(
                // Name
                "Ğdev",
                // ID
                "gdev",
                sc_service::ChainType::Live,
                move || genesis_data_to_gdev_genesis_conf(genesis_data.clone(), wasm_binary),
                // Bootnodes
                vec![],
                // Telemetry
                Some(
                    sc_service::config::TelemetryEndpoints::new(vec![(
                        "wss://telemetry.polkadot.io/submit/".to_owned(),
                        0,
                    )])
                    .expect("invalid telemetry endpoints"),
                ),
                // Protocol ID
                Some("gdev2"),
                //Fork ID
                None,
                // Properties
                Some(
                    serde_json::json!({
                        "tokenDecimals": TOKEN_DECIMALS,
                        "tokenSymbol": TOKEN_SYMBOL,
                    })
                    .as_object()
                    .expect("must be a map")
                    .clone(),
                ),
                // Extensions
                None,
            )
        },
        None,
    )
}

/// generate local network chainspects
pub fn local_testnet_config(
    initial_authorities_len: usize,
    initial_smiths_len: usize,
    initial_identities_len: usize,
) -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "wasm not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Ğdev Local Testnet",
        // ID
        "gdev_local",
        ChainType::Local,
        move || {
            gen_genesis_for_local_chain(
                wasm_binary,
                // Initial authorities len
                initial_authorities_len,
                // Initial smiths len,
                initial_smiths_len,
                // Initial identities len
                initial_identities_len,
                // Sudo account
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        //Fork ID
        None,
        // Properties
        Some(
            serde_json::json!({
                    "tokenDecimals": TOKEN_DECIMALS,
                    "tokenSymbol": TOKEN_SYMBOL,
            })
            .as_object()
            .expect("must be a map")
            .clone(),
        ),
        // Extensions
        None,
    ))
}

/// generate genesis
fn gen_genesis_for_local_chain(
    wasm_binary: &[u8],
    initial_authorities_len: usize,
    initial_smiths_len: usize,
    initial_identities_len: usize,
    root_key: AccountId,
    _enable_println: bool,
) -> gdev_runtime::GenesisConfig {
    assert!(initial_identities_len <= 6);
    assert!(initial_smiths_len <= initial_identities_len);
    assert!(initial_authorities_len <= initial_smiths_len);

    let babe_epoch_duration = get_env_u32("DUNITER_BABE_EPOCH_DURATION", 30) as u64;
    let cert_validity_period = get_env_u32("DUNITER_CERT_VALIDITY_PERIOD", 1_000);
    let first_ud = 1_000;
    let membership_period = get_env_u32("DUNITER_MEMBERSHIP_PERIOD", 1_000);
    let smith_cert_validity_period = get_env_u32("DUNITER_SMITH_CERT_VALIDITY_PERIOD", 1_000);
    let smith_membership_period = get_env_u32("DUNITER_SMITH_MEMBERSHIP_PERIOD", 1_000);
    let ud_creation_period = get_env_u32("DUNITER_UD_CREATION_PERIOD", 10);
    let ud_reeval_period = get_env_u32("DUNITER_UD_REEEVAL_PERIOD", 200);

    let initial_smiths = (0..initial_smiths_len)
        .map(|i| get_authority_keys_from_seed(NAMES[i]))
        .collect::<Vec<AuthorityKeys>>();
    let initial_identities = (0..initial_identities_len)
        .map(|i| {
            (
                IdtyName::from(NAMES[i]),
                get_account_id_from_seed::<sr25519::Public>(NAMES[i]),
            )
        })
        .collect::<BTreeMap<IdtyName, AccountId>>();

    gdev_runtime::GenesisConfig {
        system: SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
        },
        account: AccountConfig {
            accounts: initial_identities
                .iter()
                .enumerate()
                .map(|(i, (_, owner_key))| {
                    (
                        owner_key.clone(),
                        GenesisAccountData {
                            random_id: H256(blake2_256(&(i as u32, owner_key).encode())),
                            balance: first_ud,
                            is_identity: true,
                        },
                    )
                })
                .collect(),
        },
        parameters: ParametersConfig {
            parameters: GenesisParameters {
                babe_epoch_duration,
                cert_period: 15,
                cert_max_by_issuer: 10,
                cert_min_received_cert_to_issue_cert: 2,
                cert_validity_period,
                idty_confirm_period: 40,
                idty_creation_period: 50,
                membership_period,
                pending_membership_period: 500,
                ud_creation_period,
                ud_reeval_period,
                smith_cert_period: 15,
                smith_cert_max_by_issuer: 8,
                smith_cert_min_received_cert_to_issue_cert: 2,
                smith_cert_validity_period,
                smith_membership_period,
                smith_pending_membership_period: 500,
                smith_wot_first_cert_issuable_on: 20,
                smith_wot_min_cert_for_membership: 2,
                wot_first_cert_issuable_on: 20,
                wot_min_cert_for_create_idty_right: 2,
                wot_min_cert_for_membership: 2,
            },
        },
        authority_discovery: Default::default(),
        authority_members: AuthorityMembersConfig {
            initial_authorities: initial_smiths
                .iter()
                .enumerate()
                .map(|(i, keys)| (i as u32 + 1, (keys.0.clone(), i < initial_authorities_len)))
                .collect(),
        },
        balances: Default::default(),
        babe: BabeConfig {
            authorities: Vec::with_capacity(0),
            epoch_config: Some(BABE_GENESIS_EPOCH_CONFIG),
        },
        grandpa: Default::default(),
        im_online: Default::default(),
        session: SessionConfig {
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
        },
        sudo: SudoConfig {
            // Assign network admin rights.
            key: Some(root_key),
        },
        technical_committee: TechnicalCommitteeConfig {
            members: initial_smiths
                .iter()
                .map(|x| x.0.clone())
                .collect::<Vec<_>>(),
            ..Default::default()
        },
        identity: IdentityConfig {
            identities: initial_identities
                .iter()
                .enumerate()
                .map(|(i, (name, owner_key))| GenesisIdty {
                    index: i as u32 + 1,
                    name: name.clone(),
                    value: IdtyValue {
                        data: IdtyData::new(),
                        next_creatable_identity_on: Default::default(),
                        old_owner_key: None,
                        owner_key: owner_key.clone(),
                        removable_on: 0,
                        status: IdtyStatus::Validated,
                    },
                })
                .collect(),
        },
        membership: MembershipConfig {
            memberships: (1..=initial_identities.len())
                .map(|i| (i as u32, MembershipData { expire_on: 0 }))
                .collect(),
        },
        cert: CertConfig {
            apply_cert_period_at_genesis: false,
            certs_by_receiver: clique_wot(initial_identities.len()),
        },
        smith_membership: SmithMembershipConfig {
            memberships: (1..=initial_smiths_len)
                .map(|i| (i as u32, MembershipData { expire_on: 0 }))
                .collect(),
        },
        smith_cert: SmithCertConfig {
            apply_cert_period_at_genesis: false,
            certs_by_receiver: clique_wot(initial_smiths_len),
        },
        universal_dividend: UniversalDividendConfig {
            first_reeval: 100,
            first_ud,
            initial_monetary_mass: initial_identities_len as u64 * first_ud,
        },
        treasury: Default::default(),
    }
}

/// custom genesis
fn genesis_data_to_gdev_genesis_conf(
    genesis_data: super::gen_genesis_data::GenesisData<GenesisParameters, SessionKeys>,
    wasm_binary: &[u8],
) -> gdev_runtime::GenesisConfig {
    let super::gen_genesis_data::GenesisData {
        accounts,
        certs_by_receiver,
        first_ud,
        first_ud_reeval,
        identities,
        initial_authorities,
        initial_monetary_mass,
        memberships,
        parameters,
        session_keys_map,
        smith_certs_by_receiver,
        smith_memberships,
        sudo_key,
        technical_committee_members,
    } = genesis_data;

    gdev_runtime::GenesisConfig {
        system: SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
        },
        account: AccountConfig { accounts },
        parameters: ParametersConfig { parameters },
        authority_discovery: Default::default(),
        authority_members: AuthorityMembersConfig {
            initial_authorities,
        },
        balances: Default::default(),
        babe: BabeConfig {
            authorities: Vec::with_capacity(0),
            epoch_config: Some(BABE_GENESIS_EPOCH_CONFIG),
        },
        grandpa: Default::default(),
        im_online: Default::default(),
        session: SessionConfig {
            keys: session_keys_map
                .into_iter()
                .map(|(account_id, session_keys)| (account_id.clone(), account_id, session_keys))
                .collect::<Vec<_>>(),
        },
        sudo: SudoConfig { key: sudo_key },
        technical_committee: TechnicalCommitteeConfig {
            members: technical_committee_members,
            ..Default::default()
        },
        identity: IdentityConfig {
            identities: identities
                .into_iter()
                .enumerate()
                .map(|(i, (name, owner_key))| GenesisIdty {
                    index: i as u32 + 1,
                    name: common_runtime::IdtyName::from(name.as_str()),
                    value: common_runtime::IdtyValue {
                        data: IdtyData::new(),
                        next_creatable_identity_on: 0,
                        old_owner_key: None,
                        owner_key,
                        removable_on: 0,
                        status: IdtyStatus::Validated,
                    },
                })
                .collect(),
        },
        cert: CertConfig {
            apply_cert_period_at_genesis: true,
            certs_by_receiver,
        },
        membership: MembershipConfig { memberships },
        smith_cert: SmithCertConfig {
            apply_cert_period_at_genesis: true,
            certs_by_receiver: smith_certs_by_receiver,
        },
        smith_membership: SmithMembershipConfig {
            memberships: smith_memberships,
        },
        universal_dividend: UniversalDividendConfig {
            first_reeval: first_ud_reeval,
            first_ud,
            initial_monetary_mass,
        },
        treasury: Default::default(),
    }
}
