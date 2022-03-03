
mod items;
pub use crate::roles::items::*;
pub use frame_support::dispatch::DispatchResult;
pub use frame_system::{pallet_prelude::*,ensure_signed};

pub struct Investor<T,U>{
    pub account_id:T,
    pub nft:U,
}
impl<T,U> Investor<T,U>{
    pub fn new(acc: T,nft:U)-> Self{
        Investor{
            account_id: acc,
            nft: nft,
        }        
    }

}

impl<T:frame_system::Config,U> Investor<T,U>{
    
    pub fn contribute(origin:OriginFor<T>,acc:T,val:U) -> DispatchResult{
        let _c1=Contribution::new(&acc,&val);
        let _who = ensure_signed(origin)?;
		let _now = <frame_system::Pallet<T>>::block_number();

        //function taking contribution storage and amount as inputs here
        Ok(().into())


    }
}

pub struct HouseOwner<T,U>{
    pub account_id:T,
    pub nft:U,
}

pub struct Tenant<T,U>{
    pub account_id:T,
    pub rent:U,
}
impl<T,U> Tenant<T,U>{
    pub fn new(acc: T,rent:U)-> Self{
        Tenant{
            account_id: acc,
            rent: rent,
        }
        
    }
}

