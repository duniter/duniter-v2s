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

//! # Duniter Offences Pallet
//!
//! This pallet is a fork of the Substrate `offences` pallet, customized to align with the offence rules specified by the `authority-member` pallet rather than the Substrate `staking` pallet.
//!
//! ## Offences Processing
//!
//! The Duniter Offences Pallet manages various types of offences as follows:
//!
//! - **`im-online` Pallet Offences**: Offences from the `im-online` pallet necessitate disconnection of the offender.
//!
//! - **Other Offences**: For all other offences, the pallet enforces:
//!   - Disconnection of the offender.
//!   - Addition of the offender to a blacklist.
//!   - Authorization from a designated origin to remove offenders from the blacklist.
//!
//! ## Offences Triage and Slashing Execution
//!
//! This pallet handles the triage of offences, categorizing them based on predefined rules. The actual execution of slashing and other punitive measures is delegated to the `authority-member` pallet.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use core::marker::PhantomData;

use codec::Encode;
use frame_support::weights::Weight;
use scale_info::prelude::vec::Vec;
use sp_runtime::traits::Hash;
use sp_staking::offence::{Kind, Offence, OffenceDetails, OffenceError, ReportOffence};

pub use pallet::*;

pub mod traits;
use self::traits::*;

/// A binary blob which represents a SCALE codec-encoded `O::TimeSlot`.
type OpaqueTimeSlot = Vec<u8>;

/// A type alias for a report identifier.
type ReportIdOf<T> = <T as frame_system::Config>::Hash;

pub enum SlashStrategy {
    Disconnect,
    Blacklist,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    /// The pallet's config trait.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Full identification of the validator.
        type IdentificationTuple: Parameter;

        /// A handler called for every offence report.
        type OnOffenceHandler: OnOffenceHandler<Self::AccountId, Self::IdentificationTuple, Weight>;
    }

    /// The primary structure that holds all offence records keyed by report identifiers.
    #[pallet::storage]
    #[pallet::getter(fn reports)]
    pub type Reports<T: Config> = StorageMap<
        _,
        Twox64Concat,
        ReportIdOf<T>,
        OffenceDetails<T::AccountId, T::IdentificationTuple>,
    >;

    /// A vector of reports of the same kind that happened at the same time slot.
    #[pallet::storage]
    pub type ConcurrentReportsIndex<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        Kind,
        Twox64Concat,
        OpaqueTimeSlot,
        Vec<ReportIdOf<T>>,
        ValueQuery,
    >;

    /// Events type.
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event {
        /// An offense was reported during the specified time slot. This event is not deposited for duplicate slashes.
        Offence {
            kind: Kind,
            timeslot: OpaqueTimeSlot,
        },
    }
}

impl<T, O> ReportOffence<T::AccountId, T::IdentificationTuple, O> for Pallet<T>
where
    T: Config,
    O: Offence<T::IdentificationTuple>,
{
    fn report_offence(reporters: Vec<T::AccountId>, offence: O) -> Result<(), OffenceError> {
        let offenders = offence.offenders();
        let time_slot = offence.time_slot();

        // Go through all offenders in the offence report and find all offenders that were spotted
        // in unique reports.
        let TriageOutcome {
            concurrent_offenders,
        } = match Self::triage_offence_report::<O>(reporters, &time_slot, offenders) {
            Some(triage) => triage,
            None => return Err(OffenceError::DuplicateReport),
        };

        // Define the slash strategy.
        let slash_strategy = if O::ID == *b"im-online:offlin" {
            SlashStrategy::Disconnect
        } else {
            SlashStrategy::Blacklist
        };

        T::OnOffenceHandler::on_offence(
            &concurrent_offenders,
            slash_strategy,
            offence.session_index(),
        );

        Self::deposit_event(Event::Offence {
            kind: O::ID,
            timeslot: time_slot.encode(),
        });

        Ok(())
    }

    fn is_known_offence(offenders: &[T::IdentificationTuple], time_slot: &O::TimeSlot) -> bool {
        let any_unknown = offenders.iter().any(|offender| {
            let report_id = Self::report_id::<O>(time_slot, offender);
            !<Reports<T>>::contains_key(report_id)
        });

        !any_unknown
    }
}

impl<T: Config> Pallet<T> {
    /// Compute the ID for the given report properties.
    ///
    /// The report id depends on the offence kind, time slot and the id of offender.
    fn report_id<O: Offence<T::IdentificationTuple>>(
        time_slot: &O::TimeSlot,
        offender: &T::IdentificationTuple,
    ) -> ReportIdOf<T> {
        (O::ID, time_slot.encode(), offender).using_encoded(T::Hashing::hash)
    }

    /// Triages the offence report and returns the set of offenders that was involved in unique
    /// reports along with the list of the concurrent offences.
    fn triage_offence_report<O: Offence<T::IdentificationTuple>>(
        reporters: Vec<T::AccountId>,
        time_slot: &O::TimeSlot,
        offenders: Vec<T::IdentificationTuple>,
    ) -> Option<TriageOutcome<T>> {
        let mut storage = ReportIndexStorage::<T, O>::load(time_slot);

        let mut any_new = false;
        for offender in offenders {
            let report_id = Self::report_id::<O>(time_slot, &offender);

            if !<Reports<T>>::contains_key(report_id) {
                any_new = true;
                <Reports<T>>::insert(
                    report_id,
                    OffenceDetails {
                        offender,
                        reporters: reporters.clone(),
                    },
                );

                storage.insert(report_id);
            }
        }

        if any_new {
            // Load report details for the all reports happened at the same time.
            let concurrent_offenders = storage
                .concurrent_reports
                .iter()
                .filter_map(<Reports<T>>::get)
                .collect::<Vec<_>>();

            storage.save();

            Some(TriageOutcome {
                concurrent_offenders,
            })
        } else {
            None
        }
    }
}

struct TriageOutcome<T: Config> {
    /// Other reports for the same report kinds.
    concurrent_offenders: Vec<OffenceDetails<T::AccountId, T::IdentificationTuple>>,
}

/// An auxiliary struct for working with storage of indexes localized for a specific offence
/// kind (specified by the `O` type parameter).
///
/// This struct is responsible for aggregating storage writes and the underlying storage should not
/// accessed directly meanwhile.
#[must_use = "The changes are not saved without called `save`"]
struct ReportIndexStorage<T: Config, O: Offence<T::IdentificationTuple>> {
    opaque_time_slot: OpaqueTimeSlot,
    concurrent_reports: Vec<ReportIdOf<T>>,
    _phantom: PhantomData<O>,
}

impl<T: Config, O: Offence<T::IdentificationTuple>> ReportIndexStorage<T, O> {
    /// Preload indexes from the storage for the specific `time_slot` and the kind of the offence.
    fn load(time_slot: &O::TimeSlot) -> Self {
        let opaque_time_slot = time_slot.encode();

        let concurrent_reports = <ConcurrentReportsIndex<T>>::get(O::ID, &opaque_time_slot);

        Self {
            opaque_time_slot,
            concurrent_reports,
            _phantom: Default::default(),
        }
    }

    /// Insert a new report to the index.
    fn insert(&mut self, report_id: ReportIdOf<T>) {
        // Update the list of concurrent reports.
        self.concurrent_reports.push(report_id);
    }

    /// Dump the indexes to the storage.
    fn save(self) {
        <ConcurrentReportsIndex<T>>::insert(
            O::ID,
            &self.opaque_time_slot,
            &self.concurrent_reports,
        );
    }
}
