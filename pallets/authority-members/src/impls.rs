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

//! Implementation of the Slashing execution logic.
//!
//! Offences are sorted in the `offences` pallet.
//! The offences are executed here based. The offenders are disconnected and
//! can be added to a blacklist to avoid futur connection.

#![allow(clippy::type_complexity)]

use super::pallet::*;
use frame_support::{pallet_prelude::Weight, traits::Get};
use pallet_offences::{traits::OnOffenceHandler, SlashStrategy};
use sp_runtime::traits::Convert;
use sp_staking::{offence::OffenceDetails, SessionIndex};

impl<T: Config>
    OnOffenceHandler<T::AccountId, pallet_session::historical::IdentificationTuple<T>, Weight>
    for Pallet<T>
where
    T: pallet_session::Config<ValidatorId = <T as frame_system::Config>::AccountId>,
{
    fn on_offence(
        offenders: &[OffenceDetails<
            T::AccountId,
            pallet_session::historical::IdentificationTuple<T>,
        >],
        strategy: SlashStrategy,
        _slash_session: SessionIndex,
    ) -> Weight {
        let mut consumed_weight = Weight::from_parts(0, 0);
        let mut add_db_reads_writes = |reads, writes| {
            consumed_weight += T::DbWeight::get().reads_writes(reads, writes);
        };

        match strategy {
            SlashStrategy::Blacklist => {
                for offender in offenders {
                    if let Some(member_id) = T::MemberIdOf::convert(offender.offender.0.clone()) {
                        Blacklist::<T>::mutate(|blacklist| {
                            if let Err(index) = blacklist.binary_search(&member_id) {
                                blacklist.insert(index, member_id);
                                Self::deposit_event(Event::MemberAddedToBlacklist {
                                    member: member_id,
                                });
                                add_db_reads_writes(0, 1);
                            }
                            Self::insert_out(member_id);
                            add_db_reads_writes(2, 1);
                        });
                    }
                }
            }
            SlashStrategy::Disconnect => {
                for offender in offenders {
                    if let Some(member_id) = T::MemberIdOf::convert(offender.offender.0.clone()) {
                        Self::insert_out(member_id);
                        add_db_reads_writes(1, 1);
                    }
                }
            }
        }
        consumed_weight
    }
}
