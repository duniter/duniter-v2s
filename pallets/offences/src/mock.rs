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

#![cfg(test)]

use crate::Config;
use crate::{self as pallet_offences, SlashStrategy};
use codec::Encode;
use frame_support::{
    parameter_types,
    traits::{ConstU32, ConstU64},
    weights::{constants::RocksDbWeight, Weight},
};
use sp_core::H256;
use sp_runtime::BuildStorage;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    Perbill,
};
use sp_staking::{
    offence::{Kind, OffenceDetails},
    SessionIndex,
};

pub struct OnOffenceHandler;

parameter_types! {
    pub static OnOffencePerbill: Vec<Perbill> = Default::default();
    pub static OffenceWeight: Weight = Default::default();
}

impl<Reporter, Offender> pallet_offences::OnOffenceHandler<Reporter, Offender, Weight>
    for OnOffenceHandler
{
    fn on_offence(
        _offenders: &[OffenceDetails<Reporter, Offender>],
        _strategy: SlashStrategy,
        _offence_session: SessionIndex,
    ) -> Weight {
        OffenceWeight::get()
    }
}

type Block = frame_system::mocking::MockBlock<Runtime>;

frame_support::construct_runtime!(
    pub struct Runtime {
        System: frame_system,
        Offences: pallet_offences,
    }
);

impl frame_system::Config for Runtime {
    type AccountData = ();
    type AccountId = u64;
    type BaseCallFilter = frame_support::traits::Everything;
    type Block = Block;
    type BlockHashCount = ConstU64<250>;
    type BlockLength = ();
    type BlockWeights = ();
    type DbWeight = RocksDbWeight;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type Lookup = IdentityLookup<Self::AccountId>;
    type MaxConsumers = ConstU32<16>;
    type Nonce = u64;
    type OnKilledAccount = ();
    type OnNewAccount = ();
    type OnSetCode = ();
    type PalletInfo = PalletInfo;
    type RuntimeCall = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeTask = ();
    type SS58Prefix = ();
    type SystemWeightInfo = ();
    type Version = ();
}

impl Config for Runtime {
    type IdentificationTuple = u64;
    type OnOffenceHandler = OnOffenceHandler;
    type RuntimeEvent = RuntimeEvent;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::<Runtime>::default()
        .build_storage()
        .unwrap();
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

pub const KIND: [u8; 16] = *b"test_report_1234";

/// Returns all offence details for the specific `kind` happened at the specific time slot.
pub fn offence_reports(kind: Kind, time_slot: u128) -> Vec<OffenceDetails<u64, u64>> {
    <crate::ConcurrentReportsIndex<Runtime>>::get(kind, time_slot.encode())
        .into_iter()
        .map(|report_id| {
            <crate::Reports<Runtime>>::get(report_id)
                .expect("dangling report id is found in ConcurrentReportsIndex")
        })
        .collect()
}

#[derive(Clone)]
pub struct Offence {
    pub validator_set_count: u32,
    pub offenders: Vec<u64>,
    pub time_slot: u128,
}

impl pallet_offences::Offence<u64> for Offence {
    type TimeSlot = u128;

    const ID: pallet_offences::Kind = KIND;

    fn offenders(&self) -> Vec<u64> {
        self.offenders.clone()
    }

    fn validator_set_count(&self) -> u32 {
        self.validator_set_count
    }

    fn time_slot(&self) -> u128 {
        self.time_slot
    }

    fn session_index(&self) -> SessionIndex {
        1
    }

    fn slash_fraction(&self, offenders_count: u32) -> Perbill {
        Perbill::from_percent(5 + offenders_count * 100 / self.validator_set_count)
    }
}
