use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};

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
fn create_proposal() {
	ExtBuilder::default().build().execute_with(|| {
		let metadata0: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata0".to_vec().try_into().unwrap();
		let metadata1: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata1".to_vec().try_into().unwrap();
		prep_roles();
		//Charlie creates a collection
		assert_ok!(NftModule::create_collection(
			Origin::signed(CHARLIE),
			NftColl::OFFICESTEST,
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

		let coll_id = NftColl::OFFICESTEST.value();
		let item_id = pallet_nft::ItemsCount::<Test>::get()[coll_id as usize] - 1;
		let status: AssetStatus =
			Houses::<Test>::get(coll_id.clone(), item_id.clone()).unwrap().status;

		expect_events(vec![
			crate::Event::ProposalCreated {
				who: BOB,
				collection: coll_id.clone(),
				item: item_id.clone(),
				price: Some(price.clone()),
			}
			.into(),
			crate::Event::FundsReserved { from_who: BOB, amount: Some(5_000_000) }.into(),
		]);

		assert_eq!(status, AssetStatus::EDITING);

		// Bob changes the price of created proposal
		let new_price = 150_000_000;
		assert_ok!(OnboardingModule::set_price(
			Origin::signed(BOB),
			NftColl::OFFICESTEST,
			item_id.clone(),
			Some(new_price.clone())
		));

		expect_events(vec![crate::Event::TokenPriceUpdated {
			who: BOB,
			collection: coll_id.clone(),
			item: item_id.clone(),
			price: Some(new_price.clone()),
		}
		.into()]);

		let house_price = Houses::<Test>::get(coll_id.clone(), item_id.clone()).unwrap().price;
		assert_eq!(
			new_price.clone(),
			Prices::<Test>::get(coll_id.clone(), item_id.clone()).unwrap()
		);
		assert_eq!(house_price, Prices::<Test>::get(coll_id.clone(), item_id.clone()));

		//Bob finally submit the proposal without changing the price a second time
		assert_ok!(OnboardingModule::submit_awaiting(
			Origin::signed(BOB),
			NftColl::OFFICESTEST,
			item_id,
			None
		));

		let house_price = Houses::<Test>::get(coll_id.clone(), item_id.clone()).unwrap().price;
		assert_eq!(house_price, Some(150_000_000));
		let status: AssetStatus =
			Houses::<Test>::get(coll_id.clone(), item_id.clone()).unwrap().status;
		assert_eq!(status, AssetStatus::REVIEWING);
		
	});
}

#[test]
fn create_proposal_2() {
	ExtBuilder::default().build().execute_with(|| {
		let metadata0: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata0".to_vec().try_into().unwrap();
		let metadata1: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata1".to_vec().try_into().unwrap();
		prep_roles();
		//Charlie creates a collection
		assert_ok!(NftModule::create_collection(
			Origin::signed(CHARLIE),
			NftColl::OFFICESTEST,
			metadata0
		));
		// Bob creates a proposal and submit it for review
		let price = 100_000_000;
		assert_ok!(OnboardingModule::create_and_submit_proposal(
			Origin::signed(BOB),
			NftColl::OFFICESTEST,
			Some(price.clone()),
			metadata1,
			true
		));

		let coll_id = NftColl::OFFICESTEST.value();
		let item_id = pallet_nft::ItemsCount::<Test>::get()[coll_id as usize] - 1;
		let status: AssetStatus =
			Houses::<Test>::get(coll_id.clone(), item_id.clone()).unwrap().status;

		expect_events(vec![
			crate::Event::ProposalCreated {
				who: BOB,
				collection: coll_id.clone(),
				item: item_id.clone(),
				price: Some(price.clone()),
			}
			.into(),
			crate::Event::FundsReserved { from_who: BOB, amount: Some(5_000_000) }.into(),
		]);

		assert_eq!(status, AssetStatus::REVIEWING);

		//Change House status to FINALISED
	//	Houses::<Test>::mutate(coll_id.clone(),item_id.clone(),|val|{
	//		let mut v0 = val.clone().unwrap();
	//		v0.status = AssetStatus::FINALISED;
	//	})

		
	//	let status: AssetStatus =
	//		Houses::<Test>::get(coll_id.clone(), item_id.clone()).unwrap().status;
	//	assert_eq!(status, AssetStatus::PURCHASED);
		
	});
}


#[test]
fn proposal_rejections() {
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
			metadata0
		));

		// Bob creates 2 proposals and submit them for review
		let price0 = 100_000_000;
		let price1 = 150_000_000;
		assert_ok!(OnboardingModule::create_and_submit_proposal(
			Origin::signed(BOB),
			NftColl::OFFICESTEST,
			Some(price0),
			metadata1,
			true
		));
		let coll_id = NftColl::OFFICESTEST.value();
		let item_id0 = pallet_nft::ItemsCount::<Test>::get()[coll_id as usize] - 1;
		let status_0: AssetStatus =
			Houses::<Test>::get(coll_id.clone(), item_id0.clone()).unwrap().status;
		assert_eq!(status_0, AssetStatus::REVIEWING);
		let initial_balance = <Test as pallet_uniques::Config>::Currency::free_balance(&BOB);
		let fees_balance0 = <Test as pallet_uniques::Config>::Currency::total_balance(
			&OnboardingModule::account_id(),
		);

		assert_ok!(OnboardingModule::create_and_submit_proposal(
			Origin::signed(BOB),
			NftColl::OFFICESTEST,
			Some(price1),
			metadata2,
			true
		));
		let item_id1 = pallet_nft::ItemsCount::<Test>::get()[coll_id as usize] - 1;
		let status_1: AssetStatus =
			Houses::<Test>::get(coll_id.clone(), item_id0.clone()).unwrap().status;
		let balance0 = <Test as pallet_uniques::Config>::Currency::free_balance(&BOB);

		assert_eq!(status_1, AssetStatus::REVIEWING);

		//Chalie Reject_Edit first proposal
		let house0 = Houses::<Test>::get(coll_id.clone(), item_id0.clone()).unwrap();
		assert_ok!(OnboardingModule::reject_edit(
			Origin::signed(CHARLIE),
			NftColl::OFFICESTEST,
			item_id0.clone(),
			house0
		));

		expect_events(vec![crate::Event::RejectedForEditing {
			by_who: CHARLIE,
			collection: coll_id.clone(),
			item: item_id0.clone(),
		}
		.into()]);

		let status0: AssetStatus =
			Houses::<Test>::get(coll_id.clone(), item_id0.clone()).unwrap().status;
		assert_eq!(status0, AssetStatus::REJECTED);

		let fees_balance1 = <Test as pallet_uniques::Config>::Currency::total_balance(
			&OnboardingModule::account_id(),
		);
		assert_ne!(fees_balance1, fees_balance0);

		//Charlie Reject_Destroy second proposal
		let house1 = Houses::<Test>::get(coll_id.clone(), item_id1.clone()).unwrap();
		assert_ok!(OnboardingModule::reject_destroy(
			Origin::signed(CHARLIE),
			NftColl::OFFICESTEST,
			item_id1.clone(),
			house1
		));

		expect_events(vec![crate::Event::RejectedForDestruction {
			by_who: CHARLIE,
			collection: coll_id.clone(),
			item: item_id1.clone(),
		}
		.into()]);

		// Bob reserved funds are 100% slashed
		let diff = initial_balance - balance0;
		let res0 = OnboardingModule::balance_to_u64_option(price1).unwrap();
		let perc = 5;
		let res1 = perc * res0 / 100;
		let reserved = OnboardingModule::u64_to_balance_option(res1).unwrap();
		assert_eq!(diff, reserved);
		let fees_balance2 = <Test as pallet_uniques::Config>::Currency::total_balance(
			&OnboardingModule::account_id(),
		);
		assert_ne!(fees_balance1, fees_balance2);

		let status1: AssetStatus =
			Houses::<Test>::get(coll_id.clone(), item_id1.clone()).unwrap().status;
		assert_eq!(status1, AssetStatus::SLASH);
	});
}

#[test]
fn get_onboarded_houses_no_onboarded_houses() {
	ExtBuilder::default().build().execute_with(|| {
		let metadata0: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata0".to_vec().try_into().unwrap();
		let metadata1: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata1".to_vec().try_into().unwrap();

		prep_roles();

		//Charlie creates a collection
		assert_ok!(NftModule::create_collection(
			Origin::signed(CHARLIE),
			NftColl::OFFICESTEST,
			metadata0
		));
		// Bob creates a proposal without submiting for review
		assert_ok!(OnboardingModule::create_and_submit_proposal(
			Origin::signed(BOB),
			NftColl::OFFICESTEST,
			Some(100_000_000),
			metadata1,
			false
		));

		let onboarded_houses = OnboardingModule::get_onboarded_houses();
		assert_eq!(
			onboarded_houses.len(),
			0
		);
	});
}

#[test]
fn get_onboarded_houses_with_onboarded_houses() {
	ExtBuilder::default().build().execute_with(|| {
		let metadata0: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata0".to_vec().try_into().unwrap();
		let metadata1: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata1".to_vec().try_into().unwrap();
		let metadata2: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata1".to_vec().try_into().unwrap();
		prep_roles();
		//Charlie creates a collection
		assert_ok!(NftModule::create_collection(
			Origin::signed(CHARLIE),
			NftColl::OFFICESTEST,
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

		let collection_id = NftColl::OFFICESTEST.value();
		let item_id = pallet_nft::ItemsCount::<Test>::get()[collection_id as usize] - 1;

		// we simulate for the the presence of an onboarded house by changing its status
		assert_ok!(OnboardingModule::change_status(
			Origin::signed(BOB),
			NftColl::OFFICESTEST,
			item_id,
			AssetStatus::ONBOARDED,
		));

		let price2 = 200_000_000;
		// we add a new asset that won't have the ONBOARDED status
		assert_ok!(OnboardingModule::create_and_submit_proposal(
			Origin::signed(BOB),
			NftColl::OFFICESTEST,
			Some(price2.clone()),
			metadata2,
			false
		));

		// we check that the onboarded house is correctly retrieved
		let onboarded_houses = OnboardingModule::get_onboarded_houses();
		assert_eq!(
			onboarded_houses.len(),
			1
		);

		let house = onboarded_houses[0].clone();
		assert_eq!(
			house.2.status,
			AssetStatus::ONBOARDED,
		);
		assert_eq!(
			house.2.price,
			Some(price),
		);
	});
}

