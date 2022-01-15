// Copyright 2021 Axiom-Team
//
// This file is part of Substrate-Libre-Currency.
//
// Substrate-Libre-Currency is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, version 3 of the License.
//
// Substrate-Libre-Currency is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with Substrate-Libre-Currency. If not, see <https://www.gnu.org/licenses/>.

#[macro_export]
macro_rules! pallets_config {
	{$($custom:tt)*} => {
		common_runtime::pallets_config!{
			$($custom)*

			impl pallet_authority_discovery::Config for Runtime {
				type MaxAuthorities = MaxAuthorities;
			}
			impl pallet_authorship::Config for Runtime {
				type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Babe>;
				type UncleGenerations = UncleGenerations;
				type FilterUncle = ();
				type EventHandler = ImOnline;
			}
			impl pallet_babe::Config for Runtime {
				type EpochDuration = EpochDuration;
				type ExpectedBlockTime = ExpectedBlockTime;

				// session module is the trigger
				type EpochChangeTrigger = pallet_babe::ExternalTrigger;

				type DisabledValidators = Session;

				type KeyOwnerProofSystem = Historical;

				type KeyOwnerProof = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
					KeyTypeId,
					pallet_babe::AuthorityId,
				)>>::Proof;

				type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
					KeyTypeId,
					pallet_babe::AuthorityId,
				)>>::IdentificationTuple;

				type HandleEquivocation =
					pallet_babe::EquivocationHandler<Self::KeyOwnerIdentification, Offences, ReportLongevity>;

				type WeightInfo = ();

				type MaxAuthorities = MaxAuthorities;
			}

			impl pallet_im_online::Config for Runtime {
				type AuthorityId = ImOnlineId;
				type Event = Event;
				type ValidatorSet = Historical;
				type NextSessionRotation = Babe;
				type ReportUnresponsiveness = Offences;
				type UnsignedPriority = ImOnlineUnsignedPriority;
				type WeightInfo = ();
				type MaxKeys = MaxKeys;
				type MaxPeerInHeartbeats = MaxPeerInHeartbeats;
				type MaxPeerDataEncodingSize = MaxPeerDataEncodingSize;
			}
			impl pallet_offences::Config for Runtime {
				type Event = Event;
				type IdentificationTuple = pallet_session::historical::IdentificationTuple<Self>;
				type OnOffenceHandler = ();
			}
			impl pallet_session::Config for Runtime {
				type Event = Event;
				type ValidatorId = AccountId;
				type ValidatorIdOf = sp_runtime::traits::ConvertInto;
				type ShouldEndSession = Babe;
				type NextSessionRotation = Babe;
				type SessionManager = pallet_session::historical::NoteHistoricalRoot<Self, SessionManagerImpl>;
				type SessionHandler = <opaque::SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
				type Keys = opaque::SessionKeys;
				type WeightInfo = ();
			}
			impl pallet_session::historical::Config for Runtime {
				type FullIdentification = ValidatorFullIdentification;
				type FullIdentificationOf = FullIdentificationOfImpl;
			}
			impl pallet_timestamp::Config for Runtime {
				type Moment = u64;
				type OnTimestampSet = Babe;
				type MinimumPeriod = MinimumPeriod;
				type WeightInfo = ();
			}
		}
	};
}
