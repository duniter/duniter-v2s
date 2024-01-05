// Copyright 2022 Axiom-Team
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

use codec::{Decode, Encode};
use frame_support::pallet_prelude::*;
use sc_client_api::{ProvideUncles, StorageKey, StorageProvider};
use scale_info::TypeInfo;
use sp_runtime::{generic::BlockId, traits::Block as BlockT, AccountId32};
use std::path::PathBuf;

type IdtyIndex = u32;

#[derive(Debug, thiserror::Error)]
pub enum Error<B: BlockT> {
    #[error("Could not retrieve the block hash for block id: {0:?}")]
    NoHashForBlockId(BlockId<B>),
}

/// Create a new [`sp_distance::InherentDataProvider`] at the given block.
pub fn create_distance_inherent_data_provider<B, C, Backend>(
    client: &C,
    parent: B::Hash,
    distance_dir: PathBuf,
    owner_keys: &[sp_core::sr25519::Public],
) -> Result<sp_distance::InherentDataProvider<IdtyIndex>, sc_client_api::blockchain::Error>
where
    B: BlockT,
    C: ProvideUncles<B> + StorageProvider<B, Backend>,
    Backend: sc_client_api::Backend<B>,
    IdtyIndex: Decode + Encode + PartialEq + TypeInfo,
{
    let &[owner_key] = owner_keys else {
        log::error!("🧙 [distance oracle] More than one Babe owner key: oracle cannot work");
        return Ok(sp_distance::InherentDataProvider::<IdtyIndex>::new(None));
    };
    let owner_key = sp_runtime::AccountId32::new(owner_key.0);

    let session_index = client
        .storage(
            parent,
            &StorageKey(
                frame_support::storage::storage_prefix(b"Session", b"CurrentIndex").to_vec(),
            ),
        )
        .expect("CurrentIndex is Err")
        .map_or(0, |raw| {
            u32::decode(&mut &raw.0[..]).expect("cannot decode CurrentIndex")
        });

    let published_results = client
        .storage(
            parent,
            &StorageKey(
                frame_support::storage::storage_prefix(
                    b"Distance",
                    match session_index % 3 {
                        0 => b"StoragePublishedResults1",
                        1 => b"StoragePublishedResults2",
                        2 => b"StoragePublishedResults0",
                        _ => unreachable!("n%3<3"),
                    },
                )
                .to_vec(),
            ),
        )?
        .map_or_else(Default::default, |raw| {
            pallet_distance::EvaluationPool::<AccountId32, IdtyIndex>::decode(&mut &raw.0[..])
                .expect("cannot decode EvaluationPool")
        });

    // Have we already published a result for this session?
    if published_results.evaluators.contains(&owner_key) {
        log::debug!("🧙 [distance oracle] Already published a result for this session");
        return Ok(sp_distance::InherentDataProvider::<IdtyIndex>::new(None));
    }

    // Read evaluation result from file, if it exists
    log::debug!(
        "🧙 [distance oracle] Reading evaluation result from file {:?}",
        distance_dir.clone().join(session_index.to_string())
    );
    let evaluation_result = match std::fs::read(distance_dir.join(session_index.to_string())) {
        Ok(data) => data,
        Err(e) => {
            match e.kind() {
                std::io::ErrorKind::NotFound => {
                    log::debug!("🧙 [distance oracle] Evaluation result file not found");
                }
                _ => {
                    log::error!(
                        "🧙 [distance oracle] Cannot read distance evaluation result file: {e:?}"
                    );
                }
            }
            return Ok(sp_distance::InherentDataProvider::<IdtyIndex>::new(None));
        }
    };

    log::info!("🧙 [distance oracle] Providing evaluation result");
    Ok(sp_distance::InherentDataProvider::<IdtyIndex>::new(Some(
        sp_distance::ComputationResult::decode(&mut evaluation_result.as_slice()).unwrap(),
    )))
}
