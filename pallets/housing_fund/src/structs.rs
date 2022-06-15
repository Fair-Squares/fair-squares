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
    // Total balance contributed
    pub total_balance: BalanceOf<T>,
    // Share of the housing fund
    pub share: u32,
    // Indicate if the contributor has withdrawn from the housing fund
    pub has_withdrawn: bool,
    // Block number of the last contribution's update
    pub block_number: BlockNumberOf<T>,
    // History of the contributor's contribution
    pub contributions: Vec<ContributionLog<T>>
}