use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};


#[test]
fn contribution_test_should_fail() {
	new_test_ext().execute_with(|| {
		// add funds to accountId nbr 1
		let bal:u128 = 100;
		let caller:u64 = 1;
		Balances::make_free_balance_be(&caller,bal);
		// test contribute with unsufficient contribution: MinContribution is 10
		assert_noop!(HousingFundModule::contribute_to_fund(Origin::signed(1), 5),Error::<Test>::ContributionTooSmall);
		
	});
}

//#[test]
//fn contribution_test_should_work() {
	//new_test_ext().execute_with(|| {
		// add funds to accountId nbr 1
	//	let bal:u128 = 100;
	//	let caller:u64 = 1;
	//	Balances::make_free_balance_be(&caller,bal);
		// test contribute with sufficient contribution: MinContribution is 10
	//	assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(caller), 25));
		
	//});
//}

