use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use codec::Encode;
use pallet_roles::Hash;
use pallet_roles::Hooks;
use pallet_democracy::Event;

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
<<<<<<< HEAD
		// Dispatch a signed extrinsic.
		assert_ok!(VotingModule::do_something(Origin::signed(1), 42));
		// Read pallet storage and assert an expected result.
		assert_eq!(VotingModule::something(), Some(42));
	});
}

#[test]
fn correct_error_for_none_value() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when no value is present.
		assert_noop!(VotingModule::cause_error(Origin::signed(1)), Error::<Test>::NoneValue);
=======
		assert_eq!(Some(42), Some(42));
>>>>>>> main
	});
}

fn make_proposal(value: i32) -> Box<Call> {
	Box::new(Call::System(frame_system::Call::remark { remark: value.encode() }))
}

#[test]
fn submit_proposal_not_seller_should_fail() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			VotingModule::submit_proposal(
				Origin::signed(EVE),
				make_proposal(1),
				make_proposal(2),
				make_proposal(3),
				make_proposal(4)
			),
			Error::<Test>::NotASeller
		);
	});
}

#[test]
fn submit_proposal_should_succeed() {
	new_test_ext().execute_with(|| {
		// Give the investor role to the account
		assert_ok!(RoleModule::set_role(
			Origin::signed(EVE),
			crate::ROLES::Accounts::SELLER
		));

		assert_ok!(
			RoleModule::account_approval(Origin::signed(ALICE), EVE)
		);

		let proposal = make_proposal(1);

		assert_ok!(
			VotingModule::submit_proposal(
				Origin::signed(EVE),
				proposal.clone(),
				make_proposal(2),
				make_proposal(3),
				make_proposal(4)
			)
		);

		let hash = <Test as frame_system::Config>::Hashing::hash_of(&proposal);

		assert_eq!(
			VotingModule::voting_proposals(hash.clone()).is_some(),
			true
		);

		let voting_proposal = VotingModule::voting_proposals(hash.clone()).unwrap();

		assert_eq!(voting_proposal.account_id, EVE);
		assert_eq!(voting_proposal.proposal_call, proposal.clone());
		assert_eq!(voting_proposal.collective_passed_call, make_proposal(2));
		assert_eq!(voting_proposal.collective_failed_call, make_proposal(3));
		assert_eq!(voting_proposal.democracy_failed_call, make_proposal(4));
		assert_eq!(voting_proposal.proposal_hash, hash.clone());
		assert_eq!(voting_proposal.collective_index, 0);
		assert_eq!(voting_proposal.democracy_referendum_index, 0);
		assert_eq!(voting_proposal.collective_step, false);
		assert_eq!(voting_proposal.proposal_executed, false);
		assert_eq!(voting_proposal.collective_closed, false);

		let block_number = System::block_number()
			.saturating_add(<Test as crate::Config>::Delay::get())
			.saturating_add(<Test as pallet_collective::Config<pallet_collective::Instance1>>::MotionDuration::get());

		assert_eq!(
			VotingModule::collective_proposals(hash.clone()),
			Some(block_number)
		);

		assert_eq!(
			VotingModule::democracy_proposals(hash.clone()).is_none(),
			true
		);
	});
}

#[test]
fn council_vote_not_house_council_member_should_fail() {
	new_test_ext().execute_with(|| {

		// Give the investor role to the account
		assert_ok!(RoleModule::set_role(
			Origin::signed(EVE),
			crate::ROLES::Accounts::SELLER
		));

		assert_ok!(
			RoleModule::account_approval(Origin::signed(ALICE), EVE)
		);

		let proposal = make_proposal(1);

		assert_ok!(
			VotingModule::submit_proposal(
				Origin::signed(EVE),
				proposal.clone(),
				make_proposal(2),
				make_proposal(3),
				make_proposal(4)
			)
		);

		let hash = <Test as frame_system::Config>::Hashing::hash_of(&proposal);

		assert_noop!(
			VotingModule::council_vote(
				Origin::signed(EVE),
				hash.clone(),
				true,
			),
			Error::<Test>::NotAHouseCouncilMember
		);
	});
}

#[test]
fn council_vote_proposal_not_exist_should_fail() {
	new_test_ext().execute_with(|| {

		// Give the investor role to the account
		assert_ok!(RoleModule::set_role(
			Origin::signed(EVE),
			crate::ROLES::Accounts::SELLER
		));

		assert_ok!(
			RoleModule::account_approval(Origin::signed(ALICE), EVE)
		);

		let proposal = make_proposal(1);
		let hash = <Test as frame_system::Config>::Hashing::hash_of(&proposal);

		assert_noop!(
			VotingModule::council_vote(
				Origin::signed(ALICE),
				hash.clone(),
				true,
			),
			Error::<Test>::ProposalDoesNotExist
		);
	});
}

#[test]
fn council_vote_proposal_should_succeed() {
	new_test_ext().execute_with(|| {

		// Give the investor role to the account
		assert_ok!(RoleModule::set_role(
			Origin::signed(EVE),
			crate::ROLES::Accounts::SELLER
		));

		assert_ok!(
			RoleModule::account_approval(Origin::signed(ALICE), EVE)
		);
		
		let proposal = make_proposal(1);

		assert_ok!(
			VotingModule::submit_proposal(
				Origin::signed(EVE),
				proposal.clone(),
				make_proposal(2),
				make_proposal(3),
				make_proposal(4)
			)
		);

		let hash = <Test as frame_system::Config>::Hashing::hash_of(&proposal);

		assert_ok!(
			VotingModule::council_vote(
				Origin::signed(ALICE),
				hash.clone(),
				true,
			)
		);

		let event = <frame_system::Pallet<Test>>::events()
			.pop()
			.expect("Expected at least one EventRecord to be found")
			.event;

		// check that the event has been raised
		assert_eq!(
			event,
			crate::mock::Event::VotingModule(crate::Event::HouseCouncilVoted(ALICE, hash, 1)),
		);
	});
}

#[test]
fn council_close_vote_not_house_council_member_should_fail() {
	new_test_ext().execute_with(|| {

		// Give the investor role to the account
		assert_ok!(RoleModule::set_role(
			Origin::signed(EVE),
			crate::ROLES::Accounts::SELLER
		));

		assert_ok!(
			RoleModule::account_approval(Origin::signed(ALICE), EVE)
		);

		let proposal = make_proposal(1);
		let hash = <Test as frame_system::Config>::Hashing::hash_of(&proposal);

		assert_noop!(
			VotingModule::council_close_vote(
				Origin::signed(EVE),
				hash.clone(),
			),
			Error::<Test>::NotAHouseCouncilMember
		);
	});
}

#[test]
fn council_close_vote_proposal_not_exist_should_fail() {
	new_test_ext().execute_with(|| {

		// Give the investor role to the account
		assert_ok!(RoleModule::set_role(
			Origin::signed(EVE),
			crate::ROLES::Accounts::SELLER
		));

		assert_ok!(
			RoleModule::account_approval(Origin::signed(ALICE), EVE)
		);

		let proposal = make_proposal(1);
		let hash = <Test as frame_system::Config>::Hashing::hash_of(&proposal);

		assert_noop!(
			VotingModule::council_close_vote(
				Origin::signed(ALICE),
				hash.clone(),
			),
			Error::<Test>::ProposalDoesNotExist
		);
	});
}

#[test]
fn council_close_vote_proposal_not_pass_should_succeed() {
	new_test_ext().execute_with(|| {

		System::on_initialize(System::block_number());

		// Give the investor role to the account
		assert_ok!(RoleModule::set_role(
			Origin::signed(EVE),
			crate::ROLES::Accounts::SELLER
		));

		assert_ok!(
			RoleModule::account_approval(Origin::signed(ALICE), EVE)
		);
		
		let proposal = make_proposal(1);

		assert_ok!(
			VotingModule::submit_proposal(
				Origin::signed(EVE),
				proposal.clone(),
				make_proposal(2),
				make_proposal(3),
				make_proposal(4)
			)
		);

		let initial_block_number = System::block_number();

		let end_block_number = initial_block_number
			.saturating_add(<Test as crate::Config>::Delay::get())
			.saturating_add(<Test as pallet_collective::Config<pallet_collective::Instance1>>::MotionDuration::get());

		// We advance the time to reach the block number of the ending proposal vote period
		System::set_block_number(end_block_number.clone());

		let hash = <Test as frame_system::Config>::Hashing::hash_of(&proposal);

		assert_ok!(
			VotingModule::council_close_vote(
				Origin::signed(ALICE),
				hash.clone(),
			)
		);

		let voting_proposal = VotingModule::voting_proposals(hash.clone()).unwrap();

		assert_eq!(voting_proposal.collective_closed, true);
		assert_eq!(voting_proposal.collective_step, false);

		// Simulate the regular block check to have the update storage computation
		VotingModule::begin_block(end_block_number.clone() + 1);

		assert_eq!(
			VotingModule::collective_proposals(hash.clone()).is_none(),
			true
		);

		let event = <frame_system::Pallet<Test>>::events()
			.pop()
			.expect("Expected at least one EventRecord to be found")
			.event;

		// check that the event has been raised
		assert_eq!(
			event,
			crate::mock::Event::VotingModule(crate::Event::HouseCouncilClosedProposal(ALICE, hash, end_block_number)),
		);
	});
}

#[test]
fn council_close_vote_proposal_pass_should_succeed() {
	new_test_ext().execute_with(|| {

		System::on_initialize(System::block_number());

		// Give the investor role to the account
		assert_ok!(RoleModule::set_role(
			Origin::signed(EVE),
			crate::ROLES::Accounts::SELLER
		));

		assert_ok!(
			RoleModule::account_approval(Origin::signed(ALICE), EVE)
		);
		
		let proposal = make_proposal(1);

		assert_ok!(
			VotingModule::submit_proposal(
				Origin::signed(EVE),
				proposal.clone(),
				make_proposal(2),
				make_proposal(3),
				make_proposal(4)
			)
		);

		let hash = <Test as frame_system::Config>::Hashing::hash_of(&proposal);

		assert_ok!(
			VotingModule::council_vote(
				Origin::signed(ALICE),
				hash.clone(),
				true,
			)
		);

		assert_ok!(
			VotingModule::council_vote(
				Origin::signed(BOB),
				hash.clone(),
				true,
			)
		);

		assert_ok!(
			VotingModule::council_vote(
				Origin::signed(CHARLIE),
				hash.clone(),
				true,
			)
		);

		let initial_block_number = System::block_number();

		let end_block_number = initial_block_number
			.saturating_add(<Test as crate::Config>::Delay::get())
			.saturating_add(<Test as pallet_collective::Config<pallet_collective::Instance1>>::MotionDuration::get());

		// We advance the time to reach the block number of the ending proposal vote period
		System::set_block_number(end_block_number.clone());

		assert_ok!(
			VotingModule::council_close_vote(
				Origin::signed(ALICE),
				hash.clone(),
			)
		);

		let voting_proposal = VotingModule::voting_proposals(hash.clone()).unwrap();

		assert_eq!(voting_proposal.collective_closed, true);
		assert_eq!(voting_proposal.collective_step, true);

		// Simulate the regular block check to have the update storage computation
		VotingModule::begin_block(end_block_number.clone() + 1);

		assert_eq!(
			VotingModule::collective_proposals(hash.clone()).is_none(),
			true
		);

		let end_democracy_vote = end_block_number
			.saturating_add(<Test as crate::Config>::Delay::get())
			.saturating_add(<Test as pallet_democracy::Config>::VotingPeriod::get());

		assert_eq!(
			VotingModule::democracy_proposals(hash.clone()),
			Some(end_democracy_vote)
		);

		let mut events = <frame_system::Pallet<Test>>::events();
		let mut event = events
			.pop()
			.expect("Expected at least one EventRecord to be found")
			.event;

		// check that the event has been raised
		assert_eq!(
			event,
			crate::mock::Event::VotingModule(crate::Event::HouseCouncilClosedProposal(ALICE, hash, end_block_number)),
		);
	});
}

#[test]
fn investor_vote_without_having_invetsor_role_should_fail() {
	new_test_ext().execute_with(|| {

		// Give the investor role to the account
		assert_ok!(RoleModule::set_role(
			Origin::signed(EVE),
			crate::ROLES::Accounts::SELLER
		));

		assert_ok!(
			RoleModule::account_approval(Origin::signed(ALICE), EVE)
		);

		let proposal = make_proposal(1);

		assert_ok!(
			VotingModule::submit_proposal(
				Origin::signed(EVE),
				proposal.clone(),
				make_proposal(2),
				make_proposal(3),
				make_proposal(4)
			)
		);

		let hash = <Test as frame_system::Config>::Hashing::hash_of(&proposal);

		assert_noop!(
			VotingModule::investor_vote(
				Origin::signed(ALICE),
				hash.clone(),
				true,
			),
			Error::<Test>::NotAnInvestor
		);
	});
}

#[test]
fn investor_vote_with_bad_proposal_should_fail() {
	new_test_ext().execute_with(|| {

		// Give the investor role to the account
		assert_ok!(RoleModule::set_role(
			Origin::signed(ALICE),
			crate::ROLES::Accounts::INVESTOR
		));
		assert_ok!(RoleModule::set_role(
			Origin::signed(EVE),
			crate::ROLES::Accounts::SELLER
		));

		assert_ok!(
			RoleModule::account_approval(Origin::signed(ALICE), EVE)
		);

		let proposal = make_proposal(1);
		let hash = <Test as frame_system::Config>::Hashing::hash_of(&proposal);

		assert_noop!(
			VotingModule::investor_vote(
				Origin::signed(ALICE),
				hash.clone(),
				true,
			),
			Error::<Test>::ProposalDoesNotExist
		);
	});
}

#[test]
fn investor_vote_should_succeed() {
	new_test_ext().execute_with(|| {

		// Give the investor role to the account
		assert_ok!(RoleModule::set_role(
			Origin::signed(ALICE),
			crate::ROLES::Accounts::INVESTOR
		));
		assert_ok!(RoleModule::set_role(
			Origin::signed(EVE),
			crate::ROLES::Accounts::SELLER
		));

		assert_ok!(
			RoleModule::account_approval(Origin::signed(ALICE), EVE)
		);

		let proposal = make_proposal(1);
		let hash = <Test as frame_system::Config>::Hashing::hash_of(&proposal);

		assert_ok!(
			VotingModule::submit_proposal(
				Origin::signed(EVE),
				proposal.clone(),
				make_proposal(2),
				make_proposal(3),
				make_proposal(4)
			)
		);

		assert_ok!(
			VotingModule::council_vote(
				Origin::signed(ALICE),
				hash.clone(),
				true,
			)
		);

		assert_ok!(
			VotingModule::council_vote(
				Origin::signed(BOB),
				hash.clone(),
				true,
			)
		);

		assert_ok!(
			VotingModule::council_vote(
				Origin::signed(CHARLIE),
				hash.clone(),
				true,
			)
		);

		let initial_block_number = System::block_number();

		let end_block_number = initial_block_number
			.saturating_add(<Test as crate::Config>::Delay::get())
			.saturating_add(<Test as pallet_collective::Config<pallet_collective::Instance1>>::MotionDuration::get());

		// We advance the time to reach the block number of the ending proposal vote period
		System::set_block_number(end_block_number.clone());

		assert_ok!(
			VotingModule::council_close_vote(
				Origin::signed(ALICE),
				hash.clone(),
			)
		);

		// Simulate the regular block check to have the update storage computation
		VotingModule::begin_block(end_block_number.clone() + 1);
		System::set_block_number(end_block_number.clone() + 1);

		assert_ok!(
			VotingModule::investor_vote(
				Origin::signed(ALICE),
				hash.clone(),
				true,
			)
		);

		let mut event = <frame_system::Pallet<Test>>::events()
			.pop()
			.expect("Expected at least one EventRecord to be found")
			.event;

		// check that the event has been raised
		assert_eq!(
			event,
			crate::mock::Event::VotingModule(crate::Event::InvestorVoted(ALICE, hash, System::block_number())),
		);
	});
}

#[test]
fn investor_vote_proposal_fail_should_succeed() {
	new_test_ext().execute_with(|| {

		// Give the investor role to the account
		assert_ok!(RoleModule::set_role(
			Origin::signed(ALICE),
			crate::ROLES::Accounts::INVESTOR
		));
		assert_ok!(RoleModule::set_role(
			Origin::signed(EVE),
			crate::ROLES::Accounts::SELLER
		));

		assert_ok!(
			RoleModule::account_approval(Origin::signed(ALICE), EVE)
		);

		let proposal = make_proposal(1);
		let hash = <Test as frame_system::Config>::Hashing::hash_of(&proposal);

		assert_ok!(
			VotingModule::submit_proposal(
				Origin::signed(EVE),
				proposal.clone(),
				make_proposal(2),
				make_proposal(3),
				make_proposal(4)
			)
		);

		assert_ok!(
			VotingModule::council_vote(
				Origin::signed(ALICE),
				hash.clone(),
				true,
			)
		);

		assert_ok!(
			VotingModule::council_vote(
				Origin::signed(BOB),
				hash.clone(),
				true,
			)
		);

		assert_ok!(
			VotingModule::council_vote(
				Origin::signed(CHARLIE),
				hash.clone(),
				true,
			)
		);

		let initial_block_number = System::block_number();

		let end_block_number = initial_block_number
			.saturating_add(<Test as crate::Config>::Delay::get())
			.saturating_add(<Test as pallet_collective::Config<pallet_collective::Instance1>>::MotionDuration::get());

		// We advance the time to reach the block number of the ending proposal vote period
		System::set_block_number(end_block_number.clone());

		assert_ok!(
			VotingModule::council_close_vote(
				Origin::signed(ALICE),
				hash.clone(),
			)
		);

		// Simulate the regular block check to have the update storage computation
		VotingModule::begin_block(end_block_number.clone() + 1);
		Democracy::on_initialize(end_block_number.clone() + 1);
		System::set_block_number(end_block_number.clone() + 1);

		assert_ok!(
			VotingModule::investor_vote(
				Origin::signed(ALICE),
				hash.clone(),
				false,
			)
		);

		let end_democracy_vote = end_block_number
			.saturating_add(<Test as crate::Config>::Delay::get())
			.saturating_add(<Test as pallet_democracy::Config>::VotingPeriod::get());

		Democracy::on_initialize(end_democracy_vote.clone() + 1);
		VotingModule::begin_block(end_democracy_vote.clone() + 2);

		assert_eq!(
			VotingModule::democracy_proposals(hash.clone()).is_none(),
			true
		);

		let mut event = <frame_system::Pallet<Test>>::events()
			.pop()
			.expect("Expected at least one EventRecord to be found")
			.event;

		// check that the event has been raised
		assert_eq!(
			event,
			crate::mock::Event::Democracy(Event::NotPassed{ref_index: 1}),
		);
	});
}
