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

use crate::Config;

use codec::{Decode, Encode};
use frame_support::traits::IsSubType;
use frame_support::weights::DispatchInfo;
//use frame_system::Config;
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{DispatchInfoOf, Dispatchable, SignedExtension},
    transaction_validity::{TransactionValidity, TransactionValidityError},
};

#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(Runtime))]
pub struct CheckNonce<T: Config>(pub frame_system::CheckNonce<T>);

impl<T: Config> sp_std::fmt::Debug for CheckNonce<T> {
    #[cfg(feature = "std")]
    fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        write!(f, "CheckNonce({})", self.0 .0)
    }

    #[cfg(not(feature = "std"))]
    fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        Ok(())
    }
}

impl<T: Config + TypeInfo> SignedExtension for CheckNonce<T>
where
    T::Call: Dispatchable<Info = DispatchInfo> + IsSubType<crate::Call<T>>,
{
    type AccountId = <T as frame_system::Config>::AccountId;
    type Call = <T as frame_system::Config>::Call;
    type AdditionalSigned = ();
    type Pre = ();
    const IDENTIFIER: &'static str = "CheckNonce";

    fn additional_signed(&self) -> sp_std::result::Result<(), TransactionValidityError> {
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
