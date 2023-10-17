pub use super::*;
use rand::Rng;
use rand::distributions::Slice;
use sp_runtime::{BoundedVec,Percent};
use frame_support::pallet_prelude::ConstU32;
use Onboarding::Zero;



impl<T: Config> Pallet<T> {
    
    pub fn process(collection_id: T::NftCollectionId, item_id: T::NftItemId){
        //get asset price
        let asset = Onboarding::Pallet::<T>::houses(collection_id,item_id).unwrap();
        let asset_price = asset.price.unwrap();
        //Max contribution from asset Price
        //Min contribution from asset price
        
        //get the list of investors accounts
        let mut investors = vec![];
        Roles::InvestorLog::<T>::iter().map(|x| x.0).collect_into(&mut investors);
        investors.retain(|x|{
            let status = Houses::Pallet::<T>::contributions(x.clone()).unwrap();
            let contribution0 = Self::hfund_bal_to_u128(status.contributed_balance).unwrap();
            let contribution = Self::u128_to_onboarding_bal(contribution0).unwrap();
            //user contributed more than 5% of asset_price
            contribution > Percent::from_percent(5) * asset_price //should be minimun contribution calculated from asset price.
            //true

        });
        
        //Get a random sample of qualified investors 
        let inv_dist = Slice::new(&investors).unwrap();
        //let inv:BoundedVec<T,ConstU32<5>> = BoundedVec::truncate_from(inv_dist);
    }

    // Conversion of BalanceOf<T> to u128
	pub fn hfund_bal_to_u128(input: Houses::BalanceOf<T>) -> Option<u128> {
		input.try_into().ok()
	}

    // Conversion of BalanceOf<T> to u128
	pub fn u128_to_onboarding_bal(input: u128) -> Option<Onboarding::BalanceOf<T>> {
		input.try_into().ok()
	}



}

