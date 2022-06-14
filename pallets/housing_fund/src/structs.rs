pub use super::*;

use frame_support::inherent::Vec;
pub use frame_support::{
	codec::{Decode, Encode},
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement, Get, ReservableCurrency, WithdrawReasons},
};
use scale_info::TypeInfo;

pub type StorageIndex = u32;
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct ContributionLog<T: Config> {
	pub amount: BalanceOf<T>,
	pub timestamp: BlockNumberOf<T>,
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct Contribution<T: Config> {
	pub amount: BalanceOf<T>,
	pub timestamp: BlockNumberOf<T>,
}
