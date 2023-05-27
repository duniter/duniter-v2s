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
use gtest_genesis::{build_genesis, GenesisJson};
use gtest_runtime::{
    opaque::SessionKeys, AccountConfig, AccountId, AuthorityMembersConfig, BabeConfig, CertConfig,
    GenesisConfig, IdentityConfig, ImOnlineId, MembershipConfig, SessionConfig, SmithCertConfig,
    SmithMembershipConfig, SudoConfig, SystemConfig, TechnicalCommitteeConfig,
    UniversalDividendConfig, WASM_BINARY,
};
use jsonrpsee::core::JsonValue;
use sc_network_common::config::MultiaddrWithPeerId; // in the future available in sc_network::config
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use serde::Deserialize;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{blake2_256, sr25519, Encode, H256};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_membership::MembershipData;
use std::collections::BTreeMap;

pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;
pub type AuthorityKeys = (
    AccountId,
    GrandpaId,
    BabeId,
    ImOnlineId,
    AuthorityDiscoveryId,
);
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

// === client specifications ===

/// emulate client specifications to get them from json
#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ClientSpec {
    name: String,
    id: String,
    chain_type: ChainType,
    boot_nodes: Vec<MultiaddrWithPeerId>,
    telemetry_endpoints: Option<TelemetryEndpoints>,
    // protocol_id: Option<String>,
    // #[serde(default = "Default::default", skip_serializing_if = "Option::is_none")]
    // fork_id: Option<String>,
    properties: Option<serde_json::Map<std::string::String, JsonValue>>,
    // #[serde(default)]
    // code_substitutes: BTreeMap<String, Bytes>,
}

// === development chainspecs ===

/// generate development chainspec with Alice validator
// there is some code duplication because we can not use ClientSpec
pub fn development_chainspecs() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "wasm not available".to_string())?;

    // custom genesis when DUNITER_GTEST_GENESIS is set
    if let Ok(genesis_json_path) = std::env::var("DUNITER_GTEST_GENESIS") {
        // log
        log::info!("loading genesis from {genesis_json_path}");
        // open json genesis file
        let file = std::fs::File::open(&genesis_json_path)
            .map_err(|e| format!("Error opening gen conf file `{}`: {}", genesis_json_path, e))?;
        // memory map the file to avoid loading it in memory
        let bytes = unsafe {
            memmap2::Mmap::map(&file).map_err(|e| {
                format!("Error mmaping gen conf file `{}`: {}", genesis_json_path, e)
            })?
        };
        // parse the json file
        let genesis_data: GenesisJson = serde_json::from_slice(&bytes)
            .map_err(|e| format!("Error parsing gen conf file: {}", e))?;

        // return chainspecs
        Ok(ChainSpec::from_genesis(
            // Name
            "ĞTest Development",
            // ID
            "gtest_dev",
            // chain type
            sc_service::ChainType::Development,
            // genesis config constructor
            move || {
                build_genesis(
                    // genesis data built from json
                    genesis_data.clone(),
                    // wasm binary
                    wasm_binary,
                    // replace authority by Alice
                    Some(get_session_keys_from_seed("Alice").encode()),
                )
                .expect("genesis building failed")
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
                    "tokenDecimals": 2,
                    "tokenSymbol": "ĞT",
                })
                .as_object()
                .expect("must be a map")
                .clone(),
            ),
            // Extensions
            None,
        ))
    } else {
        // log
        log::info!("generating genesis");
        // generated genesis
        Ok(ChainSpec::from_genesis(
            // Name
            "ĞTest Development",
            // ID
            "gtest_dev",
            // chain type
            ChainType::Development,
            // constructor
            move || {
                generate_genesis(
                    wasm_binary,
                    // Initial authorities len
                    1,
                    // Initial smith members len
                    3,
                    // Inital identities len
                    4,
                    // Sudo account
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
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
                    "tokenDecimals": 2,
                    "tokenSymbol": "ĞT",
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

// === live chainspecs ===

/// live chainspecs
// one smith must have session keys
pub fn live_chainspecs(
    client_spec: ClientSpec,
    genesis_data: GenesisJson,
) -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "wasm not available".to_string())?;

    // return chainspecs
    Ok(ChainSpec::from_genesis(
        // Name
        client_spec.name.as_str(),
        // ID
        client_spec.id.as_str(),
        // chain type
        client_spec.chain_type,
        // genesis config constructor
        move || {
            build_genesis(
                // genesis data
                genesis_data.clone(),
                // wasm binary
                wasm_binary,
                // do not replace session keys
                None,
            )
            .expect("genesis building failed")
        },
        // Bootnodes
        client_spec.boot_nodes,
        // Telemetry (by default, enable telemetry, can be disabled with argument)
        client_spec.telemetry_endpoints,
        // Protocol ID
        None,
        // Fork ID
        None,
        // Properties
        client_spec.properties,
        // Extensions
        None,
    ))
}

/// generate a genesis with given number of smith and identities
fn generate_genesis(
    wasm_binary: &[u8],
    initial_authorities_len: usize,
    initial_smiths_len: usize,
    initial_identities_len: usize,
    root_key: AccountId,
) -> GenesisConfig {
    assert!(initial_identities_len <= 6);
    assert!(initial_smiths_len <= initial_identities_len);
    assert!(initial_authorities_len <= initial_smiths_len);

    let first_ud = 1_000;

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

    GenesisConfig {
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
        authority_discovery: Default::default(),
        authority_members: AuthorityMembersConfig {
            initial_authorities: initial_smiths
                .iter()
                .enumerate()
                .map(|(i, keys)| (i as u32 + 1, (keys.0.clone(), true)))
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
                .map(|(i, (name, owner_key))| common_runtime::GenesisIdty {
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
                .map(|i| {
                    (
                        i as u32,
                        MembershipData {
                            expire_on: gtest_runtime::MembershipPeriod::get(),
                        },
                    )
                })
                .collect(),
        },
        cert: CertConfig {
            apply_cert_period_at_genesis: false,
            certs_by_receiver: clique_wot(initial_identities.len()),
        },
        smith_membership: SmithMembershipConfig {
            memberships: (1..=initial_smiths_len)
                .map(|i| {
                    (
                        i as u32,
                        MembershipData {
                            expire_on: gtest_runtime::SmithMembershipPeriod::get(),
                        },
                    )
                })
                .collect(),
        },
        smith_cert: SmithCertConfig {
            apply_cert_period_at_genesis: false,
            certs_by_receiver: clique_wot(initial_smiths_len),
        },
        universal_dividend: UniversalDividendConfig {
            first_reeval: 100,
            first_ud: 1_000,
            initial_monetary_mass: 0,
        },
        treasury: Default::default(),
    }
}
