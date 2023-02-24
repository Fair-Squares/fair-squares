use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn test_struct_methods() {
	new_test_ext(4).execute_with(|| {
		assert_ok!(Investor::<Test>::new(Origin::signed(1)));
		//--checking investor storage if its updated----
		assert!(InvestorLog::<Test>::contains_key(1));
		assert_eq!(
			RoleModule::investors(1),
			Some(Investor { account_id: 1, age: System::block_number(), share: 0, selections: 0 })
		);

		//---HouseSeller-------
		assert_ok!(HouseSeller::<Test>::new(Origin::signed(1)));
		assert_eq!(
			RoleModule::get_pending_house_sellers(),
			vec![HouseSeller {
				account_id: 1,
				age: System::block_number(),
				activated: false,
				verifier: 4
			}]
		);
		//---house seller should fail successfully----
		assert_ne!(RoleModule::get_pending_house_sellers(), vec![]); //assert_ne! is not supported at the moment, as this expression should panic

		//-------tenant-----------
		assert_ok!(Tenant::<Test>::new(Origin::signed(1)));
		//-- checking Tenant storage------
		assert_eq!(
			RoleModule::tenants(1),
			Some(Tenant {
				account_id: 1,
				rent: 0,
				age: System::block_number(),
				asset_account: None,
				contract_start: System::block_number(),
				remaining_rent: 0,
				remaining_payments: 0,
				registered: false,
			})
		);

		//-----Servicer-----------------------------------------
		assert_ok!(Servicer::<Test>::new(Origin::signed(2)));
		//--checking storage-------------
		assert_eq!(
			RoleModule::get_pending_servicers(),
			vec![Servicer {
				account_id: 2,
				age: System::block_number(),
				activated: false,
				verifier: 4
			}]
		);

		//Representative
		assert_ok!(Representative::<Test>::new(Origin::signed(3)));
		//checking struct in Representative waiting list
		assert_eq!(
			RoleModule::get_pending_representatives(3).unwrap(),
			Representative {
				account_id: 3,
				age: System::block_number(),
				activated: false,
				assets_accounts: vec![],
				index: 0,
			}
		)
	});
}

#[test]
fn test_account_approval_rejection() {
	new_test_ext(4).execute_with(|| {
		//----testing account approval-----
		let master = Origin::signed(4);
		assert_eq!(RoleModule::get_pending_servicers().len(), 0);
		assert_eq!(RoleModule::get_pending_house_sellers().len(), 0);

		assert_eq!(RoleModule::get_pending_notaries().len(), 0);

		assert_ok!(RoleModule::set_role(Origin::signed(2), 2, Accounts::SERVICER));
		assert_ok!(RoleModule::set_role(Origin::signed(3), 3, Accounts::SELLER));
		assert_ok!(RoleModule::set_role(Origin::signed(5), 5, Accounts::SERVICER));
		assert_ok!(RoleModule::set_role(Origin::signed(6), 6, Accounts::SELLER));
		assert_ok!(RoleModule::set_role(Origin::signed(7), 7, Accounts::NOTARY));
		assert_ok!(RoleModule::set_role(Origin::signed(8), 8, Accounts::NOTARY));

		assert_eq!(RoleModule::get_requested_role(2), Some(Accounts::SERVICER));
		assert_eq!(RoleModule::get_requested_role(3), Some(Accounts::SELLER));
		assert_eq!(RoleModule::get_requested_role(5), Some(Accounts::SERVICER));
		assert_eq!(RoleModule::get_requested_role(6), Some(Accounts::SELLER));
		assert_eq!(RoleModule::get_requested_role(7), Some(Accounts::NOTARY));
		assert_eq!(RoleModule::get_requested_role(8), Some(Accounts::NOTARY));

		let servicers = RoleModule::get_pending_servicers();
		assert_eq!(servicers.len(), 2);
		assert!(!servicers[0].activated);
		assert!(!servicers[1].activated);
		assert_eq!(servicers[0].verifier, 4);
		assert_eq!(servicers[1].verifier, 4);

		let sellers = RoleModule::get_pending_house_sellers();
		assert_eq!(sellers.len(), 2);
		assert!(!sellers[0].activated);
		assert!(!sellers[1].activated);
		assert_eq!(sellers[0].verifier, 4);
		assert_eq!(sellers[1].verifier, 4);

		let notaries = RoleModule::get_pending_notaries();
		assert_eq!(notaries.len(), 2);
		assert!(!notaries[0].activated);
		assert!(!notaries[1].activated);
		assert_eq!(notaries[0].verifier, 4);
		assert_eq!(notaries[1].verifier, 4);

		assert_ok!(RoleModule::account_approval(master.clone(), 2));
		assert_ok!(RoleModule::account_approval(master.clone(), 3));
		assert_ok!(RoleModule::account_rejection(master.clone(), 5));
		assert_ok!(RoleModule::account_rejection(master.clone(), 6));
		assert_ok!(RoleModule::account_approval(master.clone(), 7));
		assert_ok!(RoleModule::account_rejection(master, 8));

		assert!(RoleModule::get_requested_role(5).is_none());
		assert!(RoleModule::get_requested_role(8).is_none());

		assert_eq!(RoleModule::get_pending_servicers().len(), 0);
		assert_eq!(RoleModule::get_pending_house_sellers().len(), 0);
		assert_eq!(RoleModule::get_pending_notaries().len(), 0);

		assert!(ServicerLog::<Test>::contains_key(2));
		assert!(RoleModule::servicers(2).unwrap().activated);
		assert_eq!(RoleModule::servicers(2).unwrap().verifier, 4);
		assert!(!ServicerLog::<Test>::contains_key(5));

		assert!(HouseSellerLog::<Test>::contains_key(3));
		assert!(RoleModule::sellers(3).unwrap().activated);
		assert_eq!(RoleModule::sellers(3).unwrap().verifier, 4);
		assert!(!HouseSellerLog::<Test>::contains_key(6));

		assert!(NotaryLog::<Test>::contains_key(7));
		assert!(RoleModule::notaries(7).unwrap().activated);
		assert_eq!(RoleModule::notaries(7).unwrap().verifier, 4);
		assert!(!NotaryLog::<Test>::contains_key(8));
	})
}

#[test]
fn test_account_creation() {
	new_test_ext(4).execute_with(|| {
		let master = Origin::signed(4);
		let user1 = Origin::signed(1);
		let user2 = Origin::signed(2);
		let user3 = Origin::signed(3);
		let user4 = Origin::signed(5);
		let user5 = Origin::signed(6);

		let wait_sell = RoleModule::get_pending_house_sellers();
		let sell_len = wait_sell.len();

		assert_ok!(RoleModule::set_role(user5.clone(), 6, Acc::SERVICER));
		assert_ok!(RoleModule::account_approval(master.clone(), 6));

		assert_ok!(RoleModule::set_role(user1.clone(), 1, Acc::INVESTOR));
		assert!(InvestorLog::<Test>::contains_key(1));
		assert_noop!(
			RoleModule::set_role(user1.clone(), 1, Acc::TENANT),
			Error::<Test>::OneRoleAllowed
		);

		assert_ok!(RoleModule::set_role(user3, 3, Acc::TENANT));
		assert!(TenantLog::<Test>::contains_key(3));

		assert_ok!(RoleModule::set_role(user2.clone(), 2, Acc::SELLER));
		assert_noop!(RoleModule::set_role(user2, 2, Acc::SELLER), Error::<Test>::AlreadyWaiting);
		let wait_sell = RoleModule::get_pending_house_sellers();
		let sell_len2 = wait_sell.len();
		assert_eq!(sell_len2, sell_len + 1);
		assert_eq!(RoleModule::total_members(), 3);
		assert_ok!(RoleModule::account_approval(master, 2));
		assert!(HouseSellerLog::<Test>::contains_key(2));
		assert_eq!(RoleModule::total_members(), 4);

		//Non Servicer user1 try to assign Investor role to 7 and fail
		assert_noop!(
			RoleModule::set_role(user1, 7, Acc::INVESTOR),
			Error::<Test>::OnlyForServicers
		);
		assert_eq!(RoleModule::total_members(), 4);
		//Servicer user5 successfully assign Investor role to 7
		assert_ok!(RoleModule::set_role(user5, 7, Acc::INVESTOR));
		//No additional member can be added
		assert_noop!(
			RoleModule::set_role(user4, 5, Acc::TENANT),
			Error::<Test>::TotalMembersExceeded
		);
	})
}

#[test]
fn test_role_notary() {
	new_test_ext(0).execute_with(|| {
		let admin = 0;
		let user1 = 1;

		// user1: set_role - notary
		assert_ok!(RoleModule::set_role(Origin::signed(user1), user1, Acc::NOTARY));

		// check notary approval list
		assert_eq!(
			RoleModule::get_pending_notaries(),
			vec![Notary {
				account_id: user1,
				activated: false,
				verifier: admin,
				age: System::block_number()
			}]
		);

		// approve notary
		assert_ok!(RoleModule::account_approval(Origin::signed(admin), user1));

		// check notary storage
		assert_eq!(
			RoleModule::notaries(user1).unwrap(),
			Notary {
				account_id: user1,
				activated: true,
				verifier: admin,
				age: System::block_number()
			}
		);

		// check total members
		assert_eq!(RoleModule::total_members(), 1);
	});
}

#[test]
fn test_set_manager() {
	new_test_ext(4).execute_with(|| {
		//-----checking existing manager--------
		assert_eq!(Sudo::key(), Some(4));
		//---changing--------------------------

		assert_ok!(RoleModule::set_manager(Origin::signed(4), 2));
		assert_eq!(Sudo::key(), Some(2));
	})
}

#[test]
fn test_genesis_config() {
	new_test_ext_with_genesis(4).execute_with(|| {
		assert_eq!(RoleModule::total_members(), 2);
		assert_eq!(RoleModule::rep_num(), 2);
		assert!(RoleModule::reps(HENRY).is_some());
		assert!(RoleModule::reps(GABRIEL).is_some());
	})
}
