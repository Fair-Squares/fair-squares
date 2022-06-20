pub use super::*;

pub use frame_support::{
    dispatch::DispatchResult,
    pallet_prelude::*,
    transactional,
    sp_runtime::traits::{AccountIdConversion, Zero},
    codec::{Encode, Decode},
    traits::{Currency, ExistenceRequirement, Get, ReservableCurrency, WithdrawReasons},
    inherent::{Vec},
    PalletId
 };
use scale_info::{ TypeInfo };
use sp_std::vec;
pub use frame_system::{pallet_prelude::*,ensure_signed};

pub type StorageIndex = u32;
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;


#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
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
    // Amount of the fund used to purchase houses
    pub contributed: BalanceOf<T>
}
impl<T: Config> FundInfo<T> {
    // Add amount to the tranferable fund
    pub fn contribute_transferable(&mut self, amount: BalanceOf<T>) {
        // add to transferable
        self.transferable += amount.clone();
        // update the total amount
        self.total += amount.clone();
    }

    pub fn can_take_off(&self, amount: BalanceOf<T>) -> bool {
        // check that amount to take off if inferior to the transferable
        amount.clone() <= self.transferable.clone()
    }

    // Withdraw from the tranferable
    pub fn withdraw_transferable(&mut self, amount: BalanceOf<T>) {
        // remove from transferable
        self.transferable -= amount.clone();
        // update the total amount
        self.total -= amount.clone();
    }

    pub fn reserve(&mut self, amount: BalanceOf<T>) {
        // remove the amount from transferable
        self.transferable -= amount.clone();
        // add the amount to reserved
        self.reserved += amount.clone();
    }
}

// Contains amount and timestamp of an account
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct ContributionLog<T: Config> {
   // Amount contributed
   pub amount: BalanceOf<T>,
   // Block numer as timestamp
   pub block_number: BlockNumberOf<T>
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
    // Share of the housing fund
    pub share: u32,
    // Indicate if the contributor has withdrawn from the housing fund
    pub has_withdrawn: bool,
    // Block number of the last contribution's update
    pub block_number: BlockNumberOf<T>,
    // History of the contributor's contribution
    pub contributions: Vec<ContributionLog<T>>,
    // History of the contributor's withdraws
    pub withdraws: Vec<ContributionLog<T>>
}
impl<T: Config> Contribution<T> {
    pub fn get_total_balance(&self) -> BalanceOf<T> {
        self.available_balance.clone() + self.reserved_balance.clone()
    }

    pub fn can_reserve(&self, amount: BalanceOf<T>) -> bool {
        amount.clone() <= self.available_balance.clone()
    }

    pub fn reserve_amount(&mut self, amount: BalanceOf<T>) {
        self.available_balance -= amount.clone();
        self.reserved_balance += amount.clone();
    }

    pub fn unreserve_amount(&mut self, amount: BalanceOf<T>) {
        self.reserved_balance -= amount.clone();
        self.available_balance += amount.clone();        
    }
}

// Contains the details of the operations that occured
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct FundOperation<T: Config> {
    // Account to which the amount is destinated
    pub account_id: AccountIdOf<T>,
    // The house identifier
    pub house_id: StorageIndex,
    // The amount of the transaction
    pub amount: BalanceOf<T>,
    // Block number of the last contribution's update
    pub block_number: BlockNumberOf<T>,
    // List of (AccountIdOf<T>, BalanceOf<T>) representing the investors and their contribution
    pub contributions: Vec<(AccountIdOf<T>, BalanceOf<T>)>
}