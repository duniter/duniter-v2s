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

use crate::DispatchError;

pub trait IsDistanceOk<IdtyId> {
    fn is_distance_ok(idty_id: &IdtyId) -> Result<(), DispatchError>;
}

pub struct DistanceAlwaysOk;

impl<IdtyId> IsDistanceOk<IdtyId> for DistanceAlwaysOk {
    fn is_distance_ok(_idty_id: &IdtyId) -> Result<(), DispatchError> {
        Ok(())
    }
}
