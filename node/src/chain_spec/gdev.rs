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
use crate::chain_spec::gen_genesis_data::{AuthorityKeys, CommonParameters, SessionKeysProvider};
use common_runtime::constants::*;
use common_runtime::entities::IdtyData;
use common_runtime::*;
use gdev_runtime::{
    opaque::SessionKeys, parameters, AccountConfig, AuthorityMembersConfig, BabeConfig, CertConfig,
    GenesisConfig, IdentityConfig, MembershipConfig, ParametersConfig, SessionConfig,
    SmithCertConfig, SmithMembershipConfig, SudoConfig, SystemConfig, TechnicalCommitteeConfig,
    UniversalDividendConfig, WASM_BINARY,
};
use sc_service::ChainType;
use sp_core::sr25519;
use sp_runtime::Perbill;

pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

type GenesisParameters = gdev_runtime::GenesisParameters<u32, u32, u64>;

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
    let parameters_from_file =
        parameters_from_file.expect("parameters must be defined in file for GDev");
    CommonParameters {
        currency_name: TOKEN_SYMBOL.to_string(),
        decimals: TOKEN_DECIMALS,
        existential_deposit: EXISTENTIAL_DEPOSIT,
        membership_period: parameters_from_file.membership_period,
        cert_period: parameters_from_file.cert_period,
        smith_membership_period: parameters_from_file.smith_membership_period,
        smith_certs_validity_period: parameters_from_file.smith_cert_validity_period,
        min_cert: parameters_from_file.wot_min_cert_for_membership,
        smith_min_cert: parameters_from_file.smith_wot_min_cert_for_membership,
        cert_max_by_issuer: parameters_from_file.cert_max_by_issuer,
        cert_validity_period: parameters_from_file.cert_validity_period,
        c2: parameters::SquareMoneyGrowthRate::get(),
        ud_creation_period: parameters_from_file.ud_creation_period,
        distance_min_accessible_referees: Perbill::from_percent(80),
        max_depth: 5, // TODO: generalize
        ud_reeval_period: parameters_from_file.ud_reeval_period,
    }
}

/// generate development chainspec with Alice validator
pub fn gdev_development_chain_spec(json_file_path: String) -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;
    Ok(ChainSpec::from_genesis(
        // Name
        "Development",
        // ID
        "gdev",
        // chain type
        sc_service::ChainType::Development,
        // constructor
        move || {
            let genesis_data =
                gen_genesis_data::generate_genesis_data::<_, _, SessionKeys, GDevSKP>(
                    json_file_path.clone(),
                    get_parameters,
                    Some("Alice".to_owned()),
                )
                .expect("Genesis Data must be buildable");
            genesis_data_to_gdev_genesis_conf(genesis_data, wasm_binary)
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

/// generate chainspecs used for benchmarks
pub fn benchmark_chain_spec() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;
    // Same as local chain
    Ok(ChainSpec::from_genesis(
        // Name
        "Development",
        // ID
        "gdev-benchmark",
        ChainType::Development,
        // constructor
        move || {
            let genesis_data = gen_genesis_data::generate_genesis_data_for_local_chain::<
                _,
                _,
                SessionKeys,
                GDevSKP,
            >(
                // Initial authorities len
                1,
                // Initial smiths members len
                3,
                // Inital identities len
                4,
                EXISTENTIAL_DEPOSIT,
                get_local_chain_parameters(),
                // Sudo account
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                true,
            )
            .expect("Genesis Data must be buildable");
            genesis_data_to_gdev_genesis_conf(genesis_data, wasm_binary)
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

/// generate live network chainspecs
pub fn gen_live_conf(json_file_path: String) -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "wasm not available".to_string())?;
    Ok(ChainSpec::from_genesis(
        // Name
        "Ğdev",
        // ID
        "gdev",
        sc_service::ChainType::Live,
        move || {
            let genesis_data =
                gen_genesis_data::generate_genesis_data::<_, _, SessionKeys, GDevSKP>(
                    json_file_path.clone(),
                    get_parameters,
                    None,
                )
                .expect("Genesis Data must be buildable");
            genesis_data_to_gdev_genesis_conf(genesis_data, wasm_binary)
        },
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
    ))
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
        // constructor
        move || {
            let genesis_data = gen_genesis_data::generate_genesis_data_for_local_chain::<
                _,
                _,
                SessionKeys,
                GDevSKP,
            >(
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
                true,
            )
            .expect("Genesis Data must be buildable");
            genesis_data_to_gdev_genesis_conf(genesis_data, wasm_binary)
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

/// custom genesis
fn genesis_data_to_gdev_genesis_conf(
    genesis_data: super::gen_genesis_data::GenesisData<GenesisParameters, SessionKeys>,
    wasm_binary: &[u8],
) -> gdev_runtime::GenesisConfig {
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
        smith_certs_by_receiver,
        smith_memberships,
        sudo_key,
        technical_committee_members,
        ud,
    } = genesis_data;

    gdev_runtime::GenesisConfig {
        system: SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
        },
        account: AccountConfig {
            accounts,
            treasury_balance,
        },
        parameters: ParametersConfig {
            parameters: parameters.expect("mandatory for GDev"),
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
                .map(|(i, (name, owner_key, old_owner_key))| GenesisIdty {
                    index: i as u32 + 1,
                    name: common_runtime::IdtyName::from(name.as_str()),
                    value: common_runtime::IdtyValue {
                        data: IdtyData::new(),
                        next_creatable_identity_on: 0,
                        old_owner_key: old_owner_key.clone().map(|address| (address, 0)),
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
            ud,
            #[cfg(test)]
            initial_members: vec![],
        },
        treasury: Default::default(),
    }
}

fn get_local_chain_parameters() -> GenesisParameters {
    let babe_epoch_duration = get_env("DUNITER_BABE_EPOCH_DURATION", 30) as u64;
    let cert_validity_period = get_env("DUNITER_CERT_VALIDITY_PERIOD", 1_000);
    let membership_period = get_env("DUNITER_MEMBERSHIP_PERIOD", 1_000);
    let smith_cert_validity_period = get_env("DUNITER_SMITH_CERT_VALIDITY_PERIOD", 1_000);
    let smith_membership_period = get_env("DUNITER_SMITH_MEMBERSHIP_PERIOD", 1_000);
    let ud_creation_period = get_env("DUNITER_UD_CREATION_PERIOD", 60_000);
    let ud_reeval_period = get_env("DUNITER_UD_REEEVAL_PERIOD", 1_200_000);
    GenesisParameters {
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
    }
}

/// get environment variable
fn get_env<T: std::str::FromStr>(env_var_name: &'static str, default_value: T) -> T {
    std::env::var(env_var_name)
        .map_or(Ok(default_value), |s| s.parse())
        .unwrap_or_else(|_| panic!("{} must be a {}", env_var_name, std::any::type_name::<T>()))
}
