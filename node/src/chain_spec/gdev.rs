// Copyright 2021 Axiom-Team
//
// This file is part of Substrate-Libre-Currency.
//
// Substrate-Libre-Currency is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, version 3 of the License.
//
// Substrate-Libre-Currency is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with Substrate-Libre-Currency. If not, see <https://www.gnu.org/licenses/>.

use super::*;
use common_runtime::constants::*;
use common_runtime::*;
use gdev_runtime::{
    opaque::SessionKeys, AccountId, AuthorityMembersConfig, BabeConfig, BalancesConfig, CertConfig,
    GenesisConfig, IdentityConfig, ImOnlineId, MembershipConfig, ParametersConfig, SessionConfig,
    SmithsCertConfig, SmithsMembershipConfig, SudoConfig, SystemConfig, UdAccountsStorageConfig,
    UniversalDividendConfig, WASM_BINARY,
};
use sc_service::ChainType;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::sr25519;
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_membership::MembershipData;
use std::collections::BTreeMap;

pub type AuthorityKeys = (
    AccountId,
    BabeId,
    GrandpaId,
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
        get_from_seed::<BabeId>(s),
        get_from_seed::<GrandpaId>(s),
        get_from_seed::<ImOnlineId>(s),
        get_from_seed::<AuthorityDiscoveryId>(s),
    )
}

pub fn development_chain_spec() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Development",
        // ID
        "gdev",
        ChainType::Development,
        move || {
            gen_genesis_conf(
                wasm_binary,
                // Initial authorities len
                1,
                // Initial smiths members len
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
                None,
                // Protocol ID
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
        Some(super::gen_genesis_data::ParamsAppliedAtGenesis {
            genesis_certs_expire_on: 100_000,
            genesis_smith_certs_expire_on: 100_000,
            genesis_memberships_expire_on: 100_000,
            genesis_memberships_renewable_on: 50,
            genesis_smith_memberships_expire_on: 100_000,
            genesis_smith_memberships_renewable_on: 50,
        }),
    )
}

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
        "gdev_local_testnet",
        ChainType::Local,
        move || {
            gen_genesis_conf(
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

fn gen_genesis_conf(
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

    let babe_epoch_duration = get_env_u32("DUNITER_BABE_EPOCH_DURATION", 600) as u64;
    let cert_validity_period = get_env_u32("DUNITER_CERT_VALIDITY_PERIOD", 1_000);
    let membership_period = get_env_u32("DUNITER_MEMBERSHIP_PERIOD", 1_000);
    let membership_renewable_period = get_env_u32("DUNITER_MEMBERSHIP_RENEWABLE_PERIOD", 50);
    let smith_cert_validity_period = get_env_u32("DUNITER_SMITH_CERT_VALIDITY_PERIOD", 1_000);
    let smith_membership_renewable_period =
        get_env_u32("DUNITER_SMITH_MEMBERSHIP_RENEWABLE_PERIOD", 20);
    let smith_membership_period = get_env_u32("DUNITER_SMITH_MEMBERSHIP_PERIOD", 1_000);

    let initial_smiths = (0..initial_smiths_len)
        .map(|i| get_authority_keys_from_seed(NAMES[i]))
        .collect::<Vec<AuthorityKeys>>();
    let initial_identities = (0..initial_smiths_len)
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
        parameters: ParametersConfig {
            parameters: GenesisParameters {
                babe_epoch_duration,
                cert_period: 15,
                cert_max_by_issuer: 10,
                cert_min_received_cert_to_issue_cert: 2,
                cert_renewable_period: 50,
                cert_validity_period,
                idty_confirm_period: 40,
                idty_creation_period: 50,
                idty_max_disabled_period: 1_000,
                membership_period,
                membership_renewable_period,
                pending_membership_period: 500,
                ud_creation_period: 10,
                ud_first_reeval: 100,
                ud_reeval_period: 20,
                ud_reeval_period_in_blocks: 200,
                smith_cert_period: 15,
                smith_cert_max_by_issuer: 8,
                smith_cert_min_received_cert_to_issue_cert: 2,
                smith_cert_renewable_period: 50,
                smith_cert_validity_period: 1_000,
                smith_membership_period: 1_000,
                smith_membership_renewable_period: 50,
                smith_pending_membership_period: 500,
                smiths_wot_first_cert_issuable_on: 20,
                smiths_wot_min_cert_for_membership: 2,
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
        balances: BalancesConfig {
            balances: Default::default(),
        },
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
        identity: IdentityConfig {
            identities: initial_identities
                .iter()
                .enumerate()
                .map(|(i, (name, owner_key))| GenesisIdty {
                    index: i as u32 + 1,
                    owner_key: owner_key.clone(),
                    name: name.clone(),
                    value: IdtyValue {
                        next_creatable_identity_on: Default::default(),
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
                            expire_on: membership_period,
                            renewable_on: membership_renewable_period,
                        },
                    )
                })
                .collect(),
        },
        cert: CertConfig {
            apply_cert_period_at_genesis: false,
            certs_by_issuer: clique_wot(initial_identities.len(), cert_validity_period),
        },
        smiths_membership: SmithsMembershipConfig {
            memberships: (1..=initial_smiths_len)
                .map(|i| {
                    (
                        i as u32,
                        MembershipData {
                            expire_on: smith_membership_period,
                            renewable_on: smith_membership_renewable_period,
                        },
                    )
                })
                .collect(),
        },
        smiths_cert: SmithsCertConfig {
            apply_cert_period_at_genesis: false,
            certs_by_issuer: clique_wot(initial_smiths_len, smith_cert_validity_period),
        },
        ud_accounts_storage: UdAccountsStorageConfig {
            ud_accounts: initial_identities
                .values()
                .cloned()
                .enumerate()
                .map(|(i, account)| (account, (i + 1) as u32))
                .collect(),
        },
        universal_dividend: UniversalDividendConfig {
            first_ud: 1_000,
            initial_monetary_mass: 0,
        },
    }
}

fn get_env_u32(env_var_name: &'static str, default_value: u32) -> u32 {
    std::env::var(env_var_name)
        .map_or(Ok(default_value), |s| s.parse())
        .unwrap_or_else(|_| panic!("{} must be a number", env_var_name))
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

fn genesis_data_to_gdev_genesis_conf(
    genesis_data: super::gen_genesis_data::GenesisData<GenesisParameters, SessionKeys>,
    wasm_binary: &[u8],
) -> gdev_runtime::GenesisConfig {
    let super::gen_genesis_data::GenesisData {
        balances,
        certs_by_issuer,
        first_ud,
        identities,
        initial_authorities,
        initial_monetary_mass,
        memberships,
        parameters,
        session_keys_map,
        smiths_certs_by_issuer,
        smiths_memberships,
        sudo_key,
        ud_accounts,
    } = genesis_data;

    gdev_runtime::GenesisConfig {
        system: SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
        },
        parameters: ParametersConfig { parameters },
        authority_discovery: Default::default(),
        authority_members: AuthorityMembersConfig {
            initial_authorities,
        },
        balances: BalancesConfig { balances },
        babe: BabeConfig {
            authorities: Vec::with_capacity(0),
            epoch_config: Some(common_runtime::constants::BABE_GENESIS_EPOCH_CONFIG),
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
        identity: IdentityConfig {
            identities: identities
                .into_iter()
                .enumerate()
                .map(|(i, (name, pubkey))| common_runtime::GenesisIdty {
                    index: i as u32 + 1,
                    owner_key: pubkey,
                    name: common_runtime::IdtyName::from(name.as_str()),
                    value: common_runtime::IdtyValue {
                        next_creatable_identity_on: 0,
                        removable_on: 0,
                        status: IdtyStatus::Validated,
                    },
                })
                .collect(),
        },
        cert: CertConfig {
            apply_cert_period_at_genesis: true,
            certs_by_issuer,
        },
        membership: MembershipConfig { memberships },
        smiths_cert: SmithsCertConfig {
            apply_cert_period_at_genesis: true,
            certs_by_issuer: smiths_certs_by_issuer,
        },
        smiths_membership: SmithsMembershipConfig {
            memberships: smiths_memberships,
        },
        ud_accounts_storage: UdAccountsStorageConfig { ud_accounts },
        universal_dividend: UniversalDividendConfig {
            first_ud,
            initial_monetary_mass,
        },
    }
}
