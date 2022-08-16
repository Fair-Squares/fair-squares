#![cfg(feature = "runtime-benchmarks")]

use super::*;

#[allow(unused)]
use crate::Pallet as Voting;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
use frame_system::{Call as SystemCall};

use pallet_roles::Hash;


// benchmarks! {

	

// 	impl_benchmark_test_suite!(Voting, crate::mock::new_test_ext(), crate::mock::Test);
// }
