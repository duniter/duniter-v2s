// Copyright 2021-2023 Axiom-Team
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

#![allow(clippy::unnecessary_cast)]

use frame_support::weights::Weight;

/// Weight functions needed for pallet.
pub trait WeightInfo {
    fn invite_smith() -> Weight;
    fn accept_invitation() -> Weight;
    fn certify_smith() -> Weight;
    fn on_removed_wot_member() -> Weight;
    fn on_removed_wot_member_empty() -> Weight;
}

impl WeightInfo for () {
    fn invite_smith() -> Weight {
        Weight::zero()
    }

    fn accept_invitation() -> Weight {
        Weight::zero()
    }

    fn certify_smith() -> Weight {
        Weight::zero()
    }

    fn on_removed_wot_member() -> Weight {
        Weight::zero()
    }

    fn on_removed_wot_member_empty() -> Weight {
        Weight::zero()
    }
}
