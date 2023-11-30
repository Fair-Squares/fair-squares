pub use super::*;
pub use sp_runtime::{BoundedVec,Percent};
pub use frame_support::{
	dispatch::{DispatchResult, GetDispatchInfo},
	ensure,
	pallet_prelude::MaxEncodedLen,
	traits::{Currency, ExistenceRequirement::KeepAlive, Get, Randomness, ReservableCurrency},
	PalletId,
};
pub use Onboarding::Zero;
pub use pallet_roles::vec;


impl<T: Config> Pallet<T> {
    
    pub fn investors_list(collection_id: T::NftCollectionId, item_id: T::NftItemId){
		//Create new round struct/Increase round count  
		let counter = Self::round_count().unwrap();		
		InvestmentRoundCount::<T>::put(counter.saturating_add(1));
		let mut round = InvestmentRound::<T>::new(collection_id,item_id);


        //get asset price
        let asset = Onboarding::Pallet::<T>::houses(collection_id,item_id).unwrap();
        let asset_price = asset.price.unwrap();
		
        let mut remaining_amount = asset_price.clone();
        //Max contribution from asset Price
		let max_contribution = T::MaxContributionper::get()*asset_price.clone();
        //Min contribution from asset price
		let min_contribution = <T as Config>::MinContributionper::get()*asset_price;
        
        //get the list of investors accounts
        let mut investors = vec![];
        Roles::InvestorLog::<T>::iter().map(|x| x.0).collect_into(&mut investors);
        investors.retain(|x|{
            let status = Houses::Pallet::<T>::contributions(x.clone()).unwrap();
            let contribution = status.contributed_balance;
            //user contributed more than 5% of asset_price to housing fund
            contribution > min_contribution //should be minimun contribution calculated from asset price.
            //ToDo: We also want to only include users that did not contributed to a purchase for y_blocks (to be defined). 

        });
		//Randomly select max number of investors per house
		let init_number = <T as Houses::Config>::MaxInvestorPerHouse::get();
		let mut inv_vec = Vec::new();
		for _i in 0..init_number+1{
			let iv = Self::choose_investor(investors.clone());
			investors.remove(iv.1);
			inv_vec.push(iv.0.unwrap());
			
		}
		let mut final_list = Vec::new();
		let mut shares = Houses::Pallet::<T>::get_contribution_share();
		//We get users shares
		shares.retain(|x|{
			inv_vec.contains(&x.account_id)
		});
		
		for investor in inv_vec{
			let status = Houses::Pallet::<T>::contributions(investor.clone()).unwrap();
			let mut fund = status.contributed_balance;
			//check if investor fund is above max contrib
			for share in shares.clone(){
				//We get user's share in the fund and recalculate contribution to the asset purchase
				if investor == share.account_id{
					if share.share< <T as Config>::MinContributionper::get(){
						fund = <T as Config>::MinContributionper::get()*fund;
					} else{
						fund = share.share*fund;
					}
					
				}
			}
			
			if fund>max_contribution{
				
				if remaining_amount>max_contribution{
				remaining_amount = remaining_amount.saturating_sub(max_contribution);
				final_list.push((investor,max_contribution));
			} else {
				final_list.push((investor,remaining_amount));
				remaining_amount = Zero::zero();
				}

			}else{
				if remaining_amount>fund{
					remaining_amount = remaining_amount.saturating_sub(fund);
					final_list.push((investor,fund));
				} else {
					final_list.push((investor,remaining_amount));
					remaining_amount = Zero::zero();
					}
			}
		}

		round.investors = BoundedVec::truncate_from(final_list);

		InvestorsList::<T>::mutate(collection_id,item_id,|val|{
			*val = Some(round);
		})
		


		


    }


    /// Randomly choose an investor from among an investors list, & returns investoraccount plus index in the list.
	/// Returns `None` if there are no investors in the list.
	fn choose_investor(investors: Vec<AccountIdOf<T>>) -> (Option<AccountIdOf<T>>,usize) {
        let total = investors.len() as u32;
		if total == 0 {
			return (None,0)
		}
		let mut random_number = Self::generate_random_number(0);

		// Best effort attempt to remove bias from modulus operator.
		for i in 1..T::MaxGenerateRandom::get() {
			if random_number < u32::MAX - u32::MAX % total && ( 0..total-1).contains(&(random_number%total)) {
				break
			}

			random_number = Self::generate_random_number(i);
		}
        let num = random_number % total; 
        let inv = investors[num as usize].clone();
		(Some(inv),num as usize)
	}


    	/// Generate a random number from a given seed.
	/// Note that there is potential bias introduced by using modulus operator.
	/// You should call this function with different seed values until the random
	/// number lies within `u32::MAX - u32::MAX % n`.
	/// TODO: deal with randomness freshness
	/// https://github.com/paritytech/substrate/issues/8311
	fn generate_random_number(seed: u32) -> u32 {
		let (random_seed, _) = T::Randomness::random(&(T::PalletId::get(), seed).encode());
		let random_number = <u32>::decode(&mut random_seed.as_ref())
			.expect("secure hashes should always be bigger than u32; qed");
		random_number
	}


}

