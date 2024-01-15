use crate::{Config, CurrentSession, Pallet};
use pallet_authority_members::SessionIndex;
use sp_runtime::traits::Convert;

impl<T: Config> pallet_authority_members::OnOutgoingMember<T::MemberId> for Pallet<T> {
    fn on_outgoing_member(member_id: T::MemberId) {
        if let Some(member_id) = T::IdtyIdOfAuthorityId::convert(member_id) {
            Pallet::<T>::on_smith_goes_offline(member_id);
        }
    }
}

/// As long as a Smith is in the authority set, he will not expire.
impl<T: Config> pallet_authority_members::OnIncomingMember<T::MemberId> for Pallet<T> {
    fn on_incoming_member(member_id: T::MemberId) {
        if let Some(member_id) = T::IdtyIdOfAuthorityId::convert(member_id) {
            Pallet::<T>::on_smith_goes_online(member_id);
        }
    }
}

///
impl<T: Config> pallet_authority_members::OnNewSession for Pallet<T> {
    fn on_new_session(index: SessionIndex) {
        CurrentSession::<T>::put(index);
        Pallet::<T>::on_exclude_expired_smiths(index);
    }
}
