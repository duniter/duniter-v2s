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

use crate::Config;

use codec::{Decode, Encode};
use frame_support::{dispatch::DispatchInfo, traits::IsSubType};
//use frame_system::Config;
use scale_info::{
    prelude::fmt::{Debug, Formatter},
    TypeInfo,
};
use sp_runtime::{
    traits::{DispatchInfoOf, Dispatchable, SignedExtension},
    transaction_validity::{TransactionValidity, TransactionValidityError},
};

/// Wrapper around `frame_system::CheckNonce<T>`.
#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(Runtime))]
pub struct CheckNonce<T: Config>(pub frame_system::CheckNonce<T>);

impl<T: Config> From<frame_system::CheckNonce<T>> for CheckNonce<T> {
    fn from(check_nonce: frame_system::CheckNonce<T>) -> Self {
        Self(check_nonce)
    }
}

impl<T: Config> Debug for CheckNonce<T> {
    #[cfg(feature = "std")]
    fn fmt(&self, f: &mut Formatter) -> scale_info::prelude::fmt::Result {
        write!(f, "CheckNonce({})", self.0 .0)
    }

    #[cfg(not(feature = "std"))]
    fn fmt(&self, _: &mut Formatter) -> scale_info::prelude::fmt::Result {
        Ok(())
    }
}

impl<T: Config + TypeInfo> SignedExtension for CheckNonce<T>
where
    T::RuntimeCall: Dispatchable<Info = DispatchInfo> + IsSubType<crate::Call<T>>,
{
    type AccountId = <T as frame_system::Config>::AccountId;
    type AdditionalSigned = ();
    type Call = <T as frame_system::Config>::RuntimeCall;
    type Pre = ();

    const IDENTIFIER: &'static str = "CheckNonce";

    fn additional_signed(&self) -> Result<(), TransactionValidityError> {
        self.0.additional_signed()
    }

    fn pre_dispatch(
        self,
        who: &Self::AccountId,
        call: &Self::Call,
        info: &DispatchInfoOf<Self::Call>,
        len: usize,
    ) -> Result<(), TransactionValidityError> {
        if let Some(
            crate::Call::consume_oneshot_account { .. }
            | crate::Call::consume_oneshot_account_with_remaining { .. },
        ) = call.is_sub_type()
        {
            Ok(())
        } else {
            self.0.pre_dispatch(who, call, info, len)
        }
    }

    fn validate(
        &self,
        who: &Self::AccountId,
        call: &Self::Call,
        info: &DispatchInfoOf<Self::Call>,
        len: usize,
    ) -> TransactionValidity {
        self.0.validate(who, call, info, len)
    }
}
