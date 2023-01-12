pub use super::*;
pub use frame_support::{
	assert_ok,
	dispatch::{DispatchResult, EncodeLike},
	inherent::Vec,
	pallet_prelude::*,
	sp_runtime::{
		traits::{AccountIdConversion, Hash, One, Saturating, StaticLookup, Zero},
		FixedU128, PerThing, Percent,
	},
	storage::child,
	traits::{
		Contains, Currency, ExistenceRequirement, Get, LockableCurrency, ReservableCurrency,
		UnfilteredDispatchable, WithdrawReasons,
	},
	weights::GetDispatchInfo,
	PalletId,
};
use codec::{Decode, Encode, HasCompact, MaxEncodedLen};
pub use frame_system::{ensure_signed, pallet_prelude::*, RawOrigin};
pub use scale_info::{prelude::vec, TypeInfo};
pub use sp_runtime::{
	traits::{BadOrigin, BlakeTwo256, IdentityLookup},
	Perbill,
};
pub use sp_std::boxed::Box;

pub type BalanceOf<T> =
	<<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;
pub type HashOf<T> = <T as frame_system::Config>::Hash;

/// The PaymentDetail struct stores information about the payment/escrow
/// A "payment" in virto network is similar to an escrow, it is used to
/// guarantee proof of funds and can be released once an agreed upon condition
/// has reached between the payment creator and recipient. The payment lifecycle
/// is tracked using the state field.
#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, MaxEncodedLen, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound(T: pallet::Config))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PaymentDetail<T: pallet::Config> {
	/// amount of asset used for payment
	#[codec(compact)]
	pub amount: BalanceOf<T>,
	/// incentive amount that is credited to creator for resolving
	#[codec(compact)]
	pub incentive_amount: BalanceOf<T>,
	/// enum to track payment lifecycle [Created, NeedsReview, RefundRequested,
	/// Requested]
	pub state: PaymentState<T>,
	/// account that can settle any disputes created in the payment
	pub resolver_account: T::AccountId,
	/// fee charged and recipient account details
	pub fee_detail: Option<(T::AccountId, BalanceOf<T>)>,
}

/// The `PaymentState` enum tracks the possible states that a payment can be in.
/// When a payment is 'completed' or 'cancelled' it is removed from storage and
/// hence not tracked by a state.
#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, MaxEncodedLen, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound(T: pallet::Config))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PaymentState<T: pallet::Config> {
	/// Amounts have been reserved and waiting for release/cancel
	Created,
	/// A judge needs to review and release manually
	NeedsReview,
	/// The user has requested refund and will be processed by `BlockNumber`
	RefundRequested { cancel_block: T::BlockNumber },
	/// The recipient of this transaction has created a request
	PaymentRequested,
}

