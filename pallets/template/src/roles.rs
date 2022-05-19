
pub use super::*;
pub use crate::items::*;

pub use frame_support::{
   dispatch::DispatchResult,
   pallet_prelude::*,
   codec::{Encode, Decode},
   traits::{Currency, ExistenceRequirement, Get, ReservableCurrency, WithdrawReasons}
};
pub use frame_system::{pallet_prelude::*,ensure_signed};
use frame_support::inherent::Vec;

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct Investor<T: Config> {
    pub account_id: AccountIdOf<T>,
}
impl<T: Config> Investor<T>{
    pub fn new(account_id: AccountIdOf<T>)-> Self{
        Self {
            account_id
        }        
    }

    pub fn create(account_id: AccountIdOf<T>) -> bool {
      if !Investors::<T>::contains_key(&account_id) {
         let investor = Investor { account_id: account_id.clone() };
         Investors::<T>::insert(account_id.clone(), investor);
         return true
      }
      false
    }

    pub fn add_contribution_fund(&self, amount: BalanceOf<T>) -> DispatchResultWithPostInfo {
      
      Ok(().into())
    }

    fn u64_to_balance_option(&self, input: u64) -> Option<BalanceOf<T>> {
      input.try_into().ok()
    }

    fn balance_to_u32_option(&self, input: BalanceOf<T>) -> Option<u32> {
      input.try_into().ok()
    }

    pub fn vote_proposal(&self, status: bool) -> DispatchResultWithPostInfo {
      
      Ok(().into())
    }
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct HouseOwner<T: Config> {
   pub account_id: AccountIdOf<T>
}
impl<T: Config> HouseOwner<T> {
   pub fn new(account_id: AccountIdOf<T>) -> Self {
      Self {
         account_id
      }
   }

   pub fn create(account_id: AccountIdOf<T>) -> bool {
      if !HouseOwners::<T>::contains_key(&account_id) {
         let house_owner = HouseOwner { account_id: account_id.clone() };
         HouseOwners::<T>::insert(account_id.clone(), house_owner);
         return true
      }
      false  
   }

   pub fn mint_house(&self) -> DispatchResultWithPostInfo {
      Ok(().into())
   }

   pub fn create_proposal(&self, valuation: BalanceOf<T>) -> DispatchResultWithPostInfo {

      Ok(().into())
   }
}

pub struct EngineProcessor<T: Config> {
   pub account_id: AccountIdOf<T>
}
impl<T: Config> EngineProcessor<T> {
   pub fn new(account_id: AccountIdOf<T>) -> Self {
      Self {
         account_id
      }
   }

   pub fn manage_proposal(&self) -> DispatchResultWithPostInfo {

      Ok(().into())
   }
}


#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct Tenant<T: Config> {
   pub account_id: AccountIdOf<T>
}
impl<T: Config> Tenant<T> {
   pub fn new(account_id: AccountIdOf<T>) -> Self {
      Self {
         account_id
      }
   }

   pub fn create(account_id: AccountIdOf<T>) -> bool {
      if !Tenants::<T>::contains_key(&account_id) {
         let tenant = Tenant { account_id: account_id.clone() };
         Tenants::<T>::insert(account_id.clone(), tenant);
         return true
      }
      false   
   }
}

