pub use super::*;
pub use crate::mock::*;
pub use frame_support::{assert_noop, assert_ok};
use frame_system::pallet_prelude::OriginFor;

pub fn prep_roles() {
	RoleModule::set_role(Origin::signed(CHARLIE).clone(), CHARLIE, Acc::SERVICER).ok();
	RoleModule::account_approval(Origin::signed(ALICE), CHARLIE).ok();
	RoleModule::set_role(Origin::signed(BOB).clone(), BOB, Acc::SELLER).ok();
	RoleModule::account_approval(Origin::signed(ALICE), BOB).ok();
	RoleModule::set_role(Origin::signed(DAVE).clone(), DAVE, Acc::INVESTOR).ok();
	RoleModule::set_role(Origin::signed(EVE).clone(), EVE, Acc::INVESTOR).ok();
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

	//Dave and EVE contribute to the fund
	assert_ok!(HousingFund::Pallet::<Test>::contribute_to_fund(Origin::signed(DAVE),50_000));
	assert_ok!(HousingFund::Pallet::<Test>::contribute_to_fund(Origin::signed(EVE),50_000));

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
	let price = 40_000;
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
		let origin2 = Origin::signed(BOB);

		//Change first asset status to FINALISED
		Onboarding::Pallet::<Test>::change_status(origin2.clone(),NftColl::OFFICESTEST,item_id0.clone(),Onboarding::AssetStatus::FINALISED).ok();		
		//Store initial owner
		let old_owner0 = pallet_nft::Pallet::<Test>::owner(coll_id0.clone(),item_id0.clone()).unwrap();
		//Execute virtual account transactions 
		assert_ok!(ShareDistributor::create_virtual(origin.clone(),coll_id0.clone(),item_id0.clone()));
		//Store new owner
		let new_owner0 = pallet_nft::Pallet::<Test>::owner(coll_id0.clone(),item_id0.clone()).unwrap();
		//Compare new & old owner
		assert_ne!(old_owner0.clone(),new_owner0.clone());
		
		// Bob creates a second proposal without submiting for review
	let price = 30_000;
	
	assert_ok!(OnboardingModule::create_and_submit_proposal(
		Origin::signed(BOB),
		NftColl::APPARTMENTSTEST,
		Some(price.clone()),
		metadata2,
		false
	));

	
		let coll_id1 = NftColl::APPARTMENTSTEST.value();
		let item_id1 = pallet_nft::ItemsCount::<Test>::get()[coll_id1 as usize] - 1;

		//Store initial owner
		let old_owner1 = pallet_nft::Pallet::<Test>::owner(coll_id1.clone(),item_id1.clone()).unwrap();

		//Change first asset status to FINALISED
		Onboarding::Pallet::<Test>::change_status(origin2.clone(),NftColl::APPARTMENTSTEST,item_id1.clone(),Onboarding::AssetStatus::FINALISED).ok();
		//Execute virtual account transactions 
		assert_ok!(ShareDistributor::create_virtual(origin,coll_id1,item_id1));
		//Store new owner
		let new_owner1 = pallet_nft::Pallet::<Test>::owner(coll_id1.clone(),item_id1.clone()).unwrap();
		//Compare new & old owner
		assert_ne!(old_owner1.clone(),new_owner1.clone());

		//Get the virtual accounts
		let virtual0 = Virtual::<Test>::get(coll_id0,item_id0).unwrap();		
		let virtual1 = Virtual::<Test>::get(coll_id1,item_id1).unwrap();

		//Check that virtual accounts are different
		println!("Virtual account nbr1:{:?}\nVirtual account nbr2:{:?}",virtual0,virtual1);
		assert_ne!(virtual0.virtual_account,virtual1.virtual_account);
		
		//Check that virtual accounts are the new owners
		assert_eq!(new_owner0,virtual0.clone().virtual_account);
		assert_eq!(new_owner1,virtual1.clone().virtual_account);


	});
}