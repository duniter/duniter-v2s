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

use common_runtime::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sp_core::{blake2_256, Decode, Encode, H256};
use std::collections::BTreeMap;

type MembershipData = sp_membership::MembershipData<u32>;

const EXISTENTIAL_DEPOSIT: u64 = 100;

#[derive(Clone)]
pub struct GenesisData<Parameters: DeserializeOwned, SessionKeys: Decode> {
    pub accounts: BTreeMap<AccountId, GenesisAccountData<u64>>,
    pub certs_by_receiver: BTreeMap<u32, BTreeMap<u32, u32>>,
    pub first_ud: u64,
    pub first_ud_reeval: u32,
    pub identities: Vec<(String, AccountId)>,
    pub initial_authorities: BTreeMap<u32, (AccountId, bool)>,
    pub initial_monetary_mass: u64,
    pub memberships: BTreeMap<u32, MembershipData>,
    pub parameters: Parameters,
    pub session_keys_map: BTreeMap<AccountId, SessionKeys>,
    pub smiths_certs_by_receiver: BTreeMap<u32, BTreeMap<u32, u32>>,
    pub smiths_memberships: BTreeMap<u32, MembershipData>,
    pub sudo_key: Option<AccountId>,
}

#[derive(Default)]
pub struct ParamsAppliedAtGenesis {
    pub genesis_certs_expire_on: u32,
    pub genesis_smith_certs_expire_on: u32,
    pub genesis_memberships_expire_on: u32,
    pub genesis_smith_memberships_expire_on: u32,
}

#[derive(Deserialize, Serialize)]
struct GenesisConfig<Parameters> {
    first_ud: u64,
    first_ud_reeval: u32,
    identities: BTreeMap<String, Idty>,
    #[serde(default)]
    parameters: Parameters,
    #[serde(rename = "smiths")]
    smith_identities: BTreeMap<String, SmithData>,
    sudo_key: Option<AccountId>,
    #[serde(default)]
    wallets: BTreeMap<AccountId, u64>,
}

#[derive(Clone, Deserialize, Serialize)]
struct Idty {
    #[serde(default)]
    balance: u64,
    #[serde(default)]
    certs: Vec<String>,
    #[serde(rename = "expire_on")]
    membership_expire_on: Option<u64>,
    pubkey: AccountId,
}

#[derive(Clone, Deserialize, Serialize)]
struct SmithData {
    session_keys: Option<String>,
    #[serde(default)]
    certs: Vec<String>,
}

pub fn generate_genesis_data<CS, P, SK, F>(
    f: F,
    maybe_force_authority: Option<Vec<u8>>,
    params_applied_at_genesis: Option<ParamsAppliedAtGenesis>,
) -> Result<CS, String>
where
    P: Default + DeserializeOwned,
    SK: Decode,
    F: Fn(GenesisData<P, SK>) -> CS,
{
    let ParamsAppliedAtGenesis {
        genesis_certs_expire_on,
        genesis_smith_certs_expire_on,
        genesis_memberships_expire_on,
        genesis_smith_memberships_expire_on,
    } = params_applied_at_genesis.unwrap_or_default();

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

    let json_file_path = std::env::var("DUNITER_GENESIS_CONFIG")
        .unwrap_or_else(|_| "duniter-gen-conf.json".to_owned());

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

    let genesis_config = serde_json::from_slice(&bytes)
        .map_err(|e| format!("Error parsing gen conf file: {}", e))?;
    let GenesisConfig {
        sudo_key,
        first_ud,
        first_ud_reeval,
        parameters,
        identities,
        smith_identities,
        wallets,
    } = genesis_config;

    // MONEY AND WOT //

    let mut accounts = BTreeMap::new();
    let mut identities_ = Vec::with_capacity(identities.len());
    let mut idty_index: u32 = 1;
    let mut idty_index_of = BTreeMap::new();
    let mut initial_monetary_mass = 0;
    let mut memberships = BTreeMap::new();
    //let mut total_dust = 0;

    // SIMPLE WALLETS //

    let mut wallet_index: u32 = 0;
    for (pubkey, balance) in wallets {
        wallet_index += 1;
        accounts.insert(
            pubkey.clone(),
            GenesisAccountData {
                random_id: H256(blake2_256(&(wallet_index, &pubkey).encode())),
                balance,
                is_identity: false,
            },
        );
    }

    // IDENTITIES //

    for (idty_name, identity) in &identities {
        if !validate_idty_name(idty_name) {
            return Err(format!("Identity name '{}' is invalid", &idty_name));
        }

        // Money
        let balance = if identity.balance >= EXISTENTIAL_DEPOSIT {
            identity.balance
        } else {
            //total_dust += identity.balance;
            0
        };
        accounts.insert(
            identity.pubkey.clone(),
            GenesisAccountData {
                random_id: H256(blake2_256(&(idty_index, &identity.pubkey).encode())),
                balance,
                is_identity: true,
            },
        );

        // We must count the money under the existential deposit because what we count is
        // the monetary mass created (for the revaluation of the DU)
        initial_monetary_mass += identity.balance;

        // Wot
        identities_.push((idty_name.clone(), identity.pubkey.clone()));
        memberships.insert(
            idty_index,
            MembershipData {
                expire_on: identity
                    .membership_expire_on
                    .map_or(genesis_memberships_expire_on, |expire_on| {
                        to_bn(genesis_timestamp, expire_on)
                    }),
            },
        );

        // Identity index
        idty_index_of.insert(idty_name, idty_index);
        idty_index += 1;
    }

    // CERTIFICATIONS //

    let mut certs_by_receiver = BTreeMap::new();
    for (idty_name, identity) in &identities {
        let issuer_index = idty_index_of
            .get(&idty_name)
            .ok_or(format!("Identity '{}' not exist", &idty_name))?;
        let mut receiver_certs = BTreeMap::new();
        for issuer in &identity.certs {
            let issuer_index = idty_index_of
                .get(issuer)
                .ok_or(format!("Identity '{}' not exist", issuer))?;
            receiver_certs.insert(*issuer_index, genesis_certs_expire_on);
        }
        certs_by_receiver.insert(*issuer_index, receiver_certs);
    }

    // SMITHS SUB-WOT //

    let mut initial_authorities = BTreeMap::new();
    let mut online_authorities_counter = 0;
    let mut session_keys_map = BTreeMap::new();
    let mut smiths_memberships = BTreeMap::new();
    let mut smiths_certs_by_receiver = BTreeMap::new();
    for (idty_name, smith_data) in smith_identities {
        let idty_index = idty_index_of
            .get(&idty_name)
            .ok_or(format!("Identity '{}' not exist", &idty_name))?;
        let identity = identities
            .get(&idty_name)
            .ok_or(format!("Identity '{}' not exist", &idty_name))?;

        if identity.balance < EXISTENTIAL_DEPOSIT {
            return Err(format!(
                "Identity '{}' have balance '{}' < EXISTENTIAL_DEPOSIT",
                idty_name, identity.balance,
            ));
        }

        // Initial authorities
        if maybe_force_authority.is_some() {
            if smith_data.session_keys.is_some() {
                return Err("session_keys field forbidden".to_owned());
            }
            if *idty_index == 1 {
                initial_authorities.insert(1, (identity.pubkey.clone(), true));
            }
        } else {
            initial_authorities.insert(
                *idty_index,
                (identity.pubkey.clone(), smith_data.session_keys.is_some()),
            );
        }

        // Session keys
        let session_keys_bytes = if let Some(ref session_keys) = smith_data.session_keys {
            online_authorities_counter += 1;
            hex::decode(&session_keys[2..])
                .map_err(|_| format!("invalid session keys for idty {}", &idty_name))?
        } else if let (1, Some(ref session_keys_bytes)) = (*idty_index, &maybe_force_authority) {
            session_keys_bytes.clone()
        } else {
            // Create fake session keys (must be unique and deterministic)
            let mut fake_session_keys_bytes = Vec::with_capacity(128);
            for _ in 0..4 {
                fake_session_keys_bytes.extend_from_slice(identity.pubkey.as_ref())
            }
            fake_session_keys_bytes
            //vec![initial_authorities.len() as u8; std::mem::size_of::<SK>()]
        };
        session_keys_map.insert(
            identity.pubkey.clone(),
            SK::decode(&mut &session_keys_bytes[..])
                .map_err(|_| format!("invalid session keys for idty {}", &idty_name))?,
        );

        // Certifications
        let mut receiver_certs = BTreeMap::new();
        for receiver in &smith_data.certs {
            let issuer_index = idty_index_of
                .get(receiver)
                .ok_or(format!("Identity '{}' not exist", receiver))?;
            receiver_certs.insert(*issuer_index, genesis_smith_certs_expire_on);
        }
        smiths_certs_by_receiver.insert(*idty_index, receiver_certs);

        // Memberships
        smiths_memberships.insert(
            *idty_index,
            MembershipData {
                expire_on: genesis_smith_memberships_expire_on,
            },
        );
    }

    if maybe_force_authority.is_none() && online_authorities_counter == 0 {
        return Err("The session_keys field must be filled in for at least one smith.".to_owned());
    }

    let genesis_data = GenesisData {
        accounts,
        certs_by_receiver,
        first_ud,
        first_ud_reeval,
        identities: identities_,
        initial_authorities,
        initial_monetary_mass,
        memberships,
        parameters,
        session_keys_map,
        smiths_certs_by_receiver,
        smiths_memberships,
        sudo_key,
    };

    Ok(f(genesis_data))
}

// Timestamp to block number
fn to_bn(genesis_timestamp: u64, timestamp: u64) -> u32 {
    let duration_in_secs = timestamp.saturating_sub(genesis_timestamp);
    (duration_in_secs / 6) as u32
}

fn validate_idty_name(name: &str) -> bool {
    name.len() <= 64
}
