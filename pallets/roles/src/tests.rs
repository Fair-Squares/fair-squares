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

		assert_eq!(RolesModule::get_pending_servicers().len(), 0);
		assert_eq!(RolesModule::get_pending_house_sellers().len(), 0);
		assert_eq!(RolesModule::get_pending_notaries().len(), 0);
		let council = Collective::members();
		assert_eq!(council.len(),3);


		//Investor & Tenant roles
		assert_ok!(RolesModule::set_role(RuntimeOrigin::signed(DAVE), DAVE, Acc::INVESTOR));
		assert_ok!(RolesModule::set_role(RuntimeOrigin::signed(EVE), EVE, Acc::TENANT));
		assert!(InvestorLog::<Test>::contains_key(DAVE));
		assert!(TenantLog::<Test>::contains_key(EVE));

		//Seller,Servicer, and Notary roles
		/*assert_ok!(RolesModule::set_role(RuntimeOrigin::signed(BOB),BOB,Acc::SELLER));
		assert_eq!(RolesModule::get_pending_house_sellers().len(),1);
		let account =RolesModule::get_pending_house_sellers()[0].account_id.clone();
		assert_eq!(account,BOB);*/

		
		//Check that collective referendum started
		//assert_eq!(Collective::proposal_count(),1); 
		//assert_eq!(Collective::proposals().len(),1);
		

		/*assert_ok!(RolesModule::council_vote(RuntimeOrigin::signed(council[0].clone()),BOB,true));
		assert_ok!(RolesModule::council_vote(RuntimeOrigin::signed(council[1].clone()),BOB,true));
		assert_ok!(RolesModule::council_vote(RuntimeOrigin::signed(council[2].clone()),BOB,true));
		assert_ok!(RolesModule::council_close(RuntimeOrigin::signed(council[2].clone()),BOB));
		let initial_block_number = System::block_number();
		let end_block_number = initial_block_number.saturating_add(<Test as crate::Config>::CheckPeriod::get());
		fast_forward_to(end_block_number);

		assert_eq!(RolesModule::get_pending_house_sellers().len(), 0);
		assert_eq!(!RolesModule::get_requested_role(BOB).is_some(),true);*/


		
	})
}
