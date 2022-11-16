pub use super::*;

impl<T: Config> Pallet<T> {
	// Helper function for approving sellers.
	pub fn approve_seller(sender: T::AccountId, who: T::AccountId) -> bool {
		let sellers = Self::get_pending_house_sellers();

		for (index, sell) in sellers.iter().enumerate() {
			if sell.account_id == who.clone() {
				let mut seller = sell.clone();
				seller.activated = true;
				seller.verifier = sender;
				HouseSellerLog::<T>::insert(&who, seller);
				SellerApprovalList::<T>::mutate(|list| {
					list.remove(index);
				});
				AccountsRolesLog::<T>::insert(&who, Accounts::SELLER);
				let now = <frame_system::Pallet<T>>::block_number();
				Self::deposit_event(Event::SellerCreated(now, who));
				return true;
			}
		}
		false
	}

	// Helper function for approving servicers
	pub fn approve_servicer(sender: T::AccountId, who: T::AccountId) -> bool {
		let servicers = Self::get_pending_servicers();

		for (index, serv) in servicers.iter().enumerate() {
			if serv.account_id == who.clone() {
				let mut servicer = serv.clone();
				servicer.activated = true;
				servicer.verifier = sender;
				ServicerLog::<T>::insert(&who, servicer);
				ServicerApprovalList::<T>::mutate(|list| {
					list.remove(index);
				});
				AccountsRolesLog::<T>::insert(&who, Accounts::SERVICER);
				let now = <frame_system::Pallet<T>>::block_number();
				Self::deposit_event(Event::ServicerCreated(now, who));
				return true;
			}
		}
		false
	}

	// Helper function for approving notaries
	pub fn approve_notary(sender: T::AccountId, who: T::AccountId) -> bool {
		let notaries = Self::get_pending_notaries();

		for (index, notary) in notaries.iter().enumerate() {
			if notary.account_id == who.clone() {
				let mut notary_ = notary.clone();
				notary_.activated = true;
				notary_.verifier = sender;
				NotaryLog::<T>::insert(&who, notary_);
				NotaryApprovalList::<T>::mutate(|list| {
					list.remove(index);
				});
				AccountsRolesLog::<T>::insert(&who, Accounts::NOTARY);
				let now = <frame_system::Pallet<T>>::block_number();
				Self::deposit_event(Event::NotaryCreated(now, who));
				return true;
			}
		}
		false
	}

	//Helper function for account creation approval by admin only
	pub fn approve_account(sender: T::AccountId, who: T::AccountId) -> DispatchResult {
		let exist = Self::approve_seller(sender.clone(), who.clone())
			|| Self::approve_servicer(sender.clone(), who.clone())
			|| Self::approve_notary(sender, who);
		ensure!(exist, Error::<T>::NotInWaitingList);
		Ok(())
	}

	pub fn check_account_role(caller: T::AccountId) -> DispatchResult {
		ensure!(!HouseSellerLog::<T>::contains_key(&caller), Error::<T>::OneRoleAllowed);
		ensure!(!InvestorLog::<T>::contains_key(&caller), Error::<T>::OneRoleAllowed);
		ensure!(!ServicerLog::<T>::contains_key(&caller), Error::<T>::OneRoleAllowed);
		ensure!(!TenantLog::<T>::contains_key(&caller), Error::<T>::OneRoleAllowed);
		ensure!(!RepresentativeLog::<T>::contains_key(&caller), Error::<T>::OneRoleAllowed);
		ensure!(Self::total_members() < T::MaxMembers::get(), Error::<T>::TotalMembersExceeded);
		Ok(())
	}

	//Helper function for account creation rejection by admin only
	pub fn reject_account(who: T::AccountId) -> DispatchResult {
		let sellers = Self::get_pending_house_sellers();
		let servicers = Self::get_pending_servicers();
		let mut exist: bool = false;
		for (index, sell) in sellers.iter().enumerate() {
			if sell.account_id == who.clone() {
				exist = true;
				SellerApprovalList::<T>::mutate(|list| {
					list.remove(index);
				});
				let now = <frame_system::Pallet<T>>::block_number();
				Self::deposit_event(Event::SellerAccountCreationRejected(now, who.clone()));
				break;
			}
		}

		for (index, serv) in servicers.iter().enumerate() {
			if serv.account_id == who.clone() {
				exist = true;
				ServicerApprovalList::<T>::mutate(|list| {
					list.remove(index);
				});
				let now = <frame_system::Pallet<T>>::block_number();
				Self::deposit_event(Event::ServicerAccountCreationRejected(now, who));
				break;
			}
		}
		ensure!(exist, Error::<T>::NotInWaitingList);
		Ok(())
	}

	pub fn check_role_approval_list(account: AccountIdOf<T>) -> DispatchResult {
		let sellers = Self::get_pending_house_sellers();
		if !sellers.is_empty() {
			for seller in sellers.iter() {
				let id = &seller.account_id;
				ensure!(&account != id, Error::<T>::AlreadyWaiting);
			}
		}
		let servicers = Self::get_pending_servicers();
		if !servicers.is_empty() {
			for servicer in servicers.iter() {
				let id = &servicer.account_id;
				ensure!(&account != id, Error::<T>::AlreadyWaiting);
			}
		}
		ensure!(!RepApprovalList::<T>::contains_key(&account), Error::<T>::AlreadyWaiting);

		Ok(())
	}

	pub fn init_representatives(representatives: Vec<AccountIdOf<T>>) {
		let now = <frame_system::Pallet<T>>::block_number();
		for account in representatives.into_iter() {
			AccountsRolesLog::<T>::insert(&account, Accounts::REPRESENTATIVE);
			RepresentativeLog::<T>::insert(
				&account,
				Representative::<T> {
					account_id: account.clone(),
					age: now,
					activated: true,
					assets_accounts: vec![],
				},
			);
		}
	}
}
