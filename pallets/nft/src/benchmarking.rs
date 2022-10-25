#![cfg(feature = "runtime-benchmarks")]

use super::*;

use crate as NFT;
//use crate::Roles::Pallet;
use frame_benchmarking::{account, benchmarks, vec};
use frame_support::traits::{tokens::nonfungibles::InspectEnumerable, Currency, Get};
use frame_system::RawOrigin;
use pallet_sudo as SUDO;
use pallet_uniques as UNQ;
use sp_runtime::traits::UniqueSaturatedInto;
use sp_std::convert::TryInto;

const SEED: u32 = 0;
const ENDOWMENT: u128 = 100_000_000_000_000_000_000;
const COLLECTION_ID_0: u32 = 4;

fn create_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);
	<T as pallet_uniques::Config>::Currency::deposit_creating(
		&caller,
		ENDOWMENT.unique_saturated_into(),
	);
	caller
}

fn do_create_collection<T: Config>(caller: T::AccountId) {
	let metadata: BoundedVec<_, _> =
		vec![0; <T as UNQ::Config>::StringLimit::get() as usize].try_into().unwrap();
	let coll_id = PossibleCollections::HOUSESTEST;
	assert!(NFT::Pallet::<T>::create_collection(
		RawOrigin::Signed(caller).into(),
		coll_id,
		metadata
	)
	.is_ok());
}

fn do_mint<T: Config>(caller: T::AccountId) {
	let metadata: BoundedVec<_, _> =
		vec![0; <T as UNQ::Config>::StringLimit::get() as usize].try_into().unwrap();
	let coll_id = PossibleCollections::HOUSESTEST;
	assert!(NFT::Pallet::<T>::mint(RawOrigin::Signed(caller).into(), coll_id, metadata).is_ok());
}

benchmarks! {
	create_collection {
		let caller = create_account::<T>("caller", 0);
		let caller_signed = <T as frame_system::Config>::Origin::from(RawOrigin::Signed(caller.clone()));
		let _ = Roles::Pallet::<T>::set_role(
			caller_signed.clone(),
			caller.clone(),
			Roles::Accounts::SERVICER
		);
		let key_account:T::AccountId = SUDO::Pallet::<T>::key().unwrap();
		let key_signed = <T as frame_system::Config>::Origin::from(RawOrigin::Signed(key_account.clone()));
		Roles::Pallet::<T>::account_approval(key_signed,caller.clone()).ok();

		let metadata: BoundedVec<_, _> = vec![0; <T as UNQ::Config>::StringLimit::get() as usize].try_into().unwrap();
	}: _(RawOrigin::Signed(caller.clone()), PossibleCollections::HOUSESTEST, metadata)
	verify {
		assert_eq!(UNQ::Pallet::<T>::collection_owner(T::NftCollectionId::from(COLLECTION_ID_0).into()), Some(caller));
	}

	mint {
		let caller = create_account::<T>("caller", 0);
		let caller1 = create_account::<T>("caller1", 0);

		let caller_signed = <T as frame_system::Config>::Origin::from(RawOrigin::Signed(caller.clone()));
		let _ = Roles::Pallet::<T>::set_role(
			caller_signed.clone(),
			caller.clone(),
			Roles::Accounts::SELLER
		);
		let caller1_signed = <T as frame_system::Config>::Origin::from(RawOrigin::Signed(caller1.clone()));
		let _ = Roles::Pallet::<T>::set_role(
			caller1_signed.clone(),
			caller1.clone(),
			Roles::Accounts::SERVICER
		);

		let key_account:T::AccountId = SUDO::Pallet::<T>::key().unwrap();
		let key_signed = <T as frame_system::Config>::Origin::from(RawOrigin::Signed(key_account.clone()));
		Roles::Pallet::<T>::account_approval(key_signed.clone(),caller.clone()).ok();
		Roles::Pallet::<T>::account_approval(key_signed,caller1.clone()).ok();

		do_create_collection::<T>(caller1.clone());
		let metadata: BoundedVec<_, _> = vec![0; <T as UNQ::Config>::StringLimit::get() as usize].try_into().unwrap();
	}: _(RawOrigin::Signed(caller.clone()), PossibleCollections::HOUSESTEST, metadata)
	verify {
		assert_eq!(UNQ::Pallet::<T>::owner(T::NftCollectionId::from(COLLECTION_ID_0).into(), T::NftItemId::from(0u32).into()), Some(caller));
	}

	transfer {
		let caller1 = create_account::<T>("caller", 1);
		let caller1_signed = <T as frame_system::Config>::Origin::from(RawOrigin::Signed(caller1.clone()));
		let _ = Roles::Pallet::<T>::set_role(
			caller1_signed.clone(),
			caller1.clone(),
			Roles::Accounts::SERVICER
		);

		let caller3 = create_account::<T>("caller3", 1);
		let caller3_signed = <T as frame_system::Config>::Origin::from(RawOrigin::Signed(caller3.clone()));
		let _ = Roles::Pallet::<T>::set_role(
			caller3_signed.clone(),
			caller3.clone(),
			Roles::Accounts::SELLER
		);

		let key_account:T::AccountId = SUDO::Pallet::<T>::key().unwrap();
		let key_signed = <T as frame_system::Config>::Origin::from(RawOrigin::Signed(key_account.clone()));
		Roles::Pallet::<T>::account_approval(key_signed.clone(),caller1.clone()).ok();
		Roles::Pallet::<T>::account_approval(key_signed,caller3.clone()).ok();

		do_create_collection::<T>(caller1.clone());
		let caller2 = create_account::<T>("caller2", 1);
		let caller2_lookup = T::Lookup::unlookup(caller2.clone());
		do_mint::<T>(caller3.clone());
	}: _(RawOrigin::Root, PossibleCollections::HOUSESTEST, 0u32.into(), caller2_lookup)
	verify {
		assert_eq!(UNQ::Pallet::<T>::owner(T::NftCollectionId::from(COLLECTION_ID_0).into(), T::NftItemId::from(0u32).into()), Some(caller2));
	}

	destroy_collection {
		let caller = create_account::<T>("caller", 1);
		let caller_signed = <T as frame_system::Config>::Origin::from(RawOrigin::Signed(caller.clone()));
		let _ = Roles::Pallet::<T>::set_role(
			caller_signed.clone(),
			caller.clone(),
			Roles::Accounts::SERVICER
		);
		let key_account:T::AccountId = SUDO::Pallet::<T>::key().unwrap();
		let key_signed = <T as frame_system::Config>::Origin::from(RawOrigin::Signed(key_account.clone()));
		Roles::Pallet::<T>::account_approval(key_signed,caller.clone()).ok();

		do_create_collection::<T>(caller.clone());
	}: _(RawOrigin::Signed(caller), PossibleCollections::HOUSESTEST)
	verify {
		assert_eq!(UNQ::Pallet::<T>::collections().count(), 0);
	}

	burn {
		let caller1 = create_account::<T>("caller", 0);
		let caller1_signed = <T as frame_system::Config>::Origin::from(RawOrigin::Signed(caller1.clone()));
		let _ = Roles::Pallet::<T>::set_role(
			caller1_signed.clone(),
			caller1.clone(),
			Roles::Accounts::SERVICER
		);

		let caller3 = create_account::<T>("caller3", 2);
		let caller3_signed = <T as frame_system::Config>::Origin::from(RawOrigin::Signed(caller3.clone()));
		let _ = Roles::Pallet::<T>::set_role(
			caller3_signed.clone(),
			caller3.clone(),
			Roles::Accounts::SELLER
		);
		let key_account:T::AccountId = SUDO::Pallet::<T>::key().unwrap();
		let key_signed = <T as frame_system::Config>::Origin::from(RawOrigin::Signed(key_account.clone()));
		Roles::Pallet::<T>::account_approval(key_signed.clone(),caller1.clone()).ok();
		Roles::Pallet::<T>::account_approval(key_signed,caller3.clone()).ok();

		do_create_collection::<T>(caller1.clone());
		do_mint::<T>(caller3.clone());
	}: _(RawOrigin::Signed(caller1.clone()), PossibleCollections::HOUSESTEST, 0u32.into())
	verify {
		assert_eq!(UNQ::Pallet::<T>::owned(&caller3).count(), 0);
	}

}

#[cfg(test)]
mod tests {
	use super::Pallet;
	use crate::mock::*;
	use frame_benchmarking::impl_benchmark_test_suite;

	impl_benchmark_test_suite!(Pallet, super::ExtBuilder::default().build(), super::Test);
}
