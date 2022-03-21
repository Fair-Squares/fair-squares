
mod items;
pub use crate::roles::items::*;
//pub use frame_support::dispatch::DispatchResult;
pub use frame_support::{
   dispatch::DispatchResult,
   pallet_prelude::*
};
pub use frame_system::{pallet_prelude::*,ensure_signed};
use frame_support::inherent::Vec;

pub struct Investor<T,U> {
    pub account_id:T,
    pub nft:U,
}
impl<T,U> Investor<T,U>{
    pub fn new(account_id: T,nft:U)-> Self{
        Self {
            account_id,
            nft,
        }        
    }
    
    pub fn contribute_to_fund<W>(investor: T, amount: u32) -> Contribution<T> {
       //let investor = &self.account_id;
       let result = Contribution::new(investor, amount);
       result
    }
    
    pub fn vote_proposal(investor: T, response: VoteStatus) -> Vote<T> {
       let result = Vote::<T>::new(investor, response);
       result
    }
}

impl<T:frame_system::Config,U> Investor<T,U>{
    
    pub fn contribute(origin:OriginFor<T>,account:T,amount:u32) -> DispatchResult{
        let _c1=Contribution::new(&account,amount);
        let _who = ensure_signed(origin)?;
		let _now = <frame_system::Pallet<T>>::block_number();

        //function taking contribution storage and amount as inputs here
        Ok(().into())
    }    
}

pub struct HouseOwner<T> {
    pub account_id:T,
    pub houses: Vec<House>
}
impl<T> HouseOwner<T> {

   pub fn new(account_id: T) -> Self {
      Self {
         account_id,
         houses: Vec::<House>::new()
      }
   }

   pub fn create_proposal(house: House, value: f32) -> Proposal<T> {
      let result = Proposal::<T>::new(house, value);
      result
   }
}

pub struct Tenant<T,U> {
    pub account_id:T,
    pub rent:U,
}
impl<T,U> Tenant<T,U> {
    pub fn new(account_id: T,rent:U)-> Self {
        Self {
            account_id,
            rent,
        }       
    }
}


