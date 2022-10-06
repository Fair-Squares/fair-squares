pub use super::*;

pub use frame_support::{
	inherent::Vec,
	sp_runtime::traits::Zero,
	traits::{Currency, ReservableCurrency},
};

pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;
