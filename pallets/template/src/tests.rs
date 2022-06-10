use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, assert_err};
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

//testing seller account creation
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
// Waiting for lib.rs to update its implementation on TENANT
#[test]
fn test_new_account_tenant_ok() {
	new_test_ext().execute_with(|| {
		assert_ok!(TemplateModule::create_account(Origin::signed(1),Accounts::TENANT));
	});
}

// testing creating asset
#[test]
fn test_create_asset_ok() {
	new_test_ext().execute_with(|| {

	})
}





// testing create proposal





//  testing all new methods for structs in the module
#[test]
fn test_investor_struct_methods() {
	new_test_ext().execute_with(|| {
		assert_eq!(roles::Investor::<Test>::new(Origin::signed(1)),
				   roles::Investor{
					   account_id: 1,
					   nft_index: Vec::new(),
					   age: System::block_number(),
					   share: 0,
					   selections:0,
				   }
		);

		//-----contribution method--------//
		// making sure the contribution is above minimum_contribution as for now is = 10
		//for testing
		assert_noop!(roles::Investor::<Test>::contribute(
			roles::Investor{
					   account_id: 1,
					   nft_index: Vec::new(),
					   age: System::block_number(),
					   share: 0,
					   selections:0,
				   },
			Origin::signed(1),
				5
			),
			Error::<Test>::ContributionTooSmall
			);
		});

	// This should work fine
	TemplateModule::create_account(Origin::signed(1),Accounts::INVESTOR);

	assert_ok!(roles::Investor::<Test>::contribute(
		roles::Investor{
					   account_id: 1,
					   nft_index: Vec::new(),
					   age: System::block_number(),
					   share: 0,
					   selections:0,
				   },
		Origin::signed(1),20)
	);
}

//lib.rs testing
