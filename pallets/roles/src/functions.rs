pub use super::*;

impl<T: Config> Pallet<T> {
	// Helper function for approving sellers.
	pub fn approve_seller(sender: T::AccountId, who: T::AccountId) -> bool {
		let sellers = Self::get_pending_house_sellers();
		let mut exist = false;

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
				exist = true;
				break
			}
		}
		exist
	}

	// Helper function for approving servicers
	pub fn approve_servicer(sender: T::AccountId, who: T::AccountId) -> bool {
		let servicers = Self::get_pending_servicers();
		let mut exist = false;

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
				exist = true;
				break
			}
		}
		exist
	}

	// Helper function for approving notaries
	pub fn approve_notary(sender: T::AccountId, who: T::AccountId) -> bool {
		let notaries = Self::get_pending_notaries();
		let mut exist = false;

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
				exist = true;
				break
			}
		}
		exist
	}

	//Helper function for account creation approval by admin only
	pub fn approve_account(sender: T::AccountId, who: T::AccountId) -> DispatchResult {
		let role = Self::get_requested_role(who.clone());
		ensure!(role.is_some(), Error::<T>::NotInWaitingList);
		let role = role.unwrap();
		let success = match role {
			Accounts::SELLER => Self::approve_seller(sender, who),
			Accounts::SERVICER => Self::approve_servicer(sender, who),
			Accounts::NOTARY => Self::approve_notary(sender, who),
			_ => false,
		};
		ensure!(success, Error::<T>::NotInWaitingList);
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

	pub fn reject_seller(who: T::AccountId) -> bool {
		let sellers = Self::get_pending_house_sellers();
		let mut exist = false;
		for (index, sell) in sellers.iter().enumerate() {
			if sell.account_id == who.clone() {
				SellerApprovalList::<T>::mutate(|list| {
					list.remove(index);
				});
				let now = <frame_system::Pallet<T>>::block_number();
				Self::deposit_event(Event::SellerAccountCreationRejected(now, who));
				exist = true;
				break
			}
		}
		exist
	}

	pub fn reject_servicer(who: T::AccountId) -> bool {
		let servicers = Self::get_pending_servicers();
		let mut exist = false;

		for (index, serv) in servicers.iter().enumerate() {
			if serv.account_id == who.clone() {
				ServicerApprovalList::<T>::mutate(|list| {
					list.remove(index);
				});
				let now = <frame_system::Pallet<T>>::block_number();
				Self::deposit_event(Event::ServicerAccountCreationRejected(now, who));
				exist = true;
				break
			}
		}
		exist
	}

	pub fn reject_notary(who: T::AccountId) -> bool {
		let notaries = Self::get_pending_notaries();
		let mut exist = false;

		for (index, notary) in notaries.iter().enumerate() {
			if notary.account_id == who.clone() {
				NotaryApprovalList::<T>::mutate(|list| {
					list.remove(index);
				});
				let now = <frame_system::Pallet<T>>::block_number();
				Self::deposit_event(Event::NotaryAccountCreationRejected(now, who));
				exist = true;
				break
			}
		}

		exist
	}

	// Helper function for account creation rejection by admin only
	pub fn reject_account(who: T::AccountId) -> DispatchResult {
		let role = Self::get_requested_role(who.clone());
		ensure!(role.is_some(), Error::<T>::NotInWaitingList);
		let role = role.unwrap();
		let success = match role {
			Accounts::SELLER => Self::reject_seller(who),
			Accounts::SERVICER => Self::reject_servicer(who),
			Accounts::NOTARY => Self::reject_notary(who),
			_ => false,
		};
		ensure!(success, Error::<T>::NotInWaitingList);
		Ok(())
	}

	pub fn balance_to_u128_option(input: BalanceOf<T>) -> Option<u128> {
		input.try_into().ok()
	}

	pub fn u128_to_balance_option(input: u128) -> Option<BalanceOf<T>> {
		input.try_into().ok()
	}

	pub fn tenant_list() -> Box<dyn Iterator<Item = T::AccountId>> {
		Box::new(TenantLog::<T>::iter_keys())
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
					activated: false,
					assets_accounts: vec![],
					index: 0
				},
			);
		}
	}
}
