#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{account, benchmarks};

use frame_system::RawOrigin;

const SEED: u32 = 0;

benchmarks! {
	investor{
		let b in 0 .. 99;
		let mut acc = Vec::<T::AccountId>::new();

		for i in 0 .. 100{
			let caller:T::AccountId= account("Kazu", i, SEED);
			acc.push(caller.clone());
			let balance = T::Currency::minimum_balance().saturating_mul(1_000_000u32.into());
			<T as pallet::Config>::Currency::make_free_balance_be(&caller,balance);
			}
			let account1 = Accounts::INVESTOR;
			let user = acc[b as usize].clone();

	}:create_account(RawOrigin::Signed(user),account1.clone())
	verify{
		assert!(InvestorLog::<T>::contains(account1),Error::<T>::NoneValue);
	}
	
	#[extra]
	tenant{
		let b in 0 .. 99;
		let mut acc = Vec::<T::AccountId>::new();

		for i in 0 .. 100{
			let caller:T::AccountId= account("Kazu", i, SEED);
			acc.push(caller.clone());
			let balance = T::Currency::minimum_balance().saturating_mul(1_000_000u32.into());
			<T as pallet::Config>::Currency::make_free_balance_be(&caller,balance);
			}
			let  account1 = Accounts::TENANT;

			let user = acc[b as usize].clone();

	}:create_account(RawOrigin::Signed(user),account1)
	
	#[extra]
	seller{
		let b in 0 .. 99;
		let mut acc = Vec::<T::AccountId>::new();

		for i in 0 .. 100{
			let caller:T::AccountId= account("Kazu", i, SEED);
			acc.push(caller.clone());
			let balance = T::Currency::minimum_balance().saturating_mul(1_000_000u32.into());
			<T as pallet::Config>::Currency::make_free_balance_be(&caller,balance);
			}
			let account1 = Accounts::SELLER;

			let user = acc[b as usize].clone();

	}:create_account(RawOrigin::Signed(user),account1)





	impl_benchmark_test_suite!(Roles, crate::tests::new_test_ext(), crate::tests::Test)
}
