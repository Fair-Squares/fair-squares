#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{account, benchmarks, whitelisted_caller};

use frame_system::RawOrigin;

const SEED: u32 = 0;

benchmarks! {
	create_account{
		let caller:T::AccountId = whitelisted_caller();
		let caller1= account("Kazu", 0, SEED);

		//let balance= u32_to_balance_option(3E100).unwrap();
		let balance = T::Currency::minimum_balance().saturating_mul(1_000_000u32.into());
		<T as pallet::Config>::Currency::make_free_balance_be(&caller1,balance);

		let account1 = Accounts::INVESTOR;

	}:create_account(RawOrigin::Signed(caller1.clone()),account1)
	verify{
		assert!(InvestorLog::<T>::contains_key(&caller1));
	}



	impl_benchmark_test_suite!(Roles, crate::tests::new_test_ext(), crate::tests::Test)
}
