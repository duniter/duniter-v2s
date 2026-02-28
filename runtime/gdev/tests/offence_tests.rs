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

mod common;

use codec::{Decode, Encode};
use common::*;
use finality_grandpa::{Equivocation as GrandpaEquivocation, Message as GrandpaMessage, Prevote};
use frame_support::{
    assert_ok,
    traits::{ValidatorSet, ValidatorSetWithIdentification},
};
use gdev_runtime::*;
use pallet_im_online as im_online;
use pallet_im_online::UnresponsivenessOffence;
use pallet_session::historical::IdentificationTuple;
use sp_core::{
    H256, Pair,
    offchain::{OffchainWorkerExt, TransactionPoolExt, testing},
};
use sp_runtime::traits::Convert;
use sp_staking::offence::ReportOffence;

fn generate_grandpa_equivocation_proof(
    set_id: sp_consensus_grandpa::SetId,
    round: sp_consensus_grandpa::RoundNumber,
    target_1: (H256, u32),
    target_2: (H256, u32),
    offender: &sp_consensus_grandpa::AuthorityPair,
) -> sp_consensus_grandpa::EquivocationProof<H256, u32> {
    let signed_prevote = |target_hash, target_number| {
        let prevote = Prevote {
            target_hash,
            target_number,
        };
        let prevote_msg = GrandpaMessage::Prevote(prevote.clone());
        let payload = sp_consensus_grandpa::localized_payload(round, set_id, &prevote_msg);
        let signature = offender.sign(&payload);
        (prevote, signature)
    };

    let (prevote1, signed1) = signed_prevote(target_1.0, target_1.1);
    let (prevote2, signed2) = signed_prevote(target_2.0, target_2.1);

    sp_consensus_grandpa::EquivocationProof::new(
        set_id,
        sp_consensus_grandpa::Equivocation::Prevote(GrandpaEquivocation {
            round_number: round,
            identity: offender.public(),
            first: (prevote1, signed1),
            second: (prevote2, signed2),
        }),
    )
}

#[test]
fn test_imonline_offence() {
    ExtBuilder::new(1, 3, 4).build().execute_with(|| {
        run_to_block(1);
        let session_index = Session::current_index();
        let current_validators = <Runtime as im_online::Config>::ValidatorSet::validators();
        // Construct an offence where all validators (member: 1) are offenders
        let offenders = current_validators
            .into_iter()
            .filter_map(|id| {
                <<Runtime as im_online::Config>::ValidatorSet as ValidatorSetWithIdentification<
                    sp_runtime::AccountId32,
                >>::IdentificationOf::convert(id.clone())
                .map(|full_id| (id, full_id))
            })
            .collect::<Vec<IdentificationTuple<Runtime>>>();
        let keys = pallet_im_online::Keys::<Runtime>::get();
        let validator_set_count = keys.len() as u32;
        let offence = UnresponsivenessOffence {
            session_index,
            validator_set_count,
            offenders,
        };
        assert_ok!(
            <Runtime as pallet_im_online::Config>::ReportUnresponsiveness::report_offence(
                vec![],
                offence
            )
        );
        // An offence is deposited
        System::assert_has_event(RuntimeEvent::Offences(pallet_offences::Event::Offence {
            kind: *b"im-online:offlin",
            timeslot: vec![0, 0, 0, 0],
        }));
        // Offenders are punished
        System::assert_has_event(RuntimeEvent::AuthorityMembers(
            pallet_authority_members::Event::MemberGoOffline { member: 1 },
        ));
        assert_eq!(AuthorityMembers::blacklist().len(), 0);
    })
}
#[test]
fn test_grandpa_offence() {
    ExtBuilder::new(1, 3, 4).build().execute_with(|| {
        run_to_block(1);
        let session_index = Session::current_index();
        let current_validators = <Runtime as pallet_im_online::Config>::ValidatorSet::validators();
        // Construct an offence where all validators (member: 1) are offenders
        let mut offenders = current_validators
            .into_iter()
            .filter_map(|id| {
                <Runtime as pallet_session::historical::Config>::FullIdentificationOf::convert(
                    id.clone(),
                )
                .map(|full_id| (id, full_id))
            })
            .collect::<Vec<IdentificationTuple<Runtime>>>();
        let keys = pallet_im_online::Keys::<Runtime>::get();
        let validator_set_count = keys.len() as u32;
        let time_slot = pallet_grandpa::TimeSlot {
            set_id: 0,
            round: 0,
        };
        let offence = pallet_grandpa::EquivocationOffence {
            time_slot,
            session_index,
            validator_set_count,
            offender: offenders.pop().unwrap(),
        };
        assert_ok!(Offences::report_offence(vec![], offence));
        // An offence is deposited
        System::assert_has_event(RuntimeEvent::Offences(pallet_offences::Event::Offence {
            kind: *b"grandpa:equivoca",
            timeslot: vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        }));
        // Offenders are punished
        System::assert_has_event(RuntimeEvent::AuthorityMembers(
            pallet_authority_members::Event::MemberGoOffline { member: 1 },
        ));
        System::assert_has_event(RuntimeEvent::AuthorityMembers(
            pallet_authority_members::Event::MemberAddedToBlacklist { member: 1 },
        ));
        assert_eq!(AuthorityMembers::blacklist().len(), 1);
    })
}
#[test]
fn test_babe_offence() {
    ExtBuilder::new(1, 3, 4).build().execute_with(|| {
        run_to_block(1);
        let session_index = Session::current_index();
        let current_validators = <Runtime as pallet_im_online::Config>::ValidatorSet::validators();
        // Construct an offence where all validators (member: 1) are offenders
        let mut offenders = current_validators
            .into_iter()
            .filter_map(|id| {
                <Runtime as pallet_session::historical::Config>::FullIdentificationOf::convert(
                    id.clone(),
                )
                .map(|full_id| (id, full_id))
            })
            .collect::<Vec<IdentificationTuple<Runtime>>>();
        let keys = pallet_im_online::Keys::<Runtime>::get();
        let validator_set_count = keys.len() as u32;
        let offence = pallet_babe::EquivocationOffence {
            slot: 0u64.into(),
            session_index,
            validator_set_count,
            offender: offenders.pop().unwrap(),
        };
        assert_ok!(Offences::report_offence(vec![], offence));
        // An offence is deposited
        System::assert_has_event(RuntimeEvent::Offences(pallet_offences::Event::Offence {
            kind: *b"babe:equivocatio",
            timeslot: vec![0, 0, 0, 0, 0, 0, 0, 0],
        }));
        // Offenders are punished
        System::assert_has_event(RuntimeEvent::AuthorityMembers(
            pallet_authority_members::Event::MemberGoOffline { member: 1 },
        ));
        System::assert_has_event(RuntimeEvent::AuthorityMembers(
            pallet_authority_members::Event::MemberAddedToBlacklist { member: 1 },
        ));
        assert_eq!(AuthorityMembers::blacklist().len(), 1);
    })
}

#[test]
fn test_grandpa_runtime_api_submit_report_equivocation_unsigned_extrinsic() {
    let mut ext = ExtBuilder::new(1, 3, 4).build();
    let (offchain, _offchain_state) = testing::TestOffchainExt::new();
    let (pool, pool_state) = testing::TestTransactionPoolExt::new();
    ext.register_extension(OffchainWorkerExt::new(offchain));
    ext.register_extension(TransactionPoolExt::new(pool));

    ext.execute_with(|| {
        run_to_block(1);

        let set_id = Grandpa::current_set_id();
        let offender_pair = sp_consensus_grandpa::AuthorityPair::from_string("//Alice", None)
            .expect("static seed should be valid");
        let offender_id: sp_consensus_grandpa::AuthorityId = offender_pair.public();

        let key_owner_proof = gdev_runtime::api::dispatch(
            "GrandpaApi_generate_key_ownership_proof",
            (set_id, offender_id).encode().as_slice(),
        )
        .expect("runtime api should return bytes");
        let key_owner_proof: Option<sp_consensus_grandpa::OpaqueKeyOwnershipProof> =
            Decode::decode(&mut key_owner_proof.as_slice())
                .expect("runtime api output should decode");
        let key_owner_proof = key_owner_proof.expect("key ownership proof should exist");

        let equivocation_proof = generate_grandpa_equivocation_proof(
            set_id,
            1,
            (H256::repeat_byte(1), 1),
            (H256::repeat_byte(2), 2),
            &offender_pair,
        );

        let submitted = gdev_runtime::api::dispatch(
            "GrandpaApi_submit_report_equivocation_unsigned_extrinsic",
            (equivocation_proof, key_owner_proof).encode().as_slice(),
        )
        .expect("runtime api should return bytes");
        let submitted: Option<()> =
            Decode::decode(&mut submitted.as_slice()).expect("runtime api output should decode");
        assert_eq!(submitted, Some(()));

        // The runtime API should submit exactly one unsigned extrinsic in the local pool.
        let txs = &pool_state.read().transactions;
        assert_eq!(txs.len(), 1);

        let xt = UncheckedExtrinsic::decode(&mut txs[0].as_slice())
            .expect("submitted transaction should decode");
        assert!(matches!(
            xt.function,
            RuntimeCall::Grandpa(pallet_grandpa::Call::report_equivocation_unsigned { .. })
        ));
    })
}
