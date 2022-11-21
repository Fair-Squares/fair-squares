//Helper functions that will be used in proposal's calls
//helper 1) get shares/owners from asset_id
pub use super::*;
pub use scale_info::prelude::boxed::Box;

pub use frame_support::pallet_prelude::*;
pub use sp_core::H256;
impl<T: Config> Pallet<T> {
	pub fn approve_representative(caller: T::AccountId, who: T::AccountId) -> DispatchResult {
		let mut representative = Roles::Pallet::<T>::get_pending_representatives(&who).unwrap();
		representative.activated = true;
		representative.assets_accounts.push(caller);
		Roles::RepresentativeLog::<T>::insert(&who, representative);
		Roles::RepApprovalList::<T>::remove(&who);
		Roles::AccountsRolesLog::<T>::insert(&who, Roles::Accounts::REPRESENTATIVE);

        Ok(())
    }

    pub fn create_proposal_hash_and_note(caller: T::AccountId,call:<T as Config>::Call) -> T::Hash {
        let origin = RawOrigin::Signed(caller);
        let call_wrap = Box::new(call);
        let proposal_hash = T::Hashing::hash_of(&call_wrap);
        let proposal_encoded: Vec<u8> = call_wrap.encode();
        match Dem::Pallet::<T>::note_preimage(origin.into(), proposal_encoded){
            Ok(_) => (),
            Err(x) if x == Error::<T>::DuplicatePreimage.into() => (),
            Err(x) => panic!("{:?}", x),
        }
        proposal_hash
    }

    pub fn caller_can_vote(caller: &T::AccountId,ownership:Share::Ownership<T>) ->bool{
        let owners = ownership.owners;
		owners.contains(caller)
    }

      pub fn balance_to_u128_option(input: <T as Assetss::Config>::Balance) -> Option<u128> {
		input.try_into().ok()
	}
    pub fn u128_to_balance_option(input: u128) -> Option<DemoBalanceOf<T>> {
		input.try_into().ok()
	}

    pub fn begin_block(now: T::BlockNumber) -> Weight {
        let max_block_weight = Weight::from_ref_time(1000_u64);
        if (now % <T as Config>::CheckPeriod::get()).is_zero() {
        let indexes = ProposalsIndexes::<T>::iter();
        for index in indexes {
            //check if the status is Finished
            let ref_infos: RefInfos<T>= Dem::Pallet::<T>::referendum_info(index.1.clone()).unwrap();
            let b = match ref_infos{
				pallet_democracy::ReferendumInfo::Finished{approved,end:_} => (1,approved),
				_=> (0,false),
			} ;
            if b.0==1{
                //get the local prop_infos and update vote result if referendum ended
                ProposalsLog::<T>::mutate(index.1,|val|{
                    let mut val0 = val.clone().unwrap();
                    if b.1==true{
                        val0.vote_result=VoteResult::ACCEPTED
                    } else {val0.vote_result=VoteResult::REJECTED}
                    *val = Some(val0)
                    
                });

            }

        }
    }
        max_block_weight    
    }
    
}
