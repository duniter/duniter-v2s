// Copyright 2023 Axiom-Team
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

use super::{gtest, gtest::runtime_types::pallet_identity, *};
use crate::{DuniterWorld, common::pair_signer::PairSigner};
use sp_keyring::sr25519::Keyring;
use subxt::{backend::rpc::RpcClient, tx::Signer, utils::AccountId32};

pub async fn request_evaluation(client: &FullClient, origin: Keyring) -> Result<()> {
    let origin = PairSigner::new(origin.pair());

    let _events = create_block_with_extrinsic(
        &client.rpc,
        client
            .client
            .tx()
            .create_signed(
                &gtest::tx().distance().request_distance_evaluation(),
                &origin,
                SubstrateExtrinsicParamsBuilder::new().build(),
            )
            .await?,
    )
    .await?;

    Ok(())
}

pub async fn run_oracle(client: &FullClient, origin: Keyring, rpc_url: String) -> Result<Vec<u32>> {
    let origin = PairSigner::new(origin.pair());
    let account_id: &AccountId32 = origin.account_id();

    if let Some((distances, current_period, _evaluation_result_path)) =
        distance_oracle::compute_distance_evaluation(
            &distance_oracle::api::client(rpc_url.clone()).await,
            &distance_oracle::Settings {
                evaluation_result_dir: PathBuf::default(),
                rpc_url,
            },
        )
        .await
    {
        let results = distances
            .iter()
            .map(|distance| distance.deconstruct())
            .collect();
        let period: u32 = client
            .client
            .constants()
            .at(&gtest::constants().distance().evaluation_period())?;
        let mut advanced = false;
        for _ in 0..=period {
            super::create_empty_block(&client.rpc).await?;
            let index = client
                .client
                .storage()
                .at_latest()
                .await?
                .fetch_or_default(&gtest::storage().distance().current_period_index())
                .await?;
            if index != current_period {
                advanced = true;
                break;
            }
        }
        anyhow::ensure!(advanced, "distance computation period did not advance");
        let call = gtest::runtime_types::gtest_runtime::RuntimeCall::Distance(
            gtest::runtime_types::pallet_distance::pallet::Call::force_update_evaluation {
                evaluator: account_id.clone(),
                computation_result: gtest::runtime_types::sp_distance::ComputationResult {
                    distances: distances
                        .into_iter()
                        .map(|res| {
                            gtest::runtime_types::sp_arithmetic::per_things::Perbill(
                                res.deconstruct(),
                            )
                        })
                        .collect(),
                },
            },
        );
        let extrinsic = client
            .client
            .tx()
            .create_signed(
                &gtest::tx().sudo().sudo(call),
                &PairSigner::new(SUDO_ACCOUNT.pair()),
                SubstrateExtrinsicParamsBuilder::new().build(),
            )
            .await?;
        create_block_with_extrinsic(&client.rpc, extrinsic).await?;
        Ok(results)
    } else {
        Err(anyhow::anyhow!(
            "oracle has no computation pool to evaluate"
        ))
    }
}
