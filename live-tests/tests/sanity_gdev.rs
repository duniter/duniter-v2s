// Copyright 2021-2022 Axiom-Team
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

#[subxt::subxt(runtime_metadata_path = "../resources/gdev_metadata.scale")]
pub mod gdev {}

use countmap::CountMap;
use sp_core::{ByteArray, H256, blake2_128, crypto::AccountId32};
use std::collections::{HashMap, HashSet};
use subxt::{backend::rpc::RpcClient, config::SubstrateConfig as GdevConfig};

const DEFAULT_ENDPOINT: &str = "ws://localhost:9944";

const EXISTENTIAL_DEPOSIT: u64 = 100;
//use hex_literal::hex;
//const TREASURY_ACCOUNT_ID: [u8; 32] =
//    hex!("6d6f646c70792f74727372790000000000000000000000000000000000000000");

type Client = subxt::OnlineClient<GdevConfig>;

// define gdev basic types
type Balance = u64;
type BlockNumber = u32;
type Index = u32;

// Define gdev types
type AccountInfo = gdev::runtime_types::frame_system::AccountInfo<
    Index,
    gdev::runtime_types::pallet_duniter_account::types::AccountData<Balance, IdtyIndex>,
>;
type IdtyData = gdev::runtime_types::common_runtime::entities::IdtyData;
type IdtyIndex = u32;
type IdtyValue =
    gdev::runtime_types::pallet_identity::types::IdtyValue<BlockNumber, AccountId32, IdtyData>;
type MembershipData = gdev::runtime_types::sp_membership::MembershipData<BlockNumber>;
use gdev::runtime_types::pallet_identity::types::{IdtyName, IdtyStatus};

struct Storage {
    accounts: HashMap<AccountId32, AccountInfo>,
    identities: HashMap<IdtyIndex, IdtyValue>,
    identity_index_of: HashMap<[u8; 16], IdtyIndex>,
    memberships: HashMap<IdtyIndex, MembershipData>,
    identities_names: HashMap<IdtyIndex, IdtyName>,
}

#[tokio::test(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let ws_rpc_endpoint =
        std::env::var("WS_RPC_ENDPOINT").unwrap_or_else(|_| DEFAULT_ENDPOINT.to_owned());
    let rpc = RpcClient::from_url(ws_rpc_endpoint)
        .await
        .expect("Failed to create the rpc backend");
    let client = Client::from_rpc_client(rpc.clone()).await.unwrap();

    let maybe_block_hash = if let Ok(block_number) = std::env::var("AT_BLOCK_NUMBER") {
        let block_number: BlockNumber = block_number.parse()?;
        println!("Run sanity tests against ĞDev at block #{block_number}.");
        // FIXME
        // client.at(block_number).await?
        None
    } else {
        println!("Run sanity tests against ĞDev at last best block");
        None
    };

    sanity_tests_at(client, maybe_block_hash).await
}

async fn sanity_tests_at(client: Client, _maybe_block_hash: Option<H256>) -> anyhow::Result<()> {
    // ===== Collect storage ===== //

    // Collect accounts
    let mut accounts: HashMap<AccountId32, AccountInfo> = HashMap::new();
    let mut account_iter = client
        .storage()
        .at_latest()
        .await
        .unwrap()
        .iter(gdev::storage().system().account_iter())
        .await?;
    while let Some(Ok(key)) = account_iter.next().await {
        let mut account_id_bytes = [0u8; 32];
        account_id_bytes.copy_from_slice(&key.key_bytes[48..]);
        accounts.insert(AccountId32::new(account_id_bytes), key.value);
    }
    println!("accounts.len(): {}.", accounts.len());

    // Collect identities
    let mut identities: HashMap<IdtyIndex, IdtyValue> = HashMap::new();
    let mut idty_iter = client
        .storage()
        .at_latest()
        .await
        .unwrap()
        .iter(gdev::storage().identity().identities_iter())
        .await?;
    while let Some(Ok(key)) = idty_iter.next().await {
        let mut idty_index_bytes = [0u8; 4];
        idty_index_bytes.copy_from_slice(&key.key_bytes[40..]);
        let idty_val = IdtyValue {
            data: key.value.data,
            next_creatable_identity_on: key.value.next_creatable_identity_on,
            old_owner_key: None, // Not used in the live test, skip the conversion
            owner_key: AccountId32::from(key.value.owner_key.0),
            next_scheduled: key.value.next_scheduled,
            status: key.value.status,
        };
        identities.insert(IdtyIndex::from_le_bytes(idty_index_bytes), idty_val);
    }
    println!("identities.len(): {}.", identities.len());

    // Collect identity_index_of
    let mut identity_index_of: HashMap<[u8; 16], IdtyIndex> = HashMap::new();
    let mut idty_index_of_iter = client
        .storage()
        .at_latest()
        .await
        .unwrap()
        .iter(gdev::storage().identity().identity_index_of_iter())
        .await?;
    while let Some(Ok(key)) = idty_index_of_iter.next().await {
        let mut blake2_128_bytes = [0u8; 16];
        blake2_128_bytes.copy_from_slice(&key.key_bytes[32..48]);
        identity_index_of.insert(blake2_128_bytes, key.value);
    }
    println!("identity_index_of.len(): {}.", identity_index_of.len());

    // Collect identity_names
    let mut identities_names: HashMap<IdtyIndex, IdtyName> = HashMap::new();
    let mut idty_name_iter = client
        .storage()
        .at_latest()
        .await
        .unwrap()
        .iter(gdev::storage().identity().identities_names_iter())
        .await?;
    while let Some(Ok(key)) = idty_name_iter.next().await {
        let name = IdtyName(key.key_bytes);
        identities_names.insert(key.value, name);
    }
    println!("identities_names.len(): {}.", identities_names.len());

    // Collect memberships
    let mut memberships: HashMap<IdtyIndex, MembershipData> = HashMap::new();
    let mut membership_iter = client
        .storage()
        .at_latest()
        .await
        .unwrap()
        .iter(gdev::storage().membership().membership_iter())
        .await?;
    while let Some(Ok(key)) = membership_iter.next().await {
        let mut idty_index_bytes = [0u8; 4];
        idty_index_bytes.copy_from_slice(&key.key_bytes[40..]);
        let membership_val = MembershipData {
            expire_on: key.value.expire_on,
        };
        memberships.insert(IdtyIndex::from_le_bytes(idty_index_bytes), membership_val);
    }
    println!("memberships.len(): {}.", memberships.len());

    let storage = Storage {
        accounts,
        identities,
        identity_index_of,
        memberships,
        identities_names,
    };

    // ===== Verify storage ===== //

    verifier::Verifier::new().verify_storage(&storage).await
}

mod verifier {
    use super::*;

    pub(super) struct Verifier {
        errors: Vec<String>,
    }

    impl Verifier {
        pub(super) fn new() -> Self {
            Self { errors: Vec::new() }
        }

        /// method to run all storage tests
        pub(super) async fn verify_storage(&mut self, storage: &Storage) -> anyhow::Result<()> {
            self.verify_accounts(&storage.accounts).await;
            self.verify_identities(&storage.accounts, &storage.identities)
                .await;
            self.verify_identity_index_of(&storage.identities, &storage.identity_index_of)
                .await;
            self.verify_identity_coherence(&storage.identities, &storage.identity_index_of)
                .await;
            self.verify_status_coherence(
                &storage.identities,
                &storage.memberships,
                &storage.identities_names,
            )
            .await;

            if self.errors.is_empty() {
                Ok(())
            } else {
                for error in &self.errors {
                    println!("{error}");
                }
                Err(anyhow::anyhow!(
                    "Storage corrupted: {} errors.",
                    self.errors.len()
                ))
            }
        }

        /// assert method to collect errors
        fn assert(&mut self, assertion: bool, error: String) {
            if !assertion {
                self.errors.push(error);
            }
        }

        /// like assert but just push error
        fn error(&mut self, error: String) {
            self.errors.push(error);
        }

        /// check accounts sufficients and consumers (specific to duniter-account pallet)
        async fn verify_accounts(&mut self, accounts: &HashMap<AccountId32, AccountInfo>) {
            for (account_id, account_info) in accounts {
                if account_info.sufficients == 0 {
                    // Rule 1: If the account is not sufficient, it should have at least one provider
                    self.assert(
                        account_info.providers > 0,
                        format!("Account {account_id} has no providers nor sufficients."),
                    );
                    // Rule 2: If the account is not sufficient, it should comply to the existential deposit
                    self.assert(
                        (account_info.data.free + account_info.data.reserved)
                            >= EXISTENTIAL_DEPOSIT,
                        format!("Account {account_id} not respect existential deposit rule."),
                    );
                }

                // Rule 3: If the account have consumers, it should have at least one provider
                if account_info.consumers > 0 {
                    // Rule 1: If the account is not sufficient [...]
                    self.assert(
                        account_info.providers > 0,
                        format!("Account {account_id} has no providers nor sufficients."),
                    );
                }
            }
        }

        /// check list of identities (account existence, sufficient)
        async fn verify_identities(
            &mut self,
            accounts: &HashMap<AccountId32, AccountInfo>,
            identities: &HashMap<IdtyIndex, IdtyValue>,
        ) {
            // counts occurence of owner key
            let mut countmap = CountMap::<AccountId32, u8>::new();
            // list owner key with multiple occurences
            let mut duplicates = HashSet::new();

            for (idty_index, idty_value) in identities {
                countmap.insert_or_increment(idty_value.owner_key.clone());
                if let Some(count) = countmap.get_count(&idty_value.owner_key)
                    && count > 1
                {
                    self.error(format!(
                        "address {} is the owner_key of {count} identities",
                        idty_value.owner_key
                    ));
                    if count == 2 {
                        duplicates.insert(idty_value.owner_key.clone());
                    }
                }

                // Rule 1: each identity should have an account
                let maybe_account = accounts.get(&idty_value.owner_key);
                self.assert(
                    maybe_account.is_some(),
                    format!("Identity {idty_index} has no account."),
                );

                if let Some(account) = maybe_account {
                    // Rule 2: each identity account should be sufficient
                    self.assert(
                        account.sufficients > 0,
                        format!(
                            "Identity {idty_index} is corrupted: idty_account.sufficients == 0"
                        ),
                    );
                }
            }

            for (idty_index, idty_value) in identities {
                if duplicates.contains(&idty_value.owner_key) {
                    self.error(format!(
                        "duplicate key {} at position {idty_index}",
                        idty_value.owner_key
                    ));
                }
            }
        }

        /// check the identity hashmap (length, identity existence, hash matches owner key)
        async fn verify_identity_index_of(
            &mut self,
            identities: &HashMap<IdtyIndex, IdtyValue>,
            identity_index_of: &HashMap<[u8; 16], IdtyIndex>,
        ) {
            // Rule1: identity_index_of should have the same lenght as identities
            self.assert(
                identities.len() == identity_index_of.len(),
                format!(
                    "identities.len({}) != identity_index_of.len({}).",
                    identities.len(),
                    identity_index_of.len()
                ),
            );

            for (blake2_128_owner_key, idty_index) in identity_index_of {
                let maybe_idty_value = identities.get(idty_index);

                // Rule2: Each identity_index_of should point to an existing identity
                self.assert(
                    maybe_idty_value.is_some(),
                    format!(
                        "Identity {idty_index} not exist, but still referenced in IdentityIndexOf."
                    ),
                );

                if let Some(idty_value) = maybe_idty_value {
                    // Rule3: identity_index_of key should correspond to the blake2_12- hash of
                    // identity owner key
                    self.assert(
                        blake2_128_owner_key == &blake2_128(idty_value.owner_key.as_slice()),
                        format!("Identity {idty_index} is referenced in IdentityIndexOf with an invalid key hash."),
                    );
                }
            }
        }

        /// check identities status and membership coherence
        async fn verify_status_coherence(
            &mut self,
            identities: &HashMap<IdtyIndex, IdtyValue>,
            memberships: &HashMap<IdtyIndex, MembershipData>,
            names: &HashMap<IdtyIndex, IdtyName>,
        ) {
            for (idty_index, idty_value) in identities {
                // Rule 1: each Status::Member
                // should have a membership and a name
                // membership should be set to expire
                // identity should have no scheduled action
                if let IdtyStatus::Member = idty_value.status {
                    self.assert(
                        memberships.get(idty_index).is_some(),
                        format!("identity number {idty_index} should have a valid membership"),
                    );
                    self.assert(
                        names.get(idty_index).is_some(),
                        format!("identity number {idty_index} should have a name"),
                    );
                    self.assert(
                        memberships.get(idty_index).unwrap().expire_on != 0,
                        format!(
                            "Member identity number {idty_index} should have a non-null expire_on value"
                        ),
                    );
                    self.assert(
                        identities.get(idty_index).unwrap().next_scheduled == 0,
                        format!(
                            "Member identity number {idty_index} should have a null next_scheduled value"
                        ),
                    );
                }

                // Rule 2: each Status::NotMember
                // should have a name but no membership
                // should have a scheduled action (auto-revocation)
                if let IdtyStatus::NotMember = idty_value.status {
                    self.assert(
                        memberships.get(idty_index).is_none(),
                        format!("identity number {idty_index} should not have a valid membership"),
                    );
                    self.assert(
                        names.get(idty_index).is_some(),
                        format!("identity number {idty_index} should have a name"),
                    );
                    self.assert(
                        identities.get(idty_index).unwrap().next_scheduled != 0,
                        format!("NotMember identity number {idty_index} should have a non-null next_scheduled value"),
                    );
                }

                // Rule 3: each Status::Revoked
                // should should have a name
                // no membership
                // should be scheduled for removal
                if let IdtyStatus::Revoked = idty_value.status {
                    self.assert(
                        memberships.get(idty_index).is_none(),
                        format!("identity number {idty_index} should not have a valid membership"),
                    );
                    self.assert(
                        names.get(idty_index).is_some(),
                        format!("identity number {idty_index} should have a name"),
                    );
                    self.assert(
                        identities.get(idty_index).unwrap().next_scheduled != 0,
                        format!("Revoked identity number {idty_index} should have a non-null next_scheduled value"),
                    );
                }

                // Rule 4: each Status::Unvalidaded
                // should have a name but no membership.
                // should be scheduled for removal
                if let IdtyStatus::Unvalidated = idty_value.status {
                    self.assert(
                        memberships.get(idty_index).is_none(),
                        format!("identity number {idty_index} should not have a valid membership"),
                    );
                    self.assert(
                        names.get(idty_index).is_some(),
                        format!("identity number {idty_index} should have a name"),
                    );
                    self.assert(
                        identities.get(idty_index).unwrap().next_scheduled != 0,
                        format!("Unvalidated identity number {idty_index} should have a non-null next_scheduled value"),
                    );
                }

                // Rule 5: each Status::Unconfirmed
                // should not have a name neither a membership.
                // should be scheduled for removal soon
                if let IdtyStatus::Unconfirmed = idty_value.status {
                    self.assert(
                        memberships.get(idty_index).is_none(),
                        format!("identity number {idty_index} should not have a valid membership"),
                    );
                    self.assert(
                        names.get(idty_index).is_none(),
                        format!("identity number {idty_index} should not have a name"),
                    );
                    self.assert(
                        identities.get(idty_index).unwrap().next_scheduled != 0,
                        format!("Unconfirmed identity number {idty_index} should have a non-null next_scheduled value"),
                    );
                }
            }
        }

        /// check coherence between identity list and identity index hashmap
        async fn verify_identity_coherence(
            &mut self,
            identities: &HashMap<IdtyIndex, IdtyValue>,
            identity_index_of: &HashMap<[u8; 16], IdtyIndex>,
        ) {
            // each identity should be correcly referenced in the hashmap
            for (idty_index, idty_value) in identities {
                // hash owner key to get key
                let blake2_128_owner_key = &blake2_128(idty_value.owner_key.as_slice());

                // get identity index from hashmap
                if let Some(index_of) = identity_index_of.get(blake2_128_owner_key) {
                    self.assert(idty_index == index_of,
                        format!("identity number {idty_index} with owner key {0} is mapped to identity index {index_of}", idty_value.owner_key));
                } else {
                    self.error(format!(
                        "identity with owner key {} is not present in hashmap",
                        idty_value.owner_key
                    ));
                }
            }
        }
    }
}
