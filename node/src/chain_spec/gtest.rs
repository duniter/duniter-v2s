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
use crate::chain_spec::gen_genesis_data::{CommonParameters, GenesisIdentity, SessionKeysProvider};
use common_runtime::constants::*;
use common_runtime::entities::IdtyData;
use common_runtime::*;
use frame_benchmarking::frame_support::traits::Get;
use gtest_runtime::SmithMembersConfig;
use gtest_runtime::{
    opaque::SessionKeys, pallet_universal_dividend, parameters, AccountConfig, AccountId,
    AuthorityMembersConfig, BabeConfig, BalancesConfig, CertConfig, GenesisConfig, IdentityConfig,
    ImOnlineId, MembershipConfig, Perbill, QuotaConfig, Runtime, SessionConfig, SudoConfig,
    SystemConfig, TechnicalCommitteeConfig, UniversalDividendConfig, WASM_BINARY,
};
use jsonrpsee::core::JsonValue;
use sc_consensus_grandpa::AuthorityId as GrandpaId;
use sc_network::config::MultiaddrWithPeerId;
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use serde::Deserialize;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use std::{env, fs};

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
        babe_epoch_duration: parameters::EpochDuration::get(),
        babe_expected_block_time: parameters::ExpectedBlockTime::get(),
        babe_max_authorities: parameters::MaxAuthorities::get(),
        timestamp_minimum_period: parameters::MinimumPeriod::get(),
        balances_existential_deposit: parameters::ExistentialDeposit::get(),
        authority_members_max_authorities: parameters::MaxAuthorities::get(),
        grandpa_max_authorities: parameters::MaxAuthorities::get(),
        universal_dividend_max_past_reevals:
            <Runtime as pallet_universal_dividend::Config>::MaxPastReeval::get(),
        universal_dividend_square_money_growth_rate: parameters::SquareMoneyGrowthRate::get(),
        universal_dividend_ud_creation_period: parameters::UdCreationPeriod::get() as u64,
        universal_dividend_ud_reeval_period: parameters::UdReevalPeriod::get() as u64,
        universal_dividend_units_per_ud:
            <Runtime as pallet_universal_dividend::Config>::UnitsPerUd::get(),
        wot_first_issuable_on: parameters::WotFirstCertIssuableOn::get(),
        wot_min_cert_for_membership: parameters::WotMinCertForMembership::get(),
        wot_min_cert_for_create_idty_right: parameters::WotMinCertForCreateIdtyRight::get(),
        identity_confirm_period: parameters::ConfirmPeriod::get(),
        identity_change_owner_key_period: parameters::ChangeOwnerKeyPeriod::get(),
        identity_idty_creation_period: parameters::IdtyCreationPeriod::get(),
        membership_membership_period: parameters::MembershipPeriod::get(),
        membership_pending_membership_period: parameters::PendingMembershipPeriod::get(),
        cert_max_by_issuer: parameters::MaxByIssuer::get(),
        cert_min_received_cert_to_be_able_to_issue_cert:
            parameters::MinReceivedCertToBeAbleToIssueCert::get(),
        cert_validity_period: parameters::ValidityPeriod::get(),
        distance_min_accessible_referees: Perbill::from_percent(80), // TODO: generalize
        distance_max_depth: 5,                                       // TODO: generalize
        smith_sub_wot_min_cert_for_membership: parameters::SmithWotMinCertForMembership::get(),
        smith_inactivity_max_duration: parameters::SmithInactivityMaxDuration::get(),
        smith_cert_max_by_issuer: parameters::SmithMaxByIssuer::get(),
        cert_cert_period: parameters::CertPeriod::get(),
        treasury_spend_period: <Runtime as pallet_treasury::Config>::SpendPeriod::get(),
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
pub fn development_chainspecs(config_file_path: String) -> Result<ChainSpec, String> {
    let wasm_binary = get_wasm_binary().ok_or_else(|| "wasm not available".to_string())?;
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
                    config_file_path.clone(),
                    get_parameters,
                    Some("Alice".to_owned()),
                )
                .expect("Genesis Data must be buildable");
            genesis_data_to_gtest_genesis_conf(genesis_data, wasm_binary.to_vec())
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
    config_file_path: String,
) -> Result<ChainSpec, String> {
    let wasm_binary = get_wasm_binary().ok_or_else(|| "wasm not available".to_string())?;
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
                    config_file_path.clone(),
                    get_parameters,
                    None,
                )
                .expect("Genesis Data must be buildable");
            genesis_data_to_gtest_genesis_conf(genesis_data, wasm_binary.to_vec())
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
    wasm_binary: Vec<u8>,
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
        parameters: _,
        common_parameters: _,
        session_keys_map,
        initial_smiths,
        sudo_key,
        technical_committee_members,
        ud,
    } = genesis_data;

    gtest_runtime::GenesisConfig {
        system: SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary,
        },
        account: AccountConfig {
            accounts,
            treasury_balance,
        },
        authority_discovery: Default::default(),
        authority_members: AuthorityMembersConfig {
            initial_authorities,
        },
        // Necessary to initialize TotalIssuence
        balances: BalancesConfig {
            total_issuance: initial_monetary_mass,
        },
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
        quota: QuotaConfig {
            identities: identities.iter().map(|i| i.idty_index).collect(),
        },
        identity: IdentityConfig {
            identities: identities
                .into_iter()
                .map(
                    |GenesisIdentity {
                         idty_index,
                         name,
                         owner_key,
                         old_owner_key,
                         active,
                     }| GenesisIdty {
                        index: idty_index,
                        name: common_runtime::IdtyName::from(name.as_str()),
                        value: common_runtime::IdtyValue {
                            data: IdtyData::new(),
                            next_creatable_identity_on: 0,
                            old_owner_key: old_owner_key.clone().map(|address| (address, 0)),
                            owner_key,
                            next_scheduled: if active { 0 } else { 2 },
                            status: IdtyStatus::Member,
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
        smith_members: SmithMembersConfig { initial_smiths },
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
