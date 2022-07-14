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
		let factor = Self::u64_to_balance_option(PERCENT_FACTOR);

		for item in contributions_iter {
			let share = factor.unwrap() * (item.1.clone().get_total_balance()) / amount;
			contribution_shares.push(ContributionShare {
				account_id: item.1.account_id.clone(),
				share: Self::balance_to_u32_option(share).unwrap()
			});
		}

		contribution_shares
	}
}
