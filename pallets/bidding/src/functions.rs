pub use super::*;
pub use sp_runtime::{BoundedVec,Percent};
pub use frame_support::pallet_prelude::ConstU32;
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



}

