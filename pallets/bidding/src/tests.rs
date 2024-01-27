use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};
pub use super::*;

fn next_block() {
	System::set_block_number(System::block_number() + 1);
	RolesModule::on_initialize(System::block_number());
	
}

fn fast_forward_to(n: u64) {
	while System::block_number() < n {
		next_block();
	}
}

#[test]
fn setting_roles(){
	new_test_ext().execute_with(||{

		//let inv_list = vec![ALICE,DAVE,BOB,CHARLIE,EVE];
		assert_eq!(RolesModule::get_pending_servicers().len(), 0);
		assert_eq!(RolesModule::get_pending_house_sellers().len(), 0);
		assert_eq!(RolesModule::get_pending_notaries().len(), 0);
		let council = Collective::members();
		assert_eq!(council.len(),3);

		//Investor & Tenant roles
		assert_ok!(RolesModule::set_role(RuntimeOrigin::signed(DAVE), DAVE, Acc::INVESTOR));
		assert_ok!(RolesModule::set_role(RuntimeOrigin::signed(EVE), EVE, Acc::TENANT));
		assert!(pallet_roles::InvestorLog::<Test>::contains_key(DAVE));
		assert!(pallet_roles::TenantLog::<Test>::contains_key(EVE));

		//Seller,Servicer, and Notary roles
		assert_ok!(RolesModule::set_role(RuntimeOrigin::signed(BOB),BOB,Acc::SELLER));
		assert_eq!(RolesModule::get_pending_house_sellers().len(),1);
		let account =RolesModule::get_pending_house_sellers()[0].account_id.clone();
		assert_eq!(account,BOB);

		//assert_ok!(RolesModule::council_vote(RuntimeOrigin::signed(council[1].clone()),BOB,true));


		

		
	})
}
