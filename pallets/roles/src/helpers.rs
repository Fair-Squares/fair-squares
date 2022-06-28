pub use super::*;

impl<T: Config> Pallet<T> {
	//Helper function for account creation approval by admin only
	pub fn approve_account(who: T::AccountId) -> DispatchResult {
		let waitlist = Self::get_pending_approvals();
		let sellers = waitlist.0;
		let servicers = waitlist.1;

		for sell in sellers.iter() {
			if sell.account_id == who.clone() {
				HouseSellerLog::<T>::insert(&who, sell.clone());
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
				ServicerLog::<T>::insert(&who, serv);
				let index = servicers.iter().position(|x| *x == *serv).unwrap();
				RoleApprovalList::<T>::mutate(|val| {
					val.1.remove(index);
				});
				AccountsRolesLog::<T>::insert(&who, Accounts::SERVICER);
				let now = <frame_system::Pallet<T>>::block_number();
				Self::deposit_event(Event::ServicerCreated(now, who.clone()));
			}
		}
		Ok(().into())
	}

	pub fn check_storage(caller: T::AccountId) -> DispatchResult {
		ensure!(HouseSellerLog::<T>::contains_key(&caller) == false, Error::<T>::OneRoleAllowed);
		ensure!(InvestorLog::<T>::contains_key(&caller) == false, Error::<T>::OneRoleAllowed);
		ensure!(ServicerLog::<T>::contains_key(&caller) == false, Error::<T>::OneRoleAllowed);
		ensure!(TenantLog::<T>::contains_key(&caller) == false, Error::<T>::OneRoleAllowed);
		Ok(().into())
	}

	//Helper function for account creation rejection by admin only
	pub fn reject_account(who: T::AccountId) -> DispatchResult {
		let waitlist = Self::get_pending_approvals();
		let sellers = waitlist.0;
		let servicers = waitlist.1;
		for sell in sellers.iter() {
			if sell.account_id == who.clone() {
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
				let index = servicers.iter().position(|x| *x == *serv).unwrap();
				RoleApprovalList::<T>::mutate(|val| {
					val.1.remove(index);
				});
				let now = <frame_system::Pallet<T>>::block_number();
				Self::deposit_event(Event::ServicerAccountCreationRejected(now, who.clone()));
			}
		}
		Ok(().into())
	}

	pub fn check_role_approval_list(account: AccountIdOf<T>) -> DispatchResult {
		let waitlists = Self::get_pending_approvals();
		let serv = waitlists.1;
		let sell = waitlists.0;
		ensure!(!HouseSellerLog::<T>::contains_key(&account), Error::<T>::OneRoleAllowed);
		ensure!(!ServicerLog::<T>::contains_key(&account), Error::<T>::OneRoleAllowed);
		if sell.len() > 0 {
			for sellers in sell.iter() {
				let id = &sellers.account_id;
				ensure!(&account != id, Error::<T>::AlreadyWaiting);
			}
		}
		if serv.len() > 0 {
			for servicers in serv.iter() {
				let id = &servicers.account_id;
				ensure!(&account != id, Error::<T>::AlreadyWaiting);
			}
		}
		Ok(().into())
	}
}
