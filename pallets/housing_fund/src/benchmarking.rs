#![cfg(feature = "runtime-benchmarks")]

use super::*;

#[allow(unused)]
use crate::Pallet as HousingFund;
use frame_benchmarking::{benchmarks, account, whitelisted_caller};
use frame_system::RawOrigin;

benchmarks! {
	contribute_to_fund {
		let caller: T::AccountId = whitelisted_caller();
		let caller_signed = <T as frame_system::Config>::Origin::from(RawOrigin::Signed(caller.clone()));
		let _ = crate::ROLES::Pallet::<T>::set_role(
			caller_signed.clone(),
			caller.clone(),
			crate::ROLES::Accounts::INVESTOR
		);
		<T as pallet::Config>::LocalCurrency::make_free_balance_be(&caller,10_000_000u32.into());
		
	}: _(RawOrigin::Signed(caller.clone()), 500u32.into())

	withdraw_fund {
		let caller: T::AccountId = whitelisted_caller();
		let caller_signed = <T as frame_system::Config>::Origin::from(RawOrigin::Signed(caller.clone()));
		let _ = crate::ROLES::Pallet::<T>::set_role(
			caller_signed.clone(),
			caller.clone(),
			crate::ROLES::Accounts::INVESTOR
		);
		<T as pallet::Config>::LocalCurrency::make_free_balance_be(&caller,10_000_000u32.into());

		let res = HousingFund::<T>::contribute_to_fund(caller_signed.clone(), 500u32.into());

	}: _(RawOrigin::Signed(caller), 200u32.into())

	house_bidding {
		let caller: T::AccountId = whitelisted_caller();
		let caller_signed = <T as frame_system::Config>::Origin::from(RawOrigin::Signed(caller.clone()));
		let house_owner_id = account("house_owner_id", 1, 1000);
		let mut contributions = Vec::new();
		let house_id = 1;

		for i in 0 .. T::MaxInvestorPerHouse::get() {
			let account_id = account("account_id", i, 1000);
			<T as pallet::Config>::LocalCurrency::make_free_balance_be(&account_id,10_000_000u32.into());
			contributions.push((
				account_id.clone(), 
				HousingFund::<T>::u64_to_balance_option(100).unwrap())
			);
			let account_signed = <T as frame_system::Config>::Origin::from(RawOrigin::Signed(account_id.clone()));
			let _ = crate::ROLES::Pallet::<T>::set_role(
				account_signed.clone(),
				account_id.clone(),
				crate::ROLES::Accounts::INVESTOR
			);
			let res = HousingFund::<T>::contribute_to_fund(account_signed.clone(), 500u32.into());
		}

	}: _(RawOrigin::Signed(caller), house_owner_id, house_id, 1000u32.into(), contributions)

	impl_benchmark_test_suite!(HousingFund, crate::mock::new_test_ext(), crate::mock::Test);
}
