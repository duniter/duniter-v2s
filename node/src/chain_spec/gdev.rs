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
    AuthorityKeys, CommonParameters, GenesisIdentity, SessionKeysProvider,
};
use common_runtime::{constants::*, entities::IdtyData, GenesisIdty, IdtyStatus};
use gdev_runtime::{
    opaque::SessionKeys, pallet_universal_dividend, parameters, Runtime, WASM_BINARY,
};
use jsonrpsee::core::JsonValue;
use sc_network::config::MultiaddrWithPeerId;
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use serde::Deserialize;
use sp_core::{sr25519, Get};
use sp_runtime::Perbill;
use std::{env, fs};

pub type ChainSpec = sc_service::GenericChainSpec;

type GenesisParameters = gdev_runtime::GenesisParameters<u32, u32, u64, u32>;

const TOKEN_DECIMALS: usize = 2;
const TOKEN_SYMBOL: &str = "ĞD";
static EXISTENTIAL_DEPOSIT: u64 = parameters::ExistentialDeposit::get();
// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

struct GDevSKP;
impl SessionKeysProvider<SessionKeys> for GDevSKP {
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

fn get_parameters(parameters_from_file: &Option<GenesisParameters>) -> CommonParameters {
    let parameters_from_file = parameters_from_file
        .clone()
        .expect("parameters must be defined in file for GDev");
    CommonParameters {
        currency_name: TOKEN_SYMBOL.to_string(),
        decimals: TOKEN_DECIMALS,
        babe_epoch_duration: parameters_from_file.babe_epoch_duration,
        babe_expected_block_time: parameters::ExpectedBlockTime::get(),
        babe_max_authorities: parameters::MaxAuthorities::get(),
        timestamp_minimum_period: parameters::MinimumPeriod::get(),
        balances_existential_deposit: EXISTENTIAL_DEPOSIT,
        authority_members_max_authorities: parameters::MaxAuthorities::get(),
        grandpa_max_authorities: parameters::MaxAuthorities::get(),
        universal_dividend_max_past_reevals:
            <Runtime as pallet_universal_dividend::Config>::MaxPastReeval::get(),
        universal_dividend_square_money_growth_rate: parameters::SquareMoneyGrowthRate::get(),
        universal_dividend_ud_creation_period: parameters_from_file.ud_creation_period,
        universal_dividend_ud_reeval_period: parameters_from_file.ud_reeval_period,
        wot_first_issuable_on: parameters_from_file.wot_first_cert_issuable_on,
        wot_min_cert_for_membership: parameters_from_file.wot_min_cert_for_membership,
        wot_min_cert_for_create_idty_right: parameters_from_file.wot_min_cert_for_create_idty_right,
        identity_confirm_period: parameters_from_file.idty_confirm_period,
        identity_change_owner_key_period: parameters::ChangeOwnerKeyPeriod::get(),
        identity_idty_creation_period: parameters_from_file.idty_creation_period,
        identity_autorevocation_period: parameters::AutorevocationPeriod::get(),
        membership_membership_period: parameters_from_file.membership_period,
        membership_membership_renewal_period: parameters_from_file.membership_renewal_period,
        cert_max_by_issuer: parameters_from_file.cert_max_by_issuer,
        cert_min_received_cert_to_be_able_to_issue_cert: parameters_from_file
            .cert_min_received_cert_to_issue_cert,
        cert_validity_period: parameters_from_file.cert_validity_period,
        distance_min_accessible_referees: Perbill::from_percent(80), // TODO: generalize
        distance_max_depth: 5,                                       // TODO: generalize
        smith_sub_wot_min_cert_for_membership: parameters_from_file
            .smith_wot_min_cert_for_membership,
        smith_cert_max_by_issuer: parameters_from_file.smith_cert_max_by_issuer,
        smith_inactivity_max_duration: parameters_from_file.smith_inactivity_max_duration,
        cert_cert_period: parameters_from_file.cert_period,
        treasury_spend_period: <Runtime as pallet_treasury::Config>::SpendPeriod::get(),
    }
}

/// generate development chainspec with Alice validator
pub fn gdev_development_chain_spec(config_file_path: String) -> Result<ChainSpec, String> {
    Ok(ChainSpec::builder(
        &get_wasm_binary().ok_or_else(|| "Development wasm not available".to_string())?,
        None,
    )
    .with_name("Development")
    .with_id("gdev")
    .with_chain_type(ChainType::Development)
    .with_genesis_config_patch({
        let genesis_data = gen_genesis_data::generate_genesis_data::<_, _, SessionKeys, GDevSKP>(
            config_file_path.clone(),
            get_parameters,
            Some("Alice".to_owned()),
        )
        .expect("Genesis Data must be buildable");
        genesis_data_to_gdev_genesis_conf(genesis_data)
    })
    .with_properties(
        serde_json::json!({
            "tokenDecimals": TOKEN_DECIMALS,
            "tokenSymbol": TOKEN_SYMBOL,
        })
        .as_object()
        .expect("must be a map")
        .clone(),
    )
    .build())
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

/// generate live network chainspecs
pub fn gen_live_conf(
    client_spec: ClientSpec,
    config_file_path: String,
) -> Result<ChainSpec, String> {
    Ok(ChainSpec::builder(
        &get_wasm_binary().ok_or_else(|| "Development wasm not available".to_string())?,
        None,
    )
    .with_name(client_spec.name.as_str())
    .with_id(client_spec.id.as_str())
    .with_chain_type(client_spec.chain_type)
    .with_genesis_config_patch({
        let genesis_data = gen_genesis_data::generate_genesis_data::<_, _, SessionKeys, GDevSKP>(
            config_file_path.clone(),
            get_parameters,
            None,
        )
        .expect("Genesis Data must be buildable");
        genesis_data_to_gdev_genesis_conf(genesis_data)
    })
    .with_telemetry_endpoints(client_spec.telemetry_endpoints.unwrap())
    .with_properties(client_spec.properties.unwrap())
    .with_boot_nodes(client_spec.boot_nodes)
    .build())
}

/// generate local network chainspects
pub fn local_testnet_config(
    initial_authorities_len: usize,
    initial_smiths_len: usize,
    initial_identities_len: usize,
) -> Result<ChainSpec, String> {
    Ok(ChainSpec::builder(
        &get_wasm_binary().ok_or_else(|| "Development wasm not available".to_string())?,
        None,
    )
    .with_name("Ğdev Local Testnet")
    .with_id("gdev_local")
    .with_chain_type(ChainType::Local)
    .with_genesis_config_patch({
        let genesis_data =
            gen_genesis_data::generate_genesis_data_for_local_chain::<_, _, SessionKeys, GDevSKP>(
                // Initial authorities len
                initial_authorities_len,
                // Initial smiths len,
                initial_smiths_len,
                // Initial identities len
                initial_identities_len,
                EXISTENTIAL_DEPOSIT,
                get_local_chain_parameters(),
                // Sudo account
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                get_parameters,
            )
            .expect("Genesis Data must be buildable");
        genesis_data_to_gdev_genesis_conf(genesis_data)
    })
    .with_properties(
        serde_json::json!({
            "tokenDecimals": TOKEN_DECIMALS,
            "tokenSymbol": TOKEN_SYMBOL,
        })
        .as_object()
        .expect("must be a map")
        .clone(),
    )
    .build())
}

/// custom genesis
fn genesis_data_to_gdev_genesis_conf(
    genesis_data: super::gen_genesis_data::GenesisData<GenesisParameters, SessionKeys>,
) -> serde_json::Value {
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
        common_parameters: _,
        session_keys_map,
        initial_smiths,
        sudo_key,
        technical_committee_members,
        ud,
    } = genesis_data;

    serde_json::json!({
        "account": {
            "accounts": accounts,
            "treasuryBalance": treasury_balance,
        },
        "parameters": {
            "parameters": parameters.expect("mandatory for GDev"),
        },
        "authorityMembers": {
            "initialAuthorities": initial_authorities,
        },
        "balances": {
            "totalIssuance": initial_monetary_mass,
        },
        "babe": {
            "epochConfig": Some(BABE_GENESIS_EPOCH_CONFIG),
        },
        "session": {
            "keys": session_keys_map
                .into_iter()
                .map(|(account_id, session_keys)| (account_id.clone(), account_id, session_keys))
                .collect::<Vec<_>>(),
        },
        "sudo": { "key": sudo_key },
        "technicalCommittee": {
            "members": technical_committee_members,
        },
        "quota": {
            "identities": identities.iter().map(|i| i.idty_index).collect::<Vec<_>>(),
        },
        "identity": {
            "identities": identities
                .into_iter()
                .map(
                    |GenesisIdentity {
                         idty_index,
                         name,
                         owner_key,
                         status,
                         expires_on,
                         revokes_on,
                     }| GenesisIdty {
                        index: idty_index,
                        name: common_runtime::IdtyName::from(name.as_str()),
                        value: common_runtime::IdtyValue {
                            data: IdtyData::new(),
                            next_creatable_identity_on: 0,
                            old_owner_key: None,
                            owner_key,
                            next_scheduled: match status {
                                IdtyStatus::Unconfirmed | IdtyStatus::Unvalidated => {
                                    panic!("Unconfirmed or Unvalidated identity in genesis")
                                }
                                IdtyStatus::Member => expires_on.expect("must have expires_on set"),
                                IdtyStatus::Revoked => 0,
                                IdtyStatus::NotMember => {
                                    revokes_on.expect("must have revokes_on set")
                                }
                            },
                            status,
                        },
                    },
                )
                .collect::<Vec<GenesisIdty<gdev_runtime::Runtime>>>(),
        },
        "certification": {
            "applyCertPeriodAtGenesis": false,
            "certsByReceiver": certs_by_receiver,
        },
        "membership": { "memberships": memberships },
        "smithMembers": { "initialSmiths": initial_smiths},
        "universalDividend": {
            "firstReeval": first_ud_reeval,
            "firstUd": first_ud,
            "initialMonetaryMass": initial_monetary_mass,
            "ud": ud,
        },
    })
}

fn get_local_chain_parameters() -> Option<GenesisParameters> {
    let babe_epoch_duration = get_env("DUNITER_BABE_EPOCH_DURATION", 30) as u64;
    let cert_validity_period = get_env("DUNITER_CERT_VALIDITY_PERIOD", 1_000);
    let membership_period = get_env("DUNITER_MEMBERSHIP_PERIOD", 1_000);
    let membership_renewal_period = get_env("DUNITER_MEMBERSHIP_RENEWAL_PERIOD", 1_000);
    let ud_creation_period = get_env("DUNITER_UD_CREATION_PERIOD", 60_000);
    let ud_reeval_period = get_env("DUNITER_UD_REEEVAL_PERIOD", 1_200_000);
    Some(GenesisParameters {
        babe_epoch_duration,
        cert_period: 15,
        cert_max_by_issuer: 10,
        cert_min_received_cert_to_issue_cert: 2,
        cert_validity_period,
        idty_confirm_period: 40,
        idty_creation_period: 50,
        membership_period,
        membership_renewal_period,
        ud_creation_period,
        ud_reeval_period,
        smith_cert_max_by_issuer: 8,
        smith_inactivity_max_duration: 48,
        smith_wot_min_cert_for_membership: 2,
        wot_first_cert_issuable_on: 20,
        wot_min_cert_for_create_idty_right: 2,
        wot_min_cert_for_membership: 2,
    })
}

/// get environment variable
fn get_env<T: std::str::FromStr>(env_var_name: &'static str, default_value: T) -> T {
    std::env::var(env_var_name)
        .map_or(Ok(default_value), |s| s.parse())
        .unwrap_or_else(|_| panic!("{} must be a {}", env_var_name, std::any::type_name::<T>()))
}

/// Get the WASM bytes either from filesytem (`WASM_FILE` env variable giving the path to the wasm blob)
/// or else get the one compiled from source code.
/// Goal: allow to provide the WASM built with srtool, which is reproductible.
fn get_wasm_binary() -> Option<Vec<u8>> {
    let wasm_bytes_from_file = if let Ok(file_path) = env::var("WASM_FILE") {
        Some(fs::read(file_path).unwrap_or_else(|e| panic!("Could not read wasm file: {}", e)))
    } else {
        None
    };
    wasm_bytes_from_file.or_else(|| WASM_BINARY.map(|bytes| bytes.to_vec()))
}
