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

#[subxt::subxt(runtime_metadata_path = "../resources/metadata.scale")]
pub mod gdev_runtime {}

use hex_literal::hex;
use sp_core::crypto::AccountId32;
use sp_core::{ByteArray, H256};
use std::collections::HashMap;
use subxt::{ClientBuilder, DefaultConfig, DefaultExtra};

const DEFAULT_ENDPOINT: &str = "wss://gdev.librelois.fr:443/ws";

const TREASURY_ACCOUNT_ID: [u8; 32] =
    hex!("6d6f646c70792f74727372790000000000000000000000000000000000000000");

type Api = gdev_runtime::RuntimeApi<DefaultConfig, DefaultExtra<DefaultConfig>>;
type Client = subxt::Client<DefaultConfig>;

// define gdev basic types
type Balance = u64;
type BlockNumber = u32;
type Index = u32;

// Define gdev types
type AccountInfo = gdev_runtime::runtime_types::frame_system::AccountInfo<
    Index,
    gdev_runtime::runtime_types::pallet_duniter_account::types::AccountData<Balance>,
>;
type IdtyIndex = u32;
type IdtyValue =
    gdev_runtime::runtime_types::pallet_identity::types::IdtyValue<BlockNumber, AccountId32>;
use gdev_runtime::runtime_types::pallet_identity::types::IdtyStatus;

struct Storage {
    accounts: HashMap<AccountId32, AccountInfo>,
    identities: HashMap<IdtyIndex, IdtyValue>,
}

#[tokio::test(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let ws_rpc_endpoint =
        std::env::var("WS_RPC_ENDPOINT").unwrap_or_else(|_| DEFAULT_ENDPOINT.to_owned());
    let client: Client = ClientBuilder::new()
        .set_url(ws_rpc_endpoint)
        .set_page_size(100)
        .build()
        .await
        .expect("fail to connect to node");

    let maybe_block_hash = if let Ok(block_number) = std::env::var("AT_BLOCK_NUMBER") {
        let block_number: BlockNumber = block_number.parse()?;
        println!("Run sanity tests against ĞDev at block #{}", block_number);
        client.rpc().block_hash(Some(block_number.into())).await?
    } else {
        println!("Run sanity tests against ĞDev at last best block");
        None
    };

    sanity_tests_at(client, maybe_block_hash).await
}

async fn sanity_tests_at(client: Client, maybe_block_hash: Option<H256>) -> anyhow::Result<()> {
    // Get API
    let api = client.clone().to_runtime_api::<Api>();

    // ===== Collect storage ===== //

    // Collect accounts
    let mut accounts = HashMap::new();
    let mut account_iter = api
        .storage()
        .system()
        .account_iter(maybe_block_hash)
        .await?;
    while let Some((key, account_info)) = account_iter.next().await? {
        let mut account_id_bytes = [0u8; 32];
        account_id_bytes.copy_from_slice(&key.0[48..]);
        accounts.insert(AccountId32::new(account_id_bytes), account_info);
    }
    println!("{} accounts.", accounts.len());

    // Collect identities
    let mut identities = HashMap::new();
    let mut idty_iter = api
        .storage()
        .identity()
        .identities_iter(maybe_block_hash)
        .await?;
    while let Some((key, idty_value)) = idty_iter.next().await? {
        let mut idty_index_bytes = [0u8; 4];
        idty_index_bytes.copy_from_slice(&key.0[40..]);
        identities.insert(IdtyIndex::from_le_bytes(idty_index_bytes), idty_value);
    }
    println!("{} identities.", identities.len());

    let storage = Storage {
        accounts,
        identities,
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

        pub(super) async fn verify_storage(&mut self, storage: &Storage) -> anyhow::Result<()> {
            self.verify_accounts(&storage.accounts).await?;
            self.verify_identities(&storage.accounts, &storage.identities)
                .await?;

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

        fn assert(&mut self, assertion: bool, error: String) {
            if !assertion {
                self.errors.push(error);
            }
        }

        async fn verify_accounts(
            &mut self,
            accounts: &HashMap<AccountId32, AccountInfo>,
        ) -> anyhow::Result<()> {
            for (account_id, account_info) in accounts {
                if account_info.sufficients == 0 {
                    // Rule 1: If the account is not sufficient, it should have at least one provider
                    self.assert(
                        account_info.providers > 0,
                        format!("Account {} has no providers nor sufficients", account_id),
                    );
                    // Rule 2: If the account is not sufficient, it should comply to the existential deposit
                    self.assert(
                        (account_info.data.free + account_info.data.reserved) >= 200,
                        format!(
                            "Account {} not respect existential deposit rule",
                            account_id
                        ),
                    );
                }

                // Rule 3: If the account have consumers, it shoul have at least one provider
                if account_info.consumers > 0 {
                    // Rule 1: If the account is not s
                    self.assert(
                        account_info.providers > 0,
                        format!("Account {} has no providers nor sufficients", account_id),
                    );
                }

                if account_id.as_slice() != TREASURY_ACCOUNT_ID {
                    // Rule 4: If the account is not a "special account",
                    // it should have a random id or a consumer
                    self.assert(
                        account_info.data.random_id.is_some() || account_info.consumers > 0,
                        format!("Account {} has no random_id nor consumer", account_id),
                    );
                }
            }
            Ok(())
        }

        async fn verify_identities(
            &mut self,
            accounts: &HashMap<AccountId32, AccountInfo>,
            identities: &HashMap<IdtyIndex, IdtyValue>,
        ) -> anyhow::Result<()> {
            for (idty_index, idty_value) in identities {
                // Rule 1: each identity should have an account
                let maybe_account = accounts.get(&idty_value.owner_key);
                self.assert(
                    maybe_account.is_some(),
                    format!("Identity {} has no account", idty_index),
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

                match idty_value.status {
                    IdtyStatus::Validated => {
                        // Rule 3: If the identity is validated, removable_on shoud be zero
                        self.assert(
                            idty_value.removable_on == 0,
                            format!(
                                "Identity {} is corrupted: removable_on > 0 on validated idty",
                                idty_index
                            ),
                        );
                    }
                    _ => {
                        // Rule 4: If the identity is not validated, next_creatable_identity_on shoud be zero
                        self.assert(
							idty_value.next_creatable_identity_on == 0,
							format!("Identity {} is corrupted: next_creatable_identity_on > 0 on non-validated idty",
							idty_index)
						);
                    }
                }
            }
            Ok(())
        }
    }
}
