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

use super::gdev;
use super::gdev::runtime_types::pallet_identity;
use super::*;
use crate::DuniterWorld;
use sp_keyring::AccountKeyring;
use subxt::backend::rpc::RpcClient;
use subxt::tx::{PairSigner, Signer};
use subxt::utils::AccountId32;

pub async fn request_evaluation(client: &FullClient, origin: AccountKeyring) -> Result<()> {
    let origin = PairSigner::new(origin.pair());

    let _events = create_block_with_extrinsic(
        &client.rpc,
        client
            .client
            .tx()
            .create_signed(
                &gdev::tx().distance().request_distance_evaluation(),
                &origin,
                SubstrateExtrinsicParamsBuilder::new().build(),
            )
            .await?,
    )
    .await?;

    Ok(())
}

pub async fn run_oracle(
    client: &FullClient,
    origin: AccountKeyring,
    rpc_url: String,
) -> Result<()> {
    let origin = PairSigner::new(origin.pair());
    let account_id: &AccountId32 = origin.account_id();

    if let Some((distances, _current_session, _evaluation_result_path)) = distance_oracle::run(
        &distance_oracle::api::client(rpc_url.clone()).await,
        &distance_oracle::Settings {
            evaluation_result_dir: PathBuf::default(),
            rpc_url,
        },
        false,
    )
    .await
    {
        for _ in 0..30 {
            super::create_empty_block(&client.rpc).await?;
        }

        let _events = create_block_with_extrinsic(
        &client.rpc,
        client.client
                .tx()
                .create_signed(
                    &gdev::tx().sudo().sudo(gdev::runtime_types::gdev_runtime::RuntimeCall::Distance(
                            gdev::runtime_types::pallet_distance::pallet::Call::force_update_evaluation {
                                evaluator: account_id.clone(),
                                computation_result:
                                    gdev::runtime_types::sp_distance::ComputationResult {
                                        distances: distances.into_iter().map(|res| unsafe{std::mem::transmute(res)}).collect(),
                                    },
                            },
                        )
                    ),
                    &origin,
                SubstrateExtrinsicParamsBuilder::new().build(),
                )
                .await?,
        )
        .await?;
    }

    Ok(())
}
