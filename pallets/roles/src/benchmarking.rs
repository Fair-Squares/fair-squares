//! Benchmarking setup for pallet-roles
#![cfg(feature = "runtime-benchmarks")]
use super::*;



#[allow(unused)]
use crate::Pallet as Roles;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn do_something() {
		let value = 100u32.into();
		let caller: T::AccountId = whitelisted_caller();
		#[extrinsic_call]
		do_something(RawOrigin::Signed(caller), value);

		assert_eq!(Something::<T>::get(), Some(value));
	}

	
	impl_benchmark_test_suite!(Roles, crate::mock::new_test_ext(), crate::mock::Test);
}

