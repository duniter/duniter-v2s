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

#[macro_export]
macro_rules! offchain_config {
    () => {
        impl<LocalCall> frame_system::offchain::CreateTransaction<LocalCall> for Runtime
        where
            RuntimeCall: From<LocalCall>,
        {
            type Extension = TxExtension;

            fn create_transaction(call: RuntimeCall, extension: TxExtension) -> UncheckedExtrinsic {
                generic::UncheckedExtrinsic::new_transaction(call, extension)
            }
        }

        impl<LocalCall> frame_system::offchain::CreateInherent<LocalCall> for Runtime
        where
            RuntimeCall: From<LocalCall>,
        {
            fn create_bare(call: RuntimeCall) -> UncheckedExtrinsic {
                generic::UncheckedExtrinsic::new_bare(call)
            }

            fn create_inherent(call: RuntimeCall) -> UncheckedExtrinsic {
                generic::UncheckedExtrinsic::new_inherent(call)
            }
        }

        impl<LocalCall> frame_system::offchain::CreateTransactionBase<LocalCall> for Runtime
        where
            RuntimeCall: From<LocalCall>,
        {
            type Extrinsic = UncheckedExtrinsic;
            type RuntimeCall = RuntimeCall;
        }
    };
}
