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

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

use crate::Pallet;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn unlink_identity() {
        let account = account("Alice", 1, 1);

        #[extrinsic_call]
        _(RawOrigin::Signed(account));
    }

    #[benchmark]
    fn on_revoke_identity() {
        let idty: IdtyIdOf<T> = 1u32.into();

        #[block]
        {
            <Pallet<T> as pallet_identity::traits::OnRemoveIdty<T>>::on_revoked(&idty);
        }
    }
}
