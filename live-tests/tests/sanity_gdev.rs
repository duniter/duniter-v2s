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

// WARNING
// these live test build but I'm not sure they actually test something
// they should be checked against corrupted storage to see that they actually fail

#[subxt::subxt(runtime_metadata_path = "../resources/metadata.scale")]
pub mod gdev {}

use countmap::CountMap;
use hex_literal::hex;
use sp_core::crypto::AccountId32;
use sp_core::{blake2_128, ByteArray, H256};
use std::collections::{HashMap, HashSet};
use subxt::backend::rpc::RpcClient;
use subxt::config::SubstrateConfig as GdevConfig;
use subxt::ext::sp_core;
// use subxt::config::substrate::SubstrateExtrinsicParamsBuilder;
// use subxt::backend::rpc::RpcParams;
// use subxt::config::SubstrateExtrinsicParams;
// use subxt::ext::{sp_core, sp_runtime};

const DEFAULT_ENDPOINT: &str = "ws://localhost:9944";

const EXISTENTIAL_DEPOSIT: u64 = 100;
const TREASURY_ACCOUNT_ID: [u8; 32] =
    hex!("6d6f646c70792f74727372790000000000000000000000000000000000000000");

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
// use gdev::runtime_types::pallet_identity::types::IdtyStatus;

struct Storage {
    accounts: HashMap<AccountId32, AccountInfo>,
    identities: HashMap<IdtyIndex, IdtyValue>,
    identity_index_of: HashMap<[u8; 16], IdtyIndex>,
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
        println!("Run sanity tests against ĞDev at block #{}.", block_number);
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
    while let Some(Ok((key, account_info))) = account_iter.next().await {
        let mut account_id_bytes = [0u8; 32];
        account_id_bytes.copy_from_slice(&key[48..]);
        accounts.insert(AccountId32::new(account_id_bytes), account_info);
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
    while let Some(Ok((key, idty_value))) = idty_iter.next().await {
        let mut idty_index_bytes = [0u8; 4];
        idty_index_bytes.copy_from_slice(&key[40..]);
        let idty_val = IdtyValue {
            data: idty_value.data,
            next_creatable_identity_on: idty_value.next_creatable_identity_on,
            old_owner_key: None, // Not used in the live test, skip the conversion
            owner_key: AccountId32::from(idty_value.owner_key.0),
            next_scheduled: idty_value.next_scheduled,
            status: idty_value.status,
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
    while let Some(Ok((key, idty_index))) = idty_index_of_iter.next().await {
        let mut blake2_128_bytes = [0u8; 16];
        blake2_128_bytes.copy_from_slice(&key[32..48]);
        identity_index_of.insert(blake2_128_bytes, idty_index);
    }
    println!("identity_index_of.len(): {}.", identity_index_of.len());

    let storage = Storage {
        accounts,
        identities,
        identity_index_of,
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

            if self.errors.is_empty() {
                Ok(())
            } else {
                for error in &self.errors {
                    println!("{}", error);
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
                        format!("Account {} has no providers nor sufficients.", account_id),
                    );
                    // Rule 2: If the account is not sufficient, it should comply to the existential deposit
                    self.assert(
                        (account_info.data.free + account_info.data.reserved)
                            >= EXISTENTIAL_DEPOSIT,
                        format!(
                            "Account {} not respect existential deposit rule.",
                            account_id
                        ),
                    );
                }

                // Rule 3: If the account have consumers, it should have at least one provider
                if account_info.consumers > 0 {
                    // Rule 1: If the account is not sufficient [...]
                    self.assert(
                        account_info.providers > 0,
                        format!("Account {} has no providers nor sufficients.", account_id),
                    );
                }

                if account_id.as_slice() != TREASURY_ACCOUNT_ID {
                    // Rule 4: If the account is not a "special account",
                    // it should have a consumer
                    self.assert(
                        account_info.consumers > 0,
                        format!("Account {} has no consumer.", account_id),
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
                if let Some(count) = countmap.get_count(&idty_value.owner_key) {
                    if count > 1 {
                        self.error(format!(
                            "address {} is the owner_key of {count} identities",
                            idty_value.owner_key
                        ));
                        if count == 2 {
                            duplicates.insert(idty_value.owner_key.clone());
                        }
                    }
                }

                // Rule 1: each identity should have an account
                let maybe_account = accounts.get(&idty_value.owner_key);
                self.assert(
                    maybe_account.is_some(),
                    format!("Identity {} has no account.", idty_index),
                );

                if let Some(account) = maybe_account {
                    // Rule 2: each identity account should be sufficient
                    self.assert(
                        account.sufficients > 0,
                        format!(
                            "Identity {} is corrupted: idty_account.sufficients == 0",
                            idty_index
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
                        "Identity {} not exist, but still referenced in IdentityIndexOf.",
                        idty_index
                    ),
                );

                if let Some(idty_value) = maybe_idty_value {
                    // Rule3: identity_index_of key should correspond to the blake2_12- hash of
                    // identity owner key
                    self.assert(
                        blake2_128_owner_key == &blake2_128(idty_value.owner_key.as_slice()),
                        format!(
                            "Identity {} is referenced in IdentityIndexOf with an invalid key hash.",
                            idty_index
                        ),
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
