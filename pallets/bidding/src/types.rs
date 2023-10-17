pub use super::*;
pub use frame_support::{
	assert_ok,
	dispatch::{DispatchResult},
	pallet_prelude::*,
	sp_runtime::traits::{AccountIdConversion, Hash, Saturating, StaticLookup, Zero},
	storage::{child,bounded_vec::BoundedVec},
	traits::{
		UnfilteredDispatchable,Currency, ExistenceRequirement, Get, LockableCurrency, ReservableCurrency, WithdrawReasons,
	},
	dispatch::GetDispatchInfo,
	PalletId,
};
pub use sp_std::vec::Vec;
pub use frame_system::{ensure_signed, ensure_root, pallet_prelude::*, RawOrigin};
pub use scale_info::{prelude::vec, TypeInfo};
pub use serde::{Deserialize, Serialize};

pub type BalanceOf<T> =
	<<T as Onboarding::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type 	BalanceOf0<T> =
<<T as Houses::Config>::LocalCurrency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type BlockNumberOf<T> = BlockNumberFor<T>;