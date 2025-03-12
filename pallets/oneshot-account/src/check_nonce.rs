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
use frame_support::{dispatch::DispatchInfo, pallet_prelude::Weight, traits::IsSubType};
//use frame_system::Config;
use scale_info::{
    prelude::fmt::{Debug, Formatter},
    TypeInfo,
};
use sp_runtime::{
    traits::{
        AsSystemOriginSigner, DispatchInfoOf, Dispatchable, PostDispatchInfoOf,
        TransactionExtension, ValidateResult,
    },
    transaction_validity::{TransactionSource, TransactionValidityError},
    DispatchResult,
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

impl<T: Config + TypeInfo> TransactionExtension<T::RuntimeCall> for CheckNonce<T>
where
    T::RuntimeCall: Dispatchable<Info = DispatchInfo>,
    <T::RuntimeCall as Dispatchable>::RuntimeOrigin: AsSystemOriginSigner<T::AccountId> + Clone,
    T::RuntimeCall: IsSubType<crate::Call<T>>,
{
    type Implicit = ();
    type Pre = <frame_system::CheckNonce<T> as TransactionExtension<T::RuntimeCall>>::Pre;
    type Val = <frame_system::CheckNonce<T> as TransactionExtension<T::RuntimeCall>>::Val;

    const IDENTIFIER: &'static str = "CheckNonce";

    fn validate(
        &self,
        origin: <T as frame_system::Config>::RuntimeOrigin,
        call: &T::RuntimeCall,
        info: &DispatchInfoOf<T::RuntimeCall>,
        len: usize,
        self_implicit: Self::Implicit,
        inherited_implication: &impl Encode,
        source: TransactionSource,
    ) -> ValidateResult<Self::Val, T::RuntimeCall> {
        self.0.validate(
            origin,
            call,
            info,
            len,
            self_implicit,
            inherited_implication,
            source,
        )
    }

    fn weight(&self, origin: &T::RuntimeCall) -> Weight {
        self.0.weight(origin)
    }

    fn prepare(
        self,
        val: Self::Val,
        origin: &T::RuntimeOrigin,
        call: &T::RuntimeCall,
        info: &DispatchInfoOf<T::RuntimeCall>,
        len: usize,
    ) -> Result<Self::Pre, TransactionValidityError> {
        if let Some(
            crate::Call::consume_oneshot_account { .. }
            | crate::Call::consume_oneshot_account_with_remaining { .. },
        ) = call.is_sub_type()
        {
            Ok(Self::Pre::NonceChecked)
        } else {
            self.0.prepare(val, origin, call, info, len)
        }
    }

    fn post_dispatch_details(
        pre: Self::Pre,
        info: &DispatchInfo,
        post_info: &PostDispatchInfoOf<T::RuntimeCall>,
        len: usize,
        result: &DispatchResult,
    ) -> Result<Weight, TransactionValidityError> {
        <frame_system::CheckNonce<T> as TransactionExtension<T::RuntimeCall>>::post_dispatch_details(
            pre, info, post_info, len, result,
        )
    }
}
