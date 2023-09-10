pub use super::*;

impl<T: Config> Pallet<T> {

	// Collective Referendum functions
    pub fn start_council_session(account:T::AccountId,coll_proposal:Coll1Proposal<T>)-> DispatchResultWithPostInfo{
        let proposal_len:u32 = coll_proposal.using_encoded(|p| p.len() as u32);
		
		let council_member = COLL::Pallet::<T,Instance1>::members()[0].clone();
		let council_origin= Self::get_origin(council_member);

		//Start Collective refererendum
		COLL::Pallet::<T,Instance1>::propose(
			council_origin,
			2,
			Box::new(coll_proposal.clone()),
			proposal_len,
		).ok();

        Ok(().into())
    }



    pub fn get_coll_formatted_call(call: T::RuntimeCallv) -> Option<Coll1Proposal<T>> {
		let call_encoded: Vec<u8> = call.encode();
		let ref_call_encoded = &call_encoded;

		if let Ok(call_formatted) = Coll1Proposal::<T>::decode(
			&mut &ref_call_encoded[..],
		) {
			Some(call_formatted)
		} else {
			None
		}
	}

	// Democracy Referendum functions
	pub fn make_proposal(call: CallOf<T>) -> BoundedCallOf<T> {
		<T as DEM::Config>::Preimages::bound(call).unwrap()
	}

    pub fn start_dem_referendum(proposal:BoundedCallOf<T> ,delay:BlockNumberFor<T>) -> DEM::ReferendumIndex{
		let threshold = DEM::VoteThreshold::SimpleMajority;    
		let referendum_index =
				DEM::Pallet::<T>::internal_start_referendum(proposal, threshold, delay);
		referendum_index
	}



  
    
	pub fn get_origin(account_id: AccountIdOf<T>) -> <T as frame_system::Config>::RuntimeOrigin {
		frame_system::RawOrigin::Signed(account_id).into()
	}

	pub fn account_vote(b: BalanceOf<T>, choice:bool) -> DEM::AccountVote<BalanceOf<T>> {
		let v = DEM::Vote { aye: choice, conviction: DEM::Conviction::Locked1x };
	
		DEM::AccountVote::Standard { vote: v, balance: b }
	}

	pub fn get_dem_formatted_call(call: T::RuntimeCallv) -> T::RuntimeCallv {
		call
	}

   
}