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
		for item in items{
			if item.0 == owner && item.2==Roles::AssetStatus::REVIEWING{
				status.push(item);
				break;
			}
		}
		let init = status.len() as u32;
		ensure!(init>0, Error::<T>::NoPendingRequest);
		let item0= &status[0];
		Roles::Asset::<T>::mutate(&item0.0,item0.1,|val|{			
			*val = Some(Roles::AssetStatus::VOTING); 
		});

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
				let mut proposal_all = ProposalOf::<T>::new(account.clone(), Some(Roles::Accounts::None),proposal_hash.clone());
				proposal_all.proposal_index = index;
				proposal_all.proposal_hash = proposal_hash;
				SellerProposal::<T>::insert(&account, proposal_all);
			}
			
		}

		
		Ok(().into())
	}
}