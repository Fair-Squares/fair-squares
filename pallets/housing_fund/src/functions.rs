pub use crate::structs::*;

impl<T: Config> Pallet<T> {
	// Conversion of u64 to BalanxceOf<T>
	pub fn u64_to_balance_option(input: u64) -> Option<BalanceOf<T>> {
		input.try_into().ok()
	}

	// Conversion of BalanceOf<T> to u32
	pub fn balance_to_u32_option(input: BalanceOf<T>) -> Option<u32> {
		input.try_into().ok()
	}

	pub fn fund_account_id() -> T::AccountId {
		T::PalletId::get().into_account_truncating()
	}

	pub fn get_contribution_share() -> Vec<ContributionShare<T>> {
		let mut contribution_shares = Vec::<ContributionShare<T>>::new();
		let amount = FundBalance::<T>::get().total;
		let contributions_iter = Contributions::<T>::iter();
		let factor = Self::u64_to_balance_option(PERCENT_FACTOR.clone());

		for item in contributions_iter {
<<<<<<< HEAD
			let factor = Self::u64_to_balance_option(PERCENT_FACTOR);
			// Calculate the share according to the new total amount of the fund
			let share = factor.unwrap() * (item.1.clone().get_total_balance()) / amount;

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
					withdraws: unwrap_val.withdraws,
				};
				*val = Some(contrib);
=======
			let share = factor.unwrap() * (item.1.clone().get_total_balance()) / amount.clone();
			contribution_shares.push(ContributionShare {
				account_id: item.1.account_id.clone(),
				share: Self::balance_to_u32_option(share).unwrap()
>>>>>>> origin/main
			});
		}

		contribution_shares
	}
}
