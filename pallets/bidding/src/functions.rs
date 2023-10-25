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
    
    pub fn initial_investors_list(collection_id: T::NftCollectionId, item_id: T::NftItemId){
        //get asset price
        let asset = Onboarding::Pallet::<T>::houses(collection_id,item_id).unwrap();
        let asset_price = asset.price.unwrap();
        let mut remaining_amount = asset_price.clone();
        //Max contribution from asset Price
        //Min contribution from asset price
        
        //get the list of investors accounts
        let mut investors = vec![];
        Roles::InvestorLog::<T>::iter().map(|x| x.0).collect_into(&mut investors);
        investors.retain(|x|{
            let status = Houses::Pallet::<T>::contributions(x.clone()).unwrap();
            let contribution = status.contributed_balance;
            //user contributed more than 5% of asset_price
            contribution > Percent::from_percent(5) * asset_price //should be minimun contribution calculated from asset price.
            //ToDo: We also want to only include users that did not contributed to a purchase for y_blocks (to be defined). 

        });
        

        //Get a random sample of qualified investors
        //Build a sample of 10 investors

        //let inv:BoundedVec<T,ConstU32<5>> = BoundedVec::truncate_from(inv_dist);

    }


    /// Randomly choose an investor from among an investors list.
	/// Returns `None` if there are no investors in the list.
	fn choose_ticket(mut investors: Vec<AccountIdOf<T>>) -> (Option<AccountIdOf<T>>,usize) {
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

