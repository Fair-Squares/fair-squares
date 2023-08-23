pub use super::*;

impl<T: Config> Pallet<T> {
	

	pub fn fund_account_id() -> T::AccountId {
		T::PalletId::get().into_account_truncating()
	}

	

	/// Check that the fund can afford the amount
	pub fn check_available_fund(value: BalanceOf<T>) -> bool {
		let fund_account = Self::fund_account_id();
		let amount = T::LocalCurrency::free_balance(&fund_account);

		amount>value
	}

	pub fn get_contributions() -> Vec<(AccountIdOf<T>, UserFundStatus<T>)> {
		Contributions::<T>::iter()
			.map(|(account_id, contribution)| (account_id, contribution))
			.collect()
	}
}

	