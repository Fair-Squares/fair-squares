#![cfg_attr(not(feature = "std"), no_std)]
pub use frame_support::{
    dispatch::{DispatchResult,EncodeLike},
    pallet_prelude::*,
    inherent::Vec,
    sp_runtime::traits::{AccountIdConversion,Hash, Zero},
    storage::{child},
    traits::{Currency, ExistenceRequirement, Get, ReservableCurrency, WithdrawReasons},
    PalletId		
 };
pub use frame_system::{pallet_prelude::*,ensure_signed};
pub use frame_support::pallet_prelude::*;
pub use scale_info::{prelude::vec,TypeInfo};
//pub use parity_codec::{Encode, Decode};





#[derive(Clone, Encode, Decode, Default, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Contribution<T,U>{
    pub account:T,
    pub amount:U,
}

impl<T,U>Contribution<T,U>{
    pub fn new(acc:T,val:U)-> Self{
        Self{
            account:acc,
            amount:val,
        }
    }
}
