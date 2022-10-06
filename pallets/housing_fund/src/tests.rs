use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_err, assert_noop, assert_ok};

#[test]
fn contribute_without_having_investor_role_should_fail() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = 1;
		// test contribute with unsufficient contribution: MinContribution is 10
		assert_noop!(
			HousingFundModule::contribute_to_fund(Origin::signed(account_id), 5),
			Error::<Test>::NotAnInvestor
		);
	});
}

#[test]
fn contribute_with_less_than_minimun_amount_should_fail() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = 1;
		// Give the investor role to the account
		assert_ok!(RoleModule::set_role(
			Origin::signed(account_id),
			account_id,
			crate::ROLES::Accounts::INVESTOR
		));
		// test contribute with unsufficient contribution: MinContribution is 10
		assert_noop!(
			HousingFundModule::contribute_to_fund(Origin::signed(account_id), 5),
			Error::<Test>::ContributionTooSmall
		);
	});
}

#[test]
fn contribute_with_with_not_enough_free_balance_should_fail() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = 1;
		// Give the investor role to the account
		assert_ok!(RoleModule::set_role(
			Origin::signed(account_id),
			account_id,
			crate::ROLES::Accounts::INVESTOR
		));
		// test contribute with unsufficient free balance: balancce is 100
		assert_noop!(
			HousingFundModule::contribute_to_fund(Origin::signed(account_id), 110),
			Error::<Test>::NotEnoughToContribute
		);
	});
}

#[test]
fn contribute_with_valid_values_should_succeed() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = 1;
		let fund_account_id = HousingFundModule::fund_account_id();
		let fund_account_balance = Balances::free_balance(&fund_account_id);
		// Give the investor role to the account
		assert_ok!(RoleModule::set_role(
			Origin::signed(account_id),
			account_id,
			crate::ROLES::Accounts::INVESTOR
		));

		// test contribute with sufficient contribution and free balance
		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(account_id), 25));

		// the fund should have been incremented
		assert_eq!(
			HousingFundModule::fund_balance(),
			FundInfo {
				total: HousingFundModule::u64_to_balance_option(25).unwrap(),
				transferable: HousingFundModule::u64_to_balance_option(25).unwrap(),
				reserved: HousingFundModule::u64_to_balance_option(0).unwrap(),
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
				has_withdrawn: false,
				block_number: 1,
				contributions: vec![ContributionLog {
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

		// Check the fund account has received the correct amount => 10 (minimum balance) + 25
		assert_eq!(
			Balances::free_balance(&fund_account_id),
			HousingFundModule::u64_to_balance_option(25).unwrap() + fund_account_balance
		);

		let event = <frame_system::Pallet<Test>>::events()
			.pop()
			.expect("Expected at least one EventRecord to be found")
			.event;

		// check that the event has been raised
		assert_eq!(
			event,
			mock::Event::HousingFundModule(crate::Event::ContributeSucceeded(1, 25, 1))
		);
	});
}

#[test]
fn contribute_update_contribution_should_succeed() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = 1;
		// Give the investor role to the account
		assert_ok!(RoleModule::set_role(
			Origin::signed(account_id),
			account_id,
			crate::ROLES::Accounts::INVESTOR
		));

		// contribute to the fund
		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(account_id), 20));
		// update the contribution
		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(account_id), 30));

		// a contribution must have been registered for the account
		assert_eq!(
			HousingFundModule::contributions(account_id),
			Some(Contribution {
				account_id: 1,
				available_balance: HousingFundModule::u64_to_balance_option(50).unwrap(),
				reserved_balance: HousingFundModule::u64_to_balance_option(0).unwrap(),
				contributed_balance: HousingFundModule::u64_to_balance_option(0).unwrap(),
				has_withdrawn: false,
				block_number: 1,
				contributions: vec![
					ContributionLog {
						amount: HousingFundModule::u64_to_balance_option(20).unwrap(),
						block_number: 1
					},
					ContributionLog {
						amount: HousingFundModule::u64_to_balance_option(30).unwrap(),
						block_number: 1
					}
				],
				withdraws: Vec::new()
			})
		);
	});
}

#[test]
fn contribute_with_valid_values_from_two_contributors_should_succeed() {
	new_test_ext().execute_with(|| {
		let first_account_id: u64 = 1;
		let second_account_id: u64 = 2;
		// Give the investor role to the accounts
		assert_ok!(RoleModule::set_role(
			Origin::signed(first_account_id),
			first_account_id,
			crate::ROLES::Accounts::INVESTOR
		));
		assert_ok!(RoleModule::set_role(
			Origin::signed(second_account_id),
			second_account_id,
			crate::ROLES::Accounts::INVESTOR
		));
		// test contribute with sufficient contribution: MinContribution is 10
		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(first_account_id), 25));
		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(second_account_id), 25));

		assert_eq!(
			HousingFundModule::fund_balance(),
			FundInfo {
				total: HousingFundModule::u64_to_balance_option(50).unwrap(),
				transferable: HousingFundModule::u64_to_balance_option(50).unwrap(),
				reserved: HousingFundModule::u64_to_balance_option(0).unwrap(),
			}
		);

		assert_eq!(
			HousingFundModule::contributions(first_account_id),
			Some(Contribution {
				account_id: 1,
				available_balance: HousingFundModule::u64_to_balance_option(25).unwrap(),
				reserved_balance: HousingFundModule::u64_to_balance_option(0).unwrap(),
				contributed_balance: HousingFundModule::u64_to_balance_option(0).unwrap(),
				has_withdrawn: false,
				block_number: 1,
				contributions: vec![ContributionLog {
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
				has_withdrawn: false,
				block_number: 1,
				contributions: vec![ContributionLog {
					amount: HousingFundModule::u64_to_balance_option(25).unwrap(),
					block_number: 1
				}],
				withdraws: Vec::new()
			})
		);
	});
}

#[test]
fn withdraw_without_being_investor_should_fail() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = 1;
		let non_contributor_account_id = 2;
		// Give the investor role to the account
		assert_ok!(RoleModule::set_role(
			Origin::signed(account_id),
			account_id,
			crate::ROLES::Accounts::INVESTOR
		));
		// test contribute with sufficient contribution: MinContribution is 10
		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(account_id), 25));

		// Try to withdraw with a non investor account
		assert_noop!(
			HousingFundModule::withdraw_fund(Origin::signed(non_contributor_account_id), 25),
			Error::<Test>::NotAnInvestor
		);
	});
}

#[test]
fn withdraw_without_being_contributor_should_fail() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = 1;
		let non_contributor_account_id = 2;
		// Give the investor role to the accounts
		assert_ok!(RoleModule::set_role(
			Origin::signed(account_id),
			account_id,
			crate::ROLES::Accounts::INVESTOR
		));
		assert_ok!(RoleModule::set_role(
			Origin::signed(non_contributor_account_id),
			non_contributor_account_id,
			crate::ROLES::Accounts::INVESTOR
		));
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
		let account_id: u64 = 1;
		// Give the investor role to the account
		assert_ok!(RoleModule::set_role(
			Origin::signed(account_id),
			account_id,
			crate::ROLES::Accounts::INVESTOR
		));

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
		let account_id: u64 = 1;
		let fund_account_id = HousingFundModule::fund_account_id();
		let fund_account_balance = Balances::free_balance(&fund_account_id);

		// Give the investor role to the account
		assert_ok!(RoleModule::set_role(
			Origin::signed(account_id),
			account_id,
			crate::ROLES::Accounts::INVESTOR
		));

		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(account_id), 25));

		// Check the state of the contribution
		assert_eq!(
			HousingFundModule::contributions(account_id),
			Some(Contribution {
				account_id: 1,
				available_balance: HousingFundModule::u64_to_balance_option(25).unwrap(),
				reserved_balance: HousingFundModule::u64_to_balance_option(0).unwrap(),
				contributed_balance: HousingFundModule::u64_to_balance_option(0).unwrap(),
				has_withdrawn: false,
				block_number: 1,
				contributions: vec![ContributionLog {
					amount: HousingFundModule::u64_to_balance_option(25).unwrap(),
					block_number: 1
				}],
				withdraws: Vec::new()
			})
		);

		assert_ok!(HousingFundModule::withdraw_fund(Origin::signed(account_id), 20));

		// check if balance has been correctly updated
		assert_eq!(
			HousingFundModule::fund_balance(),
			FundInfo {
				total: HousingFundModule::u64_to_balance_option(5).unwrap(),
				transferable: HousingFundModule::u64_to_balance_option(5).unwrap(),
				reserved: HousingFundModule::u64_to_balance_option(0).unwrap(),
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
				has_withdrawn: true,
				block_number: 1,
				contributions: vec![ContributionLog {
					amount: HousingFundModule::u64_to_balance_option(25).unwrap(),
					block_number: 1
				}],
				withdraws: vec![ContributionLog {
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

		// Check the fund account has been withdraw the correct amount
		assert_eq!(
			Balances::free_balance(&fund_account_id),
			HousingFundModule::u64_to_balance_option(5).unwrap() + fund_account_balance
		);

		let event = <frame_system::Pallet<Test>>::events()
			.pop()
			.expect("Expected at least one EventRecord to be found")
			.event;

		// check the event has been raised
		assert_eq!(
			event,
			mock::Event::HousingFundModule(crate::Event::WithdrawalSucceeded(
				1,
				20,
				crate::WithdrawalReason::NotDefined,
				1
			))
		);
	});
}

#[test]
fn withdraw_with_valid_values_from_two_contributors_should_succeed() {
	new_test_ext().execute_with(|| {
		let first_account_id: u64 = 1;
		let second_account_id: u64 = 2;
		// Give the investor role to the accounts
		assert_ok!(RoleModule::set_role(
			Origin::signed(first_account_id),
			first_account_id,
			crate::ROLES::Accounts::INVESTOR
		));
		assert_ok!(RoleModule::set_role(
			Origin::signed(second_account_id),
			second_account_id,
			crate::ROLES::Accounts::INVESTOR
		));
		// test contribute with sufficient contribution: MinContribution is 10
		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(first_account_id), 25));
		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(second_account_id), 25));

		assert_ok!(HousingFundModule::withdraw_fund(Origin::signed(first_account_id), 20));
		assert_ok!(HousingFundModule::withdraw_fund(Origin::signed(second_account_id), 20));

		assert_eq!(
			HousingFundModule::fund_balance(),
			FundInfo {
				total: HousingFundModule::u64_to_balance_option(10).unwrap(),
				transferable: HousingFundModule::u64_to_balance_option(10).unwrap(),
				reserved: HousingFundModule::u64_to_balance_option(0).unwrap(),
			}
		);

		assert_eq!(
			HousingFundModule::contributions(first_account_id),
			Some(Contribution {
				account_id: 1,
				available_balance: HousingFundModule::u64_to_balance_option(5).unwrap(),
				reserved_balance: HousingFundModule::u64_to_balance_option(0).unwrap(),
				contributed_balance: HousingFundModule::u64_to_balance_option(0).unwrap(),
				has_withdrawn: true,
				block_number: 1,
				contributions: vec![ContributionLog {
					amount: HousingFundModule::u64_to_balance_option(25).unwrap(),
					block_number: 1
				}],
				withdraws: vec![ContributionLog {
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
				has_withdrawn: true,
				block_number: 1,
				contributions: vec![ContributionLog {
					amount: HousingFundModule::u64_to_balance_option(25).unwrap(),
					block_number: 1
				}],
				withdraws: vec![ContributionLog {
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
		// Try to bid for a house without enough in pot
		assert_noop!(
			HousingFundModule::house_bidding(1, 1, 60, Vec::new()),
			Error::<Test>::NotEnoughFundForHouse
		);
	});
}

#[test]
fn house_bidding_with_an_non_contributor_account_should_fail() {
	new_test_ext().execute_with(|| {
		// Give the investor role to the account
		assert_ok!(RoleModule::set_role(Origin::signed(1), 1, crate::ROLES::Accounts::INVESTOR));

		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(1), 62));

		// Try to bid for a house without enough in pot
		// account_id 2 hadn't contributed to the fund and should not be able to be part of the bid
		assert_err!(
			HousingFundModule::house_bidding(1, 2, 50, vec![(1, 20), (2, 30)]),
			Error::<Test>::NotAContributor
		);
	});
}

#[test]
fn house_bidding_with_an_contributor_with_not_enough_available_should_fail() {
	new_test_ext().execute_with(|| {
		// Give the investor role to the accounts
		assert_ok!(RoleModule::set_role(Origin::signed(1), 1, crate::ROLES::Accounts::INVESTOR));
		assert_ok!(RoleModule::set_role(Origin::signed(2), 2, crate::ROLES::Accounts::INVESTOR));

		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(1), 42));
		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(2), 20));

		// Try to bid for a house without enough in pot
		// account_id 2 hadn't contributed to the fund and should not be able to be part of the bid
		assert_err!(
			HousingFundModule::house_bidding(1, 1, 60, vec![(1, 30), (2, 30)]),
			Error::<Test>::NotEnoughAvailableBalance
		);
	});
}

#[test]
fn house_bidding_with_valid_values_should_succeed() {
	new_test_ext().execute_with(|| {
		let fund_account_id = HousingFundModule::fund_account_id();

		// Give the investor role to the accounts
		assert_ok!(RoleModule::set_role(Origin::signed(1), 1, crate::ROLES::Accounts::INVESTOR));
		assert_ok!(RoleModule::set_role(Origin::signed(2), 2, crate::ROLES::Accounts::INVESTOR));

		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(1), 40));
		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(2), 40));

		assert_ok!(HousingFundModule::house_bidding(1, 1, 60, vec![(1, 30), (2, 30)]));

		assert_eq!(
			HousingFundModule::fund_balance(),
			FundInfo {
				total: HousingFundModule::u64_to_balance_option(80).unwrap(),
				transferable: HousingFundModule::u64_to_balance_option(20).unwrap(),
				reserved: HousingFundModule::u64_to_balance_option(60).unwrap(),
			}
		);

		assert_eq!(
			HousingFundModule::contributions(1),
			Some(Contribution {
				account_id: 1,
				available_balance: HousingFundModule::u64_to_balance_option(10).unwrap(),
				reserved_balance: HousingFundModule::u64_to_balance_option(30).unwrap(),
				contributed_balance: HousingFundModule::u64_to_balance_option(0).unwrap(),
				has_withdrawn: false,
				block_number: 1,
				contributions: vec![ContributionLog {
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
				has_withdrawn: false,
				block_number: 1,
				contributions: vec![ContributionLog {
					amount: HousingFundModule::u64_to_balance_option(40).unwrap(),
					block_number: 1
				}],
				withdraws: Vec::new()
			})
		);

		assert_eq!(
			HousingFundModule::reservations((1, 1)),
			Some(FundOperation {
				nft_collection_id: 1,
				nft_item_id: 1,
				amount: 60,
				block_number: 1,
				contributions: vec![(1, 30), (2, 30)]
			})
		);

		// Check the amount reserved for the account
		assert_eq!(
			Balances::reserved_balance(&fund_account_id),
			HousingFundModule::u64_to_balance_option(60).unwrap()
		);

		let event = <frame_system::Pallet<Test>>::events()
			.pop()
			.expect("Expected at least one EventRecord to be found")
			.event;

		assert_eq!(
			event,
			mock::Event::HousingFundModule(crate::Event::FundReservationSucceeded(1, 1, 60, 1))
		);
	});
}

#[test]
fn cancel_house_bidding_with_invalid_values_should_fail() {
	new_test_ext().execute_with(|| {
		// Try to cancel a bidding that doesn't exist
		assert_noop!(
			HousingFundModule::cancel_house_bidding(1, 1),
			Error::<Test>::NoFundReservationFound
		);
	});
}

#[test]
fn cancel_house_bidding_with_valid_values_should_succeed() {
	new_test_ext().execute_with(|| {
		let fund_account_id = HousingFundModule::fund_account_id();

		// Give the investor role to the accounts
		assert_ok!(RoleModule::set_role(Origin::signed(1), 1, crate::ROLES::Accounts::INVESTOR));
		assert_ok!(RoleModule::set_role(Origin::signed(2), 2, crate::ROLES::Accounts::INVESTOR));

		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(1), 40));
		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(2), 40));

		assert_ok!(HousingFundModule::house_bidding(1, 1, 60, vec![(1, 30), (2, 30)]));

		assert_ok!(HousingFundModule::cancel_house_bidding(1, 1));

		assert_eq!(
			HousingFundModule::fund_balance(),
			FundInfo {
				total: HousingFundModule::u64_to_balance_option(80).unwrap(),
				transferable: HousingFundModule::u64_to_balance_option(80).unwrap(),
				reserved: HousingFundModule::u64_to_balance_option(0).unwrap(),
			}
		);

		assert_eq!(
			HousingFundModule::contributions(1),
			Some(Contribution {
				account_id: 1,
				available_balance: HousingFundModule::u64_to_balance_option(40).unwrap(),
				reserved_balance: HousingFundModule::u64_to_balance_option(0).unwrap(),
				contributed_balance: HousingFundModule::u64_to_balance_option(0).unwrap(),
				has_withdrawn: false,
				block_number: 1,
				contributions: vec![ContributionLog {
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
				available_balance: HousingFundModule::u64_to_balance_option(40).unwrap(),
				reserved_balance: HousingFundModule::u64_to_balance_option(0).unwrap(),
				contributed_balance: HousingFundModule::u64_to_balance_option(0).unwrap(),
				has_withdrawn: false,
				block_number: 1,
				contributions: vec![ContributionLog {
					amount: HousingFundModule::u64_to_balance_option(40).unwrap(),
					block_number: 1
				}],
				withdraws: Vec::new()
			})
		);

		assert_eq!(HousingFundModule::reservations((1, 1)).is_none(), true);

		// Check the amount reserved for the account
		assert_eq!(
			Balances::reserved_balance(&fund_account_id),
			HousingFundModule::u64_to_balance_option(0).unwrap()
		);

		let event = <frame_system::Pallet<Test>>::events()
			.pop()
			.expect("Expected at least one EventRecord to be found")
			.event;

		assert_eq!(
			event,
			mock::Event::HousingFundModule(crate::Event::FundReservationCancelled(1, 1, 60, 1))
		);
	});
}

#[test]
fn fund_info_contribute_transferable_should_succeed() {
	new_test_ext().execute_with(|| {
		let mut fund_info = HousingFundModule::fund_balance();
		// contribute to the fund
		fund_info.contribute_transferable(100);

		// check that the values are valid
		assert_eq!(fund_info.total, 100);
		assert_eq!(fund_info.transferable, 100);
		assert_eq!(fund_info.reserved, 0);
	});
}

#[test]
fn fund_info_can_take_off_should_succeed() {
	new_test_ext().execute_with(|| {
		let mut fund_info = HousingFundModule::fund_balance();
		// contribute to the fund
		fund_info.contribute_transferable(100);
		// check that the test is correct
		assert_eq!(fund_info.can_take_off(50), true);
		assert_eq!(fund_info.can_take_off(110), false);
	});
}

#[test]
fn fund_info_withdraw_transferable_should_succeed() {
	new_test_ext().execute_with(|| {
		let mut fund_info = HousingFundModule::fund_balance();
		// contribute then withdraw
		fund_info.contribute_transferable(100);
		fund_info.withdraw_transferable(80);
		// check tha the values are valid
		assert_eq!(fund_info.total, 20);
		assert_eq!(fund_info.transferable, 20);
		assert_eq!(fund_info.reserved, 0);
	});
}

#[test]
fn fund_info_reserve_should_succeed() {
	new_test_ext().execute_with(|| {
		let mut fund_info = HousingFundModule::fund_balance();
		// reserve an amount in the fund
		fund_info.contribute_transferable(100);
		fund_info.reserve(80);
		// check that the values are valid
		assert_eq!(fund_info.total, 100);
		assert_eq!(fund_info.transferable, 20);
		assert_eq!(fund_info.reserved, 80);
	});
}

#[test]
fn contribution_get_total_balance_should_succeed() {
	new_test_ext().execute_with(|| {
		// Give the investor role to the account
		assert_ok!(RoleModule::set_role(Origin::signed(1), 1, crate::ROLES::Accounts::INVESTOR));
		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(1), 50));

		let mut contribution = HousingFundModule::contributions(1).unwrap();
		// reserve an amount from an account contribution
		contribution.reserve_amount(30);
		// check that the total balance correctly calculated
		assert_eq!(
			contribution.get_total_balance(),
			contribution.available_balance + contribution.reserved_balance
		);
	});
}

#[test]
fn contribution_can_reserve_should_succeed() {
	new_test_ext().execute_with(|| {
		// Give the investor role to the account
		assert_ok!(RoleModule::set_role(Origin::signed(1), 1, crate::ROLES::Accounts::INVESTOR));
		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(1), 50));
		// get the account contribution
		let contribution = HousingFundModule::contributions(1).unwrap();
		// check if the method respond correctly
		assert_eq!(contribution.can_reserve(30), true);
		assert_eq!(contribution.can_reserve(60), false);
	});
}

#[test]
fn contribution_reserve_amount_should_succeed() {
	new_test_ext().execute_with(|| {
		// Give the investor role to the accounts
		assert_ok!(RoleModule::set_role(Origin::signed(1), 1, crate::ROLES::Accounts::INVESTOR));
		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(1), 50));

		let mut contribution = HousingFundModule::contributions(1).unwrap();

		contribution.reserve_amount(30);
		// check that contribution balance is correctly set
		assert_eq!(contribution.available_balance, 20);
		assert_eq!(contribution.reserved_balance, 30);
	});
}

#[test]
fn contribution_unreserve_amount_should_succeed() {
	new_test_ext().execute_with(|| {
		// Give the investor role to the accounts
		assert_ok!(RoleModule::set_role(Origin::signed(1), 1, crate::ROLES::Accounts::INVESTOR));
		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(1), 50));

		let mut contribution = HousingFundModule::contributions(1).unwrap();

		contribution.reserve_amount(30);
		contribution.unreserve_amount(20);
		// check that contribution balance is correctly set
		assert_eq!(contribution.available_balance, 40);
		assert_eq!(contribution.reserved_balance, 10);
	});
}

#[test]
fn get_contribution_share_should_succeed() {
	new_test_ext().execute_with(|| {
		// Give the investor role to the accounts
		assert_ok!(RoleModule::set_role(Origin::signed(1), 1, crate::ROLES::Accounts::INVESTOR));
		assert_ok!(RoleModule::set_role(Origin::signed(2), 2, crate::ROLES::Accounts::INVESTOR));

		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(1), 40));
		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(2), 40));

		assert_eq!(
			HousingFundModule::get_contribution_share(),
			vec![
				ContributionShare { account_id: 1, share: 50000 },
				ContributionShare { account_id: 2, share: 50000 },
			]
		);
	});
}

#[test]
fn check_available_fund_not_enough_fund_should_fail() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = 1;

		assert_ok!(RoleModule::set_role(
			Origin::signed(account_id),
			account_id,
			crate::ROLES::Accounts::INVESTOR
		));

		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(account_id), 20));

		assert_eq!(HousingFundModule::check_available_fund(20), false);
	});
}

#[test]
fn check_available_fund_has_enough_fund_should_succeed() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = 1;

		assert_ok!(RoleModule::set_role(
			Origin::signed(account_id),
			account_id,
			crate::ROLES::Accounts::INVESTOR
		));

		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(account_id), 25));

		assert_eq!(HousingFundModule::check_available_fund(20), true);
	});
}

#[test]
fn get_contributions_without_contribution_should_succeed() {
	new_test_ext().execute_with(|| {
		let contributions = HousingFundModule::get_contributions();
		assert_eq!(contributions.len(), 0);
	});
}

#[test]
fn get_contributions_with_contribution_should_succeed() {
	new_test_ext().execute_with(|| {
		// Give the investor role to the accounts
		assert_ok!(RoleModule::set_role(Origin::signed(1), 1, crate::ROLES::Accounts::INVESTOR));
		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(1), 25));

		let contributions = HousingFundModule::get_contributions();
		assert_eq!(contributions.len(), 1);

		assert_eq!(
			contributions[0].1,
			Contribution {
				account_id: 1,
				available_balance: HousingFundModule::u64_to_balance_option(25).unwrap(),
				reserved_balance: HousingFundModule::u64_to_balance_option(0).unwrap(),
				contributed_balance: HousingFundModule::u64_to_balance_option(0).unwrap(),
				has_withdrawn: false,
				block_number: 1,
				contributions: vec![ContributionLog {
					amount: HousingFundModule::u64_to_balance_option(25).unwrap(),
					block_number: 1
				}],
				withdraws: Vec::new()
			}
		);
	});
}

#[test]
fn fund_info_use_reserved_should_succeed() {
	new_test_ext().execute_with(|| {
		let mut fund_info = HousingFundModule::fund_balance();
		// reserve an amount in the fund
		fund_info.contribute_transferable(100);
		fund_info.reserve(80);
		fund_info.use_reserved(50);
		// check that the values are valid
		assert_eq!(fund_info.total, 50);
		assert_eq!(fund_info.transferable, 20);
		assert_eq!(fund_info.reserved, 30);
	});
}

#[test]
fn contribution_use_reserved_amount_should_succeed() {
	new_test_ext().execute_with(|| {
		// Give the investor role to the accounts
		assert_ok!(RoleModule::set_role(Origin::signed(1), 1, crate::ROLES::Accounts::INVESTOR));
		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(1), 50));

		let mut contribution = HousingFundModule::contributions(1).unwrap();

		contribution.reserve_amount(30);
		contribution.use_reserved_amount(20);
		// check that contribution balance is correctly set
		assert_eq!(contribution.available_balance, 20);
		assert_eq!(contribution.reserved_balance, 10);
	});
}

#[test]
fn unreserve_house_bidding_amount_with_invalid_values_should_fail() {
	new_test_ext().execute_with(|| {
		// Try to unreserve form a bidding that doesn't exist
		assert_noop!(
			HousingFundModule::unreserve_house_bidding_amount(1, 1),
			Error::<Test>::NoFundReservationFound
		);
	});
}

#[test]
fn unreserve_house_bidding_amount_with_valid_values_should_succeed() {
	new_test_ext().execute_with(|| {
		let _fund_account_id = HousingFundModule::fund_account_id();

		// Give the investor role to the accounts
		assert_ok!(RoleModule::set_role(Origin::signed(1), 1, crate::ROLES::Accounts::INVESTOR));
		assert_ok!(RoleModule::set_role(Origin::signed(2), 2, crate::ROLES::Accounts::INVESTOR));

		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(1), 40));
		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(2), 40));

		assert_ok!(HousingFundModule::house_bidding(1, 1, 60, vec![(1, 30), (2, 30)]));

		assert_ok!(HousingFundModule::unreserve_house_bidding_amount(1, 1));

		assert_eq!(
			HousingFundModule::fund_balance(),
			FundInfo {
				total: HousingFundModule::u64_to_balance_option(80).unwrap(),
				transferable: HousingFundModule::u64_to_balance_option(20).unwrap(),
				reserved: HousingFundModule::u64_to_balance_option(60).unwrap(),
			}
		);

		let event = <frame_system::Pallet<Test>>::events()
			.pop()
			.expect("Expected at least one EventRecord to be found")
			.event;

		assert_eq!(
			event,
			mock::Event::HousingFundModule(crate::Event::FundUnreservedForPurchase(1, 1, 60, 1))
		);
	});
}

#[test]
fn validate_house_bidding_with_invalid_values_should_fail() {
	new_test_ext().execute_with(|| {
		// Try to validate a bidding that doesn't exist
		assert_noop!(
			HousingFundModule::validate_house_bidding(1, 1),
			Error::<Test>::NoFundReservationFound
		);
	});
}

#[test]
fn validate_house_bidding_with_valid_values_should_succeed() {
	new_test_ext().execute_with(|| {
		let _fund_account_id = HousingFundModule::fund_account_id();

		// Give the investor role to the accounts
		assert_ok!(RoleModule::set_role(Origin::signed(1), 1, crate::ROLES::Accounts::INVESTOR));
		assert_ok!(RoleModule::set_role(Origin::signed(2), 2, crate::ROLES::Accounts::INVESTOR));

		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(1), 40));
		assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(2), 40));

		assert_ok!(HousingFundModule::house_bidding(1, 1, 60, vec![(1, 30), (2, 30)]));

		assert_ok!(HousingFundModule::validate_house_bidding(1, 1));

		assert_eq!(
			HousingFundModule::fund_balance(),
			FundInfo {
				total: HousingFundModule::u64_to_balance_option(20).unwrap(),
				transferable: HousingFundModule::u64_to_balance_option(20).unwrap(),
				reserved: HousingFundModule::u64_to_balance_option(0).unwrap(),
			}
		);

		assert_eq!(
			HousingFundModule::contributions(1),
			Some(Contribution {
				account_id: 1,
				available_balance: HousingFundModule::u64_to_balance_option(10).unwrap(),
				reserved_balance: HousingFundModule::u64_to_balance_option(0).unwrap(),
				contributed_balance: HousingFundModule::u64_to_balance_option(30).unwrap(),
				has_withdrawn: false,
				block_number: 1,
				contributions: vec![ContributionLog {
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
				reserved_balance: HousingFundModule::u64_to_balance_option(0).unwrap(),
				contributed_balance: HousingFundModule::u64_to_balance_option(30).unwrap(),
				has_withdrawn: false,
				block_number: 1,
				contributions: vec![ContributionLog {
					amount: HousingFundModule::u64_to_balance_option(40).unwrap(),
					block_number: 1
				}],
				withdraws: Vec::new()
			})
		);

		assert_eq!(HousingFundModule::reservations((1, 1)).is_none(), true);
		assert_eq!(HousingFundModule::purchases((1, 1)).is_some(), true);

		let event = <frame_system::Pallet<Test>>::events()
			.pop()
			.expect("Expected at least one EventRecord to be found")
			.event;

		assert_eq!(
			event,
			mock::Event::HousingFundModule(crate::Event::PurchaseFundValidated(1, 1, 60, 1))
		);
	});
}
