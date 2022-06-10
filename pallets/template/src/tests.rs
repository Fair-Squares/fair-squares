use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use super::*;


//mod.rs testing

#[test]
fn test_new_account_investor_ok() {
	new_test_ext().execute_with(|| {
		//checking if there is account investor already stored
		assert_eq!(InvestorLog::<Test>::contains_key(1),false);
		//making sure the account is not on other registrations
		assert_eq!(HouseSellerLog::<Test>::contains_key(1),false);
		//adding new investor account
		assert_ok!(TemplateModule::create_account(Origin::signed(1),Accounts::INVESTOR));
		//checking if there is INVESTOR registered for account 1
		assert_eq!(InvestorLog::<Test>::contains_key(1),true)

	});
}

#[test]
fn test_new_account_seller_ok() {
	new_test_ext().execute_with(|| {
		//checking if there is account seller already stored
		assert_eq!(HouseSellerLog::<Test>::contains_key(1),false);
		//making sure the account is not on other registrations
		assert_eq!(InvestorLog::<Test>::contains_key(1),false);
		//adding new investor account
		assert_ok!(TemplateModule::create_account(Origin::signed(1),Accounts::SELLER));
		//checking if there is INVESTOR registered for account 1
		assert_eq!(HouseSellerLog::<Test>::contains_key(1),true)

	});
}

//for the existing logic is that tenant can be a seller even an investor.
#[test]
fn test_new_account_tenant_ok() {
	new_test_ext().execute_with(|| {
		assert_ok!(TemplateModule::create_account(Origin::signed(1),Accounts::TENANT));
	});
}

// testing creating asset






// testing create proposal





//  testing all new methods for structs in the module
#[test]
fn correct_error_for_none_value() {
	new_test_ext().execute_with(|| {
	});
}

//lib.rs testing
