use crate::{mock::*, Error,pallet::*};
use frame_support::{assert_noop, assert_ok};

#[test]
fn investor_creation_should_work() {
	new_test_ext().execute_with(|| {
		// add funds to accountId nbr 1
		let bal:u128 = 100;
		let caller:u64 = 1;
		Balances::make_free_balance_be(&caller,bal);
		// test contribute with unsufficient contribution: MinContribution is 10
		assert_ok!(RoleModule::create_account(Origin::signed(1), Accounts::INVESTOR));
		
	});
}