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

use common_runtime::constants::*;
use common_runtime::entities::IdtyData;
use common_runtime::*;
use gtest_runtime::{
    opaque::SessionKeys, parameters, AccountConfig, AccountId, AuthorityMembersConfig, BabeConfig,
    CertConfig, GenesisConfig, IdentityConfig, MembershipConfig, SessionConfig, SmithCertConfig,
    SmithMembershipConfig, SudoConfig, SystemConfig, TechnicalCommitteeConfig,
    UniversalDividendConfig,
};
use serde::Deserialize;
use sp_core::{blake2_256, Decode, Encode, H256};
use std::collections::{BTreeMap, HashMap};

type MembershipData = sp_membership::MembershipData<u32>;

// get values of parameters
static EXISTENTIAL_DEPOSIT: u64 = parameters::ExistentialDeposit::get();
static SMITH_MEMBERSHIP_EXPIRE_ON: u32 = parameters::SmithMembershipPeriod::get();
static SMITH_CERTS_EXPIRE_ON: u32 = parameters::SmithValidityPeriod::get();
static MIN_CERT: u32 = parameters::WotMinCertForMembership::get();
static SMITH_MIN_CERT: u32 = parameters::SmithWotMinCertForMembership::get();

// define structure of json
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GenesisJson {
    identities: HashMap<String, Identity>,
    smiths: HashMap<String, Smith>,
    first_ud: u64,
    first_ud_reeval: u32,
    initial_monetary_mass: u64,
    wallets: HashMap<AccountId, u64>, // u128
    sudo_key: Option<AccountId>,
    technical_committee: Vec<String>,
}

/// identities
#[derive(Clone, Deserialize)]
struct Identity {
    /// indentity index matching the order of appearance in the Ǧ1v1 blockchain
    index: u32,
    /// ss58 address in gtest network
    owner_key: AccountId,
    /// optional ss58 address in the Ğ1v1
    old_owner_key: Option<AccountId>,
    /// block at which the membership is set to expire (0 for expired members)
    membership_expire_on: u32,
    /// block at which the next cert can be emitted
    next_cert_issuable_on: u32,
    /// balance of the account of this identity
    balance: u64, // u128
    /// certs received with their expiration block
    certs_received: HashMap<String, u32>,
}

/// smith members
#[derive(Clone, Deserialize)]
struct Smith {
    /// optional pre-set session keys (at least for the smith bootstraping the blockchain)
    session_keys: Option<String>,
    /// smith certification received
    certs_received: Vec<String>,
}

// Timestamp to block number
// fn to_bn(genesis_timestamp: u64, timestamp: u64) -> u32 {
//     let duration_in_secs = timestamp.saturating_sub(genesis_timestamp);
//     (duration_in_secs / 6) as u32
// }

// copied from duniter primitives
fn validate_idty_name(idty_name: &str) -> bool {
    idty_name.len() >= 3
        && idty_name.len() <= 42 // length smaller than 42
        // all characters are alphanumeric or - or _
        && idty_name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// ============================================================================================ ///
/// build genesis from json file
pub fn build_genesis(
    // genesis data build from json
    genesis_data: GenesisJson,
    // wasm binary
    wasm_binary: &[u8],
    // useful to enforce Alice authority when developing
    maybe_force_authority: Option<Vec<u8>>,
) -> Result<GenesisConfig, String> {
    // preparatory steps

    // define genesis timestamp
    let genesis_timestamp: u64 =
        if let Ok(genesis_timestamp) = std::env::var("DUNITER_GENESIS_TIMESTAMP") {
            genesis_timestamp
                .parse()
                .map_err(|_| "DUNITER_GENESIS_TIMESTAMP must be a number".to_owned())?
        } else {
            use std::time::SystemTime;
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("SystemTime before UNIX EPOCH!")
                .as_secs()
        };
    log::info!("genesis timestamp: {}", genesis_timestamp);

    // declare variables for building genesis
    // -------------------------------------
    // track if fatal error occured, but let processing continue
    let mut fatal = false;
    // monetary mass for double check
    let mut monetary_mass = 0u64; // u128
                                  // wallet index to generate random id
    let mut wallet_index: u32 = 0;
    // counter for online authorities at genesis
    let mut counter_online_authorities = 0;
    // track identity index
    let mut identity_index = HashMap::new();
    // counter for certifications
    let mut counter_cert = 0u32;
    // counter for smith certifications
    let mut counter_smith_cert = 0u32;
    // track inactive identities
    let mut inactive_identities = HashMap::<u32, &str>::new();

    // declare variables to fill in genesis
    // -------------------------------------
    // account inserted in genesis
    let mut accounts = BTreeMap::new();
    // members of technical committee
    let mut technical_committee_members = Vec::new();
    // memberships
    let mut memberships = BTreeMap::new();
    // identities
    let mut identities = Vec::new();
    // certifications
    let mut certs_by_receiver = BTreeMap::new();
    // initial authorities
    let mut initial_authorities = BTreeMap::new();
    // session keys
    let mut session_keys_map = BTreeMap::new();
    // smith memberships
    let mut smith_memberships = BTreeMap::new();
    // smith certifications
    let mut smith_certs_by_receiver = BTreeMap::new();

    // SIMPLE WALLETS //
    for (pubkey, balance) in &genesis_data.wallets {
        // check existential deposit
        if balance < &EXISTENTIAL_DEPOSIT {
            log::warn!("wallet {pubkey} has {balance} cǦT which is below {EXISTENTIAL_DEPOSIT}");
            fatal = true;
        }

        // double check the monetary mass
        monetary_mass += balance;

        wallet_index += 1;
        // json prevents duplicate wallets
        accounts.insert(
            pubkey.clone(),
            GenesisAccountData {
                random_id: H256(blake2_256(&(wallet_index, &pubkey).encode())),
                balance: *balance,
                is_identity: false,
            },
        );
    }

    // IDENTITIES //
    for (name, identity) in &genesis_data.identities {
        // identity name
        if !validate_idty_name(&name) {
            return Err(format!("Identity name '{}' is invalid", &name));
        }

        // check existential deposit
        if identity.balance < EXISTENTIAL_DEPOSIT {
            if identity.membership_expire_on != 0 {
                log::warn!(
                    "expired identity {name} has {} cǦT which is below {EXISTENTIAL_DEPOSIT}",
                    identity.balance
                );
                fatal = true;
            } else {
                // member identities can still be below existential deposit thanks to sufficient
                log::info!(
                    "identity {name} has {} cǦT which is below {EXISTENTIAL_DEPOSIT}",
                    identity.balance
                );
            }
        }

        // Money
        // check that wallet with same owner_key does not exist
        if accounts.get(&identity.owner_key).is_some() {
            log::warn!(
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
        monetary_mass += identity.balance;

        // insert identity
        // check that index does not already exist
        if let Some(other_name) = identity_index.get(&identity.index) {
            log::warn!(
                "{other_name} already has identity index {} of {name}",
                identity.index
            );
            fatal = true;
        }
        identity_index.insert(identity.index, name);

        // only add the identity if not expired
        if identity.membership_expire_on != 0 {
            identities.push(GenesisIdty {
                index: identity.index,
                name: common_runtime::IdtyName::from(name.as_str()),
                value: common_runtime::IdtyValue {
                    data: IdtyData::new(),
                    next_creatable_identity_on: identity.next_cert_issuable_on,
                    old_owner_key: match identity.old_owner_key.clone() {
                        Some(address) => Some((address, 0)), // FIXME old owner key expiration
                        None => None,
                    },
                    // old_owner_key: None,
                    owner_key: identity.owner_key.clone(),
                    // TODO remove the removable_on field of identity
                    removable_on: 0,
                    status: IdtyStatus::Validated,
                },
            });
        } else {
            inactive_identities.insert(identity.index, name);
        };

        // insert the membershup data (only if not expired)
        if identity.membership_expire_on != 0 {
            memberships.insert(
                identity.index,
                MembershipData {
                    expire_on: identity.membership_expire_on,
                },
            );
        }
    }

    // Technical Comittee //
    // NOTE : when changing owner key, the technical committee is not changed
    for name in &genesis_data.technical_committee {
        if let Some(identity) = &genesis_data.identities.get(name) {
            technical_committee_members.push(identity.owner_key.clone());
        } else {
            log::error!("Identity '{}' does not exist", name);
            fatal = true;
        }
    }

    // CERTIFICATIONS //
    for (_, identity) in &genesis_data.identities {
        let mut certs = BTreeMap::new();
        for (issuer, expire_on) in &identity.certs_received {
            if let Some(issuer) = &genesis_data.identities.get(issuer) {
                certs.insert(issuer.index, Some(expire_on.clone()));
                counter_cert += 1;
            } else {
                log::error!("Identity '{}' does not exist", issuer);
                fatal = true;
            };
        }
        certs_by_receiver.insert(identity.index, certs);
    }

    // SMITHS SUB-WOT //
    for (name, smith_data) in &genesis_data.smiths {
        // check that smith exists
        if let Some(identity) = &genesis_data.identities.get(&name.clone()) {
            // Initial authorities and session keys
            let session_keys_bytes = if let Some(declared_session_keys) = &smith_data.session_keys {
                counter_online_authorities += 1;
                // insert authority as online
                initial_authorities.insert(identity.index, (identity.owner_key.clone(), true));
                // decode session keys or force to given value
                match maybe_force_authority {
                    Some(ref bytes) => bytes.clone(),
                    None => hex::decode(&declared_session_keys[2..])
                        .map_err(|_| format!("invalid session keys for idty {}", &name))?,
                }
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
                SessionKeys::decode(&mut &session_keys_bytes[..]).unwrap(),
            );

            // smith certifications
            let mut certs = BTreeMap::new();
            for issuer in &smith_data.certs_received {
                let issuer_index = &genesis_data
                    .identities
                    .get(issuer)
                    .ok_or(format!("Identity '{}' does not exist", issuer))?
                    .index;
                certs.insert(*issuer_index, Some(SMITH_CERTS_EXPIRE_ON));
                counter_smith_cert += 1;
            }
            smith_certs_by_receiver.insert(identity.index, certs);

            // smith memberships
            smith_memberships.insert(
                identity.index,
                MembershipData {
                    expire_on: SMITH_MEMBERSHIP_EXPIRE_ON,
                },
            );
        } else {
            log::error!("Smith '{}' does not correspond to exising identity", &name);
            fatal = true;
        }
    }

    // Verify certifications coherence (can be ignored for old users)
    for (idty_index, receiver_certs) in &certs_by_receiver {
        if receiver_certs.len() < MIN_CERT as usize {
            let name = identity_index.get(idty_index).unwrap();
            let identity = genesis_data.identities.get(name.clone()).unwrap();
            if identity.membership_expire_on != 0 {
                log::warn!(
                    "[{}] has received only {}/{} certifications",
                    name,
                    receiver_certs.len(),
                    MIN_CERT
                );
                fatal = true;
            }
        }
    }

    // Verify smith certifications coherence
    for (idty_index, certs) in &smith_certs_by_receiver {
        if certs.len() < SMITH_MIN_CERT as usize {
            log::warn!(
                "[{}] has received only {}/{} smith certifications",
                identity_index.get(idty_index).unwrap(),
                certs.len(),
                SMITH_MIN_CERT
            );
            fatal = true;
        }
    }

    // check number of online authorities
    if counter_online_authorities != 1 {
        log::error!("one and only one smith must be online, not {counter_online_authorities}");
    }

    // check monetary mass
    if monetary_mass != genesis_data.initial_monetary_mass {
        log::warn!(
            "actuel monetary_mass ({monetary_mass}) and initial_monetary_mass ({}) do not match",
            genesis_data.initial_monetary_mass
        );
        fatal = true;
    }

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
        accounts.len(),
        identity_index.len(),
        &genesis_data.wallets.len(),
        identity_index.len(),
        identities.len(),
        inactive_identities.len(),
        smith_memberships.len(),
        counter_online_authorities,
        counter_cert,
        counter_smith_cert,
        technical_committee_members.len(),
    );

    // some more checks
    assert_eq!(identities.len(), memberships.len());
    assert_eq!(smith_memberships.len(), initial_authorities.len());
    assert_eq!(smith_memberships.len(), session_keys_map.len());
    assert_eq!(
        identity_index.len(),
        identities.len() + inactive_identities.len()
    );
    assert_eq!(
        accounts.len(),
        identity_index.len() + &genesis_data.wallets.len()
    );
    // no inactive tech comm
    for tech_com_member in &genesis_data.technical_committee {
        assert!(!inactive_identities.values().any(|&v| v == tech_com_member));
    }
    // no inactive smith
    for (smith, _) in &genesis_data.smiths {
        assert!(!inactive_identities.values().any(|&v| v == smith));
    }

    // check the logs to see all the fatal error preventing from starting gtest currency
    if fatal {
        log::error!("some previously logged error prevent from building a sane genesis");
        panic!();
    }

    // return genesis config
    Ok(gtest_runtime::GenesisConfig {
        system: SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
        },
        account: AccountConfig { accounts },
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
        sudo: SudoConfig {
            key: genesis_data.sudo_key,
        },
        technical_committee: TechnicalCommitteeConfig {
            members: technical_committee_members,
            ..Default::default()
        },
        identity: IdentityConfig { identities },
        cert: CertConfig {
            apply_cert_period_at_genesis: false,
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
            first_reeval: genesis_data.first_ud_reeval,
            first_ud: genesis_data.first_ud,
            initial_monetary_mass: genesis_data.initial_monetary_mass,
        },
        treasury: Default::default(),
    })
}
