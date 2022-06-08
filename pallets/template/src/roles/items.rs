#![cfg_attr(not(feature = "std"), no_std)]
pub use super::*;
pub use frame_support::{
    dispatch::{DispatchResult,EncodeLike},
    pallet_prelude::*,
    inherent::Vec,
    sp_runtime::traits::{AccountIdConversion,Hash, Zero,Saturating},
    storage::{child},
    traits::{Currency, ExistenceRequirement, Get, ReservableCurrency,LockableCurrency, WithdrawReasons},
    PalletId,
    assert_ok,		
 };
pub use frame_system::{pallet_prelude::*,ensure_signed};
pub use frame_support::pallet_prelude::*;
pub use scale_info::{prelude::vec,TypeInfo};
#[cfg(feature = "std")]
use frame_support::serde::{Deserialize, Serialize};


#[derive(Hash,Clone, Encode, Decode,PartialEq, Eq, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct House<T:Config> {
    pub owners:Vec<T::AccountId>,
    pub nft_index:u32,
    pub age:BlockNumberOf<T>,
    pub index:HouseIndex
}
impl<T:Config> Default for House<T>{
    fn default() -> Self{
        Self{owners: Vec::new(),nft_index:0,age:<frame_system::Pallet<T>>::block_number(),index:0}
    }
}



#[derive(Clone, Encode, Decode, Default, PartialEq, Eq, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Contribution<T:Config>{    
    pub amount:BalanceOf<T>,
    pub age:BlockNumberOf<T>,
    pub index: u32,
}

impl<T:Config>Contribution<T>{
    pub fn new(val:BalanceOf<T>)-> Self{
        Self{
            amount:val,
            age: <frame_system::Pallet<T>>::block_number(),
            index: 0,
        }
    }
}
