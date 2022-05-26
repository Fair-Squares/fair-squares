
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

// #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
// #[scale_info(skip_type_params(T))]
// pub struct StructToDevelop<T: Config> {
//     pub account_id: AccountIdOf<T>,
// }
// impl<T: Config> StructToDevelop<T>{
//     pub fn new(account_id: AccountIdOf<T>)-> Self{
//         Self {
//             account_id
//         }        
//     }
// }


