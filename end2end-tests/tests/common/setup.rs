//! Scenario state is built with the runtime's genesis builders and installed through sudo.
//! Consensus state and the authoring key remain those of the isolated local node.

use super::*;
use codec::Decode;
use common_runtime::{
    GenesisAccountData, GenesisIdty, IdtyName, IdtyStatus, IdtyValue, entities::IdtyData,
};
use gtest_runtime::{RuntimeCall, RuntimeGenesisConfig, SystemCall};
use sp_core::{Pair, sr25519};
use sp_membership::MembershipData;
use sp_runtime::BuildStorage;

pub async fn sudo(client: &FullClient, call: RuntimeCall) -> Result<()> {
    let call = gtest::runtime_types::gtest_runtime::RuntimeCall::decode(&mut &call.encode()[..])?;
    create_block_with_extrinsic(
        &client.rpc,
        client
            .client
            .tx()
            .create_signed(
                &gtest::tx().sudo().sudo(call),
                &pair_signer::PairSigner::new(SUDO_ACCOUNT.pair()),
                SubstrateExtrinsicParamsBuilder::new().build(),
            )
            .await?,
    )
    .await?;
    Ok(())
}

pub async fn prepare(client: &FullClient, fixture: &str) -> Result<()> {
    let spec = client.client.runtime_version();
    anyhow::ensure!(
        spec.spec_version == gtest_runtime::VERSION.spec_version,
        "unexpected runtime version"
    );
    let key = client
        .client
        .storage()
        .at_latest()
        .await?
        .fetch(&gtest::storage().sudo().key())
        .await?;
    anyhow::ensure!(
        key == Some(SUDO_ACCOUNT.to_raw_public().into()),
        "the local sudo key must be //Alice ed25519"
    );
    let period = client
        .client
        .constants()
        .at(&gtest::constants().certification().cert_period())?;
    anyhow::ensure!(period == 150, "Cucumber requires the fast gtest runtime");

    // The ed25519 sudo account owns Alice's genesis identity. Unlink it before
    // replacing the WoT so it cannot inherit the fixture's sr25519 Alice quota.
    let sudo_account = client
        .client
        .storage()
        .at_latest()
        .await?
        .fetch_or_default(
            &gtest::storage()
                .system()
                .account(SUDO_ACCOUNT.to_raw_public().into()),
        )
        .await?;
    if sudo_account.data.linked_idty.is_some() {
        let extrinsic = client
            .client
            .tx()
            .create_signed(
                &gtest::tx().account().unlink_identity(),
                &pair_signer::PairSigner::new(SUDO_ACCOUNT.pair()),
                SubstrateExtrinsicParamsBuilder::new().build(),
            )
            .await?;
        create_block_with_extrinsic(&client.rpc, extrinsic).await?;
    }

    // Six members form a valid clique under the production threshold of five certifications.
    let mut groups = vec![vec!["Alice", "Bob", "Charlie", "One", "Two", "Support"]];
    match fixture {
        "default" => {}
        "wot" => groups[0].push("Dave"),
        // Both cliques are internally valid. A newcomer certified by the first clique
        // reaches six of twelve referees, below the required 80 percent.
        "bad_distance" => groups.push(vec![
            "RemoteA", "RemoteB", "RemoteC", "RemoteD", "RemoteE", "RemoteF",
        ]),
        _ => return Err(anyhow!("unknown scenario fixture: {fixture}")),
    }
    let mut genesis = RuntimeGenesisConfig::default();
    let mut members = Vec::new();
    let mut next_index = 1u32;
    for group in &groups {
        let indices: Vec<u32> = (next_index..next_index + group.len() as u32).collect();
        for (name, index) in group.iter().zip(indices.iter().copied()) {
            let account: gtest_runtime::AccountId =
                sr25519::Pair::from_string(&format!("//{name}"), None)?
                    .public()
                    .into();
            genesis.account.accounts.insert(
                account.clone(),
                GenesisAccountData {
                    balance: if *name == "Alice" { 900 } else { 1000 },
                    idty_id: Some(index),
                },
            );
            genesis.identity.identities.push(GenesisIdty {
                index,
                name: IdtyName::from(*name),
                value: IdtyValue {
                    data: IdtyData {
                        first_eligible_ud:
                            gtest_runtime::pallet_universal_dividend::FirstEligibleUd::min(),
                    },
                    next_creatable_identity_on: 0,
                    old_owner_key: None,
                    owner_key: account,
                    next_scheduled: 0,
                    status: IdtyStatus::Member,
                },
            });
            genesis.membership.memberships.insert(
                index,
                MembershipData {
                    expire_on: u32::MAX,
                },
            );
            genesis.certification.certs_by_receiver.insert(
                index,
                indices
                    .iter()
                    .filter(|other| **other != index)
                    .map(|other| (*other, Some(u32::MAX)))
                    .collect(),
            );
            members.push(index);
        }
        next_index += group.len() as u32;
    }
    if fixture == "wot" {
        // Leave Dave -> Alice available for the certification scenario.
        genesis
            .certification
            .certs_by_receiver
            .get_mut(&1)
            .unwrap()
            .remove(&7);
        for (name, status, cert_count) in [
            ("Eve", IdtyStatus::Revoked, 0),
            ("Ferdie", IdtyStatus::Unvalidated, 4),
        ] {
            let account: gtest_runtime::AccountId =
                sr25519::Pair::from_string(&format!("//{name}"), None)?
                    .public()
                    .into();
            let index = next_index;
            next_index += 1;
            genesis.account.accounts.insert(
                account.clone(),
                GenesisAccountData {
                    balance: 1000,
                    idty_id: Some(index),
                },
            );
            genesis.identity.identities.push(GenesisIdty {
                index,
                name: IdtyName::from(name),
                value: IdtyValue {
                    data: IdtyData {
                        first_eligible_ud: Default::default(),
                    },
                    next_creatable_identity_on: 0,
                    old_owner_key: None,
                    owner_key: account,
                    next_scheduled: 0,
                    status,
                },
            });
            genesis.certification.certs_by_receiver.insert(
                index,
                (1..=cert_count)
                    .map(|issuer| (issuer, Some(u32::MAX)))
                    .collect(),
            );
        }
    }
    genesis.certification.apply_cert_period_at_genesis = false;
    genesis.quota.identities = members;
    genesis.account.treasury_balance = 100;
    let sudo_account = client
        .client
        .storage()
        .at_latest()
        .await?
        .fetch_or_default(
            &gtest::storage()
                .system()
                .account(SUDO_ACCOUNT.to_raw_public().into()),
        )
        .await?;
    let mass = sudo_account.data.free
        + genesis
            .account
            .accounts
            .values()
            .map(|account| account.balance)
            .sum::<u64>()
        + 100;
    genesis.balances.total_issuance = mass;
    genesis.universal_dividend.ud = 1000;
    genesis.universal_dividend.initial_monetary_mass = mass;
    // None anchors the schedules to the first timestamp after setup.
    genesis.universal_dividend.first_ud = None;
    genesis.universal_dividend.first_reeval = None;

    let mut storage = sp_core::storage::Storage::default();
    genesis
        .account
        .assimilate_storage(&mut storage)
        .map_err(anyhow::Error::msg)?;
    genesis
        .balances
        .assimilate_storage(&mut storage)
        .map_err(anyhow::Error::msg)?;
    genesis
        .identity
        .assimilate_storage(&mut storage)
        .map_err(anyhow::Error::msg)?;
    genesis
        .membership
        .assimilate_storage(&mut storage)
        .map_err(anyhow::Error::msg)?;
    genesis
        .certification
        .assimilate_storage(&mut storage)
        .map_err(anyhow::Error::msg)?;
    genesis
        .quota
        .assimilate_storage(&mut storage)
        .map_err(anyhow::Error::msg)?;
    genesis
        .universal_dividend
        .assimilate_storage(&mut storage)
        .map_err(anyhow::Error::msg)?;
    genesis
        .distance
        .assimilate_storage(&mut storage)
        .map_err(anyhow::Error::msg)?;
    let prefixes: Vec<Vec<u8>> = [
        "Balances",
        "Identity",
        "Membership",
        "Certification",
        "Quota",
        "UniversalDividend",
        "Distance",
        "OneshotAccount",
    ]
    .iter()
    .map(|name| sp_core::twox_128(name.as_bytes()).to_vec())
    .collect();
    for prefix in &prefixes {
        let mut key = prefix.clone();
        key.extend(sp_core::twox_128(b":__STORAGE_VERSION__:"));
        let value: Option<String> = client
            .rpc
            .request(
                "state_getStorage",
                rpc_params![format!("0x{}", hex::encode(&key))],
            )
            .await?;
        if let Some(value) = value {
            storage
                .top
                .insert(key, hex::decode(value.trim_start_matches("0x"))?);
        }
    }
    let mut accounts_prefix = sp_core::twox_128(b"System").to_vec();
    accounts_prefix.extend(sp_core::twox_128(b"Account"));
    let mut sudo_key = accounts_prefix.clone();
    let sudo_public = SUDO_ACCOUNT.to_raw_public();
    sudo_key.extend(sp_core::blake2_128(&sudo_public));
    sudo_key.extend(sudo_public);
    let mut account_keys = keys_under(client, &accounts_prefix).await?;
    account_keys.retain(|key| *key != sudo_key);
    let balances_prefix = sp_core::twox_128(b"Balances").to_vec();
    let mut balance_keys = keys_under(client, &balances_prefix).await?;
    // Fee withdrawal creates a Holds entry before dispatch. Never clear the
    // sudo payer's balance maps while its transaction is executing.
    balance_keys.retain(|key| !key.ends_with(&sudo_public));
    account_keys.extend(balance_keys);
    let mut calls: Vec<_> = prefixes
        .into_iter()
        .filter(|prefix| *prefix != balances_prefix)
        .map(|prefix| {
            RuntimeCall::System(SystemCall::kill_prefix {
                prefix,
                subkeys: 1000,
            })
        })
        .collect();
    calls.push(RuntimeCall::System(SystemCall::kill_storage {
        keys: account_keys,
    }));
    calls.push(RuntimeCall::System(SystemCall::set_storage {
        items: storage.top.into_iter().collect(),
    }));
    sudo(
        client,
        RuntimeCall::Utility(pallet_utility::Call::batch_all { calls }),
    )
    .await?;

    // Check installed invariants through RPC before any scenario action.
    let latest = client.client.storage().at_latest().await?;
    let member_count = latest
        .fetch_or_default(&gtest::storage().membership().counter_for_membership())
        .await?;
    anyhow::ensure!(
        member_count as usize == groups.iter().map(Vec::len).sum::<usize>(),
        "fixture member count mismatch"
    );
    let issuance = latest
        .fetch_or_default(&gtest::storage().balances().total_issuance())
        .await?;
    anyhow::ensure!(issuance == mass, "fixture issuance mismatch");
    let mut accounts = latest
        .iter(gtest::storage().system().account_iter())
        .await?;
    let mut total = 0u64;
    while let Some(account) = accounts.next().await {
        let account = account?.value.data;
        total += account.free + account.reserved;
    }
    anyhow::ensure!(
        total == issuance,
        "fixture account sum {total} differs from issuance {issuance}"
    );
    Ok(())
}

async fn keys_under(client: &FullClient, prefix: &[u8]) -> Result<Vec<Vec<u8>>> {
    let keys: Vec<String> = client
        .rpc
        .request(
            "state_getKeysPaged",
            rpc_params![
                format!("0x{}", hex::encode(prefix)),
                1000,
                Option::<String>::None
            ],
        )
        .await?;
    anyhow::ensure!(
        keys.len() < 1000,
        "scenario setup exceeds its bounded storage key count"
    );
    keys.into_iter()
        .map(|key| hex::decode(key.trim_start_matches("0x")).map_err(Into::into))
        .collect()
}
