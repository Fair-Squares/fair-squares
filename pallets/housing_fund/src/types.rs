pub use super::*;
pub use frame_support::{
	assert_ok,
	dispatch::{DispatchResult},
	pallet_prelude::*,
	sp_runtime::{traits::{AccountIdConversion, Hash, Saturating, StaticLookup, Zero},Percent},
	storage::{child,bounded_vec::BoundedVec},
	traits::{
		UnfilteredDispatchable,Currency, ExistenceRequirement, Get, LockableCurrency, ReservableCurrency, WithdrawReasons,
	},
	dispatch::GetDispatchInfo,
	PalletId,
};
pub use frame_system::{ensure_signed, ensure_root, pallet_prelude::*, RawOrigin};
pub use scale_info::{prelude::vec::Vec, TypeInfo};
pub use serde::{Deserialize, Serialize};

pub type BalanceOf<T> = <<T as Config>::LocalCurrency as Currency<AccountIdOf<T>>>::Balance;
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type BlockNumberOf<T> = BlockNumberFor<T>;

pub type StorageIndex = u32;
pub type NftCollectionId<T> = <T as pallet_nft::Config>::NftCollectionId;
pub type NftItemId<T> = <T as pallet_nft::Config>::NftItemId;

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum WithdrawalReason {
	NotDefined,
}

// Contains amount and timestamp of an account
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct UserOperationsLog<T: Config> {
	// Amount transfered
	pub amount: BalanceOf<T>,
	// Block number as timestamp
	pub block_number: BlockNumberOf<T>,
}

// Contains the contributed amount of an account, ist share and his contributions history
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct UserFundStatus<T: Config> {
	// Account of the contributor
	pub account_id: AccountIdOf<T>,
	// Amount available for transactions
	pub available_balance: BalanceOf<T>,
	// Amount reserved for house bidding
	pub reserved_balance: BalanceOf<T>,
	// Amount used to purchase houses
	pub contributed_balance: BalanceOf<T>,
	// Indicate if the contributor has withdrawn from the housing fund
	pub has_withdrawn: bool,
	// Block number of the last contribution's update
	pub block_number: BlockNumberOf<T>,
	// History of the contributor's contribution
	pub contributions: Vec<UserOperationsLog<T>>,
	// History of the contributor's withdraws
	pub withdraws: Vec<UserOperationsLog<T>>,
}
impl<T: Config> UserFundStatus<T> {
	pub fn get_total_user_balance(&self) -> BalanceOf<T> {
		self.available_balance.saturating_add(self.reserved_balance)
	}

	pub fn can_reserve(&self, amount: BalanceOf<T>) -> bool {
		amount <= self.available_balance
	}

	pub fn reserve_amount(&mut self, amount: BalanceOf<T>) {
		self.available_balance = self.available_balance.saturating_sub(amount);
		self.reserved_balance = self.reserved_balance.saturating_add(amount);
	}

	pub fn unreserve_amount(&mut self, amount: BalanceOf<T>) {
		self.reserved_balance -= amount;
		self.available_balance += amount;
	}

	pub fn use_reserved_amount(&mut self, amount: BalanceOf<T>) {
		self.reserved_balance -= amount;
		self.contributed_balance += amount;
	}
}

// Contains the details of the operations that occured
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct HousingFundOperation<T: Config> {
	// The house identifier
	pub nft_collection_id: T::NftCollectionId,
	pub nft_item_id: T::NftItemId,
	// The amount of the transaction
	pub amount: BalanceOf<T>,
	// Block number of the last contribution's update
	pub block_number: BlockNumberOf<T>,
	// List of (AccountIdOf<T>, BalanceOf<T>) representing the investors and their contribution
	pub contributions: Vec<(AccountIdOf<T>, BalanceOf<T>)>,
}

// Contains the share of each investor
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct ContributionShare<T: Config> {
	// Account of the contributor
	pub account_id: AccountIdOf<T>,
	// Share of the fund
	pub share: Percent,
}

