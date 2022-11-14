//Helper functions that will be used in proposal's calls
//helper 1) get shares/owners from asset_id  
pub use super::*;

pub use frame_support::pallet_prelude::*;
impl<T: Config> Pallet<T> {
    pub fn approve_representative(caller: T::AccountId, who:T::AccountId) -> DispatchResult{
        let mut representative = Roles::Pallet::<T>::get_pending_representatives(&who).unwrap();
        representative.activated = true;
        representative.assets_accounts.push(caller.clone());
        Roles::RepresentativeLog::<T>::insert(&who,representative);
        Roles::RepApprovalList::<T>::remove(&who);
        Roles::AccountsRolesLog::<T>::insert(&who, Roles::Accounts::REPRESENTATIVE);

        Ok(())
    }
    
}
