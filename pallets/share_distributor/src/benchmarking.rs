//! Benchmarking setup for pallet-template

use super::*;

#[allow(unused)]
use crate::Pallet as Template;
use frame_benchmarking::{benchmarks, whitelisted_caller};
pub use frame_support::{
	dispatch::{DispatchResult, EncodeLike},
	inherent::Vec,
	pallet_prelude::*,
};
use frame_system::RawOrigin;
pub type NftColl = Nft::PossibleCollections;

benchmarks! {
	create_virtual{
		let s in 0 .. 100;
		let caller: T::AccountId = whitelisted_caller();
		let coll_id:T::NftCollectionId = NftColl::APPARTMENTSTEST.value().into();

	}: _(RawOrigin::Root,coll_id,s.into())

	impl_benchmark_test_suite!(Template, crate::mock::new_test_ext(), crate::mock::Test);
}
