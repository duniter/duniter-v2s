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
use g1_runtime::{
    opaque::SessionKeys, pallet_universal_dividend, parameters, Runtime, WASM_BINARY,
};
use sc_service::ChainType;
use serde::Deserialize;
use sp_core::{sr25519, Get};
use std::{env, fs};

pub type ChainSpec = sc_service::GenericChainSpec;

#[derive(Default, Clone, Deserialize)]
// No parameters for G1 (unlike GDev)
struct GenesisParameters {}

const TOKEN_DECIMALS: usize = 2;
const TOKEN_SYMBOL: &str = "Ğ";
static EXISTENTIAL_DEPOSIT: u64 = parameters::ExistentialDeposit::get();

struct G1SKP;
impl SessionKeysProvider<SessionKeys> for G1SKP {
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

fn get_parameters(_parameters_from_file: &Option<GenesisParameters>) -> CommonParameters {
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
        wot_first_issuable_on: parameters::WotFirstCertIssuableOn::get(),
        wot_min_cert_for_membership: parameters::WotMinCertForMembership::get(),
        wot_min_cert_for_create_idty_right: parameters::WotMinCertForCreateIdtyRight::get(),
        identity_confirm_period: parameters::ConfirmPeriod::get(),
        identity_change_owner_key_period: parameters::ChangeOwnerKeyPeriod::get(),
        identity_idty_creation_period: parameters::IdtyCreationPeriod::get(),
        identity_autorevocation_period: parameters::AutorevocationPeriod::get(),
        membership_membership_period: parameters::MembershipPeriod::get(),
        membership_membership_renewal_period: parameters::MembershipRenewalPeriod::get(),
        cert_max_by_issuer: parameters::MaxByIssuer::get(),
        cert_min_received_cert_to_be_able_to_issue_cert:
            parameters::MinReceivedCertToBeAbleToIssueCert::get(),
        cert_validity_period: parameters::ValidityPeriod::get(),
        distance_min_accessible_referees: parameters::MinAccessibleReferees::get(),
        distance_max_depth: parameters::MaxRefereeDistance::get(),
        smith_sub_wot_min_cert_for_membership: parameters::SmithWotMinCertForMembership::get(),
        smith_inactivity_max_duration: parameters::SmithInactivityMaxDuration::get(),
        smith_cert_max_by_issuer: parameters::SmithMaxByIssuer::get(),
        cert_cert_period: parameters::CertPeriod::get(),
        treasury_spend_period: <Runtime as pallet_treasury::Config>::SpendPeriod::get(),
    }
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
    .with_name("Ğ1 Local Testnet")
    .with_id("g1_local")
    .with_chain_type(ChainType::Local)
    .with_genesis_config_patch({
        let genesis_data =
            gen_genesis_data::generate_genesis_data_for_local_chain::<_, _, SessionKeys, G1SKP>(
                // Initial authorities len
                initial_authorities_len,
                // Initial smiths len,
                initial_smiths_len,
                // Initial identities len
                initial_identities_len,
                EXISTENTIAL_DEPOSIT,
                None,
                // Sudo account
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                get_parameters,
            )
            .expect("Genesis Data must be buildable");
        genesis_data_to_g1_genesis_conf(genesis_data)
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
fn genesis_data_to_g1_genesis_conf(
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
        parameters: _,
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
                .collect::<Vec<GenesisIdty<g1_runtime::Runtime>>>(),
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
