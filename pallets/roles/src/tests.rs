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
			RoleModule::get_pending_approvals(),
			(
				vec![HouseSeller {
					account_id: 1,
					age: System::block_number(),
					activated: false,
					verifier: 4
				}],
				vec![]
			)
		);
		//---house seller should fail successfully----
		assert_ne!(RoleModule::get_pending_approvals(), (vec![], vec![])); //assert_ne! is not supported at the moment, as this expression should panick

		//-------tenant-----------
		assert_ok!(Tenant::<Test>::new(Origin::signed(1)));
		//-- checking Tenant storage------
		assert_eq!(
			RoleModule::tenants(1),
			Some(Tenant { account_id: 1, rent: 0, age: System::block_number() })
		);

		//-----Servicer-----------------------------------------
		assert_ok!(Servicer::<Test>::new(Origin::signed(2)));
		//--checking storage-------------
		assert_eq!(
			RoleModule::get_pending_approvals(),
			(
				vec![HouseSeller {
					account_id: 1,
					age: System::block_number(),
					activated: false,
					verifier: 4
				}],
				vec![Servicer {
					account_id: 2,
					age: System::block_number(),
					activated: false,
					verifier: 4
				}]
			)
		)
	});
}

#[test]
fn test_account_approval_rejection() {
	new_test_ext(4).execute_with(|| {
		//----testing account approval-----
		let master = Origin::signed(4);
		let wait0 = RoleModule::get_pending_approvals();
		let serv0 = wait0.1;
		let sell0 = wait0.0;
		assert_eq!(serv0.len(), 0);
		assert_eq!(sell0.len(), 0);

		assert_ok!(Servicer::<Test>::new(Origin::signed(2)));
		assert_ok!(HouseSeller::<Test>::new(Origin::signed(3)));
		assert_ok!(Servicer::<Test>::new(Origin::signed(5)));
		assert_ok!(HouseSeller::<Test>::new(Origin::signed(6)));

		let wait1 = RoleModule::get_pending_approvals();
		let serv1 = wait1.1;
		let sell1 = wait1.0;
		assert_eq!(serv1.len(), 2);
		assert_eq!(sell1.len(), 2);
		assert_eq!(serv1[0].activated, false);
		assert_eq!(serv1[1].activated, false);
		assert_eq!(serv1[0].verifier, 4);
		assert_eq!(serv1[1].verifier, 4);
		assert_eq!(sell1[0].activated, false);
		assert_eq!(sell1[1].activated, false);
		assert_eq!(sell1[0].verifier, 4);
		assert_eq!(sell1[1].verifier, 4);

		assert_ok!(RoleModule::account_approval(master.clone(), 2));
		assert_ok!(RoleModule::account_approval(master.clone(), 3));
		assert_ok!(RoleModule::account_rejection(master.clone(), 5));
		assert_ok!(RoleModule::account_rejection(master, 6));

		let wait2 = RoleModule::get_pending_approvals();
		let serv2 = wait2.1;
		let sell2 = wait2.0;
		assert_eq!(serv2.len(), 0);
		assert_eq!(sell2.len(), 0);
		assert!(ServicerLog::<Test>::contains_key(2));
		assert_eq!(RoleModule::servicers(2).unwrap().activated, true);
		assert_eq!(RoleModule::servicers(2).unwrap().verifier, 4);
		assert!(!ServicerLog::<Test>::contains_key(5));
		assert!(HouseSellerLog::<Test>::contains_key(3));
		assert_eq!(RoleModule::sellers(3).unwrap().activated, true);
		assert_eq!(RoleModule::sellers(3).unwrap().verifier, 4);
		assert!(!HouseSellerLog::<Test>::contains_key(6));
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

		let wait_sell = RoleModule::get_pending_approvals().0;
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
		let wait_sell = RoleModule::get_pending_approvals().0;
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
fn test_set_manager() {
	new_test_ext(4).execute_with(|| {
		//-----checking existing manager--------
		assert_eq!(Sudo::key(), Some(4));
		//---changing--------------------------

		assert_ok!(RoleModule::set_manager(Origin::signed(4), 2));
		assert_eq!(Sudo::key(), Some(2));
	})
}
