
#![cfg(feature = "runtime-benchmarks")]


use super::*;

#[allow(unused)]
use crate::Pallet as Voting;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
use frame_system::{Call as SystemCall};
use pallet_roles::Hash;


benchmarks! {
	do_something {
		let s in 0 .. 100;
		let caller: T::AccountId = whitelisted_caller();
	}: _(RawOrigin::Signed(caller), s)
	verify {
		assert_eq!(Something::<T>::get(), Some(s));
	}

	impl_benchmark_test_suite!(Voting, crate::mock::new_test_ext(), crate::mock::Test);
}




