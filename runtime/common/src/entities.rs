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

use super::AccountId;
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

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

            // Dummy implementation only for benchmarking
            impl Default for SessionKeysWrapper {
                fn default() -> Self {
                    SessionKeysWrapper(SessionKeys{
                        grandpa: sp_core::ed25519::Public([0u8; 32]).into(),
                        babe: sp_core::sr25519::Public([0u8; 32]).into(),
                        im_online: sp_core::sr25519::Public([0u8; 32]).into(),
                        authority_discovery: sp_core::sr25519::Public([0u8; 32]).into(),
                    })
                }
            }
        }
    }
}

#[cfg_attr(feature = "std", derive(Deserialize, Serialize))]
#[derive(Clone, Encode, Decode, Default, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct IdtyData {
    pub first_eligible_ud: pallet_universal_dividend::FirstEligibleUd,
}

#[cfg(feature = "std")]
impl IdtyData {
    pub fn new() -> Self {
        Self {
            first_eligible_ud: pallet_universal_dividend::FirstEligibleUd::min(),
        }
    }
}

impl From<IdtyData> for pallet_universal_dividend::FirstEligibleUd {
    fn from(idty_data: IdtyData) -> Self {
        idty_data.first_eligible_ud
    }
}

pub struct NewOwnerKeySigner(sp_core::sr25519::Public);

impl sp_runtime::traits::IdentifyAccount for NewOwnerKeySigner {
    type AccountId = crate::AccountId;
    fn into_account(self) -> crate::AccountId {
        <[u8; 32]>::from(self.0).into()
    }
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct NewOwnerKeySignature(sp_core::sr25519::Signature);

impl sp_runtime::traits::Verify for NewOwnerKeySignature {
    type Signer = NewOwnerKeySigner;
    fn verify<L: sp_runtime::traits::Lazy<[u8]>>(&self, msg: L, signer: &crate::AccountId) -> bool {
        use sp_core::crypto::ByteArray as _;
        match sp_core::sr25519::Public::from_slice(signer.as_ref()) {
            Ok(signer) => self.0.verify(msg, &signer),
            Err(()) => false,
        }
    }
}

impl From<sp_core::sr25519::Signature> for NewOwnerKeySignature {
    fn from(a: sp_core::sr25519::Signature) -> Self {
        NewOwnerKeySignature(a)
    }
}

#[cfg_attr(feature = "std", derive(Deserialize, Serialize))]
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct SmithMembershipMetaData<SessionKeysWrapper> {
    pub owner_key: AccountId,
    pub p2p_endpoint: sp_runtime::RuntimeString,
    pub session_keys: SessionKeysWrapper,
}

impl<SessionKeysWrapper: Default> Default for SmithMembershipMetaData<SessionKeysWrapper> {
    #[cfg(not(feature = "runtime-benchmarks"))]
    fn default() -> Self {
        unreachable!()
    }
    #[cfg(feature = "runtime-benchmarks")]
    // dummy implementation for benchmarking
    fn default() -> Self {
        SmithMembershipMetaData {
            owner_key: AccountId::from([
                // Dave (FIXME avoid stupid metadata)
                48, 103, 33, 33, 29, 84, 4, 189, 157, 168, 142, 2, 4, 54, 10, 26, 154, 184, 184,
                124, 102, 193, 188, 47, 205, 211, 127, 60, 34, 34, 204, 32,
            ]),
            p2p_endpoint: sp_runtime::RuntimeString::default(),
            session_keys: SessionKeysWrapper::default(),
        }
    }
}

impl<SessionKeysWrapper> sp_membership::traits::Validate<AccountId>
    for SmithMembershipMetaData<SessionKeysWrapper>
{
    fn validate(&self, who: &AccountId) -> bool {
        &self.owner_key == who
    }
}

#[cfg_attr(feature = "std", derive(Deserialize, Serialize))]
#[derive(
    Encode, Decode, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, RuntimeDebug, TypeInfo,
)]
pub struct ValidatorFullIdentification;
