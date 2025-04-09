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

use super::{gdev, gdev::runtime_types::pallet_identity, *};
use crate::{common::pair_signer::PairSigner, DuniterWorld};
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
                &gdev::tx().distance().request_distance_evaluation(),
                &origin,
                SubstrateExtrinsicParamsBuilder::new().build(),
            )
            .await?,
    )
    .await?;

    Ok(())
}

pub async fn run_oracle(client: &FullClient, origin: Keyring, rpc_url: String) -> Result<()> {
    let origin = PairSigner::new(origin.pair());
    let account_id: &AccountId32 = origin.account_id();

    if let Some((distances, _current_session, _evaluation_result_path)) =
        distance_oracle::compute_distance_evaluation(
            &distance_oracle::api::client(rpc_url.clone()).await,
            &distance_oracle::Settings {
                evaluation_result_dir: PathBuf::default(),
                rpc_url,
            },
        )
        .await
    {
        // Distance evaluation period is 7 blocks
        for _ in 0..7 {
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
        /*for event in events.iter() {
            let event = event.unwrap();
            println!(
                "Event: {}::{} -> {:?}\n\n",
                event.pallet_name(),
                event.variant_name(),
                event.field_values()
            );
        }*/
    }

    Ok(())
}
