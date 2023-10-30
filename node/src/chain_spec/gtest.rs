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
use crate::chain_spec::gen_genesis_data::{
    CommonParameters, GenesisData, GenesisIdentity, SessionKeysProvider,
};
use common_runtime::constants::*;
use common_runtime::entities::IdtyData;
use common_runtime::*;
use gtest_runtime::{
    opaque::SessionKeys, parameters, AccountConfig, AccountId, AuthorityMembersConfig, BabeConfig,
    CertConfig, GenesisConfig, IdentityConfig, ImOnlineId, MembershipConfig, Perbill,
    SessionConfig, SmithCertConfig, SmithMembershipConfig, SudoConfig, SystemConfig,
    TechnicalCommitteeConfig, UniversalDividendConfig, WASM_BINARY,
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

const TOKEN_DECIMALS: usize = 2;
const TOKEN_SYMBOL: &str = "ĞT";

#[derive(Default, Clone, Deserialize)]
// No parameters for GTest (unlike GDev)
struct GenesisParameters {}

struct GTestSKP;
impl SessionKeysProvider<SessionKeys> for GTestSKP {
    fn session_keys(keys: &AuthorityKeys) -> SessionKeys {
        let cloned = keys.clone();
        SessionKeys {
            grandpa: cloned.1,
            babe: cloned.2,
            im_online: cloned.3,
            authority_discovery: cloned.4,
        }
    }
}

fn get_parameters(_: &Option<GenesisParameters>) -> CommonParameters {
    CommonParameters {
        currency_name: TOKEN_SYMBOL.to_string(),
        decimals: TOKEN_DECIMALS,
        existential_deposit: parameters::ExistentialDeposit::get(),
        membership_period: parameters::MembershipPeriod::get(),
        cert_period: parameters::CertPeriod::get(),
        smith_membership_period: parameters::SmithMembershipPeriod::get(),
        smith_certs_validity_period: parameters::SmithValidityPeriod::get(),
        min_cert: parameters::WotMinCertForMembership::get(),
        smith_min_cert: parameters::SmithWotMinCertForMembership::get(),
        cert_max_by_issuer: parameters::MaxByIssuer::get(),
        cert_validity_period: parameters::ValidityPeriod::get(),
        c2: parameters::SquareMoneyGrowthRate::get(),
        ud_creation_period: parameters::UdCreationPeriod::get() as u64, // TODO: cast?
        distance_min_accessible_referees: Perbill::from_percent(80),    // TODO: generalize
        max_depth: 5,                                                   // TODO: generalize value
        ud_reeval_period: parameters::UdReevalPeriod::get() as u64,     // TODO: cast?
    }
}

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
pub fn development_chainspecs(json_file_path: String) -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "wasm not available".to_string())?;
    Ok(ChainSpec::from_genesis(
        // Name
        "ĞTest Development",
        // ID
        "gtest_dev",
        // chain type
        sc_service::ChainType::Development,
        // constructor
        move || {
            let genesis_data =
                gen_genesis_data::generate_genesis_data::<_, _, SessionKeys, GTestSKP>(
                    json_file_path.clone(),
                    get_parameters,
                    Some("Alice".to_owned()),
                )
                .expect("Genesis Data must be buildable");
            genesis_data_to_gtest_genesis_conf(genesis_data, wasm_binary)
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

// === live chainspecs ===

/// live chainspecs
// one smith must have session keys
pub fn live_chainspecs(
    client_spec: ClientSpec,
    json_file_path: String,
) -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "wasm not available".to_string())?;
    Ok(ChainSpec::from_genesis(
        // Name
        client_spec.name.as_str(),
        // ID
        client_spec.id.as_str(),
        // chain type
        client_spec.chain_type,
        // genesis config constructor
        move || {
            let genesis_data =
                gen_genesis_data::generate_genesis_data::<_, _, SessionKeys, GTestSKP>(
                    json_file_path.clone(),
                    get_parameters,
                    None,
                )
                .expect("Genesis Data must be buildable");
            genesis_data_to_gtest_genesis_conf(genesis_data, wasm_binary)
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

/// custom genesis
fn genesis_data_to_gtest_genesis_conf(
    genesis_data: super::gen_genesis_data::GenesisData<GenesisParameters, SessionKeys>,
    wasm_binary: &[u8],
) -> gtest_runtime::GenesisConfig {
    let super::gen_genesis_data::GenesisData {
        accounts,
        treasury_balance,
        certs_by_receiver,
        first_ud,
        first_ud_reeval,
        identities,
        initial_authorities,
        initial_monetary_mass,
        memberships,
        parameters,
        common_parameters,
        session_keys_map,
        smith_certs_by_receiver,
        smith_memberships,
        sudo_key,
        technical_committee_members,
        ud,
    } = genesis_data;

    gtest_runtime::GenesisConfig {
        system: SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
        },
        account: AccountConfig {
            accounts,
            treasury_balance,
        },
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
                .map(
                    |(
                        i,
                        GenesisIdentity {
                            idty_index,
                            name,
                            owner_key,
                            old_owner_key,
                            active,
                        },
                    )| GenesisIdty {
                        index: idty_index,
                        name: common_runtime::IdtyName::from(name.as_str()),
                        value: common_runtime::IdtyValue {
                            data: IdtyData::new(),
                            next_creatable_identity_on: 0,
                            old_owner_key: old_owner_key.clone().map(|address| (address, 0)),
                            owner_key,
                            removable_on: if active { 0 } else { 2 },
                            status: IdtyStatus::Validated,
                        },
                    },
                )
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
            ud,
            #[cfg(test)]
            initial_members: vec![],
        },
        treasury: Default::default(),
    }
}
