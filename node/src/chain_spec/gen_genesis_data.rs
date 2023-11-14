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

use crate::chain_spec::{
    clique_wot, get_account_id_from_seed, get_from_seed, AccountPublic, NAMES,
};
use common_runtime::constants::DAYS;
use common_runtime::*;
use log::{error, warn};
use num_format::{Locale, ToFormattedString};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::crypto::AccountId32;
use sp_core::{blake2_256, ed25519, sr25519, Decode, Encode, H256};
use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_runtime::{MultiSignature, Perbill};
use std::collections::{BTreeMap, HashMap};
use std::fmt::{Display, Formatter};
use std::fs;
use std::ops::{Add, Sub};

static G1_DUNITER_V1_EXISTENTIAL_DEPOSIT: u64 = 100;
static G1_DUNITER_V1_DECIMALS: usize = 2;
static G1_DUNITER_V1_DT: u64 = 86400;
static G1_DUNITER_V1_SIGPERIOD: u32 = 432000;
static G1_DUNITER_V1_SIGSTOCK: u32 = 100;
static G1_DUNITER_V1_SIGVALIDITY: u32 = 63115200;
static G1_DUNITER_V1_SIGQTY: u32 = 5;
static G1_DUNITER_V1_MSVALIDITY: u32 = 31557600;
static G1_DUNITER_V1_STEPMAX: u32 = 5;
static G1_DUNITER_V1_DTREEVAL: u64 = 15778800;
// Warning: Duniter V1 "days" are expressed in seconds, while V2 are expressed in blocks
static DUNITER_V1_DAYS: u32 = 3600 * 24;
// Not used in V2S
// static G1_DUNITER_V1_SIGWINDOW: u32 = 5259600; // no more pool
// static G1_DUNITER_V1_IDTYWINDOW: u32 = 5259600; // no more pool
// static G1_DUNITER_V1_MSWINDOW: u32 = 5259600; // no more pool
// static G1_DUNITER_V1_PERCENTROT: f32 = 0.67; // no more PoW
// static G1_DUNITER_V1_MEDIANTIMEBLOCKS: u32 = 24; // no more PoW
// static G1_DUNITER_V1_AVGGENTIME: u32 = 300; // no more PoW
// static G1_DUNITER_V1_DTDIFFEVAL: u32 = 12; // no more PoW
// static G1_DUNITER_V1_UD0: u32 = 1000; // new value
// static G1_DUNITER_V1_MSPERIOD: u32 = 5259600; // no more used
// static G1_DUNITER_V1_UDTIME0: u32 = 1488970800; // duniter v1 specific
// static G1_DUNITER_V1_UDREEVALTIME0: u32 = 1490094000; // duniter v1 specific

type MembershipData = sp_membership::MembershipData<u32>;

#[derive(Clone)]
pub struct GenesisData<Parameters: DeserializeOwned, SessionKeys: Decode> {
    pub accounts: BTreeMap<AccountId, GenesisAccountData<u64>>,
    pub treasury_balance: u64,
    pub certs_by_receiver: BTreeMap<u32, BTreeMap<u32, Option<u32>>>,
    pub first_ud: Option<u64>,
    pub first_ud_reeval: Option<u64>,
    pub identities: Vec<GenesisIdentity>,
    pub initial_authorities: BTreeMap<u32, (AccountId, bool)>,
    pub initial_monetary_mass: u64,
    pub memberships: BTreeMap<u32, MembershipData>,
    pub parameters: Option<Parameters>,
    pub common_parameters: Option<CommonParameters>,
    pub session_keys_map: BTreeMap<AccountId, SessionKeys>,
    pub smith_certs_by_receiver: BTreeMap<u32, BTreeMap<u32, Option<u32>>>,
    pub smith_memberships: BTreeMap<u32, MembershipData>,
    pub sudo_key: Option<AccountId>,
    pub technical_committee_members: Vec<AccountId>,
    pub ud: u64,
}

#[derive(Clone)]
pub struct GenesisIdentity {
    pub idty_index: u32,
    pub name: String,
    pub owner_key: AccountId,
    pub old_owner_key: Option<AccountId>,
    pub active: bool,
}

#[derive(Deserialize, Serialize)]
struct GenesisInput<Parameters> {
    first_ud: Option<u64>,
    first_ud_reeval: Option<u64>,
    #[serde(default)]
    parameters: Option<Parameters>,
    #[serde(rename = "smiths")]
    smith_identities: Option<BTreeMap<String, RawSmith>>,
    clique_smiths: Option<Vec<CliqueSmith>>,
    sudo_key: Option<AccountId>,
    treasury_funder_pubkey: Option<PubkeyV1>,
    treasury_funder_address: Option<AccountId>,
    technical_committee: Vec<String>,
    ud: u64,
}

#[derive(Deserialize, Serialize)]
pub struct GenesisIndexerExport {
    first_ud: Option<u64>,
    first_ud_reeval: Option<u64>,
    genesis_parameters: CommonParameters,
    identities: HashMap<String, IdentityV2>,
    smiths: BTreeMap<String, SmithData>,
    sudo_key: Option<AccountId>,
    technical_committee: Vec<String>,
    ud: u64,
    wallets: BTreeMap<AccountId, u64>,
    transactions_history: Option<BTreeMap<AccountId, Vec<TransactionV2>>>,
}

#[derive(Deserialize, Serialize)]
struct TransactionV1 {
    issuer: PubkeyV1,
    amount: String,
    written_time: Option<u32>,
    comment: String,
}

#[derive(Deserialize, Serialize)]
struct TransactionV2 {
    issuer: AccountId,
    amount: String,
    written_time: Option<u32>,
    comment: String,
}

#[derive(Deserialize, Serialize)]
struct GenesisMigrationData {
    initial_monetary_mass: u64,
    identities: BTreeMap<String, IdentityV1>,
    #[serde(default)]
    wallets: BTreeMap<PubkeyV1, u64>,
    transactions_history: Option<BTreeMap<PubkeyV1, Vec<TransactionV1>>>,
}

// Base58 encoded Ed25519 public key
#[derive(Clone, Deserialize, Serialize, Ord, PartialOrd, Eq, PartialEq)]
struct PubkeyV1(String);
// Timestamp
#[derive(Clone, Deserialize, Serialize)]
struct TimestampV1(u32);

impl Display for PubkeyV1 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// identities
#[derive(Clone, Deserialize, Serialize)]
struct IdentityV1 {
    /// indentity index matching the order of appearance in the Ǧ1v1 blockchain
    index: u32,
    /// Base58 public key in Ğ1v1
    owner_pubkey: Option<PubkeyV1>,
    /// Ğ1v2 address
    owner_address: Option<AccountId>,
    /// Optional Base58 public key in Ğ1v1
    old_owner_key: Option<PubkeyV1>,
    /// timestamp at which the membership is set to expire (0 for expired members)
    membership_expire_on: TimestampV1,
    /// timestamp at which the next cert can be emitted
    next_cert_issuable_on: TimestampV1, // TODO: unused?
    /// balance of the account of this identity
    balance: u64,
    /// certs received with their expiration timestamp
    certs_received: HashMap<String, TimestampV1>,
}

/// identities
#[derive(Clone, Deserialize, Serialize)]
struct IdentityV2 {
    /// indentity index matching the order of appearance in the Ǧ1v1 blockchain
    index: u32,
    /// ss58 address in gx network
    owner_key: AccountId,
    /// optional ss58 address in the Ğ1v1
    old_owner_key: Option<AccountId>,
    /// block at which the membership is set to expire (0 for expired members)
    membership_expire_on: u32,
    /// block at which the next cert can be emitted
    next_cert_issuable_on: u32,
    /// balance of the account of this identity
    balance: u64,
    /// certs received with their expiration block
    certs_received: HashMap<String, u32>,
}

#[derive(Clone, Deserialize, Serialize)]
struct RawSmith {
    name: String,
    /// optional pre-set session keys (at least for the smith bootstraping the blockchain)
    session_keys: Option<String>,
    /// optional pre-set account migration
    migration_address: Option<AccountId>,
    #[serde(default)]
    certs_received: Vec<String>,
}

#[derive(Clone, Deserialize, Serialize)]
struct SmithData {
    idty_index: u32,
    name: String,
    account: AccountId,
    /// optional pre-set session keys (at least for the smith bootstraping the blockchain)
    session_keys: Option<String>,
    #[serde(default)]
    certs_received: Vec<String>,
}

#[derive(Clone, Deserialize, Serialize)]
struct CliqueSmith {
    name: String,
    migration_address: Option<AccountId>,
    session_keys: Option<String>,
}

struct SmithWoT<SK: Decode> {
    smith_certs_by_receiver: BTreeMap<u32, BTreeMap<u32, Option<u32>>>,
    smith_memberships: BTreeMap<u32, sp_membership::MembershipData<u32>>,
    session_keys_map:
        BTreeMap<<<MultiSignature as Verify>::Signer as IdentifyAccount>::AccountId, SK>,
}

struct GenesisInfo<'a> {
    accounts: &'a BTreeMap<AccountId32, GenesisAccountData<u64>>,
    genesis_data_wallets_count: &'a usize,
    inactive_identities: &'a HashMap<u32, String>,
    identities: &'a Vec<GenesisIdentity>,
    identity_index: &'a HashMap<u32, String>,
    smith_memberships: &'a BTreeMap<u32, MembershipData>,
    counter_online_authorities: &'a u32,
    counter_cert: &'a u32,
    counter_smith_cert: &'a u32,
    technical_committee_members: &'a Vec<AccountId32>,
    common_parameters: &'a CommonParameters,
}

/// generate genesis data from a json file
/// takes DUNITER_GENESIS_CONFIG env var if present or duniter-gen-conf.json by default
// this function is targeting dev chainspecs, do not use in production network
pub fn generate_genesis_data<P, SK, SessionKeys: Encode, SKP>(
    json_file_path: String,
    get_common_parameters: fn(&Option<P>) -> CommonParameters,
    maybe_force_authority: Option<String>,
) -> Result<GenesisData<P, SK>, String>
where
    P: Default + DeserializeOwned,
    SK: Decode,
    SKP: SessionKeysProvider<SessionKeys>,
{
    let genesis_timestamp: u64 = get_genesis_timestamp()?;

    // Per network input
    let GenesisInput {
        sudo_key,
        treasury_funder_pubkey,
        treasury_funder_address,
        first_ud,
        first_ud_reeval,
        parameters,
        smith_identities,
        clique_smiths,
        technical_committee,
        ud,
    } = get_genesis_input::<P>(
        std::env::var("DUNITER_GENESIS_CONFIG").unwrap_or_else(|_| json_file_path.to_owned()),
    )?;

    // Per network parameters
    let common_parameters = get_common_parameters(&parameters);

    // Per network smiths (without link to an account yet — identified by their pseudonym)
    let mut smiths = build_smiths_wot(&clique_smiths, smith_identities)?;

    // G1 data migration (common to all networks)
    let mut genesis_data = get_genesis_migration_data()?;
    check_parameters_consistency(&genesis_data.wallets, &first_ud, &first_ud_reeval, &ud)?;
    check_genesis_data_and_filter_expired_certs_since_export(
        &mut genesis_data,
        genesis_timestamp,
        &common_parameters,
    );
    let mut identities_v2: HashMap<String, IdentityV2> =
        genesis_data_to_identities_v2(genesis_data.identities, genesis_timestamp, &smiths);
    check_identities_v2(&identities_v2, &common_parameters);

    // MONEY AND WOT //
    // declare variables for building genesis
    // -------------------------------------
    // track if fatal error occured, but let processing continue
    let mut fatal = false;
    // initial Treasury balance
    let mut treasury_balance = 0;
    // track identity index
    let mut identity_index = HashMap::new();
    // track inactive identities
    let mut inactive_identities = HashMap::<u32, String>::new();

    // declare variables to fill in genesis
    // -------------------------------------
    // members of technical committee
    let mut technical_committee_members: Vec<AccountId> = Vec::new();
    // memberships
    let mut memberships = BTreeMap::new();
    // certifications
    let mut certs_by_receiver = BTreeMap::new();
    // initial authorities
    let mut initial_authorities = BTreeMap::new();
    //let mut total_dust = 0;

    // FORCED AUTHORITY //
    // If this authority is defined (most likely Alice), then it must exist in both _identities_
    // and _smiths_. We create all this, for *development purposes* (used with `gdev_dev` or `gtest_dev` chains).
    if let Some(authority_name) = &maybe_force_authority {
        make_authority_exist::<SessionKeys, SKP>(
            &mut identities_v2,
            &mut smiths,
            &common_parameters,
            authority_name,
        );
    }

    // SIMPLE WALLETS //
    let genesis_data_wallets_count = genesis_data.wallets.len();
    let (was_fatal, mut monetary_mass, mut accounts, invalid_wallets) =
        v1_wallets_to_v2_accounts(genesis_data.wallets, &common_parameters);
    if was_fatal {
        fatal = true;
    }

    // Technical Comittee //
    // NOTE : when changing owner key, the technical committee is not changed
    for name in &technical_committee {
        if let Some(identity) = &identities_v2.get(name) {
            technical_committee_members.push(identity.owner_key.clone());
        } else {
            log::error!("Identity '{}' does not exist", name);
            fatal = true;
        }
    }

    // IDENTITIES //
    let (was_fatal, identities) = feed_identities(
        &mut accounts,
        &mut identity_index,
        &mut monetary_mass,
        &mut inactive_identities,
        &mut memberships,
        &identities_v2,
    )?;
    if was_fatal {
        fatal = true;
    }

    // CERTIFICATIONS //
    // counter for certifications
    let (was_fatal, counter_cert) = feed_certs_by_receiver(&mut certs_by_receiver, &identities_v2);
    if was_fatal {
        fatal = true;
    }

    // SMITHS SUB-WOT //

    // Authorities
    if let Some(name) = &maybe_force_authority {
        check_authority_exists_in_both_wots(name, &identities_v2, &smiths);
    }

    let smiths = decorate_smiths_with_identity(smiths, &identity_index, &identities_v2);

    // counter for online authorities at genesis
    let (
        was_fatal,
        counter_online_authorities,
        counter_smith_cert,
        SmithWoT {
            smith_certs_by_receiver,
            smith_memberships,
            session_keys_map,
        },
    ) = create_smith_wot(
        &mut initial_authorities,
        &identities_v2,
        &smiths,
        &common_parameters,
        &clique_smiths,
    )?;
    if was_fatal {
        fatal = true;
    }

    // Verify certifications coherence (can be ignored for old users)
    for (idty_index, receiver_certs) in &certs_by_receiver {
        if receiver_certs.len() < common_parameters.min_cert as usize {
            let name = identity_index.get(idty_index).unwrap();
            let identity = identities_v2.get(&(*name).clone()).unwrap();
            if identity.membership_expire_on != 0 {
                log::error!(
                    "[{}] has received only {}/{} certifications",
                    name,
                    receiver_certs.len(),
                    common_parameters.min_cert
                );
                fatal = true;
            }
        }
    }

    // Verify smith certifications coherence
    for (idty_index, certs) in &smith_certs_by_receiver {
        if certs.len() < common_parameters.smith_min_cert as usize {
            log::error!(
                "[{}] has received only {}/{} smith certifications",
                identity_index.get(idty_index).unwrap(),
                certs.len(),
                common_parameters.smith_min_cert
            );
            fatal = true;
        }
    }

    // check number of online authorities
    if maybe_force_authority.is_none() && counter_online_authorities != 1 {
        log::error!("one and only one smith must be online, not {counter_online_authorities}");
    }

    // check monetary mass
    if monetary_mass != genesis_data.initial_monetary_mass {
        log::warn!(
            "actual monetary_mass ({}) and initial_monetary_mass ({}) do not match",
            monetary_mass.to_formatted_string(&Locale::en),
            genesis_data
                .initial_monetary_mass
                .to_formatted_string(&Locale::en)
        );
        if monetary_mass > genesis_data.initial_monetary_mass {
            log::error!("money has been created");
            fatal = true;
        }
    }

    // treasury balance must come from existing money
    let treasury_funder: AccountId = match (treasury_funder_address, treasury_funder_pubkey) {
        (Some(address), None) => address,
        (None, Some(pubkey)) => {
            v1_pubkey_to_account_id(pubkey).expect("treasury founder must have a valid public key")
        }
        _ => panic!("One of treasury_funder_address or treasury_funder_pubkey must be set"),
    };
    if let Some(existing_account) = accounts.get_mut(&treasury_funder) {
        existing_account.balance = existing_account
            .balance
            .checked_sub(common_parameters.existential_deposit)
            .expect("should have enough money to fund Treasury");
        treasury_balance = common_parameters.existential_deposit;
    }
    if treasury_balance < common_parameters.existential_deposit {
        log::error!(
            "Treasury balance {} is inferior to existential deposit {}",
            treasury_balance,
            common_parameters.existential_deposit
        );
        fatal = true;
    }

    smiths.iter().for_each(|smith| {
        log::info!(
            "[Smith] {} ({} - {})",
            smith.idty_index,
            smith.account,
            smith.name.clone()
        );
    });

    initial_authorities
        .iter()
        .for_each(|(index, (authority_account, online))| {
            log::info!(
                "[Authority] {} : {} ({} - {})",
                index,
                if *online { "online" } else { "offline" },
                authority_account,
                identity_index
                    .get(index)
                    .expect("authority should have an identity")
            );
        });

    let genesis_info = GenesisInfo {
        accounts: &accounts,
        genesis_data_wallets_count: &genesis_data_wallets_count,
        identities: &identities,
        inactive_identities: &inactive_identities,
        identity_index: &identity_index,
        smith_memberships: &smith_memberships,
        counter_online_authorities: &counter_online_authorities,
        counter_cert: &counter_cert,
        counter_smith_cert: &counter_smith_cert,
        technical_committee_members: &technical_committee_members,
        common_parameters: &common_parameters,
    };

    dump_genesis_info(genesis_info);

    if parameters.is_some() {
        let g1_duniter_v1_c = 0.0488;
        let g1_duniter_v1_xpercent: Perbill = Perbill::from_float(0.8);
        let c = f32::sqrt(common_parameters.c2.deconstruct() as f32 / 1_000_000_000f32);

        // static parameters (GTest or G1)
        if common_parameters.decimals != G1_DUNITER_V1_DECIMALS {
            warn!(
                "parameter `decimals` value ({}) is different from Ğ1 value ({})",
                common_parameters.decimals, G1_DUNITER_V1_DECIMALS
            )
        }
        if common_parameters.existential_deposit != G1_DUNITER_V1_EXISTENTIAL_DEPOSIT {
            warn!(
                "parameter `existential_deposit` value ({}) is different from Ğ1 value ({})",
                common_parameters.existential_deposit, G1_DUNITER_V1_EXISTENTIAL_DEPOSIT
            )
        }
        if common_parameters.membership_period / DAYS != G1_DUNITER_V1_MSVALIDITY / DUNITER_V1_DAYS
        {
            warn!(
                "parameter `membership_period` ({} days) is different from Ğ1's ({} days)",
                common_parameters.membership_period as f32 / DAYS as f32,
                G1_DUNITER_V1_MSVALIDITY as f32 / DUNITER_V1_DAYS as f32
            )
        }
        if common_parameters.cert_period / DAYS != G1_DUNITER_V1_SIGPERIOD / DUNITER_V1_DAYS {
            warn!(
                "parameter `cert_period` ({} days) is different from Ğ1's ({} days)",
                common_parameters.cert_period as f32 / DAYS as f32,
                G1_DUNITER_V1_SIGPERIOD as f32 / DUNITER_V1_DAYS as f32
            )
        }
        if common_parameters.cert_validity_period / DAYS
            != G1_DUNITER_V1_SIGVALIDITY / DUNITER_V1_DAYS
        {
            warn!(
                "parameter `cert_validity_period` ({} days) is different from Ğ1's ({} days)",
                common_parameters.cert_validity_period as f32 / DAYS as f32,
                G1_DUNITER_V1_SIGVALIDITY as f32 / DUNITER_V1_DAYS as f32
            )
        }
        if common_parameters.min_cert != G1_DUNITER_V1_SIGQTY {
            warn!(
                "parameter `min_cert` value ({}) is different from Ğ1 value ({})",
                common_parameters.min_cert, G1_DUNITER_V1_SIGQTY
            )
        }
        if common_parameters.cert_max_by_issuer != G1_DUNITER_V1_SIGSTOCK {
            warn!(
                "parameter `cert_max_by_issuer` value ({}) is different from Ğ1 value ({})",
                common_parameters.cert_max_by_issuer, G1_DUNITER_V1_SIGSTOCK
            )
        }
        if c != g1_duniter_v1_c {
            warn!(
                "parameter `c` value ({}) is different from Ğ1 value ({})",
                c, g1_duniter_v1_c
            )
        }
        if common_parameters.ud_creation_period as f32 / DAYS as f32
            != G1_DUNITER_V1_DT as f32 / DUNITER_V1_DAYS as f32
        {
            warn!(
                "parameter `ud_creation_period` value ({} days) is different from Ğ1 value ({} days)",
                common_parameters.ud_creation_period as f32 / DAYS as f32, G1_DUNITER_V1_DT as f32 / DUNITER_V1_DAYS as f32
            )
        }
        if common_parameters.ud_reeval_period as f32 / DAYS as f32
            != G1_DUNITER_V1_DTREEVAL as f32 / DUNITER_V1_DAYS as f32
        {
            warn!(
                "parameter `ud_reeval_period` value ({} days) is different from Ğ1 value ({} days)",
                common_parameters.ud_reeval_period as f32 / DAYS as f32,
                G1_DUNITER_V1_DTREEVAL as f32 / DUNITER_V1_DAYS as f32
            )
        }
        if common_parameters.distance_min_accessible_referees != g1_duniter_v1_xpercent {
            warn!(
                "parameter `distance_min_accessible_referees` value ({}) is different from Ğ1 value ({})",
                format!("{:?}", common_parameters.distance_min_accessible_referees), format!("{:?}", g1_duniter_v1_xpercent)
            )
        }
        if common_parameters.max_depth != G1_DUNITER_V1_STEPMAX {
            warn!(
                "parameter `max_depth` value ({}) is different from Ğ1 value ({})",
                common_parameters.max_depth, G1_DUNITER_V1_STEPMAX
            )
        }
        let count_uds = common_parameters.ud_reeval_period / common_parameters.ud_creation_period;
        if count_uds == 0 {
            error!(
                "the `ud_reeval_period / ud_creation_period` is zero ({} days/{} days)",
                common_parameters.ud_reeval_period / DAYS as u64,
                common_parameters.ud_creation_period / DAYS as u64
            );
            fatal = true;
        }
    }

    // some more checks
    assert_eq!(
        identities.len() - inactive_identities.len(),
        memberships.len()
    );
    assert_eq!(smith_memberships.len(), initial_authorities.len());
    assert_eq!(smith_memberships.len(), session_keys_map.len());
    assert_eq!(identity_index.len(), identities.len());
    assert_eq!(
        accounts.len(),
        identity_index.len() + genesis_data_wallets_count.sub(invalid_wallets)
    );
    smiths_and_technical_committee_checks(&inactive_identities, &technical_committee, &smiths);

    // check the logs to see all the fatal error preventing from starting gtest currency
    if fatal {
        log::error!("some previously logged error prevent from building a sane genesis");
        panic!();
    }

    // Indexer output
    if let Ok(path) = std::env::var("DUNITER_GENESIS_EXPORT") {
        // genesis_certs_min_received => min_cert
        // genesis_memberships_expire_on => membership_period
        // genesis_smith_certs_min_received => smith_min_cert
        // genesis_smith_memberships_expire_on => smith_membership_period
        let export = GenesisIndexerExport {
            first_ud,
            first_ud_reeval,
            genesis_parameters: common_parameters.clone(),
            identities: identities_v2,
            sudo_key: sudo_key.clone(),
            technical_committee,
            ud,
            wallets: accounts
                .iter()
                .map(|(account_id, data)| (account_id.clone(), data.balance))
                .collect(),
            smiths: (smiths)
                .iter()
                .map(|smith| {
                    (
                        smith.name.clone(),
                        SmithData {
                            idty_index: smith.idty_index,
                            name: smith.name.clone(),
                            account: smith.account.clone(),
                            session_keys: smith.session_keys.clone(),
                            certs_received: smith.certs_received.clone(),
                        },
                    )
                })
                .collect::<BTreeMap<String, SmithData>>(),
            transactions_history: genesis_data.transactions_history.map(|history| {
                history
                    .iter()
                    // Avoid wrong pubkeys in tx history
                    .filter(|(pubkey, _)| v1_pubkey_to_account_id((*pubkey).clone()).is_ok())
                    .map(|(pubkey, txs)| {
                        (
                            v1_pubkey_to_account_id(pubkey.clone())
                                .expect("already checked account"),
                            txs.iter()
                                // Avoid wrong pubkeys in tx history
                                .filter(|tx| v1_pubkey_to_account_id(tx.issuer.clone()).is_ok())
                                .map(|tx| TransactionV2 {
                                    issuer: v1_pubkey_to_account_id(tx.issuer.clone())
                                        .expect("already checked tx.issuer"),
                                    amount: tx.amount.clone(),
                                    written_time: tx.written_time,
                                    comment: tx.comment.clone(),
                                })
                                .collect::<Vec<TransactionV2>>(),
                        )
                    })
                    .collect::<BTreeMap<AccountId, Vec<TransactionV2>>>()
            }),
        };
        fs::write(
            &path,
            serde_json::to_string_pretty(&export).expect("should be serializable"),
        )
        .unwrap_or_else(|_| panic!("Could not export genesis data to {}", &path));
    }

    let genesis_data = GenesisData {
        accounts,
        treasury_balance,
        certs_by_receiver,
        first_ud,
        first_ud_reeval,
        identities,
        initial_authorities,
        initial_monetary_mass: genesis_data.initial_monetary_mass,
        memberships,
        parameters,
        common_parameters: Some(common_parameters),
        session_keys_map,
        smith_certs_by_receiver,
        smith_memberships,
        sudo_key,
        technical_committee_members,
        ud,
    };

    Ok(genesis_data)
}

fn dump_genesis_info(info: GenesisInfo) {
    // give genesis info
    log::info!(
        "prepared genesis with:
        - {} accounts ({} identities, {} simple wallets)
        - {} total identities ({} active, {} inactive)
        - {} smiths
        - {} initial online authorities
        - {} certifications
        - {} smith certifications
        - {} members in technical committee",
        info.accounts.len(),
        info.identities.len() - info.inactive_identities.len(),
        info.genesis_data_wallets_count,
        info.identity_index.len(),
        info.identities.len() - info.inactive_identities.len(),
        info.inactive_identities.len(),
        info.smith_memberships.len(),
        info.counter_online_authorities,
        info.counter_cert,
        info.counter_smith_cert,
        info.technical_committee_members.len(),
    );

    // give genesis info
    log::info!(
        "currency parameters:
        - existential deposit: {} {}
        - currency decimals: {}
        - membership validity: {} days
        - certification period: {} days
        - certification validity duration: {} days
        - smith membership validity: {} days
        - smith certification validity: {} days
        - required certifications: {}
        - smith required certifications: {}
        - max certifications by issuer: {}
        - money growth rate: {}% every {} days
        - UD creation period: {} days
        - distance percent of required referees: {}%
        - distance max depth: {}",
        info.common_parameters.existential_deposit,
        info.common_parameters.currency_name,
        info.common_parameters.decimals,
        info.common_parameters.membership_period as f32 / DAYS as f32,
        info.common_parameters.cert_period as f32 / DAYS as f32,
        info.common_parameters.cert_validity_period as f32 / DAYS as f32,
        info.common_parameters.smith_membership_period as f32 / DAYS as f32,
        info.common_parameters.smith_certs_validity_period as f32 / DAYS as f32,
        info.common_parameters.min_cert,
        info.common_parameters.smith_min_cert,
        info.common_parameters.cert_max_by_issuer,
        f32::sqrt(info.common_parameters.c2.deconstruct() as f32 / 1_000_000_000f32) * 100f32,
        info.common_parameters.ud_reeval_period as f32 / DAYS as f32,
        info.common_parameters.ud_creation_period as f32 / DAYS as f32,
        info.common_parameters
            .distance_min_accessible_referees
            .deconstruct() as f32
            / 1_000_000_000f32
            * 100f32,
        info.common_parameters.max_depth,
    );
}

fn smiths_and_technical_committee_checks(
    inactive_identities: &HashMap<u32, String>,
    technical_committee: &Vec<String>,
    smiths: &Vec<SmithData>,
) {
    // no inactive tech comm
    for tech_com_member in technical_committee {
        let inactive_commitee_member = inactive_identities.values().any(|v| v == tech_com_member);
        if inactive_commitee_member {
            log::error!(
                "{} is an inactive technical commitee member",
                tech_com_member
            );
            assert!(!inactive_commitee_member);
        }
    }
    // no inactive smith
    for SmithData { name: smith, .. } in smiths {
        let inactive_smiths: Vec<_> = inactive_identities
            .values()
            .filter(|v| *v == smith)
            .collect();
        inactive_smiths
            .iter()
            .for_each(|s| log::warn!("Smith {} is inactive", s));
        assert_eq!(inactive_smiths.len(), 0);
    }
}

fn create_smith_wot<SK: Decode>(
    initial_authorities: &mut BTreeMap<u32, (AccountId32, bool)>,
    identities_v2: &HashMap<String, IdentityV2>,
    smiths: &Vec<SmithData>,
    common_parameters: &CommonParameters,
    clique_smiths: &Option<Vec<CliqueSmith>>,
) -> Result<(bool, u32, u32, SmithWoT<SK>), String> {
    let mut fatal = false;
    let mut counter_online_authorities = 0;
    // counter for smith certifications
    let mut counter_smith_cert = 0;
    let mut smith_certs_by_receiver = BTreeMap::new();
    // smith memberships
    let mut smith_memberships = BTreeMap::new();
    let mut session_keys_map = BTreeMap::new();
    // Then create the smith WoT
    for smith in smiths {
        // check that smith exists
        let identities_v2_clone = identities_v2.clone();
        if let Some(identity) = identities_v2.get(&smith.name.clone()) {
            counter_online_authorities = set_smith_session_keys_and_authority_status(
                initial_authorities,
                &mut session_keys_map,
                &smith,
                identity,
            )?;

            // smith certifications
            counter_smith_cert += feed_smith_certs_by_receiver(
                &mut smith_certs_by_receiver,
                clique_smiths,
                &smith,
                identity,
                &identities_v2_clone,
                common_parameters,
            )?;

            // smith memberships
            smith_memberships.insert(
                identity.index,
                MembershipData {
                    expire_on: common_parameters.smith_membership_period,
                },
            );
        } else {
            log::error!(
                "Smith '{}' does not correspond to exising identity",
                &smith.name
            );
            fatal = true;
        }
    }
    Ok((
        fatal,
        counter_online_authorities,
        counter_smith_cert,
        SmithWoT {
            smith_certs_by_receiver,
            smith_memberships,
            session_keys_map,
        },
    ))
}

fn v1_wallets_to_v2_accounts(
    wallets: BTreeMap<PubkeyV1, u64>,
    common_parameters: &CommonParameters,
) -> (
    bool,
    u64,
    BTreeMap<AccountId32, GenesisAccountData<u64>>,
    usize,
) {
    // monetary mass for double check
    let mut monetary_mass = 0u64;
    // account inserted in genesis
    let mut accounts: BTreeMap<AccountId, GenesisAccountData<u64>> = BTreeMap::new();
    let mut invalid_wallets = 0;
    let mut fatal = false;
    for (pubkey, balance) in wallets {
        // check existential deposit
        if balance < common_parameters.existential_deposit {
            log::error!(
                "wallet {pubkey} has {balance} cǦT which is below {}",
                common_parameters.existential_deposit
            );
            fatal = true;
        }

        // double check the monetary mass
        monetary_mass += balance;

        // json prevents duplicate wallets
        if let Ok(owner_key) = v1_pubkey_to_account_id(pubkey.clone()) {
            accounts.insert(
                owner_key.clone(),
                GenesisAccountData {
                    random_id: H256(blake2_256(&(balance, &owner_key).encode())),
                    balance,
                    is_identity: false,
                },
            );
        } else {
            log::warn!("wallet {pubkey} has wrong format");
            invalid_wallets = invalid_wallets.add(1);
        }
    }
    (fatal, monetary_mass, accounts, invalid_wallets)
}

fn check_identities_v2(
    identities_v2: &HashMap<String, IdentityV2>,
    common_parameters: &CommonParameters,
) {
    // // Identities whose membership was lost since export
    // identities_v2.iter_mut()
    //     .filter(|(name, i)| (i.membership_expire_on as u64) < genesis_timestamp)
    //     .for_each(|(name, i)| {
    //         log::warn!("{} membership expired since export", name);
    //         i.membership_expire_on = 0;
    //     });

    // // Identities that are no more members because of a lack of certs
    // identities_v2.iter_mut()
    //     .filter(|(name, i)| i.membership_expire_on != 0 && (i.certs_received.len() as u32) < common_parameters.min_cert)
    //     .for_each(|(name, i)| {
    //         log::warn!("{} lost membership because of lost certifications since export", name);
    //         i.membership_expire_on = 0;
    //     });

    // Check that members have enough certs
    identities_v2
        .iter()
        .filter(|(_, i)| i.membership_expire_on != 0)
        .for_each(|(name, i)| {
            let nb_certs = i.certs_received.len() as u32;
            if nb_certs < common_parameters.min_cert {
                log::warn!("{} has only {} valid certifications", name, nb_certs);
            }
        });
}

fn check_genesis_data_and_filter_expired_certs_since_export(
    genesis_data: &mut GenesisMigrationData,
    genesis_timestamp: u64,
    common_parameters: &CommonParameters,
) {
    // Remove expired certs since export
    genesis_data
        .identities
        .iter_mut()
        .for_each(|(receiver, i)| {
            i.certs_received.retain(|issuer, v| {
                let retain = (v.0 as u64) >= genesis_timestamp;
                if !retain {
                    log::warn!("{} -> {} cert expired since export", issuer, receiver);
                }
                retain
            });
        });

    genesis_data.identities.iter_mut().for_each(|(name, i)| {
        if (i.membership_expire_on.0 as u64) < genesis_timestamp {
            i.membership_expire_on = TimestampV1(0);
            log::warn!("{} membership expired since export", name);
        }
    });

    genesis_data.identities.iter_mut().for_each(|(name, i)| {
        if i.membership_expire_on.0 != 0
            && i.certs_received.len() < common_parameters.min_cert as usize
        {
            i.membership_expire_on = TimestampV1(0);
            log::warn!(
                "{} lost membership because of lost certifications since export",
                name
            );
        }
    });

    genesis_data.identities.iter().for_each(|(name, i)| {
        if i.owner_pubkey.is_some() && i.owner_address.is_some() {
            log::warn!(
                "{} both has a pubkey and an address defined - address will be used",
                name
            );
        }
        if i.owner_pubkey.is_none() && i.owner_address.is_none() {
            log::error!("{} neither has a pubkey and an address defined", name);
        }
    });
}

fn genesis_data_to_identities_v2(
    genesis_identities: BTreeMap<String, IdentityV1>,
    genesis_timestamp: u64,
    smiths: &[RawSmith],
) -> HashMap<String, IdentityV2> {
    let key_migrations: HashMap<String, AccountId> = smiths
        .iter()
        .filter(|s| s.migration_address.is_some())
        .map(|s| {
            (
                s.name.clone(),
                s.migration_address.clone().expect("already filtered"),
            )
        })
        .collect();
    genesis_identities
        .into_iter()
        .map(|(name, i)| {
            let legacy_account = i
                .owner_pubkey
                .map(|pubkey| {
                    v1_pubkey_to_account_id(pubkey)
                        .expect("a G1 identity necessarily has a valid pubkey")
                })
                .unwrap_or_else(|| {
                    i.owner_address.unwrap_or_else(|| {
                        panic!("neither pubkey nor address is defined for {}", name)
                    })
                });
            let migration = key_migrations.get(name.as_str());
            let owner_key = if let Some(migrated_account) = migration {
                migrated_account.clone()
            } else {
                legacy_account.clone()
            };
            let old_owner_key = if migration.is_none() {
                None
            } else {
                Some(legacy_account)
            };
            (
                name,
                IdentityV2 {
                    index: i.index,
                    owner_key,
                    old_owner_key,
                    membership_expire_on: timestamp_to_relative_blocs(
                        i.membership_expire_on,
                        genesis_timestamp,
                    ),
                    next_cert_issuable_on: timestamp_to_relative_blocs(
                        i.next_cert_issuable_on,
                        genesis_timestamp,
                    ),
                    balance: i.balance,
                    certs_received: i
                        .certs_received
                        .into_iter()
                        .map(|(issuer, timestamp)| {
                            (
                                issuer,
                                timestamp_to_relative_blocs(timestamp, genesis_timestamp),
                            )
                        })
                        .collect(),
                },
            )
        })
        .collect()
}

fn make_authority_exist<SessionKeys: Encode, SKP: SessionKeysProvider<SessionKeys>>(
    identities_v2: &mut HashMap<String, IdentityV2>,
    smiths: &mut Vec<RawSmith>,
    common_parameters: &CommonParameters,
    authority_name: &String,
) {
    // The identity might already exist, notably: G1 "Alice" already exists
    if let Some(authority) = identities_v2.get_mut(authority_name) {
        // Force authority to be active
        authority.membership_expire_on = common_parameters.membership_period;
    } else {
        // Not found: we must create it
        identities_v2.insert(
            authority_name.clone(),
            IdentityV2 {
                index: (identities_v2.len() as u32 + 1),
                owner_key: get_account_id_from_seed::<sr25519::Public>(authority_name),
                balance: common_parameters.existential_deposit,
                certs_received: HashMap::new(),
                membership_expire_on: common_parameters.membership_period,
                old_owner_key: None,
                next_cert_issuable_on: 0,
            },
        );
    };
    // Forced authority gets its required certs from first "minCert" WoT identities (fake certs)
    let mut new_certs: HashMap<String, u32> = HashMap::new();
    let certs_of_authority = &identities_v2.get(authority_name).unwrap().certs_received;
    identities_v2
        .keys()
        // Identities which are not the authority and have not already certified her
        .filter(|issuer| {
            issuer != &authority_name
                && !certs_of_authority
                    .iter()
                    .any(|(authority_issuer, _)| issuer == &authority_issuer)
        })
        .take(common_parameters.min_cert as usize)
        .map(String::clone)
        .for_each(|issuer| {
            new_certs.insert(issuer, common_parameters.cert_period);
        });
    let authority = identities_v2
        .get_mut(authority_name)
        .expect("authority must exist or be created");
    new_certs.into_iter().for_each(|(issuer, c)| {
        authority.certs_received.insert(issuer, c);
    });
    let sk: SessionKeys = SKP::session_keys(&get_authority_keys_from_seed(authority_name.as_str()));
    let forced_authority_session_keys = format!("0x{}", hex::encode(sk.encode()));
    // Add forced authority to smiths (whether explicit smith WoT or clique)
    if let Some(smith) = smiths.iter_mut().find(|s| &s.name == authority_name) {
        smith.session_keys = Some(forced_authority_session_keys);
        smith.migration_address = None;
    } else {
        smiths.push(RawSmith {
            name: authority_name.clone(),
            session_keys: Some(forced_authority_session_keys),
            migration_address: None,
            certs_received: vec![],
        })
    }
}

fn feed_identities(
    accounts: &mut BTreeMap<AccountId32, GenesisAccountData<u64>>,
    identity_index: &mut HashMap<u32, String>,
    monetary_mass: &mut u64,
    inactive_identities: &mut HashMap<u32, String>,
    memberships: &mut BTreeMap<u32, MembershipData>,
    identities_v2: &HashMap<String, IdentityV2>,
) -> Result<(bool, Vec<GenesisIdentity>), String> {
    let mut fatal = false;
    let mut identities: Vec<GenesisIdentity> = Vec::new();
    for (name, identity) in identities_v2 {
        // identity name
        if !validate_idty_name(name) {
            return Err(format!("Identity name '{}' is invalid", &name));
        }

        // TODO: re-check this code origin and wether it should be included or not
        // do not check existential deposit of identities
        // // check existential deposit
        // if identity.balance < common_parameters.existencial_deposit {
        //     if identity.membership_expire_on == 0 {
        //         log::warn!(
        //             "expired identity {name} has {} cǦT which is below {}",
        //             identity.balance, common_parameters.existencial_deposit
        //         );
        //         fatal = true;
        //     } else {
        //         member identities can still be below existential deposit thanks to sufficient
        //         log::info!(
        //             "identity {name} has {} cǦT which is below {}",
        //             identity.balance, common_parameters.existencial_deposit
        //         );
        //     }
        // }

        // Money
        // check that wallet with same owner_key does not exist
        if accounts.get(&identity.owner_key).is_some() {
            log::error!(
                "{name} owner_key {} already exists as a simple wallet",
                identity.owner_key
            );
            fatal = true;
        }
        // insert as an account
        accounts.insert(
            identity.owner_key.clone(),
            GenesisAccountData {
                random_id: H256(blake2_256(&(identity.index, &identity.owner_key).encode())),
                balance: identity.balance,
                is_identity: true,
            },
        );

        // double check the monetary mass
        *monetary_mass += identity.balance;

        // insert identity
        // check that index does not already exist
        if let Some(other_name) = identity_index.get(&identity.index) {
            log::error!(
                "{other_name} already has identity index {} of {name}",
                identity.index
            );
            fatal = true;
        }
        identity_index.insert(identity.index, name.to_owned());

        let expired = identity.membership_expire_on == 0;
        // only add the identity if not expired
        if expired {
            inactive_identities.insert(identity.index, name.clone());
        };
        identities.push(GenesisIdentity {
            // N.B.: every **non-expired** identity on Genesis is considered to have:
            //  - removable_on: 0,
            //  - next_creatable_identity_on: 0,
            //  - status: IdtyStatus::Validated,
            idty_index: identity.index,
            name: name.to_owned().clone(),
            owner_key: identity.owner_key.clone(),
            old_owner_key: identity.old_owner_key.clone(),
            // but expired identities will just have their pseudonym reserved in the storage
            active: !expired,
        });

        // insert the membershup data (only if not expired)
        if !expired {
            memberships.insert(
                identity.index,
                MembershipData {
                    expire_on: identity.membership_expire_on,
                },
            );
        }
    }
    // sort the identities by index for reproducibility (should have been a vec in json)
    identities.sort_unstable_by(|a, b| a.idty_index.cmp(&b.idty_index));

    Ok((fatal, identities))
}

fn set_smith_session_keys_and_authority_status<SK>(
    initial_authorities: &mut BTreeMap<
        u32,
        (
            <<MultiSignature as Verify>::Signer as IdentifyAccount>::AccountId,
            bool,
        ),
    >,
    session_keys_map: &mut BTreeMap<
        <<MultiSignature as Verify>::Signer as IdentifyAccount>::AccountId,
        SK,
    >,
    smith: &&SmithData,
    identity: &IdentityV2,
) -> Result<u32, String>
where
    SK: Decode,
{
    let mut counter_online_authorities = 0;
    // Initial authorities and session keys
    let session_keys_bytes = if let Some(declared_session_keys) = &smith.session_keys {
        counter_online_authorities += 1;
        // insert authority as online
        initial_authorities.insert(identity.index, (identity.owner_key.clone(), true));
        // decode session keys or force to given value
        hex::decode(&declared_session_keys[2..])
            .map_err(|_| format!("invalid session keys for idty {}", smith.name))?
    } else {
        // still authority but offline
        initial_authorities.insert(identity.index, (identity.owner_key.clone(), false));
        // fake session keys
        let mut fake_bytes = Vec::with_capacity(128);
        for _ in 0..4 {
            fake_bytes.extend_from_slice(identity.owner_key.as_ref())
        }
        fake_bytes
    };

    // insert session keys to map
    session_keys_map.insert(
        identity.owner_key.clone(),
        SK::decode(&mut &session_keys_bytes[..]).unwrap(),
    );

    Ok(counter_online_authorities)
}

fn feed_smith_certs_by_receiver(
    smith_certs_by_receiver: &mut BTreeMap<u32, BTreeMap<u32, Option<u32>>>,
    clique_smiths: &Option<Vec<CliqueSmith>>,
    smith: &&SmithData,
    identity: &IdentityV2,
    identities_v2: &HashMap<String, IdentityV2>,
    common_parameters: &CommonParameters,
) -> Result<u32, String> {
    let mut counter_smith_cert = 0;
    let mut certs = BTreeMap::new();
    if clique_smiths.is_some() {
        // All initial smiths are considered to be certifying all each other
        clique_smiths
            .as_ref()
            .unwrap()
            .iter()
            .filter(|other_smith| *other_smith.name.as_str() != *smith.name)
            .for_each(|other_smith| {
                let issuer_index = &identities_v2
                    .get(other_smith.name.as_str())
                    .unwrap_or_else(|| {
                        panic!("Identity '{}' does not exist", other_smith.name.as_str())
                    })
                    .index;
                certs.insert(*issuer_index, None);
                counter_smith_cert += 1;
            });
    } else {
        for issuer in &smith.certs_received {
            let issuer_index = &identities_v2
                .get(issuer)
                .ok_or(format!("Identity '{}' does not exist", issuer))?
                .index;
            certs.insert(
                *issuer_index,
                Some(common_parameters.smith_certs_validity_period),
            );
            counter_smith_cert += 1;
        }
    }
    smith_certs_by_receiver.insert(identity.index, certs);
    Ok(counter_smith_cert)
}

fn feed_certs_by_receiver(
    certs_by_receiver: &mut BTreeMap<u32, BTreeMap<u32, Option<u32>>>,
    identities_v2: &HashMap<String, IdentityV2>,
) -> (bool, u32) {
    let mut fatal = false;
    let mut counter_cert = 0;
    for identity in identities_v2.values() {
        let mut certs = BTreeMap::new();
        for (issuer, expire_on) in &identity.certs_received {
            if let Some(issuer) = &identities_v2.get(issuer) {
                certs.insert(issuer.index, Some(*expire_on));
                counter_cert += 1;
            } else {
                log::error!("Identity '{}' does not exist", issuer);
                fatal = true;
            };
        }
        certs_by_receiver.insert(identity.index, certs);
    }
    (fatal, counter_cert)
}

fn check_authority_exists_in_both_wots(
    name: &String,
    identities_v2: &HashMap<String, IdentityV2>,
    smiths: &[RawSmith],
) {
    identities_v2
        .get(name)
        .ok_or(format!("Identity '{}' not exist", name))
        .expect("Initial authority must have an identity");
    smiths
        .iter()
        .find(|smith| &smith.name == name)
        .expect("Forced authority must be present in smiths");
}

fn build_smiths_wot(
    clique_smiths: &Option<Vec<CliqueSmith>>,
    smith_identities: Option<BTreeMap<String, RawSmith>>,
) -> Result<Vec<RawSmith>, String> {
    if smith_identities.is_some() && clique_smiths.is_some() {
        return Err(
            "'smiths' and 'clique_smiths' cannot be both defined at the same time".to_string(),
        );
    }
    // Create a single source of smiths
    let smiths = if let Some(clique) = &clique_smiths {
        // From a clique
        clique
            .iter()
            .map(|smith| RawSmith {
                name: smith.name.clone(),
                session_keys: smith.session_keys.clone(),
                certs_received: vec![],
                migration_address: smith.migration_address.clone(),
            })
            .collect::<Vec<RawSmith>>()
    } else {
        // From explicit smith WoT
        smith_identities
            .expect("existence has been tested earlier")
            .into_values()
            .collect::<Vec<RawSmith>>()
    };
    Ok(smiths)
}

fn decorate_smiths_with_identity(
    smiths: Vec<RawSmith>,
    identity_index: &HashMap<u32, String>,
    identities_v2: &HashMap<String, IdentityV2>,
) -> Vec<SmithData> {
    smiths
        .into_iter()
        .map(|smith| SmithData {
            idty_index: identity_index
                .iter()
                .find(|(_, v)| ***v == smith.name)
                .map(|(k, _)| *k)
                .expect("smith must have an identity"),
            account: identities_v2
                .get(smith.name.as_str())
                .map(|i| i.owner_key.clone())
                .expect("identity must exist"),
            name: smith.name,
            session_keys: smith.session_keys,
            certs_received: smith.certs_received,
        })
        .collect()
}

#[cfg(feature = "gdev")]
pub fn generate_genesis_data_for_local_chain<P, SK, SessionKeys: Encode, SKP>(
    initial_authorities_len: usize,
    initial_smiths_len: usize,
    initial_identities_len: usize,
    treasury_balance: u64,
    parameters: Option<P>,
    root_key: AccountId,
    get_common_parameters: fn(&Option<P>) -> CommonParameters,
) -> Result<GenesisData<P, SK>, String>
where
    P: Default + DeserializeOwned,
    SK: Decode,
    SKP: SessionKeysProvider<SessionKeys>,
{
    assert!(initial_identities_len <= 6);
    assert!(initial_smiths_len <= initial_identities_len);
    assert!(initial_authorities_len <= initial_smiths_len);
    let ud = 1_000;
    let idty_index_start: u32 = 1;
    let common_parameters = get_common_parameters(&parameters);

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

    let mut session_keys_map = BTreeMap::new();
    initial_smiths.iter().for_each(|x| {
        let session_keys_bytes = SKP::session_keys(x).encode();
        let sk = SK::decode(&mut &session_keys_bytes[..])
            .map_err(|_| format!("invalid session keys for idty {}", x.0.clone()))
            .unwrap();
        session_keys_map.insert(x.0.clone(), sk);
    });

    let identities: Vec<GenesisIdentity> = initial_identities
        .iter()
        .enumerate()
        .map(|(i, (name, owner_key))| GenesisIdentity {
            idty_index: i as u32 + idty_index_start,
            name: String::from_utf8(name.0.clone()).unwrap(),
            owner_key: owner_key.clone(),
            old_owner_key: None,
            active: true,
        })
        .collect();

    let (certs_by_receiver, counter_cert) = clique_wot(initial_identities.len());

    let accounts = initial_identities
        .iter()
        .enumerate()
        .map(|(i, (_, owner_key))| {
            (
                owner_key.clone(),
                GenesisAccountData {
                    random_id: H256(blake2_256(
                        &(i as u32 + idty_index_start, owner_key).encode(),
                    )),
                    balance: ud,
                    is_identity: true,
                },
            )
        })
        .collect();

    let identity_index = identities
        .iter()
        .map(|i| (i.idty_index, i.name.clone()))
        .collect();

    let genesis_data_wallets_count = 0;
    let inactive_identities = HashMap::new();

    let smith_memberships = (1..=initial_smiths_len)
        .map(|i| (i as u32, MembershipData { expire_on: 0 }))
        .collect();

    let (smith_certs_by_receiver, counter_smith_cert) = clique_wot(initial_smiths_len);

    let initial_authorities: BTreeMap<
        u32,
        (
            <<MultiSignature as Verify>::Signer as IdentifyAccount>::AccountId,
            bool,
        ),
    > = initial_smiths
        .iter()
        .enumerate()
        .map(|(i, keys)| {
            (
                i as u32 + idty_index_start,
                (keys.0.clone(), i < initial_authorities_len),
            )
        })
        .collect();

    let counter_online_authorities = initial_authorities
        .iter()
        .filter(|(_, authority)| authority.1)
        .count() as u32;

    let technical_committee_members = initial_smiths
        .iter()
        .map(|x| x.0.clone())
        .collect::<Vec<_>>();

    let genesis_info = GenesisInfo {
        accounts: &accounts,
        genesis_data_wallets_count: &genesis_data_wallets_count,
        identities: &identities,
        inactive_identities: &inactive_identities,
        identity_index: &identity_index,
        smith_memberships: &smith_memberships,
        counter_online_authorities: &counter_online_authorities,
        counter_cert: &counter_cert,
        counter_smith_cert: &counter_smith_cert,
        technical_committee_members: &technical_committee_members,
        common_parameters: &common_parameters,
    };

    dump_genesis_info(genesis_info);

    let genesis_data = GenesisData {
        accounts,
        // Treasury balance is created out of nothing for local blockchain
        treasury_balance,
        certs_by_receiver,
        first_ud: None,
        first_ud_reeval: None,
        identities,
        initial_authorities,
        initial_monetary_mass: initial_identities_len as u64 * ud,
        memberships: (1..=initial_identities.len())
            .map(|i| (i as u32, MembershipData { expire_on: 0 }))
            .collect(),
        parameters,
        common_parameters: None,
        session_keys_map,
        smith_certs_by_receiver,
        smith_memberships,
        sudo_key: Some(root_key),
        technical_committee_members,
        ud,
    };

    Ok(genesis_data)
}

fn check_parameters_consistency(
    wallets: &BTreeMap<PubkeyV1, u64>,
    first_ud: &Option<u64>,
    first_reeval: &Option<u64>,
    ud: &u64,
) -> Result<(), String> {
    // No empty wallet
    if let Some((account, _)) = wallets.iter().find(|(_, amount)| **amount == 0) {
        return Err(format!("Wallet {} is empty", account));
    }

    if let (Some(first_ud), Some(first_reeval)) = (first_ud, first_reeval) {
        if first_ud > first_reeval {
            return Err(format!(
                "`first_ud` ({}) should be lower than `first_ud_reeval` ({})",
                first_ud, first_reeval
            ));
        }
    }
    if *ud == 0 {
        return Err("`ud` is expected to be > 0".to_owned());
    }
    Ok(())
}

fn get_genesis_input<P: Default + DeserializeOwned>(
    json_file_path: String,
) -> Result<GenesisInput<P>, String> {
    // We mmap the file into memory first, as this is *a lot* faster than using
    // `serde_json::from_reader`. See https://github.com/serde-rs/json/issues/160
    let file = std::fs::File::open(&json_file_path)
        .map_err(|e| format!("Error opening gen conf file `{}`: {}", json_file_path, e))?;
    // SAFETY: `mmap` is fundamentally unsafe since technically the file can change
    //         underneath us while it is mapped; in practice it's unlikely to be a problem
    let bytes = unsafe {
        memmap2::Mmap::map(&file)
            .map_err(|e| format!("Error mmaping gen conf file `{}`: {}", json_file_path, e))?
    };
    serde_json::from_slice::<GenesisInput<P>>(&bytes)
        .map_err(|e| format!("Error parsing gen conf file: {}", e))
}

fn get_genesis_migration_data() -> Result<GenesisMigrationData, String> {
    let json_file_path = std::env::var("DUNITER_GENESIS_DATA")
        .unwrap_or_else(|_| "./resources/g1-data.json".to_owned());
    let file = std::fs::File::open(&json_file_path).map_err(|e| {
        format!(
            "Error opening gen migration file `{}`: {}",
            json_file_path, e
        )
    })?;
    let bytes = unsafe {
        memmap2::Mmap::map(&file).map_err(|e| {
            format!(
                "Error mmaping gen migration file `{}`: {}",
                json_file_path, e
            )
        })?
    };
    serde_json::from_slice::<GenesisMigrationData>(&bytes)
        .map_err(|e| format!("Error parsing gen migration file: {}", e))
}

fn get_genesis_timestamp() -> Result<u64, String> {
    if let Ok(genesis_timestamp) = std::env::var("DUNITER_GENESIS_TIMESTAMP") {
        genesis_timestamp
            .parse()
            .map_err(|_| "DUNITER_GENESIS_TIMESTAMP must be a number".to_owned())
    } else {
        use std::time::SystemTime;
        Ok(SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("SystemTime before UNIX EPOCH!")
            .as_secs())
    }
}

fn validate_idty_name(name: &str) -> bool {
    name.len() <= 64
}

pub type AuthorityKeys = (
    AccountId,
    GrandpaId,
    BabeId,
    ImOnlineId,
    AuthorityDiscoveryId,
);

/// Because SessionKeys struct is defined by each Runtime, it cannot be constructed here.
/// Its construction must be provided.
pub trait SessionKeysProvider<SessionKeys: Encode> {
    fn session_keys(keys: &AuthorityKeys) -> SessionKeys;
}

#[derive(Default, Deserialize, Serialize, Clone)]
pub struct CommonParameters {
    // TODO: replace u32 by BlockNumber when appropriate
    pub currency_name: String,
    pub decimals: usize,
    pub existential_deposit: u64,
    pub membership_period: u32,
    pub cert_period: u32,
    pub smith_membership_period: u32,
    pub smith_certs_validity_period: u32,
    pub min_cert: u32,
    pub smith_min_cert: u32,
    pub cert_max_by_issuer: u32,
    pub cert_validity_period: u32,
    pub c2: Perbill,
    pub ud_creation_period: u64,
    pub distance_min_accessible_referees: Perbill,
    pub max_depth: u32,
    pub ud_reeval_period: u64,
}

/// Generate an authority keys.
fn get_authority_keys_from_seed(s: &str) -> AuthorityKeys {
    (
        get_account_id_from_seed::<sr25519::Public>(s),
        get_from_seed::<GrandpaId>(s),
        get_from_seed::<BabeId>(s),
        get_from_seed::<ImOnlineId>(s),
        get_from_seed::<AuthorityDiscoveryId>(s),
    )
}

/// Converts a Duniter v1 public key (Ed25519) to an Account Id.
/// No need to convert to address.
fn v1_pubkey_to_account_id(pubkey: PubkeyV1) -> Result<AccountId, String> {
    let bytes = bs58::decode(pubkey.0)
        .into_vec()
        .expect("Duniter v1 pubkey should be decodable");
    if bytes.len() > 32 {
        return Err("Pubkey is too long".to_string());
    }
    let prepend = vec![0u8; 32 - &bytes.len()];
    let bytes: [u8; 32] = [prepend.as_slice(), bytes.as_slice()]
        .concat()
        .as_slice()
        .try_into()
        .expect("incorrect pubkey length");
    Ok(AccountPublic::from(ed25519::Public::from_raw(bytes)).into_account())
}

fn timestamp_to_relative_blocs(timestamp: TimestampV1, start: u64) -> u32 {
    let diff = (timestamp.0 as u64).saturating_sub(start);
    seconds_to_blocs(diff as u32)
}

/// Converts a number of seconds to a number of 6-seconds blocs
/// use lower approximation
/// example : 2 seconds will be block 0
/// example : 7 seconds will be block 1
fn seconds_to_blocs(seconds: u32) -> u32 {
    seconds / 6
}

#[cfg(test)]
mod tests {
    use super::*;
    use sp_core::crypto::{Ss58AddressFormat, Ss58Codec};
    use sp_core::ByteArray;
    use std::str::FromStr;

    #[test]
    fn test_timestamp_to_relative_blocs() {
        assert_eq!(seconds_to_blocs(2), 0);
        assert_eq!(seconds_to_blocs(6), 1);
        assert_eq!(seconds_to_blocs(7), 1);
    }

    #[test]
    fn test_v1_pubkey_to_v2_address_translation() {
        assert_eq!(
            v1_pubkey_to_account_id(PubkeyV1(
                "2ny7YAdmzReQxAayyJZsyVYwYhVyax2thKcGknmQy5nQ".to_string()
            ))
            .unwrap()
            .to_ss58check_with_version(Ss58AddressFormat::custom(42)),
            "5CfdJjEgh3jDkg3bzmZ1ED1xVhXAARtNmZJWbcXh53rU8z5a".to_owned()
        );
    }

    #[test]
    fn test_pubkey_with_33_bytes() {
        assert_eq!(
            v1_pubkey_to_account_id(PubkeyV1(
                "d2meevcahfts2gqmvmrw5hzi25jddikk4nc4u1fkwrau".to_string()
            )),
            Err("Pubkey is too long".to_owned())
        );
    }

    #[test]
    fn test_address_to_pubkey_v1() {
        let account = AccountId::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")
            .expect("valid address");
        let pubkey = bs58::encode(account.as_slice()).into_vec();
        let pubkey = String::from_utf8(pubkey).expect("valid conversion");
        assert_eq!(pubkey, "FHNpKmJrUtusuvKPGomAygQqeiks98bdV6yD61Stb6vg");
    }
}
