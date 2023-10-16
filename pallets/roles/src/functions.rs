
pub use super::*;

impl<T: Config> Pallet<T> {

    // Helper function for approving sellers.
	pub fn approve_seller(who: T::AccountId) -> bool {
		let sellers = Self::get_pending_house_sellers();
		let mut exist = false;
		

		for (index, sell) in sellers.iter().enumerate() {
			if sell.account_id == who.clone() {
				let mut seller = sell.clone();
				seller.activated = true;
				HouseSellerLog::<T>::insert(&who, seller);
				SellerApprovalList::<T>::mutate(|list| {
					list.remove(index);
				});
				
				AccountsRolesLog::<T>::mutate(&who,|val|{					
					val.try_push(Accounts::SELLER).ok();					
				});
				//AccountsRolesLog::<T>::insert(&who, Accounts::SELLER);
				let now = <frame_system::Pallet<T>>::block_number();
				Self::deposit_event(Event::SellerCreated(now, who));
				exist = true;
				break;
			}
		}
		exist
	}



    // Helper function for approving servicers
	pub fn approve_servicer(who: T::AccountId) -> bool {
		let servicers = Self::get_pending_servicers();
		let mut exist = false;

		for (index, serv) in servicers.iter().enumerate() {
			if serv.account_id == who.clone() {
				let mut servicer = serv.clone();
				servicer.activated = true;
				ServicerLog::<T>::insert(&who, servicer);
				ServicerApprovalList::<T>::mutate(|list| {
					list.remove(index);
				});
				AccountsRolesLog::<T>::mutate(&who,|val|{
					val.try_push(Accounts::SERVICER).ok();
				});
				//AccountsRolesLog::<T>::insert(&who, Accounts::SERVICER);
				let now = <frame_system::Pallet<T>>::block_number();
				Self::deposit_event(Event::ServicerCreated(now, who));
				exist = true;
				break;
			}
		}
		exist
	}

	// Helper function for approving notaries
	pub fn approve_notary(who: T::AccountId) -> bool {
		let notaries = Self::get_pending_notaries();
		let mut exist = false;

		for (index, notary) in notaries.iter().enumerate() {
			if notary.account_id == who.clone() {
				let mut notary_ = notary.clone();
				notary_.activated = true;
				NotaryLog::<T>::insert(&who, notary_);
				NotaryApprovalList::<T>::mutate(|list| {
					list.remove(index);
				});

				AccountsRolesLog::<T>::mutate(&who,|val|{
					val.try_push(Accounts::NOTARY).ok();
				});
				//AccountsRolesLog::<T>::insert(&who, Accounts::NOTARY);
				let now = <frame_system::Pallet<T>>::block_number();
				Self::deposit_event(Event::NotaryCreated(now, who));
				exist = true;
				break;
			}
		}
		exist
	}

    //Helper function for account creation approval by admin only
	pub fn approve_account(who: T::AccountId) -> DispatchResultWithPostInfo{
		let role = Self::get_requested_role(who.clone()).unwrap().role;
		ensure!(role.is_some(), Error::<T>::NotInWaitingList);
		let role = role.unwrap();
		if !AccountsRolesLog::<T>::contains_key(who.clone()){
			Self::increase_total_members().ok();
		}
		let success = match role {
			Accounts::SELLER => Self::approve_seller( who.clone()),
			Accounts::SERVICER => Self::approve_servicer(who.clone()),
			Accounts::NOTARY => Self::approve_notary(who.clone()),
			_ => false,
		};
		ensure!(success, Error::<T>::NotInWaitingList);
		
		Ok(().into())
	}

    // TODO: This function can be updated
	pub fn check_account_role(_caller: T::AccountId) -> DispatchResult {
		//ensure!(!HouseSellerLog::<T>::contains_key(&caller), Error::<T>::OneRoleAllowed);
		//ensure!(!InvestorLog::<T>::contains_key(&caller), Error::<T>::OneRoleAllowed);
		//ensure!(!ServicerLog::<T>::contains_key(&caller), Error::<T>::OneRoleAllowed);
		//ensure!(!TenantLog::<T>::contains_key(&caller), Error::<T>::OneRoleAllowed);
		//ensure!(!RepresentativeLog::<T>::contains_key(&caller), Error::<T>::OneRoleAllowed);
		ensure!(Self::total_members() < <T as Config>::MaxMembers::get(), Error::<T>::TotalMembersExceeded);
		Ok(().into())
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
				break;
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
				break;
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
				break;
			}
		}

		exist
	}

    // Helper function for account creation rejection by admin only
	pub fn reject_account(who: T::AccountId) -> DispatchResultWithPostInfo {
		let role = Self::get_requested_role(who.clone()).unwrap().role;
		ensure!(role.is_some(), Error::<T>::NotInWaitingList);
		let role = role.unwrap();
		let success = match role {
			Accounts::SELLER => Self::reject_seller(who),
			Accounts::SERVICER => Self::reject_servicer(who),
			Accounts::NOTARY => Self::reject_notary(who),
			_ => false,
		};
		ensure!(success, Error::<T>::NotInWaitingList);
		Ok(().into())
	}

	pub fn tenant_list() -> Box<dyn Iterator<Item = T::AccountId>> {
		Box::new(TenantLog::<T>::iter_keys())
	}

    pub fn increase_total_members() -> DispatchResult {
		let members: u32 = Self::total_members();
		ensure!(members < <T as Config>::MaxMembers::get(), Error::<T>::TotalMembersExceeded);
		TotalMembers::<T>::put(members.saturating_add(1));

		Ok(().into())
	}


	pub fn start_council_session(account: T::AccountId,account_type: Accounts) -> DispatchResultWithPostInfo{
		//Create proposal
		let proposal0 = 
			Call::<T>::account_approval{
				account: account.clone()
			};
		let proposal = Self::get_formatted_call(proposal0.into()).unwrap();

		
						
		let proposal_len:u32 = proposal.using_encoded(|p| p.len() as u32);
		
		let council_member = Coll::Pallet::<T,Instance2>::members()[0].clone();
		let council_origin= Self::get_origin(council_member);

		//Start Collective refererendum
		Coll::Pallet::<T,Instance2>::propose(
			council_origin,
			2,
			Box::new(proposal.clone()),
			proposal_len,
		).ok();
		let mut index:u32 = Coll::Pallet::<T,Instance2>::proposal_count();
		index = index.saturating_sub(1);

		//Update proposal index and hash
		let proposal_hashes =  Coll::Pallet::<T,Instance2>::proposals().into_iter();
		for proposal_hash in proposal_hashes{
			let prop0 = Coll::Pallet::<T,Instance2>::proposal_of(proposal_hash.clone()).unwrap();
			if proposal == prop0{
				let mut proposal_all = Proposal::<T>::new(account.clone(), Some(account_type),proposal_hash.clone());
				proposal_all.proposal_index = index;
				proposal_all.proposal_hash = proposal_hash;
				RequestedRoles::<T>::insert(&account, proposal_all);
			}
			
		}

		
		Ok(().into())
	}

	pub fn vote_action(caller: T::AccountId,candidate_account: T::AccountId,approve:bool) -> DispatchResultWithPostInfo{
		
		// Check that the caller is a backgroundcouncil member
		ensure!(
			Coll::Pallet::<T, Instance2>::members().contains(&caller),
			Error::<T>::NotACouncilMember
		);
		// Check that the proposal exists
		ensure!(
			RequestedRoles::<T>::contains_key(&candidate_account),
			Error::<T>::ProposalDoesNotExist
		);
		let proposal_all = Self::get_requested_role(candidate_account.clone()).unwrap();
		let proposal_hash = proposal_all.proposal_hash;
		let proposal_index = proposal_all.proposal_index;
		let origin = Self::get_origin(caller.clone());
		// Execute the council vote
		Coll::Pallet::<T, Instance2>::vote(
			origin,
			proposal_hash,
			proposal_index,
			approve,
		).ok();

		Ok(().into())
	}

	pub fn closing_vote(caller: T::AccountId,candidate_account: T::AccountId) -> DispatchResultWithPostInfo{

		// Check that the caller is a backgroundcouncil member
		ensure!(
			Coll::Pallet::<T, Instance2>::members().contains(&caller),
			Error::<T>::NotACouncilMember
		);
		// Check that the proposal exists
		ensure!(
			RequestedRoles::<T>::contains_key(&candidate_account),
			Error::<T>::ProposalDoesNotExist
		);
		let proposal_all = Self::get_requested_role(candidate_account.clone()).unwrap();
		let proposal_hash = proposal_all.proposal_hash;
		let proposal = Coll::Pallet::<T,Instance2>::proposal_of(proposal_hash.clone()).unwrap();
		let proposal_len = proposal.clone().encoded_size();
		let index = proposal_all.proposal_index;
		let proposal_weight = proposal.get_dispatch_info().weight;
		let origin = Self::get_origin(caller.clone());
		Coll::Pallet::<T,Instance2>::close(
			origin,
			proposal_hash,
			index,
			proposal_weight,
			proposal_len as u32,
		).ok();

		RequestedRoles::<T>::mutate(&candidate_account,|val|{
			let mut proposal = val.clone().unwrap();
			proposal.session_closed = true;
			*val = Some(proposal);
			});

		Ok(().into())

	}

	pub fn begin_block(now: BlockNumberOf<T>) -> Weight{
		let max_block_weight = Weight::from_parts(1000_u64,0);
		if (now % T::CheckPeriod::get()).is_zero(){
			let proposal_iter = RequestedRoles::<T>::iter();
			for proposal_all in proposal_iter{
				let test = (proposal_all.1.session_closed,proposal_all.1.approved); 
				let prop = match test{
					(true,false) => 0,
					_ => 1,
				};
				if prop == 0 {
					let proposal = Call::<T>::account_rejection
					{
						account: proposal_all.0
					};

					let council_member = Coll::Pallet::<T,Instance2>::members()[0].clone();
					proposal.dispatch_bypass_filter(frame_system::RawOrigin::Signed(council_member).into()).ok();
				}
			}
			
		}
		max_block_weight
	}

	pub fn get_formatted_call(call: <T as Config>::RuntimeCall) -> Option<<T as Coll::Config<Instance2>>::Proposal> {
		let call_encoded: Vec<u8> = call.encode();
		let ref_call_encoded = &call_encoded;

		if let Ok(call_formatted) = <T as pallet_collective::Config<Instance2>>::Proposal::decode(
			&mut &ref_call_encoded[..],
		) {
			Some(call_formatted)
		} else {
			None
		}
	}

	pub fn get_origin(account_id: AccountIdOf<T>) -> <T as frame_system::Config>::RuntimeOrigin {
		frame_system::RawOrigin::Signed(account_id).into()
	}
}