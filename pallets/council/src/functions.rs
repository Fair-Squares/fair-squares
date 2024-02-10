pub use super::*;
impl<T: Config> Pallet<T> {
    pub fn get_formatted_call(call: <T as Config>::RuntimeCall) -> Option<<T as Coll::Config<Instance1>>::Proposal> {
		let call_encoded: Vec<u8> = call.encode();
		let ref_call_encoded = &call_encoded;

		if let Ok(call_formatted) = <T as Coll::Config<Instance1>>::Proposal::decode(
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

	pub fn status(owner: AccountIdOf<T>)->DispatchResult {
		let items = Roles::Asset::<T>::iter();
		let mut status = vec![];
		let mut block:BlockNumberFor<T>=<frame_system::Pallet<T>>::block_number();
		for item in items{
			if item.0 == owner && item.2==Roles::AssetStatus::REVIEWING{
				status.push(item.clone());
				block = item.1;
				break;
			}
		}
		let init = status.len() as u32;
		ensure!(init>0, Error::<T>::NoPendingRequest);
		let item0= &status[0];

		Roles::Asset::<T>::mutate(&item0.0,item0.1,|val|{			
			*val = Some(Roles::AssetStatus::VOTING); 
		});
		ensure!(Roles::Pallet::<T>::status(owner,block).unwrap()==Roles::AssetStatus::VOTING,"Operation failed!!");

		Ok(())


	}

    pub fn start_house_council_session(account: T::AccountId,collection: Nft::PossibleCollections,item_id: T::NftItemId) -> DispatchResultWithPostInfo{
		//Create proposal
		let proposal0 = 
			Call::<T>::collective_approval{
				collection,
				item_id
			};
		let proposal = Self::get_formatted_call(proposal0.into()).unwrap();		
						
		let proposal_len:u32 = proposal.using_encoded(|p| p.len() as u32);
		
		let council_member = Coll::Pallet::<T,Instance1>::members()[0].clone();
		let council_origin= Self::get_origin(council_member);

		//Start Collective refererendum
		Coll::Pallet::<T,Instance1>::propose(
			council_origin,
			2,
			Box::new(proposal.clone()),
			proposal_len,
		).ok();
		let mut index:u32 = Coll::Pallet::<T,Instance1>::proposal_count();
		index = index.saturating_sub(1);

		//Update proposal index and hash
		let proposal_hashes =  Coll::Pallet::<T,Instance1>::proposals().into_iter();
		for proposal_hash in proposal_hashes{
			let prop0 = Coll::Pallet::<T,Instance1>::proposal_of(proposal_hash.clone()).unwrap();
			if proposal == prop0{
				let mut proposal_all = ProposalOf::<T>::new(account.clone(), Some(Roles::Accounts::NONE),proposal_hash.clone());
				proposal_all.proposal_index = index;
				proposal_all.proposal_hash = proposal_hash;
				SellerProposal::<T>::insert(&account, proposal_all);
			}
			
		}

		
		Ok(().into())
	}

	pub fn vote_action(caller: T::AccountId,seller_account: T::AccountId,approve:bool) -> DispatchResultWithPostInfo{
		
		// Check that the caller is a backgroundcouncil member
		ensure!(
			Coll::Pallet::<T, Instance1>::members().contains(&caller),
			Error::<T>::NotACouncilMember
		);
		// Check that the proposal exists
		ensure!(
			SellerProposal::<T>::contains_key(&seller_account),
			Error::<T>::ProposalDoesNotExist
		);
		let proposal_all = Self::get_submitted_proposal(seller_account.clone()).unwrap();
		let proposal_hash = proposal_all.proposal_hash;
		let proposal_index = proposal_all.proposal_index;
		let origin = Self::get_origin(caller.clone());
		// Execute the council vote
		Coll::Pallet::<T, Instance1>::vote(
			origin,
			proposal_hash,
			proposal_index,
			approve,
		).ok();

		Ok(().into())
	}
	

	pub fn closing_vote(caller: T::AccountId,seller_account: T::AccountId) -> DispatchResultWithPostInfo{

		// Check that the caller is a backgroundcouncil member
		ensure!(
			Coll::Pallet::<T, Instance1>::members().contains(&caller),
			Error::<T>::NotACouncilMember
		);
		// Check that the proposal exists
		ensure!(
			SellerProposal::<T>::contains_key(&seller_account),
			Error::<T>::ProposalDoesNotExist
		);
		let proposal_all = Self::get_submitted_proposal(seller_account.clone()).unwrap();
		let proposal_hash = proposal_all.proposal_hash;
		let proposal = Coll::Pallet::<T,Instance1>::proposal_of(proposal_hash.clone()).unwrap();
		let proposal_len = proposal.clone().encoded_size();
		let index = proposal_all.proposal_index;
		let proposal_weight = proposal.get_dispatch_info().weight;
		let origin = Self::get_origin(caller.clone());
		Coll::Pallet::<T,Instance1>::close(
			origin,
			proposal_hash,
			index,
			proposal_weight,
			proposal_len as u32,
		).ok();

		SellerProposal::<T>::mutate(&seller_account,|val|{
			let mut proposal = val.clone().unwrap();
			proposal.session_closed = true;
			*val = Some(proposal);
			});

		Ok(().into())

	}

	pub fn begin_block(now: BlockNumberFor<T>) -> Weight{
		let max_block_weight = Weight::from_parts(1000_u64,0);
		if (now % <T as Config>::CheckPeriod::get()).is_zero(){
			let proposal_iter = SellerProposal::<T>::iter();
			for proposal_all in proposal_iter{
				let test = (proposal_all.1.session_closed,proposal_all.1.approved); 
				let prop = match test{
					(true,Roles::Approvals::NO) => 0,
					(true,Roles::Approvals::YES) => 1,
					_ => 2,
				};
				if prop == 0 {
					let proposal = Call::<T>::proposal_rejection
					{
						account: proposal_all.0.clone()
					};

					let council_member = Coll::Pallet::<T,Instance1>::members()[0].clone();
					proposal.dispatch_bypass_filter(frame_system::RawOrigin::Signed(council_member).into()).ok();
					SellerProposal::<T>::remove(&proposal_all.0.clone());
				} else if prop == 1 {
					SellerProposal::<T>::remove(&proposal_all.0);
				}

			}
			
		}
		max_block_weight
	}
}