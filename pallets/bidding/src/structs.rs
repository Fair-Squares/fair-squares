pub use super::*;

pub use frame_support::{
	traits::{Currency, ReservableCurrency },
<<<<<<< HEAD
    inherent::Vec,
=======
	inherent::Vec,
>>>>>>> 98461e11f4d88e4240dc1213d69fa6dd8e41b7c9
};

pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;
