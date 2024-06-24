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

use codec::{Decode, Encode, Error, Input, MaxEncodedLen, Output};
use core::num::NonZeroU16;
use sp_runtime::RuntimeDebug;

pub type UdIndex = u16;

/// Represents the first eligible Universal Dividend.
#[derive(
    Clone, Copy, Default, Eq, PartialEq, RuntimeDebug, serde::Deserialize, serde::Serialize,
)]
pub struct FirstEligibleUd(pub Option<NonZeroU16>);

impl FirstEligibleUd {
    pub fn min() -> Self {
        Self(Some(NonZeroU16::new(1).expect("unreachable")))
    }
}

impl From<UdIndex> for FirstEligibleUd {
    fn from(ud_index: UdIndex) -> Self {
        FirstEligibleUd(NonZeroU16::new(ud_index))
    }
}

impl From<FirstEligibleUd> for Option<UdIndex> {
    fn from(first_eligible_ud: FirstEligibleUd) -> Self {
        first_eligible_ud.0.map(|ud_index| ud_index.get())
    }
}

impl Encode for FirstEligibleUd {
    fn size_hint(&self) -> usize {
        self.as_u16().size_hint()
    }

    fn encode_to<W: Output + ?Sized>(&self, dest: &mut W) {
        self.as_u16().encode_to(dest)
    }

    fn encode(&self) -> sp_std::vec::Vec<u8> {
        self.as_u16().encode()
    }

    fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
        self.as_u16().using_encoded(f)
    }
}

impl codec::EncodeLike for FirstEligibleUd {}

impl Decode for FirstEligibleUd {
    fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
        Ok(match NonZeroU16::new(Decode::decode(input)?) {
            Some(non_zero_u16) => Self(Some(non_zero_u16)),
            None => Self(None),
        })
    }
}

impl MaxEncodedLen for FirstEligibleUd {
    fn max_encoded_len() -> usize {
        u16::max_encoded_len()
    }
}

impl scale_info::TypeInfo for FirstEligibleUd {
    type Identity = UdIndex;

    fn type_info() -> scale_info::Type {
        Self::Identity::type_info()
    }
}

impl FirstEligibleUd {
    // private
    #[inline(always)]
    fn as_u16(&self) -> UdIndex {
        self.0.map(|ud_index| ud_index.get()).unwrap_or_default()
    }
}
