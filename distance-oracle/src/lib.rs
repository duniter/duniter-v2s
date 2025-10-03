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

//! # Distance Oracle
//!
//! The **Distance Oracle** is a standalone program designed to calculate the distances between identities in the Duniter Web of Trust (WoT). This process is computationally intensive and is therefore decoupled from the main runtime. It allows smith users to choose whether to run the oracle and provide results to the network.
//!
//! The **oracle** works in conjunction with the **Inherent Data Provider** and the **Distance Pallet** in the runtime to deliver periodic computation results. The **Inherent Data Provider** fetches and supplies these results to the runtime, ensuring that the necessary data for distance evaluations is available to be processed at the appropriate time in the runtime lifecycle.
//!
//! ## Structure
//!
//! The Distance Oracle is organized into the following modules:
//!
//! 1. **`/distance-oracle/`**: Contains the main binary for executing the distance computation.
//! 2. **`/primitives/distance/`**: Defines primitive types shared between the client and runtime.
//! 3. **`/client/distance/`**: Exposes the `create_distance_inherent_data_provider`, which feeds data into the runtime through the Inherent Data Provider.
//! 4. **`/pallets/distance/`**: A pallet that handles distance-related types, traits, storage, and hooks in the runtime, coordinating the interaction between the oracle, inherent data provider, and runtime.
//!
//! ## How it works
//! - The **Distance Pallet** adds an evaluation request at period `i` in the runtime.
//! - The **Distance Oracle** evaluates this request at period `i + 1`, computes the necessary results and stores them on disk.
//! - The **Inherent Data Provider** reads this evaluation result from disk at period `i + 2` and provides it to the runtime to perform the required operations.
//!
//! ## Usage
//!
//! ### Docker Integration
//!
//! To run the Distance Oracle, use the provided Docker setup. Refer to the [docker-compose.yml](../docker-compose.yml) file for an example configuration.
//!
//! Example Output:
//! ```text
//! 2023-12-09T14:45:05.942Z INFO [distance_oracle] Nothing to do: Pool does not exist
//! Waiting 1800 seconds before next execution...
//! ```

#[cfg(not(test))]
pub mod api;
#[cfg(test)]
pub mod mock;
#[cfg(test)]
mod tests;

#[cfg(test)]
pub use mock as api;

use api::{AccountId, EvaluationPool, H256, IdtyIndex};

use codec::Encode;
use fnv::{FnvHashMap, FnvHashSet};
use log::{debug, info, warn};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{io::Write, path::PathBuf};

/// The file version must match the version used by the inherent data provider.
/// This ensures that the smith avoids accidentally submitting invalid data
/// in case there are changes in logic between the runtime and the oracle,
/// thereby preventing potential penalties.
const VERSION_PREFIX: &str = "001-";

#[cfg(feature = "gdev")]
#[subxt::subxt(runtime_metadata_path = "../resources/gdev_metadata.scale")]
pub mod runtime {}
#[cfg(feature = "gtest")]
#[subxt::subxt(runtime_metadata_path = "../resources/gtest_metadata.scale")]
pub mod runtime {}
#[cfg(feature = "g1")]
#[subxt::subxt(runtime_metadata_path = "../resources/g1_metadata.scale")]
pub mod runtime {}

pub enum RuntimeConfig {}
impl subxt::config::Config for RuntimeConfig {
    type AccountId = AccountId;
    type Address = sp_runtime::MultiAddress<Self::AccountId, u32>;
    type AssetId = ();
    type ExtrinsicParams = subxt::config::substrate::SubstrateExtrinsicParams<Self>;
    type Hasher = subxt::config::substrate::BlakeTwo256;
    type Header =
        subxt::config::substrate::SubstrateHeader<u32, subxt::config::substrate::BlakeTwo256>;
    type Signature = sp_runtime::MultiSignature;
}

/// Represents a tipping amount.
#[derive(Copy, Clone, Debug, Default, Encode)]
pub struct Tip {
    #[codec(compact)]
    tip: u64,
}

impl Tip {
    pub fn new(amount: u64) -> Self {
        Tip { tip: amount }
    }
}

impl From<u64> for Tip {
    fn from(n: u64) -> Self {
        Self::new(n)
    }
}

/// Represents configuration parameters.
pub struct Settings {
    pub evaluation_result_dir: PathBuf,
    pub rpc_url: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            evaluation_result_dir: PathBuf::from("/tmp/duniter/chains/gdev/distance"),
            rpc_url: String::from("ws://127.0.0.1:9944"),
        }
    }
}

/// Runs the evaluation process, saves the results, and cleans up old files.
///
/// This function performs the following steps:
/// 1. Runs the evaluation task by invoking `compute_distance_evaluation`, which provides:
///    - The evaluation results.
///    - The current period index.
///    - The file path where the results should be stored.
/// 2. Saves the evaluation results to a file in the specified directory.
/// 3. Cleans up outdated evaluation files.
pub async fn run(client: &api::Client, settings: &Settings) {
    let Some((evaluation, current_period_index, evaluation_result_path)) =
        compute_distance_evaluation(client, settings).await
    else {
        return;
    };

    debug!("Saving distance evaluation result to file `{evaluation_result_path:?}`");
    let mut evaluation_result_file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&evaluation_result_path)
        .unwrap_or_else(|e| {
            panic!(
                "Cannot open distance evaluation result file `{evaluation_result_path:?}`: {e:?}"
            )
        });
    evaluation_result_file
        .write_all(
            &sp_distance::ComputationResult {
                distances: evaluation,
            }
            .encode(),
        )
        .unwrap_or_else(|e| {
            panic!(
                "Cannot write distance evaluation result to file `{evaluation_result_path:?}`: {e:?}"
            )
        });

    // When a new result is written, remove old results except for the current period used by the inherent logic and the next period that was just generated.
    settings
        .evaluation_result_dir
        .read_dir()
        .unwrap_or_else(|e| {
            panic!(
                "Cannot read distance evaluation result directory `{:?}`: {:?}",
                settings.evaluation_result_dir, e
            )
        })
        .flatten()
        .filter_map(|entry| {
            entry
                .file_name()
                .to_str()
                .and_then(|name| {
                    name.split('-')
                        .next_back()?
                        .parse::<u32>()
                        .ok()
                        .filter(|&pool| {
                            pool != current_period_index && pool != current_period_index + 1
                        })
                })
                .map(|_| entry.path())
        })
        .for_each(|path| {
            std::fs::remove_file(&path)
                .unwrap_or_else(|e| warn!("Cannot remove file `{:?}`: {:?}", path, e));
        });
}

/// Evaluates distance for the current period and prepares results for storage.
///
/// This function performs the following steps:
/// 1. Prepares the evaluation context using `prepare_evaluation_context`. If the context is not
///    ready (e.g., no pending evaluations, or results already exist), it returns `None`.
/// 2. Evaluates distances for all identities in the evaluation pool.
/// 3. Returns the evaluation results, the current period index, and the path to store the results.
///
pub async fn compute_distance_evaluation(
    client: &api::Client,
    settings: &Settings,
) -> Option<(Vec<sp_runtime::Perbill>, u32, PathBuf)> {
    let (evaluation_block, current_period_index, evaluation_pool, evaluation_result_path) =
        prepare_evaluation_context(client, settings).await?;

    info!("Evaluating distance for period {}", current_period_index);

    let max_depth = api::max_referee_distance(client).await;

    // member idty -> issued certs
    let mut members = FnvHashMap::<IdtyIndex, u32>::default();

    let mut members_iter = api::member_iter(client, evaluation_block).await;
    while let Some(member_idty) = members_iter
        .next()
        .await
        .expect("Cannot fetch next members")
    {
        members.insert(member_idty, 0);
    }

    let min_certs_for_referee = (members.len() as f32).powf(1. / (max_depth as f32)).ceil() as u32;

    // idty -> received certs
    let mut received_certs = FnvHashMap::<IdtyIndex, Vec<IdtyIndex>>::default();

    let mut certs_iter = api::cert_iter(client, evaluation_block).await;
    while let Some((receiver, issuers)) = certs_iter
        .next()
        .await
        .expect("Cannot fetch next certification")
    {
        if (issuers.len() as u32) < min_certs_for_referee {
            // This member is not referee
            members.remove(&receiver);
        }
        for (issuer, _removable_on) in issuers.iter() {
            if let Some(issued_certs) = members.get_mut(issuer) {
                *issued_certs += 1;
            }
        }
        received_certs.insert(
            receiver,
            issuers
                .into_iter()
                .map(|(issuer, _removable_on)| issuer)
                .collect(),
        );
    }

    // Only retain referees
    members.retain(|_idty, issued_certs| *issued_certs >= min_certs_for_referee);
    let referees = members;

    let evaluation = evaluation_pool
        .evaluations
        .0
        .as_slice()
        .par_iter()
        .map(|(idty, _)| distance_rule(&received_certs, &referees, max_depth, *idty))
        .collect();

    Some((evaluation, current_period_index, evaluation_result_path))
}

/// Prepares the context for the next evaluation task.
///
/// This function performs the following steps:
/// 1. Fetches the parent hash of the latest block from the API.
/// 2. Determines the current period index.
/// 3. Retrieves the evaluation pool for the current period.
///    - If the pool does not exist or is empty, it returns `None`.
/// 4. Checks if the evaluation result file for the next period already exists.
///    - If it exists, the task has already been completed, so the function returns `None`.
/// 5. Ensures the evaluation result directory is available, creating it if necessary.
/// 6. Retrieves the block number of the evaluation.
///
async fn prepare_evaluation_context(
    client: &api::Client,
    settings: &Settings,
) -> Option<(H256, u32, EvaluationPool, PathBuf)> {
    let parent_hash = api::parent_hash(client).await;

    let current_period_index = api::current_period_index(client, parent_hash).await;

    // Fetch the pending identities
    let Some(evaluation_pool) =
        api::current_pool(client, parent_hash, current_period_index % 3).await
    else {
        info!("Nothing to do: Pool does not exist");
        return None;
    };

    // Stop if nothing to evaluate
    if evaluation_pool.evaluations.0.is_empty() {
        info!("Nothing to do: Pool is empty");
        return None;
    }

    // The result is saved in a file named `current_period_index + 1`.
    // It will be picked up during the next period by the inherent.
    let evaluation_result_path = settings
        .evaluation_result_dir
        .join(VERSION_PREFIX.to_owned() + &(current_period_index + 1).to_string());

    // Stop if already evaluated
    if evaluation_result_path
        .try_exists()
        .expect("Result path unavailable")
    {
        info!("Nothing to do: File already exists");
        return None;
    }

    #[cfg(not(test))]
    std::fs::create_dir_all(&settings.evaluation_result_dir).unwrap_or_else(|e| {
        panic!(
            "Cannot create distance evaluation result directory `{0:?}`: {e:?}",
            settings.evaluation_result_dir
        );
    });

    Some((
        api::evaluation_block(client, parent_hash).await,
        current_period_index,
        evaluation_pool,
        evaluation_result_path,
    ))
}

/// Recursively explores the certification graph to identify referees accessible within a given depth.
fn distance_rule_recursive(
    received_certs: &FnvHashMap<IdtyIndex, Vec<IdtyIndex>>,
    referees: &FnvHashMap<IdtyIndex, u32>,
    idty: IdtyIndex,
    accessible_referees: &mut FnvHashSet<IdtyIndex>,
    known_idties: &mut FnvHashMap<IdtyIndex, u32>,
    depth: u32,
) {
    // Do not re-explore identities that have already been explored at least as deeply
    match known_idties.entry(idty) {
        std::collections::hash_map::Entry::Occupied(mut entry) => {
            if *entry.get() >= depth {
                return;
            } else {
                *entry.get_mut() = depth;
            }
        }
        std::collections::hash_map::Entry::Vacant(entry) => {
            entry.insert(depth);
        }
    }

    // If referee, add it to the list
    if referees.contains_key(&idty) {
        accessible_referees.insert(idty);
    }

    // If reached the maximum distance, stop exploring
    if depth == 0 {
        return;
    }

    // Explore certifiers
    for &certifier in received_certs.get(&idty).unwrap_or(&vec![]).iter() {
        distance_rule_recursive(
            received_certs,
            referees,
            certifier,
            accessible_referees,
            known_idties,
            depth - 1,
        );
    }
}

/// Calculates the fraction of accessible referees to total referees for a given identity.
fn distance_rule(
    received_certs: &FnvHashMap<IdtyIndex, Vec<IdtyIndex>>,
    referees: &FnvHashMap<IdtyIndex, u32>,
    depth: u32,
    idty: IdtyIndex,
) -> sp_runtime::Perbill {
    debug!("Evaluating distance for idty {}", idty);
    let mut accessible_referees =
        FnvHashSet::<IdtyIndex>::with_capacity_and_hasher(referees.len(), Default::default());
    let mut known_idties =
        FnvHashMap::<IdtyIndex, u32>::with_capacity_and_hasher(referees.len(), Default::default());
    distance_rule_recursive(
        received_certs,
        referees,
        idty,
        &mut accessible_referees,
        &mut known_idties,
        depth,
    );
    let result = if referees.contains_key(&idty) {
        sp_runtime::Perbill::from_rational(
            accessible_referees.len() as u32 - 1,
            referees.len() as u32 - 1,
        )
    } else {
        sp_runtime::Perbill::from_rational(accessible_referees.len() as u32, referees.len() as u32)
    };
    info!(
        "Distance for idty {}: {}/{} = {}%",
        idty,
        accessible_referees.len(),
        referees.len(),
        result.deconstruct() as f32 / 1_000_000_000f32 * 100f32
    );
    result
}
