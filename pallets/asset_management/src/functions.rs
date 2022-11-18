//Helper functions that will be used in proposal's calls
//helper 1) get shares/owners from asset_id  
pub use super::*;
pub use scale_info::prelude::boxed::Box;

pub use frame_support::pallet_prelude::*;
pub use sp_core::H256;
impl<T: Config> Pallet<T> {
    pub fn approve_representative(caller: T::AccountId, who:T::AccountId) -> DispatchResult{
        let mut representative = Roles::Pallet::<T>::get_pending_representatives(&who).unwrap();
        representative.activated = true;
        representative.assets_accounts.push(caller);
        Roles::RepresentativeLog::<T>::insert(&who,representative);
        Roles::RepApprovalList::<T>::remove(&who);
        Roles::AccountsRolesLog::<T>::insert(&who, Roles::Accounts::REPRESENTATIVE);

        Ok(())
    }

    pub fn create_proposal_hash_and_note(caller: T::AccountId,call:<T as pallet::Config>::Call) -> T::Hash {
        let origin = RawOrigin::Signed(caller);
        let call_wrap = Box::new(call);
        let proposal_hash = T::Hashing::hash_of(&call_wrap);
        let proposal_encoded: Vec<u8> = proposal_hash.encode();
        match Dem::Pallet::<T>::note_preimage(origin.into(), proposal_encoded.clone()) {
            Ok(_) => (),
            Err(x) if x == Error::<T>::DuplicatePreimage.into() => (),
            Err(x) => panic!("{:?}", x),
        }
        proposal_hash
    }
    
}
