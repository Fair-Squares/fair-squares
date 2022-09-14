#![cfg(feature = "runtime-benchmarks")]

use super::*;

#[allow(unused)]
use crate::Pallet as HousingFund;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
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

	impl_benchmark_test_suite!(HousingFund, crate::mock::new_test_ext(), crate::mock::Test);
}
