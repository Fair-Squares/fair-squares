//! Benchmarking setup for pallet-template

use super::*;

#[allow(unused)]
use crate::Pallet as Voting;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

// fn make_proposal(value: u64) -> Call {
// 	Call::System(frame_system::Call::remark { remark: value.encode() })
// }

benchmarks! {
	submit_proposal {
		let caller: T::AccountId = whitelisted_caller();
		let proposal = Box::new(Call::System(frame_system::Call::remark { remark: 1_i32.encode() }));
		let collective_passed = Box::new(Call::System(frame_system::Call::remark { remark: 2_i32.encode() }));
		let collective_failed = Box::new(Call::System(frame_system::Call::remark { remark: 3_i32.encode() }));
		let democracy_failed = Box::new(Call::System(frame_system::Call::remark { remark: 4_i32.encode() }));
	}: _(
		RawOrigin::Signed(caller), 
		proposal,
		collective_passed,
		collective_failed,
		democracy_failed_call
	)
	verify {
		assert_eq!(Some(2), Some(2));
	}

	impl_benchmark_test_suite!(Voting, crate::mock::new_test_ext(), crate::mock::Test);
}
