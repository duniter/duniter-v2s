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

#[cfg(not(test))]
pub mod api;
#[cfg(test)]
pub mod mock;
#[cfg(test)]
mod tests;

#[cfg(test)]
pub use mock as api;

use api::{AccountId, IdtyIndex};

use codec::Encode;
use fnv::{FnvHashMap, FnvHashSet};
use log::{debug, error, info};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{io::Write, path::PathBuf};

// TODO select metadata file using features
#[subxt::subxt(runtime_metadata_path = "../resources/metadata.scale")]
pub mod runtime {}

pub enum RuntimeConfig {}
impl subxt::config::Config for RuntimeConfig {
    type AccountId = AccountId;
    type Address = subxt::ext::sp_runtime::MultiAddress<Self::AccountId, u32>;
    type AssetId = ();
    type ExtrinsicParams = subxt::config::substrate::SubstrateExtrinsicParams<Self>;
    type Hash = subxt::utils::H256;
    type Hasher = subxt::config::substrate::BlakeTwo256;
    type Header =
        subxt::config::substrate::SubstrateHeader<u32, subxt::config::substrate::BlakeTwo256>;
    type Signature = subxt::ext::sp_runtime::MultiSignature;
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

/// Asynchronously runs a computation using the provided client and saves the result to a file.
pub async fn run_and_save(client: &api::Client, settings: &Settings) {
    let Some((evaluation, _current_pool_index, evaluation_result_path)) =
        run(client, settings, true).await
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
}

/// Asynchronously runs a computation based on the provided client and settings.
/// Returns `Option<(evaluation, current_pool_index, evaluation_result_path)>`.
pub async fn run(
    client: &api::Client,
    settings: &Settings,
    handle_fs: bool,
) -> Option<(Vec<sp_runtime::Perbill>, u32, PathBuf)> {
    let parent_hash = api::parent_hash(client).await;

    let max_depth = api::max_referee_distance(client).await;

    let current_pool_index = api::current_pool_index(client, parent_hash).await;

    // Fetch the pending identities
    let Some(evaluation_pool) = api::current_pool(client, parent_hash, current_pool_index).await
    else {
        info!("Nothing to do: Pool does not exist");
        return None;
    };

    // Stop if nothing to evaluate
    if evaluation_pool.evaluations.0.is_empty() {
        info!("Nothing to do: Pool is empty");
        return None;
    }

    let evaluation_result_path = settings
        .evaluation_result_dir
        .join(((current_pool_index + 1) % 3).to_string());

    if handle_fs {
        // Stop if already evaluated
        if evaluation_result_path
            .try_exists()
            .expect("Result path unavailable")
        {
            info!("Nothing to do: File already exists");
            return None;
        }

        std::fs::create_dir_all(&settings.evaluation_result_dir).unwrap_or_else(|e| {
            error!(
                "Cannot create distance evaluation result directory `{0:?}`: {e:?}",
                settings.evaluation_result_dir
            );
        });
    }

    info!("Evaluating distance for pool {}", current_pool_index);
    let evaluation_block = api::evaluation_block(client, parent_hash).await;

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

    Some((evaluation, current_pool_index, evaluation_result_path))
}

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

/// Returns the fraction `nb_accessible_referees / nb_referees`
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
