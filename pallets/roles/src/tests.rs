use super::*;
use crate::{
	mock::*,
	Error,
};
use frame_support::{assert_noop, assert_ok};

#[test]
fn test_struct_methods() {
	new_test_ext(4).execute_with(|| {
		assert_ok!(Investor::<Test>::new(Origin::signed(1)));
		//--checking investor storage if its updated----
		assert!(InvestorLog::<Test>::contains_key(1));
		assert_eq!(
			InvestorLog::<Test>::get(1),
			Some(Investor {
				account_id: 1,
				nft_index: Vec::new(),
				age: System::block_number(),
				share: 0,
				selections: 0,
			})
		);

		//---HouseSeller-------
		assert_ok!(HouseSeller::<Test>::new(Origin::signed(1)));
		assert_eq!(
			WaitingList::<Test>::get(),
			(
				vec![HouseSeller {
					account_id: 1,
					nft_index: Vec::new(),
					age: System::block_number(),
				}],
				vec![]
			)
		);
		//---house seller should fail successfully----
		assert_ne!(WaitingList::<Test>::get(), (vec![], vec![])); //assert_ne! is not supported at the moment, as this expression should panick

		//-------tenant-----------
		assert_ok!(Tenant::<Test>::new(Origin::signed(1)));
		//-- checking Tenant storage------
		assert_eq!(
			TenantLog::<Test>::get(1),
			Some(Tenant { account_id: 1, rent: 0, age: System::block_number() })
		);

		//-----Servicer-----------------------------------------
		assert_ok!(Servicer::<Test>::new(Origin::signed(2)));
		//--checking storage-------------
		assert_eq!(
			WaitingList::<Test>::get(),
			(
				vec![HouseSeller {
					account_id: 1,
					nft_index: Vec::new(),
					age: System::block_number(),
				}],
				vec![Servicer { account_id: 2, age: System::block_number() }]
			)
		)
	});
}

#[test]
fn test_account_approval_rejection() {
	new_test_ext(4).execute_with(|| {
		//----testing account approval-----
		let master = Origin::signed(4);
		let wait0 = WaitingList::<Test>::get();
		let serv0 = wait0.1;
		let sell0 = wait0.0;
		assert_eq!(serv0.len(), 0);
		assert_eq!(sell0.len(), 0);

		assert_ok!(Servicer::<Test>::new(Origin::signed(2)));
		assert_ok!(HouseSeller::<Test>::new(Origin::signed(3)));
		assert_ok!(Servicer::<Test>::new(Origin::signed(5)));
		let wait1 = WaitingList::<Test>::get();
		let serv1 = wait1.1;
		let sell1 = wait1.0;
		assert_eq!(serv1.len(), 2);
		assert_eq!(sell1.len(), 1);

		assert_ok!(RoleModule::account_approval(master.clone(),2));
		assert_ok!(RoleModule::account_approval(master.clone(),3));
		assert_ok!(RoleModule::account_rejection(master,5));
		
		let wait2 = WaitingList::<Test>::get();
		let serv2 = wait2.1;
		let sell2 = wait2.0;
		assert_eq!(serv2.len(), 0);
		assert_eq!(sell2.len(), 0);
		assert!(ServicerLog::<Test>::contains_key(2));
		assert!(!ServicerLog::<Test>::contains_key(5));
		assert!(HouseSellerLog::<Test>::contains_key(3));
	})
}

#[test]
fn test_account_creation() {
	new_test_ext(4).execute_with(|| {
		let master = Origin::signed(4);
		let user1 = Origin::signed(1);
		let user2 = Origin::signed(2);
		let wait_sell = WaitingList::<Test>::get().0;
		let sell_len = wait_sell.len();

		assert_ok!(RoleModule::create_account(user1.clone(), Acc::INVESTOR));
		assert!(InvestorLog::<Test>::contains_key(1));
		assert_noop!(RoleModule::create_account(user1, Acc::TENANT), Error::<Test>::OneRoleAllowed);

		assert_ok!(RoleModule::create_account(user2.clone(), Acc::SELLER));
		assert_noop!(RoleModule::create_account(user2.clone(), Acc::SELLER),Error::<Test>::AlreadyWaiting);
		let wait_sell = WaitingList::<Test>::get().0;
		let sell_len2 = wait_sell.len();
		assert_eq!(sell_len2, sell_len + 1);
		assert_ok!(RoleModule::account_approval(master,2));
		assert!(HouseSellerLog::<Test>::contains_key(2));
	})
}
