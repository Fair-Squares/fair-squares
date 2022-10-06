pub use super::*;

impl<T: Config> Pallet<T> {
	//Helper function for account creation approval by admin only
	pub fn approve_account(sender: T::AccountId, who: T::AccountId) -> DispatchResult {
		let (sellers, servicers) = Self::get_pending_approvals();
		let mut exist: bool = false;

		for (index, sell) in sellers.iter().enumerate() {
			if sell.account_id == who.clone() {
				exist = true;
				let mut seller = sell.clone();
				seller.activated = true;
				seller.verifier = sender.clone();
				HouseSellerLog::<T>::insert(&who, seller);
				RoleApprovalList::<T>::mutate(|val| {
					val.0.remove(index);
				});
				AccountsRolesLog::<T>::insert(&who, Accounts::SELLER);
				let now = <frame_system::Pallet<T>>::block_number();
				Self::deposit_event(Event::SellerCreated(now, who.clone()));
				break
			}
		}
		for (index, serv) in servicers.iter().enumerate() {
			if serv.account_id == who.clone() {
				exist = true;
				let mut servicer = serv.clone();
				servicer.activated = true;
				servicer.verifier = sender;
				ServicerLog::<T>::insert(&who, servicer);
				RoleApprovalList::<T>::mutate(|val| {
					val.1.remove(index);
				});
				AccountsRolesLog::<T>::insert(&who, Accounts::SERVICER);
				let now = <frame_system::Pallet<T>>::block_number();
				Self::deposit_event(Event::ServicerCreated(now, who));
				break
			}
		}
		ensure!(exist, Error::<T>::NotInWaitingList);
		Ok(())
	}

	pub fn check_account_role(caller: T::AccountId) -> DispatchResult {
		ensure!(!HouseSellerLog::<T>::contains_key(&caller), Error::<T>::OneRoleAllowed);
		ensure!(!InvestorLog::<T>::contains_key(&caller), Error::<T>::OneRoleAllowed);
		ensure!(!ServicerLog::<T>::contains_key(&caller), Error::<T>::OneRoleAllowed);
		ensure!(!TenantLog::<T>::contains_key(&caller), Error::<T>::OneRoleAllowed);
		ensure!(Self::total_members() < T::MaxMembers::get(), Error::<T>::TotalMembersExceeded);
		Ok(())
	}

	//Helper function for account creation rejection by admin only
	pub fn reject_account(who: T::AccountId) -> DispatchResult {
		let (sellers, servicers) = Self::get_pending_approvals();
		let mut exist: bool = false;
		for (index, sell) in sellers.iter().enumerate() {
			if sell.account_id == who.clone() {
				exist = true;
				RoleApprovalList::<T>::mutate(|val| {
					val.0.remove(index);
				});
				let now = <frame_system::Pallet<T>>::block_number();
				Self::deposit_event(Event::SellerAccountCreationRejected(now, who.clone()));
				break
			}
		}

		for (index, serv) in servicers.iter().enumerate() {
			if serv.account_id == who.clone() {
				exist = true;
				RoleApprovalList::<T>::mutate(|val| {
					val.1.remove(index);
				});
				let now = <frame_system::Pallet<T>>::block_number();
				Self::deposit_event(Event::ServicerAccountCreationRejected(now, who));
				break
			}
		}
		ensure!(exist, Error::<T>::NotInWaitingList);
		Ok(())
	}

	pub fn check_role_approval_list(account: AccountIdOf<T>) -> DispatchResult {
		let (sellers, servicers) = Self::get_pending_approvals();
		if !sellers.is_empty() {
			for seller in sellers.iter() {
				let id = &seller.account_id;
				ensure!(&account != id, Error::<T>::AlreadyWaiting);
			}
		}
		if !servicers.is_empty() {
			for servicer in servicers.iter() {
				let id = &servicer.account_id;
				ensure!(&account != id, Error::<T>::AlreadyWaiting);
			}
		}
		Ok(())
	}
}
