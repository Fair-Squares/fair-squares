pub use super::*;

pub use frame_support::{
	codec::{Decode, Encode},
	dispatch::DispatchResult,
	inherent::Vec,
	pallet_prelude::*,
	sp_runtime::traits::{AccountIdConversion, Zero},
	traits::{Currency, ExistenceRequirement, Get, ReservableCurrency, WithdrawReasons},
	transactional, PalletId,
};
pub use frame_system::{ensure_signed, pallet_prelude::*};
use scale_info::TypeInfo;

pub type StorageIndex = u32;
pub type NftCollectionId<T> = <T as pallet_nft::Config>::NftCollectionId;
pub type NftItemId<T> = <T as pallet_nft::Config>::NftItemId;
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type BalanceOf<T> = <<T as Config>::LocalCurrency as Currency<AccountIdOf<T>>>::Balance;
pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum WithdrawalReason {
	NotDefined,
}

// Represents the state of the housing fund balance
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct FundInfo<T: Config> {
	// The total balance of the fund : transferable + reserved + frozen
	pub total: BalanceOf<T>,
	// The amount that can be used for fund transfer as withdraw or house bidding
	pub transferable: BalanceOf<T>,
	// The amount used project funding
	pub reserved: BalanceOf<T>,
}
impl<T: Config> FundInfo<T> {
	// Add amount to the tranferable fund
	pub fn contribute_transferable(&mut self, amount: BalanceOf<T>) {
		// add to transferable
		self.transferable += amount;
		// update the total amount
		self.total += amount;
	}

	pub fn can_take_off(&self, amount: BalanceOf<T>) -> bool {
		// check that amount to take off if inferior to the transferable
		self.transferable > T::FundThreshold::get() &&
			amount <= self.transferable - T::FundThreshold::get()
	}

	// Withdraw from the tranferable
	pub fn withdraw_transferable(&mut self, amount: BalanceOf<T>) {
		// remove from transferable
		self.transferable -= amount;
		// update the total amount
		self.total -= amount;
	}

	pub fn reserve(&mut self, amount: BalanceOf<T>) {
		// remove the amount from transferable
		self.transferable -= amount;
		// add the amount to reserved
		self.reserved += amount;
	}

	pub fn unreserve(&mut self, amount: BalanceOf<T>) {
		// remove the amount from reserved
		self.reserved -= amount;
		// add the amount to transferable
		self.transferable += amount;
	}

	pub fn use_reserved(&mut self, amount: BalanceOf<T>) {
		// remove the amount from reserved
		self.reserved -= amount;
		self.total -= amount;
	}
}

// Contains amount and timestamp of an account
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct ContributionLog<T: Config> {
	// Amount contributed
	pub amount: BalanceOf<T>,
	// Block numer as timestamp
	pub block_number: BlockNumberOf<T>,
}

// Contains the contributed amount of an account, ist share and his contributions history
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct Contribution<T: Config> {
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
	pub contributions: Vec<ContributionLog<T>>,
	// History of the contributor's withdraws
	pub withdraws: Vec<ContributionLog<T>>,
}
impl<T: Config> Contribution<T> {
	pub fn get_total_balance(&self) -> BalanceOf<T> {
		self.available_balance + self.reserved_balance
	}

	pub fn can_reserve(&self, amount: BalanceOf<T>) -> bool {
		amount <= self.available_balance
	}

	pub fn reserve_amount(&mut self, amount: BalanceOf<T>) {
		self.available_balance -= amount;
		self.reserved_balance += amount;
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
pub struct FundOperation<T: Config> {
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
	pub share: u32,
}
