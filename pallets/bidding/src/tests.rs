use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};
pub use super::*;



#[test]
fn setting_roles(){
	new_test_ext().execute_with(||{

		let inv_list = vec![ALICE,DAVE,BOB,CHARLIE,EVE];
		let rand_list = BiddingModule::generate_random_number(inv_list);
		println!("selected indexes: {:?}",rand_list);



		

		
	})
}
