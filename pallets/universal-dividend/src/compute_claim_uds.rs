// Copyright 2021 Axiom-Team
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

use super::UdIndex;
use core::iter::DoubleEndedIterator;
use sp_arithmetic::traits::{AtLeast32BitUnsigned, Zero};

pub(super) fn compute_claim_uds<Balance: AtLeast32BitUnsigned>(
    mut current_ud_index: UdIndex,
    first_ud_index: UdIndex,
    past_reevals: impl DoubleEndedIterator<Item = (UdIndex, Balance)>,
) -> (UdIndex, Balance) {
    let mut total_amount = Zero::zero();
    let mut total_count = 0;
    for (ud_index, ud_amount) in past_reevals.rev() {
        if ud_index <= first_ud_index {
            let count = current_ud_index - first_ud_index;
            total_amount += Balance::from(count) * ud_amount;
            total_count += count;
            break;
        } else {
            let count = current_ud_index - ud_index;
            total_amount += Balance::from(count) * ud_amount;
            total_count += count;
            current_ud_index = ud_index;
        }
    }

    (total_count, total_amount)
}

#[cfg(test)]
#[allow(clippy::unnecessary_cast)]
mod tests {
    use super::*;

    type Balance = u64;

    #[test]
    fn empty_case() {
        let past_reevals = Vec::<(UdIndex, Balance)>::new();
        assert_eq!(compute_claim_uds(11, 1, past_reevals.into_iter()), (0, 0));
    }

    #[test]
    fn ten_uds_after_genesis() {
        let past_reevals = vec![(1, 1_000 as Balance)];
        assert_eq!(
            compute_claim_uds(11, 1, past_reevals.into_iter()),
            (10, 10_000)
        );
    }

    #[test]
    fn three_uds_after_one_reeval() {
        let past_reevals = vec![(1, 1_000 as Balance), (8, 1_100 as Balance)];
        assert_eq!(
            compute_claim_uds(11, 1, past_reevals.into_iter()),
            (10, 10_300)
        );
    }

    #[test]
    fn just_at_a_reeval() {
        let past_reevals = vec![(1, 1_000 as Balance), (8, 1_100 as Balance)];
        assert_eq!(
            compute_claim_uds(9, 1, past_reevals.into_iter()),
            (8, 8_100)
        );
    }

    #[test]
    fn first_at_current() {
        let past_reevals = vec![(1, 1_000 as Balance)];
        assert_eq!(compute_claim_uds(1, 1, past_reevals.into_iter()), (0, 0));
    }

    #[test]
    fn only_one_ud() {
        let past_reevals = vec![(1, 1_000 as Balance)];
        assert_eq!(
            compute_claim_uds(2, 1, past_reevals.into_iter()),
            (1, 1_000)
        );
    }

    #[test]
    fn ud_for_joiner_after_reeval() {
        let past_reevals = vec![
            (1, 1_000 as Balance),
            (2, 10_000 as Balance),
            (3, 100_000 as Balance),
        ];
        assert_eq!(
            compute_claim_uds(4, 2, past_reevals.into_iter()),
            (2, 110_000)
        );
    }
}
