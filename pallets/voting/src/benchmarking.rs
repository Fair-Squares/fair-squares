//! Benchmarking setup for pallet-template

use super::*;
use ic_types::{
    crypto::CryptoHash,
    state_sync::{decode_manifest, encode_manifest, ChunkInfo, FileInfo, Manifest},
    CryptoHashOfState,
};

#[allow(unused)]
use crate::Pallet as Voting;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

benchmarks! {
	investor_vote {
		let hash = CryptoHashOfState::from(CryptoHash(vec![1u8; 32]));
		let approve = true;
		let caller: T::AccountId = whitelisted_caller();
	}: _(RawOrigin::Signed(caller), hash, approve)
	verify {
		assert_eq!(Some(2), Some(2));
	}

	impl_benchmark_test_suite!(Voting, crate::mock::new_test_ext(), crate::mock::Test);
}
