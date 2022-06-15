use crate::{mock::*, Error};
use frame_support::{assert_noop,assert_err, assert_ok};
use super::*;
use structs;



// -------- Struct methods----------------
#[test]
fn struct_methods_ok() {
	new_test_ext().execute_with(|| {
		//--checking if there is any registered investor
		assert_eq!(InvestorLog::<Test>::contains_key(1),false);
		//---instantiate new Investor struct -----
		assert_eq!(Investor::<Test>::new(Origin::signed(1)),
			Investor{
				account_id:1,
				nft_index: Vec::new(),
				age:0,
				share:0,
				selections:0,
			}
		);
		//---checking storage----------------
		assert_eq!(InvestorLog::<Test>::contains_key(1),true);

		//--trying creating same investor account,it should fail
		assert_noop!(RoleModule::create_account(Origin::signed(1),Accounts::INVESTOR),
			Error::<Test>::NoneValue
		);
		//----same test with other account creation------

	});
}

#[test]
fn correct_error_for_none_value() {
	new_test_ext().execute_with(|| {

	});
}
