pub use super::*;
pub use crate::mock::*;
pub use frame_support::{assert_noop, assert_ok};
use frame_system::pallet_prelude::OriginFor;

pub fn prep_roles() {
	RoleModule::set_role(Origin::signed(CHARLIE).clone(), CHARLIE, Acc::SERVICER).ok();
	RoleModule::account_approval(Origin::signed(ALICE), CHARLIE).ok();
	RoleModule::set_role(Origin::signed(EVE).clone(), EVE, Acc::SERVICER).ok();
	RoleModule::account_approval(Origin::signed(ALICE), EVE).ok();
	RoleModule::set_role(Origin::signed(BOB).clone(), BOB, Acc::SELLER).ok();
	RoleModule::account_approval(Origin::signed(ALICE), BOB).ok();
	RoleModule::set_role(Origin::signed(DAVE).clone(), DAVE, Acc::INVESTOR).ok();
	RoleModule::set_role(
		Origin::signed(ACCOUNT_WITH_NO_BALANCE0).clone(),
		ACCOUNT_WITH_NO_BALANCE0,
		Acc::SERVICER,
	)
	.ok();
	RoleModule::account_approval(Origin::signed(ALICE), ACCOUNT_WITH_NO_BALANCE0).ok();
}

#[test]
fn virtual0(){
	ExtBuilder::default().build().execute_with(|| {
		let metadata0: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
		b"metadata0".to_vec().try_into().unwrap();
	let metadata1: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
		b"metadata1".to_vec().try_into().unwrap();
	let metadata2: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
		b"metadata2".to_vec().try_into().unwrap();
	prep_roles();
	//Charlie creates a collection
	assert_ok!(NftModule::create_collection(
		Origin::signed(CHARLIE),
		NftColl::OFFICESTEST,
		metadata0.clone()
	));
	//Charlie creates a second collection
	assert_ok!(NftModule::create_collection(
		Origin::signed(CHARLIE),
		NftColl::APPARTMENTSTEST,
		metadata0
	));
	// Bob creates a proposal without submiting for review
	let price = 100_000_000;
	assert_ok!(OnboardingModule::create_and_submit_proposal(
		Origin::signed(BOB),
		NftColl::OFFICESTEST,
		Some(price.clone()),
		metadata1,
		false
	));

	

		let coll_id0 = NftColl::OFFICESTEST.value();
		let item_id0 = pallet_nft::ItemsCount::<Test>::get()[coll_id0 as usize] - 1;
		let origin: OriginFor<Test> = frame_system::RawOrigin::Root.into();

		assert_ok!(ShareDistributor::create_virtual(origin.clone(),coll_id0,item_id0));

		// Bob creates a second proposal without submiting for review
	let price = 100_000_000;
	assert_ok!(OnboardingModule::create_and_submit_proposal(
		Origin::signed(BOB),
		NftColl::APPARTMENTSTEST,
		Some(price.clone()),
		metadata2,
		false
	));
		let coll_id1 = NftColl::APPARTMENTSTEST.value();
		let item_id1 = pallet_nft::ItemsCount::<Test>::get()[coll_id1 as usize] - 1;
		

		assert_ok!(ShareDistributor::create_virtual(origin,coll_id1,item_id1));
		let virtual0 = Virtual::<Test>::get(coll_id0,item_id0).unwrap();
		
		let virtual1 = Virtual::<Test>::get(coll_id1,item_id1).unwrap();
		println!("Virtual account nbr1:{:?}\nVirtual account nbr2:{:?}",virtual0,virtual1);
		assert_ne!(virtual0.virtual_account,virtual1.virtual_account);


	});
}