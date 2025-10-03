// Copyright 2022 Axiom-Team
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

pub use crate::{MAX_EVALUATIONS_PER_SESSION, MAX_EVALUATORS_PER_SESSION, median::*};
pub use sp_distance::ComputationResult;

use codec::{Decode, Encode};
use frame_support::pallet_prelude::*;
use sp_runtime::Perbill;

/// Status of the distance evaluation of an identity.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum DistanceStatus {
    /// Identity is in evaluation.
    Pending,
    /// Identity respects the distance.
    Valid,
    /// Identity doesn't respect the distance.
    Invalid,
}

/// Represents a pool where distance evaluation requests and results are stored.
///
/// Depending on the pool rotation, this may not be complete and may still be accepting
/// new evaluation requests (with empty median accumulators) or new evaluations (with evaluators and new samples in the median accumulators).
#[derive(Encode, Decode, Clone, RuntimeDebug, TypeInfo)]
pub struct EvaluationPool<AccountId: Ord, IdtyIndex> {
    /// List of identities with their evaluation result.
    /// The result is the median of all the evaluations.
    pub evaluations: BoundedVec<
        (IdtyIndex, MedianAcc<Perbill, MAX_EVALUATORS_PER_SESSION>),
        ConstU32<MAX_EVALUATIONS_PER_SESSION>,
    >,
    /// Evaluators who have published a result.
    /// Its length should be the same as the number of samples
    /// in each evaluation result `MedianAcc`.
    /// An evaluator is not allowed to publish twice in a single session.
    pub evaluators: BoundedBTreeSet<AccountId, ConstU32<MAX_EVALUATORS_PER_SESSION>>,
}

impl<AccountId: Ord, IdtyIndex> Default for EvaluationPool<AccountId, IdtyIndex> {
    fn default() -> Self {
        Self {
            evaluations: BoundedVec::default(),
            evaluators: BoundedBTreeSet::new(),
        }
    }
}
