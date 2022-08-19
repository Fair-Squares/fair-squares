#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{account, benchmarks};
use frame_system::RawOrigin;

const SEED: u32 = 0;

benchmarks! {
	investor{
		let b in 0 .. T::MaxMembers::get();
		let mut acc = Vec::<T::AccountId>::new();

		for i in 0 .. T::MaxMembers::get()+1{
			let caller:T::AccountId= account("Kazu", i, SEED);
			acc.push(caller.clone());
			let balance = T::Currency::minimum_balance().saturating_mul(1_000_000u32.into());
			<T as pallet::Config>::Currency::make_free_balance_be(&caller,balance);
			}
		let account1 = Accounts::INVESTOR;
		let user = acc[b as usize].clone();

	}:set_role(RawOrigin::Signed(user.clone()),user.clone(),account1.clone())
	verify{
		assert!(InvestorLog::<T>::contains_key(user),"Investor account missing");
	}

	#[extra]
	tenant{
		let b in 0 .. T::MaxMembers::get();
		let mut acc = Vec::<T::AccountId>::new();

		for i in 0 .. T::MaxMembers::get()+1{
			let caller:T::AccountId= account("Kazu", i, SEED);
			acc.push(caller.clone());
			let balance = T::Currency::minimum_balance().saturating_mul(1_000_000u32.into());
			<T as pallet::Config>::Currency::make_free_balance_be(&caller,balance);
			}
		let  account1 = Accounts::TENANT;
		let user = acc[b as usize].clone();

	}:set_role(RawOrigin::Signed(user.clone()),user.clone(),account1)
	verify{
		assert!(TenantLog::<T>::contains_key(user),"Tenant account missing");
	}

	#[extra]
	seller{
		let b in 0 .. T::MaxMembers::get();
		let mut acc = Vec::<T::AccountId>::new();

		for i in 0 .. T::MaxMembers::get()+1{
			let caller:T::AccountId= account("Kazu", i, SEED);
			acc.push(caller.clone());
			let balance = T::Currency::minimum_balance().saturating_mul(1_000_000u32.into());
			<T as pallet::Config>::Currency::make_free_balance_be(&caller,balance);
			}
		let account1 = Accounts::SELLER;
		let user = acc[b as usize].clone();

	}:set_role(RawOrigin::Signed(user.clone()),user.clone(),account1)

	#[extra]
	servicers{
		let b in 0 .. T::MaxMembers::get();
		let mut acc = Vec::<T::AccountId>::new();

		for i in 0 .. T::MaxMembers::get()+1{
			let caller:T::AccountId= account("Kazu", i, SEED);
			acc.push(caller.clone());
			let balance = T::Currency::minimum_balance().saturating_mul(1_000_000u32.into());
			<T as pallet::Config>::Currency::make_free_balance_be(&caller,balance);
			}
		let account1 = Accounts::SERVICER;
		let user = acc[b as usize].clone();

	}:set_role(RawOrigin::Signed(user.clone()),user.clone(),account1)


	approval{
		let b in 0 .. T::MaxMembers::get();
		let mut acc = Vec::<T::AccountId>::new();
		let key_account:T::AccountId = SUDO::Pallet::<T>::key().unwrap();

		for i in 0 .. T::MaxMembers::get()+1{
			let caller:T::AccountId= account("Kazu", i, SEED);
			acc.push(caller.clone());
			let balance = T::Currency::minimum_balance().saturating_mul(1_000_000u32.into());
			<T as pallet::Config>::Currency::make_free_balance_be(&caller,balance);
			}
		let account1 = Accounts::SELLER;
		let user = acc[b as usize].clone();
		Pallet::<T>::set_role(RawOrigin::Signed(user.clone()).into(),user.clone(),account1.clone()).ok();


	}:account_approval(RawOrigin::Signed(key_account.clone()),user.clone())
	verify{
		ensure!(HouseSellerLog::<T>::contains_key(&user)== true, "Seller not added");
	}

	rejection{
		let b in 0 .. T::MaxMembers::get();
		let mut acc = Vec::<T::AccountId>::new();
		let key_account:T::AccountId = SUDO::Pallet::<T>::key().unwrap();

		for i in 0 .. T::MaxMembers::get()+1{
			let caller:T::AccountId= account("Kazu", i, SEED);
			acc.push(caller.clone());
			let balance = T::Currency::minimum_balance().saturating_mul(1_000_000u32.into());
			<T as pallet::Config>::Currency::make_free_balance_be(&caller,balance);
			}
		let account1 = Accounts::SELLER;
		let user = acc[b as usize].clone();
		Pallet::<T>::set_role(RawOrigin::Signed(user.clone()).into(),user.clone(),account1.clone()).ok();


	}:account_rejection(RawOrigin::Signed(key_account.clone()),user.clone())
	verify{
		ensure!(Pallet::<T>::get_pending_approvals().0.len() == 0, "Seller not removed");
	}

	set_admin{
		let b in 0 .. T::MaxMembers::get();
		let mut acc = Vec::<T::AccountId>::new();
		let key_account:T::AccountId = SUDO::Pallet::<T>::key().unwrap();

		for i in 0 .. T::MaxMembers::get()+1{
			let caller:T::AccountId= account("Kazu", i, SEED);
			acc.push(caller.clone());
			let balance = T::Currency::minimum_balance().saturating_mul(1_000_000u32.into());
			<T as pallet::Config>::Currency::make_free_balance_be(&caller,balance);
			}

		let account1 = Accounts::SELLER;
		let user = acc[b as usize].clone();
		let user_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(user.clone());
	}:set_manager(RawOrigin::Signed(key_account.clone()),user_lookup.clone())
	verify{
		ensure!(key_account != SUDO::Pallet::<T>::key().unwrap(), "Admin is unchanged." )
	}




	impl_benchmark_test_suite!(Roles, crate::tests::new_test_ext(), crate::tests::Test)
}
