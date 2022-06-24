pub use crate::structs::*;

use frame_support::{
    sp_runtime::traits::{Hash},
    storage::child
 };
use frame_support::inherent::Vec;
use scale_info::{ prelude::vec };

impl<T: Config> Pallet<T> {
    
    // Conversion of u64 to BalanxceOf<T>
    pub fn u64_to_balance_option(input: u64) -> Option<BalanceOf<T>> {
        input.try_into().ok()
    }

    // Concersion of BalanceOf<T> to u32
    pub fn balance_to_u32_option(input: BalanceOf<T>) -> Option<u32> {
        input.try_into().ok()
    }

    // Update the shares of the contributions
    pub fn update_contribution_share(amount: BalanceOf<T>) {

        let contributions_iter = Contributions::<T>::iter();

        for item in contributions_iter {
            let factor = Self::u64_to_balance_option(PERCENT_FACTOR.clone());
            // Calculate the share according to the new total amount of the fund
            let share = factor.unwrap() * (item.1.clone().get_total_balance()) / amount.clone();

            Contributions::<T>::mutate(item.0, |val| {
                let unwrap_val = val.clone().unwrap();
                let contrib = Contribution {
                    account_id: unwrap_val.account_id,
                    available_balance: unwrap_val.available_balance,
                    reserved_balance: unwrap_val.reserved_balance,
                    contributed_balance: unwrap_val.contributed_balance,
                    share: Self::balance_to_u32_option(share).unwrap(),
                    has_withdrawn: unwrap_val.has_withdrawn,
                    block_number: unwrap_val.block_number,
                    contributions: unwrap_val.contributions.clone(),
                    withdraws: unwrap_val.withdraws.clone(),
                };
                *val = Some(contrib);
            });
        }
    }
}