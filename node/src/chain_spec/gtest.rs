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
use common_runtime::entities::IdtyName;
use gtest_runtime::{
    opaque::SessionKeys, AccountId, AuthorityMembersConfig, BabeConfig, BalancesConfig, CertConfig,
    GenesisConfig, IdentityConfig, IdtyValue, ImOnlineId, MembershipConfig, SessionConfig,
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

const TOKEN_DECIMALS: usize = 2;
const TOKEN_SYMBOL: &str = "ĞT";
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
    let wasm_binary = WASM_BINARY.ok_or_else(|| "wasm not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Ğtest Development",
        // ID
        "gtest_dev",
        ChainType::Development,
        move || {
            gen_genesis_conf(
                wasm_binary,
                // Initial authorities
                1,
                // Inital identities
                3,
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

pub fn local_testnet_config(
    initial_authorities_len: usize,
    initial_identities_len: usize,
) -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "wasm not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Ğtest Local Testnet",
        // ID
        "gtest_local_testnet",
        ChainType::Local,
        move || {
            gen_genesis_conf(
                wasm_binary,
                // Initial authorities len
                initial_authorities_len,
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
    initial_identities_len: usize,
    root_key: AccountId,
    _enable_println: bool,
) -> GenesisConfig {
    let initial_authorities = (0..initial_authorities_len)
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
        authority_discovery: Default::default(),
        authority_members: AuthorityMembersConfig {
            initial_authorities: initial_authorities
                .iter()
                .enumerate()
                .map(|(i, keys)| (i as u32 + 1, (keys.0.clone(), true)))
                .collect(),
        },
        balances: BalancesConfig {
            balances: Vec::with_capacity(0),
        },
        babe: BabeConfig {
            authorities: Vec::with_capacity(0),
            epoch_config: Some(BABE_GENESIS_EPOCH_CONFIG),
        },
        grandpa: Default::default(),
        im_online: Default::default(),
        session: SessionConfig {
            keys: initial_authorities
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
                .map(|(name, account)| IdtyValue {
                    data: Default::default(),
                    name: name.clone(),
                    next_creatable_identity_on: Default::default(),
                    owner_key: account.clone(),
                    removable_on: 0,
                    status: gtest_runtime::IdtyStatus::Validated,
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
                            renewable_on: gtest_runtime::RenewablePeriod::get(),
                        },
                    )
                })
                .collect(),
        },
        cert: CertConfig {
            apply_cert_period_at_genesis: false,
            certs_by_issuer: clique_wot(
                initial_identities.len(),
                gtest_runtime::parameters::ValidityPeriod::get(),
            ),
        },
        smiths_membership: SmithsMembershipConfig {
            memberships: (1..=initial_authorities_len)
                .map(|i| {
                    (
                        i as u32,
                        MembershipData {
                            expire_on: gtest_runtime::SmithMembershipPeriod::get(),
                            renewable_on: gtest_runtime::SmithRenewablePeriod::get(),
                        },
                    )
                })
                .collect(),
        },
        smiths_cert: SmithsCertConfig {
            apply_cert_period_at_genesis: false,
            certs_by_issuer: clique_wot(
                initial_authorities_len,
                gtest_runtime::parameters::SmithValidityPeriod::get(),
            ),
        },
        ud_accounts_storage: UdAccountsStorageConfig {
            ud_accounts: initial_identities.values().cloned().collect(),
        },
        universal_dividend: UniversalDividendConfig {
            first_ud: 1_000,
            initial_monetary_mass: 0,
        },
    }
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
