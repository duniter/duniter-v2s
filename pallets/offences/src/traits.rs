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

use sp_staking::{offence::OffenceDetails, SessionIndex};

use crate::SlashStrategy;

/// Trait for handling offences.
pub trait OnOffenceHandler<Reporter, Offender, Res> {
    /// Handle an offence committed by one or more offenders.
    fn on_offence(
        offenders: &[OffenceDetails<Reporter, Offender>],
        slash_strategy: SlashStrategy,
        session: SessionIndex,
    ) -> Res;
}
