//! Benchmarking setup for pallet-template

// use super::*;

// #[allow(unused)]
// use crate::Pallet as Finalizer;
// use crate::HousingFund::NftItemId;
// use frame_benchmarking::{benchmarks, whitelisted_caller};
// use frame_system::RawOrigin;

// benchmarks! {
// 	validate_transaction_asset {
// 		let s in 0 .. 100;
// 		let caller: T::AccountId = whitelisted_caller();
// 		let collection = pallet_nft::PossibleCollections::OFFICESTEST.value();
// 		let item_id: u32 = 1;
// 	}: _(RawOrigin::Signed(caller), collection.into(), item_id.into())
// 	verify {
// 		assert_eq!(2, 2);
// 	}

// 	impl_benchmark_test_suite!(Finalizer, crate::mock::new_test_ext(), crate::mock::Test);
// }
