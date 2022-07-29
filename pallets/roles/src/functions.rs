pub use super::*;

impl<T: Config> Pallet<T> {
	//Helper function for account creation approval by admin only
	pub fn approve_account(sender: T::AccountId, who: T::AccountId) -> DispatchResult {
		let waitlist = Self::get_pending_approvals();
		let sellers = waitlist.0;
		let servicers = waitlist.1;
		let mut exist: bool = false;

		for sell in sellers.iter() {
			if sell.account_id == who.clone() {
				exist = true;
				let mut sell0 = sell.clone();
				sell0.activated = true;
				sell0.verifier = sender.clone();
				HouseSellerLog::<T>::insert(&who, sell0);
				let index = sellers.iter().position(|x| *x == *sell).unwrap();
				RoleApprovalList::<T>::mutate(|val| {
					val.0.remove(index);
				});
				AccountsRolesLog::<T>::insert(&who, Accounts::SELLER);
				let now = <frame_system::Pallet<T>>::block_number();
				Self::deposit_event(Event::SellerCreated(now, who.clone()));
			}
		}
		for serv in servicers.iter() {
			if serv.account_id == who.clone() {
				exist = true;
				let mut serv0 = serv.clone();
				serv0.activated = true;
				serv0.verifier = sender.clone();
				ServicerLog::<T>::insert(&who, serv0);
				let index = servicers.iter().position(|x| *x == *serv).unwrap();
				RoleApprovalList::<T>::mutate(|val| {
					val.1.remove(index);
				});
				AccountsRolesLog::<T>::insert(&who, Accounts::SERVICER);
				let now = <frame_system::Pallet<T>>::block_number();
				Self::deposit_event(Event::ServicerCreated(now, who.clone()));
			}
		}
		ensure!(exist, Error::<T>::NotInWaitingList);
		Ok(())
	}

	pub fn check_storage(caller: T::AccountId) -> DispatchResult {
		ensure!(!HouseSellerLog::<T>::contains_key(&caller), Error::<T>::OneRoleAllowed);
		ensure!(!InvestorLog::<T>::contains_key(&caller), Error::<T>::OneRoleAllowed);
		ensure!(!ServicerLog::<T>::contains_key(&caller), Error::<T>::OneRoleAllowed);
		ensure!(!TenantLog::<T>::contains_key(&caller), Error::<T>::OneRoleAllowed);
		ensure!(Self::total_members() < T::MaxMembers::get(), Error::<T>::TotalMembersExceeded);
		Ok(())
	}

	//Helper function for account creation rejection by admin only
	pub fn reject_account(who: T::AccountId) -> DispatchResult {
		let waitlist = Self::get_pending_approvals();
		let sellers = waitlist.0;
		let servicers = waitlist.1;
		let mut exist: bool = false;
		for sell in sellers.iter() {
			if sell.account_id == who.clone() {
				exist = true;
				let index = sellers.iter().position(|x| *x == *sell).unwrap();
				RoleApprovalList::<T>::mutate(|val| {
					val.0.remove(index);
				});
				let now = <frame_system::Pallet<T>>::block_number();
				Self::deposit_event(Event::SellerAccountCreationRejected(now, who.clone()));
			}
		}

		for serv in servicers.iter() {
			if serv.account_id == who.clone() {
				exist = true;
				let index = servicers.iter().position(|x| *x == *serv).unwrap();
				RoleApprovalList::<T>::mutate(|val| {
					val.1.remove(index);
				});
				let now = <frame_system::Pallet<T>>::block_number();
				Self::deposit_event(Event::ServicerAccountCreationRejected(now, who.clone()));
			}
		}
		ensure!(exist, Error::<T>::NotInWaitingList);
		Ok(())
	}

	pub fn check_role_approval_list(account: AccountIdOf<T>) -> DispatchResult {
		let waitlists = Self::get_pending_approvals();
		let serv = waitlists.1;
		let sell = waitlists.0;
		ensure!(!HouseSellerLog::<T>::contains_key(&account), Error::<T>::OneRoleAllowed);
		ensure!(!ServicerLog::<T>::contains_key(&account), Error::<T>::OneRoleAllowed);
		if !sell.is_empty() {
			for sellers in sell.iter() {
				let id = &sellers.account_id;
				ensure!(&account != id, Error::<T>::AlreadyWaiting);
			}
		}
		if !serv.is_empty() {
			for servicers in serv.iter() {
				let id = &servicers.account_id;
				ensure!(&account != id, Error::<T>::AlreadyWaiting);
			}
		}
		Ok(())
	}
}
