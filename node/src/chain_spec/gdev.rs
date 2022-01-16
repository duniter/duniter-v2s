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
use common_runtime::entities::IdtyName;
use gdev_runtime::{
    AccountId, BalancesConfig, GenesisConfig, GrandpaConfig, IdentityConfig, IdtyRight, IdtyValue,
    StrongCertConfig, SudoConfig, SystemConfig, UdAccountsStorageConfig, UniversalDividendConfig,
    WASM_BINARY,
};
use maplit::btreemap;
use sc_service::ChainType;
use sp_core::sr25519;
use sp_finality_grandpa::AuthorityId as GrandpaId;
use std::collections::BTreeMap;

pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

const TOKEN_DECIMALS: usize = 2;
const TOKEN_SYMBOL: &str = "ĞD";
// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Generate an authority keys.
pub fn get_authority_keys_from_seed(s: &str) -> GrandpaId {
    get_from_seed::<GrandpaId>(s)
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
            devnet_genesis(
                wasm_binary,
                // Initial authorities
                vec![get_authority_keys_from_seed("Alice")],
                // Inital identities
                btreemap![
                    IdtyName::from("Alice") => get_account_id_from_seed::<sr25519::Public>("Alice"),
                    IdtyName::from("Bob") => get_account_id_from_seed::<sr25519::Public>("Bob"),
                    IdtyName::from("Charlie") => get_account_id_from_seed::<sr25519::Public>("Charlie"),
                ],
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

fn devnet_genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<GrandpaId>,
    initial_identities: BTreeMap<IdtyName, AccountId>,
    root_key: AccountId,
    _enable_println: bool,
) -> gdev_runtime::GenesisConfig {
    gdev_runtime::GenesisConfig {
        system: SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
        },
        balances: BalancesConfig {
            balances: Default::default(),
        },
        grandpa: GrandpaConfig {
            authorities: initial_authorities.iter().map(|x| (x.clone(), 1)).collect(),
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
                    expire_on: gdev_runtime::MaxInactivityPeriod::get(),
                    owner_key: account.clone(),
                    name: name.clone(),
                    next_creatable_identity_on: Default::default(),
                    removable_on: 0,
                    renewable_on: gdev_runtime::StrongCertRenewablePeriod::get(),
                    rights: vec![
                        (IdtyRight::CreateIdty, None),
                        (IdtyRight::StrongCert, None),
                        (IdtyRight::Ud, None),
                    ],
                    status: gdev_runtime::IdtyStatus::Validated,
                })
                .collect(),
        },
        strong_cert: StrongCertConfig {
            certs_by_issuer: clique_wot(
                initial_identities.len(),
                gdev_runtime::parameters::ValidityPeriod::get(),
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
