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



    pub fn get_coll_formatted_call(call: <T as frame_system::Config>::RuntimeCall) -> Option<Coll1Proposal<T>> {
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

    pub fn start_dem_referendum(proposal0:<T as frame_system::Config>::RuntimeCall ,delay:BlockNumberFor<T>) -> DEM::ReferendumIndex{
		//let proposal:<T as frame_system::Config>::RuntimeCall = proposal0.into();
		let bounded_proposal = <T as DEM::Config>::Preimages::bound(proposal0).unwrap();
		let threshold = DEM::VoteThreshold::SimpleMajority;    
		let referendum_index =
				DEM::Pallet::<T>::internal_start_referendum(bounded_proposal, threshold, delay);
		referendum_index
	}

	pub fn call_dispatch(
		account_id: AccountIdOf<T>,
		proposal_id: u32,
		proposal: UtilCall<T>,
	) -> UtilCall<T> {
		let origin = Self::get_origin(account_id);
		ensure_root(origin.clone()).ok();

		// We set the flag making the democracy pass vote
		let mut vote_proposal = VotingProposals::<T>::get(proposal_id).unwrap();
		vote_proposal.proposal_executed = true;

		VotingProposals::<T>::mutate(proposal_id, |val| {
			*val = Some(vote_proposal);
		});
		let dispatch_prop = vec!(proposal.clone()); 
		// The proposal is executed
		UTIL::Pallet::<T>::batch(origin.clone(),dispatch_prop).ok();
		
		proposal
	}


  
    
	pub fn get_origin(account_id: AccountIdOf<T>) -> <T as frame_system::Config>::RuntimeOrigin {
		frame_system::RawOrigin::Signed(account_id).into()
	}

	pub fn account_vote(b: BalanceOf<T>, choice:bool) -> DEM::AccountVote<BalanceOf<T>> {
		let v = DEM::Vote { aye: choice, conviction: DEM::Conviction::Locked1x };
	
		DEM::AccountVote::Standard { vote: v, balance: b }
	}

	pub fn get_dem_formatted_call(call: <T as frame_system::Config>::RuntimeCall) -> UtilCall<T>{
		let call1:UtilCall<T>=call.into();
		call1
	}
	

   
}