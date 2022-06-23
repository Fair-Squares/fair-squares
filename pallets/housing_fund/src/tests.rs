use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};


#[test]
fn contribute_with_less_that_minimun_amount_should_fail(){
	new_test_ext().execute_with(|| {
		let account_id: u64 = 1;
		// test contribute with unsufficient contribution: MinContribution is 10
		assert_noop!(
			HousingFundModule::contribute_to_fund(Origin::signed(account_id), 5), 
			Error::<Test>::ContributionTooSmall
		);
	});
}

#[test]
fn contribute_with_with_not_enough_free_balance_should_fail(){
	new_test_ext().execute_with(|| {
		let account_id: u64 = 1;
		// test contribute with unsufficient free balance: balancce is 100
		assert_noop!(
			HousingFundModule::contribute_to_fund(Origin::signed(account_id), 110), 
			Error::<Test>::NotEnoughToContribute
		);
	});
}

#[test]
fn contribution_with_valid_values_should_succeed() {
	new_test_ext().execute_with(|| {
		let account_id:u64 = 1;
		// test contribute with sufficient contribution and free balance
		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(account_id), 25));

		// the fund should have been incremented
		assert_eq!(
			HousingFundModule::fund_balance(), 
			FundInfo{ 
				total: HousingFundModule::u64_to_balance_option(25).unwrap(),
				transferable: HousingFundModule::u64_to_balance_option(25).unwrap(),
				reserved: HousingFundModule::u64_to_balance_option(0).unwrap(),
				contributed: HousingFundModule::u64_to_balance_option(0).unwrap(),
			}
		);

		// a contribution must have been registered for the account
		assert_eq!(
			HousingFundModule::contributions(account_id),
			Some(Contribution {
				account_id: 1,
				available_balance: HousingFundModule::u64_to_balance_option(25).unwrap(),
				reserved_balance: HousingFundModule::u64_to_balance_option(0).unwrap(),
				contributed_balance: HousingFundModule::u64_to_balance_option(0).unwrap(),
				share: 100000,
				has_withdrawn: false,
				block_number: 1,
				contributions: vec![
					ContributionLog { 
						amount: HousingFundModule::u64_to_balance_option(25).unwrap(),
						block_number: 1
					}],
				withdraws: Vec::new()
			})
		);

		// check the account balance
		assert_eq!(
			Balances::free_balance(account_id),
			HousingFundModule::u64_to_balance_option(75).unwrap()
		);
		
		let event = <frame_system::Pallet<Test>>::events().pop()
            .expect("Expected at least one EventRecord to be found").event;
		
		// check that the event has been raised
		assert_eq!(
			event, 
			mock::Event::HousingFundModule(crate::Event::ContributeSucceeded(1, 25, 1))
		);
	});
}

#[test]
fn contribution_with_valid_values_from_two_contributors_should_succeed() {
	new_test_ext().execute_with(|| {
		let first_account_id:u64 = 1;
		let second_account_id:u64 = 2;
		// test contribute with sufficient contribution: MinContribution is 10
		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(first_account_id), 25));
		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(second_account_id), 25));

		assert_eq!(
			HousingFundModule::fund_balance(), 
			FundInfo{ 
				total: HousingFundModule::u64_to_balance_option(50).unwrap(),
				transferable: HousingFundModule::u64_to_balance_option(50).unwrap(),
				reserved: HousingFundModule::u64_to_balance_option(0).unwrap(),
				contributed: HousingFundModule::u64_to_balance_option(0).unwrap(),
			}
		);

		assert_eq!(
			HousingFundModule::contributions(first_account_id),
			Some(Contribution {
				account_id: 1,
				available_balance: HousingFundModule::u64_to_balance_option(25).unwrap(),
				reserved_balance: HousingFundModule::u64_to_balance_option(0).unwrap(),
				contributed_balance: HousingFundModule::u64_to_balance_option(0).unwrap(),
				share: 50000,
				has_withdrawn: false,
				block_number: 1,
				contributions: vec![
					ContributionLog { 
						amount: HousingFundModule::u64_to_balance_option(25).unwrap(),
						block_number: 1
					}],
				withdraws: Vec::new()
			})
		);

		assert_eq!(
			HousingFundModule::contributions(second_account_id),
			Some(Contribution {
				account_id: 2,
				available_balance: HousingFundModule::u64_to_balance_option(25).unwrap(),
				reserved_balance: HousingFundModule::u64_to_balance_option(0).unwrap(),
				contributed_balance: HousingFundModule::u64_to_balance_option(0).unwrap(),
				share: 50000,
				has_withdrawn: false,
				block_number: 1,
				contributions: vec![
					ContributionLog { 
						amount: HousingFundModule::u64_to_balance_option(25).unwrap(),
						block_number: 1
					}],
				withdraws: Vec::new()
			})
		);
	});
}

#[test]
fn withdraw_without_being_contributor_should_fail() {
	new_test_ext().execute_with(|| {
		let account_id:u64 = 1;
		let non_contributor_account_id = 2;
		// test contribute with sufficient contribution: MinContribution is 10
		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(account_id), 25));

		// Try to withdraw with a non contributor account 
		assert_noop!(
			HousingFundModule::withdraw_fund(Origin::signed(non_contributor_account_id), 25),
			Error::<Test>::NotAContributor
		);
	});
}

#[test]
fn withdraw_more_than_contributed_should_fail() {
	new_test_ext().execute_with(|| {
		let account_id:u64 = 1;
		
		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(account_id), 25));

		// Try to withdraw more than contributed
		assert_noop!(
			HousingFundModule::withdraw_fund(Origin::signed(account_id), 30),
			Error::<Test>::NotEnoughFundToWithdraw
		);
	});
}

#[test]
fn withdraw_with_valid_values_should_succeed() {
	new_test_ext().execute_with(|| {
		let account_id:u64 = 1;
		
		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(account_id), 25));

		assert_ok!(HousingFundModule::withdraw_fund(Origin::signed(account_id), 20));

		// check if balance has been correctly updated
		assert_eq!(
			HousingFundModule::fund_balance(), 
			FundInfo{ 
				total: HousingFundModule::u64_to_balance_option(5).unwrap(),
				transferable: HousingFundModule::u64_to_balance_option(5).unwrap(),
				reserved: HousingFundModule::u64_to_balance_option(0).unwrap(),
				contributed: HousingFundModule::u64_to_balance_option(0).unwrap(),
			}
		);

		// check account's contribution amount and history
		assert_eq!(
			HousingFundModule::contributions(account_id),
			Some(Contribution {
				account_id: 1,
				available_balance: HousingFundModule::u64_to_balance_option(5).unwrap(),
				reserved_balance: HousingFundModule::u64_to_balance_option(0).unwrap(),
				contributed_balance: HousingFundModule::u64_to_balance_option(0).unwrap(),
				share: 100000,
				has_withdrawn: true,
				block_number: 1,
				contributions: vec![
					ContributionLog { 
						amount: HousingFundModule::u64_to_balance_option(25).unwrap(),
						block_number: 1
					}],
				withdraws: vec![
					ContributionLog { 
						amount: HousingFundModule::u64_to_balance_option(20).unwrap(),
						block_number: 1
					}]
			})
		);

		// check that balance is correct
		assert_eq!(
			Balances::free_balance(account_id),
			HousingFundModule::u64_to_balance_option(95).unwrap()
		);

		let event = <frame_system::Pallet<Test>>::events().pop()
            .expect("Expected at least one EventRecord to be found").event;

		// check the event has been raised
		assert_eq!(
			event, 
			mock::Event::HousingFundModule(crate::Event::WithdrawalSucceeded(1, 20, crate::WithdrawalReason::NotDefined, 1))
		);
	});
}

#[test]
fn withdraw_with_valid_values_from_two_contributors_should_succeed() {
	new_test_ext().execute_with(|| {
		let first_account_id:u64 = 1;
		let second_account_id:u64 = 2;
		// test contribute with sufficient contribution: MinContribution is 10
		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(first_account_id), 25));
		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(second_account_id), 25));

		assert_ok!(HousingFundModule::withdraw_fund(Origin::signed(first_account_id), 20));
		assert_ok!(HousingFundModule::withdraw_fund(Origin::signed(second_account_id), 20));

		assert_eq!(
			HousingFundModule::fund_balance(), 
			FundInfo{ 
				total: HousingFundModule::u64_to_balance_option(10).unwrap(),
				transferable: HousingFundModule::u64_to_balance_option(10).unwrap(),
				reserved: HousingFundModule::u64_to_balance_option(0).unwrap(),
				contributed: HousingFundModule::u64_to_balance_option(0).unwrap(),
			}
		);

		assert_eq!(
			HousingFundModule::contributions(first_account_id),
			Some(Contribution {
				account_id: 1,
				available_balance: HousingFundModule::u64_to_balance_option(5).unwrap(),
				reserved_balance: HousingFundModule::u64_to_balance_option(0).unwrap(),
				contributed_balance: HousingFundModule::u64_to_balance_option(0).unwrap(),
				share: 50000,
				has_withdrawn: true,
				block_number: 1,
				contributions: vec![
					ContributionLog { 
						amount: HousingFundModule::u64_to_balance_option(25).unwrap(),
						block_number: 1
					}],
				withdraws: vec![
					ContributionLog { 
						amount: HousingFundModule::u64_to_balance_option(20).unwrap(),
						block_number: 1
					}]
			})
		);

		assert_eq!(
			HousingFundModule::contributions(second_account_id),
			Some(Contribution {
				account_id: 2,
				available_balance: HousingFundModule::u64_to_balance_option(5).unwrap(),
				reserved_balance: HousingFundModule::u64_to_balance_option(0).unwrap(),
				contributed_balance: HousingFundModule::u64_to_balance_option(0).unwrap(),
				share: 50000,
				has_withdrawn: true,
				block_number: 1,
				contributions: vec![
					ContributionLog { 
						amount: HousingFundModule::u64_to_balance_option(25).unwrap(),
						block_number: 1
					}],
				withdraws: vec![
					ContributionLog { 
						amount: HousingFundModule::u64_to_balance_option(20).unwrap(),
						block_number: 1
					}]
			})
		);
	});
}

#[test]
fn house_bidding_without_enough_in_fund_should_fail() {
	new_test_ext().execute_with(|| {
		let origin:u64 = 1;
		let account_id:u64 = 1;

		// Try to bid for a house without enough in pot
		assert_noop!(
			HousingFundModule::house_bidding(
				Origin::signed(origin), 
				account_id, 
				1, 
				60, 
				Vec::new()
			),
			Error::<Test>::NotEnoughAvailableBalance
		);
	});
}

#[test]
fn house_bidding_with_an_non_contributor_account_should_fail() {
	new_test_ext().execute_with(|| {
		let origin:u64 = 1;
		let account_id:u64 = 1;

		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(1), 60));

		// Try to bid for a house without enough in pot
		// account_id 2 hadn't contributed to the fund and should not be able to be part of the bid
		assert_noop!(
			HousingFundModule::house_bidding(
				Origin::signed(origin), 
				account_id, 
				1, 
				60, 
				vec![(1, 20), (2,40)]
			),
			Error::<Test>::NotAContributor
		);
	});
}

#[test]
fn house_bidding_with_an_contributor_with_not_enough_available_should_fail() {
	new_test_ext().execute_with(|| {
		let origin:u64 = 1;
		let account_id:u64 = 1;

		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(1), 40));
		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(2), 20));

		// Try to bid for a house without enough in pot
		// account_id 2 hadn't contributed to the fund and should not be able to be part of the bid
		assert_noop!(
			HousingFundModule::house_bidding(
				Origin::signed(origin), 
				account_id, 
				1, 
				60, 
				vec![(1, 30), (2,30)]
			),
			Error::<Test>::NotEnoughAvailableBalance
		);
	});
}

#[test]
fn house_bidding_with_valid_values_should_succeed() {
	new_test_ext().execute_with(|| {
		let origin:u64 = 1;
		let account_id:u64 = 1;

		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(1), 40));
		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(2), 40));

		assert_ok!(HousingFundModule::house_bidding(
			Origin::signed(origin), 
			account_id, 
			1, 
			60, 
			vec![(1, 30), (2,30)]
		));
		
		assert_eq!(
			HousingFundModule::fund_balance(), 
			FundInfo{ 
				total: HousingFundModule::u64_to_balance_option(80).unwrap(),
				transferable: HousingFundModule::u64_to_balance_option(20).unwrap(),
				reserved: HousingFundModule::u64_to_balance_option(60).unwrap(),
				contributed: HousingFundModule::u64_to_balance_option(0).unwrap(),
			}
		);

		assert_eq!(
			HousingFundModule::contributions(1),
			Some(Contribution {
				account_id: 1,
				available_balance: HousingFundModule::u64_to_balance_option(10).unwrap(),
				reserved_balance: HousingFundModule::u64_to_balance_option(30).unwrap(),
				contributed_balance: HousingFundModule::u64_to_balance_option(0).unwrap(),
				share: 50000,
				has_withdrawn: false,
				block_number: 1,
				contributions: vec![
					ContributionLog { 
						amount: HousingFundModule::u64_to_balance_option(40).unwrap(),
						block_number: 1
					}],
				withdraws: Vec::new()
			})
		);

		assert_eq!(
			HousingFundModule::contributions(2),
			Some(Contribution {
				account_id: 2,
				available_balance: HousingFundModule::u64_to_balance_option(10).unwrap(),
				reserved_balance: HousingFundModule::u64_to_balance_option(30).unwrap(),
				contributed_balance: HousingFundModule::u64_to_balance_option(0).unwrap(),
				share: 50000,
				has_withdrawn: false,
				block_number: 1,
				contributions: vec![
					ContributionLog { 
						amount: HousingFundModule::u64_to_balance_option(40).unwrap(),
						block_number: 1
					}],
				withdraws: Vec::new()
			})
		);

		assert_eq!(
			HousingFundModule::reservations(1),
			Some(FundOperation {
				account_id: 1,
				house_id: 1,
				amount: 60,
				block_number: 1,
				contributions: vec![(1, 30), (2, 30)]
			})
		);

		let event = <frame_system::Pallet<Test>>::events().pop()
            .expect("Expected at least one EventRecord to be found").event;

		assert_eq!(
			event, 
			mock::Event::HousingFundModule(crate::Event::FundReservationSucceeded(1, 1, 60, 1))
		);
	});
}
