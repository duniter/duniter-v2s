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

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

use crate::Pallet;

#[benchmarks(
        where
            T::IdtyIndex: From<u32>
)]
mod benchmarks {
    use super::*;
    use crate::types::SmithMeta;
    use scale_info::prelude::vec::Vec;

    fn assert_has_event<T: Config>(generic_event: <T as frame_system::Config>::RuntimeEvent) {
        frame_system::Pallet::<T>::assert_has_event(generic_event);
    }

    fn near_limit_issued_certs<T: Config>(receiver: T::IdtyIndex) -> Vec<T::IdtyIndex> {
        let near_limit = T::MaxByIssuer::get().max(1) - 1;
        (0..near_limit)
            .map(|i| (10_000 + i).into())
            .filter(|id| *id != receiver)
            .collect()
    }

    #[benchmark]
    fn invite_smith() {
        let issuer: T::IdtyIndex = 1.into();
        let caller: T::AccountId = T::IdtyAttr::owner_key(issuer).unwrap();
        Pallet::<T>::on_smith_goes_online(1.into());
        // Should be the last identities from the local_testnet_config
        let receiver: T::IdtyIndex = 6.into();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), receiver);

        assert_has_event::<T>(Event::<T>::InvitationSent { receiver, issuer }.into());
    }

    #[benchmark]
    fn accept_invitation() -> Result<(), BenchmarkError> {
        let issuer: T::IdtyIndex = 1.into();
        let caller: T::AccountId = T::IdtyAttr::owner_key(issuer).unwrap();
        Pallet::<T>::on_smith_goes_online(1.into());
        let caller_origin: <T as frame_system::Config>::RuntimeOrigin =
            RawOrigin::Signed(caller.clone()).into();
        // Should be the last identities from the local_testnet_config
        let receiver: T::IdtyIndex = 6.into();
        Pallet::<T>::invite_smith(caller_origin, receiver)?;
        let issuer: T::IdtyIndex = 6.into();
        let caller: T::AccountId = T::IdtyAttr::owner_key(issuer).unwrap();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller));

        assert_has_event::<T>(
            Event::<T>::InvitationAccepted {
                idty_index: receiver,
            }
            .into(),
        );
        Ok(())
    }

    #[benchmark]
    fn certify_smith() -> Result<(), BenchmarkError> {
        let issuer: T::IdtyIndex = 1.into();
        let caller: T::AccountId = T::IdtyAttr::owner_key(issuer).unwrap();
        Pallet::<T>::on_smith_goes_online(1.into());
        let caller_origin: <T as frame_system::Config>::RuntimeOrigin =
            RawOrigin::Signed(caller.clone()).into();
        // Should be the last identities from the local_testnet_config
        let receiver: T::IdtyIndex = 6.into();
        Pallet::<T>::invite_smith(caller_origin, receiver)?;
        let issuer: T::IdtyIndex = receiver;
        let caller: T::AccountId = T::IdtyAttr::owner_key(issuer).unwrap();
        let caller_origin: <T as frame_system::Config>::RuntimeOrigin =
            RawOrigin::Signed(caller.clone()).into();
        Pallet::<T>::accept_invitation(caller_origin)?;
        let issuer: T::IdtyIndex = 1.into();
        let caller: T::AccountId = T::IdtyAttr::owner_key(issuer).unwrap();
        // Keep issuer very close to MaxByIssuer to benchmark near-bound insertion/sort costs.
        Smiths::<T>::mutate(issuer, |maybe_smith_meta| {
            if let Some(smith_meta) = maybe_smith_meta {
                smith_meta.issued_certs = near_limit_issued_certs::<T>(receiver);
                smith_meta.issued_certs.sort();
            }
        });

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), receiver);

        assert_has_event::<T>(Event::<T>::SmithCertAdded { receiver, issuer }.into());
        Ok(())
    }

    #[benchmark]
    fn on_removed_wot_member() {
        let idty: T::IdtyIndex = 1.into();
        let max = T::MaxByIssuer::get().max(1);
        let mut issuers = Vec::new();
        for i in 0..max {
            let issuer: T::IdtyIndex = (20_000 + i).into();
            issuers.push(issuer);
            // Keep `receiver` present while filling the issuer close to MaxByIssuer.
            // Sorting puts `receiver` first, making remove(index) shift the most elements.
            let mut issued_certs = near_limit_issued_certs::<T>(idty);
            issued_certs.push(idty);
            issued_certs.sort();
            Smiths::<T>::insert(
                issuer,
                SmithMeta {
                    status: SmithStatus::Smith,
                    expires_on: None,
                    issued_certs,
                    received_certs: Vec::new(),
                    last_online: None,
                },
            );
        }
        // Build a dense received list so exclusion loops over many issuers and prunes issued_certs.
        Smiths::<T>::insert(
            idty,
            SmithMeta {
                status: SmithStatus::Smith,
                expires_on: None,
                issued_certs: Vec::new(),
                received_certs: issuers,
                last_online: None,
            },
        );

        #[block]
        {
            Pallet::<T>::on_removed_wot_member(idty);
        }
    }

    #[benchmark]
    fn on_removed_wot_member_empty() {
        let idty: T::IdtyIndex = 100.into();
        assert!(Smiths::<T>::get(idty).is_none());

        #[block]
        {
            Pallet::<T>::on_removed_wot_member(idty);
        }
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::new_test_ext(crate::GenesisConfig {
            initial_smiths: maplit::btreemap![
                1 => (false, vec![2, 3]),
                2 => (false, vec![1, 3]),
                3 => (false, vec![1, 2]),
            ],
        }),
        crate::mock::Runtime
    );
}
