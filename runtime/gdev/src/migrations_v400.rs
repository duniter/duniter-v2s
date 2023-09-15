// Copyright 2021-2022 Axiom-Team
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

use crate::*;

pub struct MigrationsV400;
impl frame_support::traits::OnRuntimeUpgrade for MigrationsV400 {
    fn on_runtime_upgrade() -> Weight {
        let mut weight = Weight::from_parts(1_000_000_000, 0); // Safety margin

        type OldvalueType = AccountId;

        pallet_membership::PendingMembership::<Runtime, Instance1>::translate_values(
            |_: OldvalueType| {
                *weight.ref_time_mut() += <Runtime as frame_system::Config>::DbWeight::get().write;
                Some(())
            },
        );

        weight
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<frame_benchmarking::Vec<u8>, &'static str> {
        Ok(Vec::new())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(_state: frame_benchmarking::Vec<u8>) -> Result<(), &'static str> {
        Ok(())
    }
}
