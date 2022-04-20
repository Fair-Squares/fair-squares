
pub use super::*;
mod items;
pub use crate::roles::items::*;
//pub use frame_support::dispatch::DispatchResult;
pub use frame_support::{
   dispatch::DispatchResult,
   pallet_prelude::*,
   codec::{Encode, Decode},
   traits::{Currency, ExistenceRequirement, Get, ReservableCurrency, WithdrawReasons}
};
pub use frame_system::{pallet_prelude::*,ensure_signed};
use frame_support::inherent::Vec;

use scale_info::TypeInfo;




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
}


//#[derive(Clone, Encode, Decode, Default, PartialEq, Eq, TypeInfo)]
//#[cfg_attr(feature = "std", derive(Debug))]
pub struct HouseOwner<T> {
    pub account_id: T,
    pub owner_id: u32
}
impl<T> HouseOwner<T> {

   pub fn new(account_id: T, owner_id: u32) -> Self {
      Self {
         account_id,
         owner_id
      }
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


#[derive(Clone, Encode, Decode, Default, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct House {
    pub house_nft: u32,
    pub token_id: u32
}
impl House {
   pub fn new(house_nft: u32, token_id: u32) -> Self {
      Self {
         house_nft,
         token_id
      }
   }
}
impl MaxEncodedLen for House {
   fn max_encoded_len() -> usize {
      //std::mem::size_of::<House>()
      10000
   }
}


#[derive(Clone, Encode, Decode, Default, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Proposal {
   pub house_id: u32,
   pub account_id: u32,
   pub valuation: u32,
   pub active: bool,
   pub funded: bool
}
impl Proposal {
   pub fn new(house_id: u32, account_id: u32, valuation: u32) -> Self {
      Self {
         house_id,
         account_id,
         valuation,
         active: true,
         funded: false
      }
   } 
}
impl MaxEncodedLen for Proposal {
   fn max_encoded_len() -> usize {
      10000
   }
}


#[derive(Clone, Encode, Decode, Default, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Vote {
   pub proposal_id: u32,
   pub account_id: u32,
   pub status: bool
}
impl Vote {
   pub fn new(proposal_id: u32, account_id: u32, status: bool) -> Self {
      Self {
         proposal_id,
         account_id,
         status
      }
   }
}
impl MaxEncodedLen for Vote {
   fn max_encoded_len() -> usize {
      //std::mem::size_of::<House>()
      10000
   }
}


#[derive(Clone, Encode, Decode, Default, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Ownership {
  pub account_id: u32,
  pub house_id: u32,
  pub percentage: u32
}

impl Ownership {
   pub fn new(account_id: u32, house_id: u32, percentage: u32) -> Self {
      Self {
         account_id,
         house_id,
         percentage
      }
   }
}
impl MaxEncodedLen for Ownership {
   fn max_encoded_len() -> usize {
      //std::mem::size_of::<House>()
      10000
   }
}


#[derive(Clone, Encode, Decode, Default, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Role {
   pub roles: Vec<u16>
}
impl Role {
   pub fn new() -> Self {
      Self {
         roles: Vec::<u16>::new()
      }
   }
}
impl MaxEncodedLen for Role {
   fn max_encoded_len() -> usize {
      //std::mem::size_of::<House>()
      10000
   }
}


