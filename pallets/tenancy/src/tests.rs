pub use super::*;
pub use frame_support::{assert_err, assert_ok};
use frame_system::pallet_prelude::OriginFor;
use mock::*;

pub type Bvec<Test> = BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit>;

pub fn prep_roles() {
	RoleModule::set_role(Origin::signed(CHARLIE), CHARLIE, Acc::SERVICER).ok();
	RoleModule::account_approval(Origin::signed(ALICE), CHARLIE).ok();
	RoleModule::set_role(Origin::signed(BOB), BOB, Acc::SELLER).ok();
	RoleModule::account_approval(Origin::signed(ALICE), BOB).ok();
	RoleModule::set_role(Origin::signed(DAVE), DAVE, Acc::INVESTOR).ok();
	RoleModule::set_role(Origin::signed(EVE), EVE, Acc::INVESTOR).ok();
	RoleModule::set_role(Origin::signed(FERDIE), FERDIE, Acc::REPRESENTATIVE).ok(); //FERDIE approval will be tested
	RoleModule::set_role(Origin::signed(GERARD), GERARD, Acc::TENANT).ok();
	RoleModule::set_role(Origin::signed(HUNTER), HUNTER, Acc::TENANT).ok();
}

fn next_block() {
	System::set_block_number(System::block_number() + 1);
	Scheduler::on_initialize(System::block_number());
	Democracy::on_initialize(System::block_number());
	AssetManagement::begin_block(System::block_number());
}

fn fast_forward_to(n: u64) {
	while System::block_number() < n {
		next_block();
	}
}

