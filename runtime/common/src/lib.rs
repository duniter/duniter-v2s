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

#![cfg_attr(not(feature = "std"), no_std)]

mod apis;
pub mod constants;
pub mod entities;
pub mod fees;
pub mod handlers;
mod pallets_config;
pub mod providers;

pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as sp_runtime::traits::Verify>::Signer as sp_runtime::traits::IdentifyAccount>::AccountId;

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;

/// Balance of an account.
pub type Balance = u64;

/// Bock type.
pub type Block = sp_runtime::generic::Block<Header, UncheckedExtrinsic>;

/// Block identifier type.
pub type BlockId = sp_runtime::generic::BlockId<Block>;

/// An index to a block.
pub type BlockNumber = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// Block header type
pub type Header = sp_runtime::generic::Header<BlockNumber, sp_runtime::traits::BlakeTwo256>;

/// Index of an identity
pub type IdtyIndex = u32;

/// Index of a transaction in the chain.
pub type Index = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = sp_runtime::MultiSignature;

pub struct FullIdentificationOfImpl;
impl sp_runtime::traits::Convert<AccountId, Option<entities::ValidatorFullIdentification>>
    for FullIdentificationOfImpl
{
    fn convert(_: AccountId) -> Option<entities::ValidatorFullIdentification> {
        Some(entities::ValidatorFullIdentification)
    }
}

pub struct IdtyNameValidatorImpl;
impl pallet_identity::traits::IdtyNameValidator for IdtyNameValidatorImpl {
    fn validate(idty_name: &pallet_identity::IdtyName) -> bool {
        idty_name.0.len() >= 3 && idty_name.0.len() <= 64
    }
}

pub struct OwnerKeyOfImpl<Runtime>(core::marker::PhantomData<Runtime>);

impl<
        Runtime: frame_system::Config<AccountId = AccountId>
            + pallet_identity::Config<IdtyIndex = IdtyIndex>,
    > sp_runtime::traits::Convert<IdtyIndex, Option<AccountId>> for OwnerKeyOfImpl<Runtime>
{
    fn convert(idty_index: IdtyIndex) -> Option<AccountId> {
        pallet_identity::Pallet::<Runtime>::identity(idty_index)
            .map(|idty_value| idty_value.owner_key)
    }
}

#[macro_export]
macro_rules! declare_session_keys {
	{} => {
		pub mod opaque {
			use super::*;

			impl_opaque_keys! {
				pub struct SessionKeys {
					pub grandpa: Grandpa,
					pub babe: Babe,
					pub im_online: ImOnline,
					pub authority_discovery: AuthorityDiscovery,
				}
			}

			#[derive(Clone, codec::Decode, Debug, codec::Encode, Eq, PartialEq)]
			pub struct SessionKeysWrapper(pub SessionKeys);

			impl From<SessionKeysWrapper> for SessionKeys {
				fn from(keys_wrapper: SessionKeysWrapper) -> SessionKeys {
					keys_wrapper.0
				}
			}

			impl scale_info::TypeInfo for SessionKeysWrapper {
				type Identity = [u8; 128];

				fn type_info() -> scale_info::Type {
					Self::Identity::type_info()
				}
			}
		}
	}
}
