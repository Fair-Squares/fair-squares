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

   /* pub fn start_house_council_session(account: T::AccountId,account_type: Accounts) -> DispatchResultWithPostInfo{
		//Create proposal
		let proposal0 = 
			Call::<T>::account_approval{
				account: account.clone()
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
				let mut proposal_all = ProposalOf::<T>::new(account.clone(), Some(account_type),proposal_hash.clone());
				proposal_all.proposal_index = index;
				proposal_all.proposal_hash = proposal_hash;
				SellerProposal::<T>::insert(&account, proposal_all);
			}
			
		}

		
		Ok(().into())
	}*/
}