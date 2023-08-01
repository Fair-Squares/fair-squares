use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};


#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		assert_ok!(RolesModule::do_something(RuntimeOrigin::signed(1), 42));
		// Read pallet storage and assert an expected result.
		assert_eq!(RolesModule::something(), Some(42));
		// Assert that the correct event was deposited
		System::assert_last_event(Event::SomethingStored { something: 42, who: 1 }.into());
	});
}

#[test]
fn correct_error_for_none_value() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when no value is present.
		assert_noop!(
			RolesModule::cause_error(RuntimeOrigin::signed(1)),
			Error::<Test>::NoneValue
		);
	});
}

#[test]
fn set_investor_role(){
	new_test_ext().execute_with(||{

		assert_eq!(RolesModule::get_pending_servicers().len(), 0);
		assert_eq!(RolesModule::get_pending_house_sellers().len(), 0);
		assert_eq!(RolesModule::get_pending_notaries().len(), 0);
		 
	})
}
