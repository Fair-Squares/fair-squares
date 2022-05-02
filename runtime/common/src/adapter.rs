use frame_support::dispatch::DispatchError;
use frame_support::sp_runtime::DispatchResult;
use frame_support::traits::BalanceStatus;
use orml_traits::currency::TransferAll;
use orml_traits::{
	LockIdentifier, MultiCurrency, MultiCurrencyExtended, MultiLockableCurrency, MultiReservableCurrency,
};

pub struct OrmlTokensAdapter<T>(sp_std::marker::PhantomData<T>);

impl<T: orml_tokens::Config + frame_system::Config> MultiCurrency<T::AccountId> for OrmlTokensAdapter<T> {
	type CurrencyId = <T as orml_tokens::Config>::CurrencyId;
	type Balance = <T as orml_tokens::Config>::Balance;

	fn minimum_balance(currency_id: Self::CurrencyId) -> Self::Balance {
		<orml_tokens::Pallet<T> as MultiCurrency<T::AccountId>>::minimum_balance(currency_id)
	}

	fn total_issuance(currency_id: Self::CurrencyId) -> Self::Balance {
		<orml_tokens::Pallet<T> as MultiCurrency<T::AccountId>>::total_issuance(currency_id)
	}

	fn total_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
		<orml_tokens::Pallet<T> as MultiCurrency<T::AccountId>>::total_balance(currency_id, who)
	}

	fn free_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
		<orml_tokens::Pallet<T> as MultiCurrency<T::AccountId>>::free_balance(currency_id, who)
	}

	fn ensure_can_withdraw(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		<orml_tokens::Pallet<T> as MultiCurrency<T::AccountId>>::ensure_can_withdraw(currency_id, who, amount)
	}

	fn transfer(
		currency_id: Self::CurrencyId,
		from: &T::AccountId,
		to: &T::AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		let res = <orml_tokens::Pallet<T> as MultiCurrency<T::AccountId>>::transfer(currency_id, from, to, amount);

		if res.is_ok() {
			<frame_system::Pallet<T>>::deposit_event(
				<T as orml_tokens::Config>::Event::from(orml_tokens::Event::Transfer {
					currency_id,
					from: from.clone(),
					to: to.clone(),
					amount,
				})
				.into(),
			);
		}

		res
	}

	fn deposit(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		<orml_tokens::Pallet<T> as MultiCurrency<T::AccountId>>::deposit(currency_id, who, amount)
	}

	fn withdraw(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		<orml_tokens::Pallet<T> as MultiCurrency<T::AccountId>>::withdraw(currency_id, who, amount)
	}

	fn can_slash(currency_id: Self::CurrencyId, who: &T::AccountId, value: Self::Balance) -> bool {
		<orml_tokens::Pallet<T> as MultiCurrency<T::AccountId>>::can_slash(currency_id, who, value)
	}

	fn slash(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> Self::Balance {
		<orml_tokens::Pallet<T> as MultiCurrency<T::AccountId>>::slash(currency_id, who, amount)
	}
}

impl<T: orml_tokens::Config + frame_system::Config> MultiCurrencyExtended<T::AccountId> for OrmlTokensAdapter<T> {
	type Amount = <T as orml_tokens::Config>::Amount;

	fn update_balance(currency_id: Self::CurrencyId, who: &T::AccountId, by_amount: Self::Amount) -> DispatchResult {
		<orml_tokens::Pallet<T> as MultiCurrencyExtended<T::AccountId>>::update_balance(currency_id, who, by_amount)
	}
}

impl<T: orml_tokens::Config + frame_system::Config> MultiReservableCurrency<T::AccountId> for OrmlTokensAdapter<T> {
	fn can_reserve(currency_id: Self::CurrencyId, who: &T::AccountId, value: Self::Balance) -> bool {
		<orml_tokens::Pallet<T> as MultiReservableCurrency<T::AccountId>>::can_reserve(currency_id, who, value)
	}

	fn slash_reserved(currency_id: Self::CurrencyId, who: &T::AccountId, value: Self::Balance) -> Self::Balance {
		<orml_tokens::Pallet<T> as MultiReservableCurrency<T::AccountId>>::slash_reserved(currency_id, who, value)
	}

	fn reserved_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
		<orml_tokens::Pallet<T> as MultiReservableCurrency<T::AccountId>>::reserved_balance(currency_id, who)
	}

	fn reserve(currency_id: Self::CurrencyId, who: &T::AccountId, value: Self::Balance) -> DispatchResult {
		<orml_tokens::Pallet<T> as MultiReservableCurrency<T::AccountId>>::reserve(currency_id, who, value)
	}

	fn unreserve(currency_id: Self::CurrencyId, who: &T::AccountId, value: Self::Balance) -> Self::Balance {
		<orml_tokens::Pallet<T> as MultiReservableCurrency<T::AccountId>>::unreserve(currency_id, who, value)
	}

	fn repatriate_reserved(
		currency_id: Self::CurrencyId,
		slashed: &T::AccountId,
		beneficiary: &T::AccountId,
		value: Self::Balance,
		status: BalanceStatus,
	) -> Result<Self::Balance, DispatchError> {
		<orml_tokens::Pallet<T> as MultiReservableCurrency<T::AccountId>>::repatriate_reserved(
			currency_id,
			slashed,
			beneficiary,
			value,
			status,
		)
	}
}

impl<T: orml_tokens::Config + frame_system::Config> MultiLockableCurrency<T::AccountId> for OrmlTokensAdapter<T> {
	type Moment = T::BlockNumber;

	fn set_lock(
		lock_id: LockIdentifier,
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		<orml_tokens::Pallet<T> as MultiLockableCurrency<T::AccountId>>::set_lock(lock_id, currency_id, who, amount)
	}

	fn extend_lock(
		lock_id: LockIdentifier,
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		<orml_tokens::Pallet<T> as MultiLockableCurrency<T::AccountId>>::extend_lock(lock_id, currency_id, who, amount)
	}

	fn remove_lock(lock_id: LockIdentifier, currency_id: Self::CurrencyId, who: &T::AccountId) -> DispatchResult {
		<orml_tokens::Pallet<T> as MultiLockableCurrency<T::AccountId>>::remove_lock(lock_id, currency_id, who)
	}
}

impl<T: orml_tokens::Config> TransferAll<T::AccountId> for OrmlTokensAdapter<T> {
	fn transfer_all(source: &T::AccountId, dest: &T::AccountId) -> DispatchResult {
		<orml_tokens::Pallet<T> as TransferAll<T::AccountId>>::transfer_all(source, dest)
	}
}