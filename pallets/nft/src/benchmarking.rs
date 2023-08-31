//! Benchmarking setup for pallet-nft
#![cfg(feature = "runtime-benchmarks")]
use super::*;



#[allow(unused)]
use crate::Pallet as NFT;
use frame_support::{traits::{tokens::nonfungibles::InspectEnumerable, Currency, Get},assert_noop};
use pallet_sudo as SUDO;
use pallet_uniques as UNQ;
use pallet_roles as Roles;
use sp_runtime::traits::UniqueSaturatedInto;
use sp_std::convert::TryInto;

const SEED: u32 = 0;
const ENDOWMENT: u128 = 100_000_000_000_000_000_000;
const COLLECTION_ID_0: u32 = 4;

use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

fn create_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);
	<T as pallet_uniques::Config>::Currency::deposit_creating(
		&caller,
		ENDOWMENT.unique_saturated_into(),
	);
	caller
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn create_collection(){
        let caller = create_account::<T>("caller", 0);
		let caller_signed = <T as frame_system::Config>::RuntimeOrigin::from(RawOrigin::Signed(caller.clone()));
        let _ = Roles::Pallet::<T>::set_role(
			caller_signed.clone(),
			caller.clone(),
			Roles::Accounts::SERVICER
		);

        assert_eq!(Roles::Pallet::<T>::get_requested_role(caller.clone()).is_some(),true);
        let coll_id = PossibleCollections::HOUSESTEST;
        #[extrinsic_call]
        create_collection(
            RawOrigin::Signed(caller),
            coll_id,
            vec![0; <T as UNQ::Config>::StringLimit::get() as usize].try_into().unwrap()
        );
        
    }
    impl_benchmark_test_suite!(NFT, crate::mock::new_test_ext(), crate::mock::Test);
}