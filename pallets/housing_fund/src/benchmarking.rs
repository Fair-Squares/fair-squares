//! Benchmarking setup for pallet-template

use super::*;

#[allow(unused)]
use crate::Pallet as Template;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

benchmarks! {
	withdraw_fund {
		let caller: T::AccountId = whitelisted_caller();
		let value: u32 = 1000;
		let amount: BalanceOf<T> = value.into();
	}: _(RawOrigin::Signed(caller), amount)
	verify {
		assert_eq!(true, true);
	}

	impl_benchmark_test_suite!(Template, crate::mock::new_test_ext(), crate::mock::Test);
}