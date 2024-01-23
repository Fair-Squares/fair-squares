use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};
pub use super::*;



#[test]
fn setting_roles(){
	new_test_ext().execute_with(||{

		let inv_list = vec![ALICE,DAVE,BOB,CHARLIE,EVE];
		//let selected = BiddingModule::choose_investor(inv_list);
		//println!("selected is {:?}",selected);


		

		
	})
}
