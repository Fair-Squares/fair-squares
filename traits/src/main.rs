pub use frame_support::{
	assert_ok,
	dispatch::{DispatchResult, EncodeLike,Vec},
	pallet_prelude::*,
	sp_runtime::traits::{AccountIdConversion, Hash, Saturating, StaticLookup, Zero},
	storage::{child,bounded_vec::BoundedVec},
	traits::{
		UnfilteredDispatchable,Currency, ExistenceRequirement, Get, LockableCurrency, ReservableCurrency, WithdrawReasons,
	},
	dispatch::GetDispatchInfo,
	PalletId,
};
pub use frame_system::{ensure_signed, ensure_root, pallet_prelude::*, RawOrigin};
pub use scale_info::{prelude::vec, TypeInfo};
pub use serde::{Deserialize, Serialize};

pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

#[derive(Clone, Encode, Decode, PartialEq, Eq, TypeInfo, Copy)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub enum AssetStatus {
    EDITING,
    REVIEWING,
    VOTING,
    ONBOARDED,
    FINALISING,
    FINALISED,
    PURCHASED,
    REJECTED,
    SLASH,
    CANCELLED,
}
