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
macro_rules! runtime_apis {
	{$($custom:tt)*} => {
		common_runtime::runtime_apis! {
			$($custom)*

			impl sp_authority_discovery::AuthorityDiscoveryApi<Block> for Runtime {
				fn authorities() -> Vec<sp_authority_discovery::AuthorityId> {
					AuthorityDiscovery::authorities()
				}
			}

			impl sp_consensus_babe::BabeApi<Block> for Runtime {
				fn configuration() -> sp_consensus_babe::BabeGenesisConfiguration {
					// The choice of `c` parameter (where `1 - c` represents the
					// probability of a slot being empty), is done in accordance to the
					// slot duration and expected target block time, for safely
					// resisting network delays of maximum two seconds.
					// <https://research.web3.foundation/en/latest/polkadot/BABE/Babe/#6-practical-results>
					sp_consensus_babe::BabeGenesisConfiguration {
						slot_duration: Babe::slot_duration(),
						epoch_length: EpochDuration::get(),
						c: BABE_GENESIS_EPOCH_CONFIG.c,
						genesis_authorities: Babe::authorities().to_vec(),
						randomness: Babe::randomness(),
						allowed_slots: BABE_GENESIS_EPOCH_CONFIG.allowed_slots,
					}
				}

				fn current_epoch_start() -> sp_consensus_babe::Slot {
					Babe::current_epoch_start()
				}

				fn current_epoch() -> sp_consensus_babe::Epoch {
					Babe::current_epoch()
				}

				fn next_epoch() -> sp_consensus_babe::Epoch {
					Babe::next_epoch()
				}

				fn generate_key_ownership_proof(
					_slot: sp_consensus_babe::Slot,
					authority_id: sp_consensus_babe::AuthorityId,
				) -> Option<sp_consensus_babe::OpaqueKeyOwnershipProof> {
					use codec::Encode;

					Historical::prove((sp_consensus_babe::KEY_TYPE, authority_id))
						.map(|p| p.encode())
						.map(sp_consensus_babe::OpaqueKeyOwnershipProof::new)
				}

				fn submit_report_equivocation_unsigned_extrinsic(
					equivocation_proof: sp_consensus_babe::EquivocationProof<<Block as BlockT>::Header>,
					key_owner_proof: sp_consensus_babe::OpaqueKeyOwnershipProof,
				) -> Option<()> {
					let key_owner_proof = key_owner_proof.decode()?;

					Babe::submit_unsigned_equivocation_report(
						equivocation_proof,
						key_owner_proof,
					)
				}
			}
		}
	};
}
