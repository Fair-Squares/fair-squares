use crate::{mock::*};
use super::*;
use frame_support::{assert_noop, assert_ok};

pub fn prep_roles(){
    RoleModule::set_role(Origin::signed(CHARLIE).clone(), Acc::SERVICER).ok();
    RoleModule::account_approval(Origin::signed(ALICE),CHARLIE).ok();
    RoleModule::set_role(Origin::signed(EVE).clone(), Acc::SERVICER).ok();
    RoleModule::account_approval(Origin::signed(ALICE),EVE).ok();
    RoleModule::set_role(Origin::signed(BOB).clone(), Acc::SELLER).ok();
    RoleModule::account_approval(Origin::signed(ALICE),BOB).ok();
    RoleModule::set_role(Origin::signed(DAVE).clone(), Acc::INVESTOR).ok();
    RoleModule::set_role(Origin::signed(ACCOUNT_WITH_NO_BALANCE0).clone(), Acc::SERVICER).ok();
    RoleModule::account_approval(Origin::signed(ALICE),ACCOUNT_WITH_NO_BALANCE0).ok();

}


#[test]
fn create_proposal() {
	ExtBuilder::default().build().execute_with(|| {
		let metadata0: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
            b"metadata0".to_vec().try_into().unwrap();
			let metadata1: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
            b"metadata1".to_vec().try_into().unwrap();
        prep_roles();
		//Charlie creates a collection
		assert_ok!(NftModule::create_collection(Origin::signed(CHARLIE),NftColl::OFFICESTEST,metadata0));
		// Bob creates a proposal without submiting for review
		let price =  100_000_000;
		assert_ok!(OnboardingModule::create_and_submit_proposal(Origin::signed(BOB), NftColl::OFFICESTEST,Some(price),metadata1,false));
        let coll_id = NftColl::OFFICESTEST.value();
        let item_id = pallet_nft::ItemsCount::<Test>::get()[coll_id as usize] -1;
        let status: AssetStatus = Houses::<Test>::get(coll_id.clone(),item_id.clone()).unwrap().status;
        assert_eq!( status, AssetStatus::EDITING);

        // Bob changes the price of created proposal
        let new_price =  150_000_000;        
        assert_ok!(OnboardingModule::set_price(Origin::signed(BOB),NftColl::OFFICESTEST,item_id.clone(),Some(new_price)));
        let house_price = Houses::<Test>::get(coll_id.clone(),item_id.clone()).unwrap().price;
        assert_eq!(new_price,Prices::<Test>::get(coll_id.clone(),item_id.clone()).unwrap());
        assert_eq!(house_price,Prices::<Test>::get(coll_id.clone(),item_id.clone()));

        //Bob finally submit the proposal without changing the price a second time
        assert_ok!(OnboardingModule::submit_awaiting(Origin::signed(BOB),NftColl::OFFICESTEST,item_id,None));
        let house_price = Houses::<Test>::get(coll_id.clone(),item_id.clone()).unwrap().price;
        assert_eq!(house_price, Some(150_000_000));
        let status: AssetStatus = Houses::<Test>::get(coll_id.clone(),item_id.clone()).unwrap().status;
        assert_eq!( status, AssetStatus::REVIEWING);

		
	});
}
#[test]
fn proposal_rejections(){
    ExtBuilder::default().build().execute_with(|| {
        let metadata0: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
            b"metadata0".to_vec().try_into().unwrap();
			let metadata1: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
            b"metadata1".to_vec().try_into().unwrap();
            let metadata2: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
            b"metadata2".to_vec().try_into().unwrap();
        prep_roles();
		//Charlie creates a collection
		assert_ok!(NftModule::create_collection(Origin::signed(CHARLIE),NftColl::OFFICESTEST,metadata0));
		// Bob creates 2 proposals and submit them for review
		let price0 =  100_000_000;
        let price1 =  150_000_000;
		assert_ok!(OnboardingModule::create_and_submit_proposal(Origin::signed(BOB), NftColl::OFFICESTEST,Some(price0),metadata1,true));
        let coll_id = NftColl::OFFICESTEST.value();
        let item_id0 = pallet_nft::ItemsCount::<Test>::get()[coll_id as usize] -1;
        let status_0: AssetStatus = Houses::<Test>::get(coll_id.clone(),item_id0.clone()).unwrap().status;
        assert_eq!( status_0, AssetStatus::REVIEWING);

        assert_ok!(OnboardingModule::create_and_submit_proposal(Origin::signed(BOB), NftColl::OFFICESTEST,Some(price1),metadata2,true));
        let item_id1 = pallet_nft::ItemsCount::<Test>::get()[coll_id as usize] -1;
        let status_1: AssetStatus = Houses::<Test>::get(coll_id.clone(),item_id0.clone()).unwrap().status;
        assert_eq!( status_1, AssetStatus::REVIEWING);
        
        //Chalie Reject_Edit first proposal
        let house0 = Houses::<Test>::get(coll_id.clone(),item_id0.clone()).unwrap();
        assert_ok!(OnboardingModule::reject_edit(Origin::signed(CHARLIE),NftColl::OFFICESTEST,item_id0.clone(),house0));
        let status0: AssetStatus = Houses::<Test>::get(coll_id.clone(),item_id0.clone()).unwrap().status;
        assert_eq!( status0, AssetStatus::REJECTEDIT);

        //Charlie Reject_Destroy second proposal
        let house1 = Houses::<Test>::get(coll_id.clone(),item_id1.clone()).unwrap();
        assert_ok!(OnboardingModule::reject_destroy(Origin::signed(CHARLIE),NftColl::OFFICESTEST,item_id1.clone(),house1));
        let status1: AssetStatus = Houses::<Test>::get(coll_id.clone(),item_id1.clone()).unwrap().status;
        assert_eq!( status1, AssetStatus::REJECTBURN);

    });

}

