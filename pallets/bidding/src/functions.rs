pub use super::*;
use rand::Rng;
use rand::distributions::Slice;
use sp_runtime::BoundedVec;
use frame_support::pallet_prelude::ConstU32;
use Onboarding::Zero;



impl<T: Config> Pallet<T> {
    
    pub fn process(){
        //get asset price
        //Max contribution from asset Price
        //Min contribution from asset price
        
        //get the list of investors accounts
        let mut investors = vec![];
        Roles::InvestorLog::<T>::iter().map(|x| x.0).collect_into(&mut investors);
        investors.retain(|x|{
            let status = Houses::Pallet::<T>::contributions(x.clone()).unwrap();
            let contribution = status.contributed_balance;
            //user contributed more than 2x_min_contribution
            contribution > Zero::zero() //should be minimun contribution calculated from asset price.

        });
        
        //Get a random sample of qualified investors 
        let inv_dist = Slice::new(&investors).unwrap();
        //let inv:BoundedVec<T,ConstU32<5>> = BoundedVec::truncate_from(inv_dist);
    }

}