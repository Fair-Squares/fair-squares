use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, BoundedVec};

#[test]
fn validate_transaction_asset_no_notary_role_should_fail() {
	new_test_ext().execute_with(|| {
		assert_ok!(RoleModule::set_role(
			Origin::signed(KEZIA),
			KEZIA,
			crate::Onboarding::HousingFund::ROLES::Accounts::SERVICER
		));
		assert_ok!(RoleModule::account_approval(Origin::signed(ALICE), KEZIA));

		assert_ok!(RoleModule::set_role(
			Origin::signed(AMANI),
			AMANI,
			crate::Onboarding::HousingFund::ROLES::Accounts::SELLER
		));
		assert_ok!(RoleModule::account_approval(Origin::signed(ALICE), AMANI));

		let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata0".to_vec().try_into().unwrap();

		assert_ok!(NftModule::create_collection(
			Origin::signed(KEZIA),
			NftCollection::OFFICESTEST,
			metadata.clone()
		));

		assert_ok!(OnboardingModule::create_and_submit_proposal(
			Origin::signed(AMANI),
			NftCollection::OFFICESTEST,
			Some(100),
			metadata,
			false,
			3
		));

		let collection_id = NftCollection::OFFICESTEST.value();
		let item_id = pallet_nft::ItemsCount::<Test>::get()[collection_id as usize] - 1;

		assert_noop!(
			FinalizerModule::validate_transaction_asset(
				Origin::signed(AMANI),
				collection_id,
				item_id,
			),
			Error::<Test>::NotANotary
		);
	});
}

#[test]
fn validate_transaction_asset_no_existing_house_should_fail() {
	new_test_ext().execute_with(|| {
		assert_ok!(RoleModule::set_role(
			Origin::signed(DAN),
			DAN,
			crate::Onboarding::HousingFund::ROLES::Accounts::NOTARY
		));
		assert_ok!(RoleModule::account_approval(Origin::signed(ALICE), DAN));

		assert_ok!(RoleModule::set_role(
			Origin::signed(AMANI),
			AMANI,
			crate::Onboarding::HousingFund::ROLES::Accounts::SELLER
		));
		assert_ok!(RoleModule::account_approval(Origin::signed(ALICE), AMANI));

		let collection_id = NftCollection::OFFICESTEST.value();

		assert_noop!(
			FinalizerModule::validate_transaction_asset(Origin::signed(DAN), collection_id, 1,),
			Error::<Test>::AssetDoesNotExist
		);
	});
}

#[test]
fn validate_transaction_asset_no_finalising_status_should_fail() {
	new_test_ext().execute_with(|| {
		assert_ok!(RoleModule::set_role(
			Origin::signed(KEZIA),
			KEZIA,
			crate::Onboarding::HousingFund::ROLES::Accounts::SERVICER
		));
		assert_ok!(RoleModule::account_approval(Origin::signed(ALICE), KEZIA));

		assert_ok!(RoleModule::set_role(
			Origin::signed(DAN),
			DAN,
			crate::Onboarding::HousingFund::ROLES::Accounts::NOTARY
		));
		assert_ok!(RoleModule::account_approval(Origin::signed(ALICE), DAN));

		assert_ok!(RoleModule::set_role(
			Origin::signed(AMANI),
			AMANI,
			crate::Onboarding::HousingFund::ROLES::Accounts::SELLER
		));
		assert_ok!(RoleModule::account_approval(Origin::signed(ALICE), AMANI));

		let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata0".to_vec().try_into().unwrap();

		assert_ok!(NftModule::create_collection(
			Origin::signed(KEZIA),
			NftCollection::OFFICESTEST,
			metadata.clone()
		));

		assert_ok!(OnboardingModule::create_and_submit_proposal(
			Origin::signed(AMANI),
			NftCollection::OFFICESTEST,
			Some(100),
			metadata,
			false,
			3
		));

		let collection_id = NftCollection::OFFICESTEST.value();
		let item_id = pallet_nft::ItemsCount::<Test>::get()[collection_id as usize] - 1;

		assert_noop!(
			FinalizerModule::validate_transaction_asset(
				Origin::signed(DAN),
				collection_id,
				item_id,
			),
			Error::<Test>::HouseHasNotFinalisingStatus
		);
	});
}

#[test]
fn validate_transaction_asset_should_succeed() {
	new_test_ext().execute_with(|| {
		assert_ok!(RoleModule::set_role(
			Origin::signed(KEZIA),
			KEZIA,
			crate::Onboarding::HousingFund::ROLES::Accounts::SERVICER
		));
		assert_ok!(RoleModule::account_approval(Origin::signed(ALICE), KEZIA));

		assert_ok!(RoleModule::set_role(
			Origin::signed(DAN),
			DAN,
			crate::Onboarding::HousingFund::ROLES::Accounts::NOTARY
		));
		assert_ok!(RoleModule::account_approval(Origin::signed(ALICE), DAN));

		assert_ok!(RoleModule::set_role(
			Origin::signed(AMANI),
			AMANI,
			crate::Onboarding::HousingFund::ROLES::Accounts::SELLER
		));
		assert_ok!(RoleModule::account_approval(Origin::signed(ALICE), AMANI));

		let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata0".to_vec().try_into().unwrap();

		assert_ok!(NftModule::create_collection(
			Origin::signed(KEZIA),
			NftCollection::OFFICESTEST,
			metadata.clone()
		));

		assert_ok!(OnboardingModule::create_and_submit_proposal(
			Origin::signed(AMANI),
			NftCollection::OFFICESTEST,
			Some(100),
			metadata,
			false,
			3
		));

		let collection_id = NftCollection::OFFICESTEST.value();
		let item_id = pallet_nft::ItemsCount::<Test>::get()[collection_id as usize] - 1;

		assert_ok!(OnboardingModule::change_status(
			Origin::signed(AMANI),
			NftCollection::OFFICESTEST,
			item_id,
			crate::Onboarding::AssetStatus::FINALISING
		));

		assert_ok!(FinalizerModule::validate_transaction_asset(
			Origin::signed(DAN),
			collection_id,
			item_id,
		));

		let house = OnboardingModule::houses(collection_id, item_id).unwrap();
		assert_eq!(house.status, crate::Onboarding::AssetStatus::FINALISED);

		let event = <frame_system::Pallet<Test>>::events()
			.pop()
			.expect("Expected at least one EventRecord to be found")
			.event;

		// check that the event has been raised
		assert_eq!(
			event,
			mock::Event::FinalizerModule(crate::Event::NotaryValidatedAssetTransaction(
				DAN,
				collection_id,
				item_id
			))
		);
	});
}

#[test]
fn reject_transaction_asset_no_notary_role_should_fail() {
	new_test_ext().execute_with(|| {
		assert_ok!(RoleModule::set_role(
			Origin::signed(KEZIA),
			KEZIA,
			crate::Onboarding::HousingFund::ROLES::Accounts::SERVICER
		));
		assert_ok!(RoleModule::account_approval(Origin::signed(ALICE), KEZIA));

		assert_ok!(RoleModule::set_role(
			Origin::signed(AMANI),
			AMANI,
			crate::Onboarding::HousingFund::ROLES::Accounts::SELLER
		));
		assert_ok!(RoleModule::account_approval(Origin::signed(ALICE), AMANI));

		let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata0".to_vec().try_into().unwrap();

		assert_ok!(NftModule::create_collection(
			Origin::signed(KEZIA),
			NftCollection::OFFICESTEST,
			metadata.clone()
		));

		assert_ok!(OnboardingModule::create_and_submit_proposal(
			Origin::signed(AMANI),
			NftCollection::OFFICESTEST,
			Some(100),
			metadata,
			false,
			3
		));

		let collection_id = NftCollection::OFFICESTEST.value();
		let item_id = pallet_nft::ItemsCount::<Test>::get()[collection_id as usize] - 1;

		assert_noop!(
			FinalizerModule::reject_transaction_asset(
				Origin::signed(AMANI),
				collection_id,
				item_id,
			),
			Error::<Test>::NotANotary
		);
	});
}

#[test]
fn reject_transaction_asset_no_existing_house_should_fail() {
	new_test_ext().execute_with(|| {
		assert_ok!(RoleModule::set_role(
			Origin::signed(DAN),
			DAN,
			crate::Onboarding::HousingFund::ROLES::Accounts::NOTARY
		));
		assert_ok!(RoleModule::account_approval(Origin::signed(ALICE), DAN));

		assert_ok!(RoleModule::set_role(
			Origin::signed(AMANI),
			AMANI,
			crate::Onboarding::HousingFund::ROLES::Accounts::SELLER
		));
		assert_ok!(RoleModule::account_approval(Origin::signed(ALICE), AMANI));

		let collection_id = NftCollection::OFFICESTEST.value();

		assert_noop!(
			FinalizerModule::reject_transaction_asset(Origin::signed(DAN), collection_id, 1,),
			Error::<Test>::AssetDoesNotExist
		);
	});
}

#[test]
fn reject_transaction_asset_no_finalising_status_should_fail() {
	new_test_ext().execute_with(|| {
		assert_ok!(RoleModule::set_role(
			Origin::signed(KEZIA),
			KEZIA,
			crate::Onboarding::HousingFund::ROLES::Accounts::SERVICER
		));
		assert_ok!(RoleModule::account_approval(Origin::signed(ALICE), KEZIA));

		assert_ok!(RoleModule::set_role(
			Origin::signed(DAN),
			DAN,
			crate::Onboarding::HousingFund::ROLES::Accounts::NOTARY
		));
		assert_ok!(RoleModule::account_approval(Origin::signed(ALICE), DAN));

		assert_ok!(RoleModule::set_role(
			Origin::signed(AMANI),
			AMANI,
			crate::Onboarding::HousingFund::ROLES::Accounts::SELLER
		));
		assert_ok!(RoleModule::account_approval(Origin::signed(ALICE), AMANI));

		let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata0".to_vec().try_into().unwrap();

		assert_ok!(NftModule::create_collection(
			Origin::signed(KEZIA),
			NftCollection::OFFICESTEST,
			metadata.clone()
		));

		assert_ok!(OnboardingModule::create_and_submit_proposal(
			Origin::signed(AMANI),
			NftCollection::OFFICESTEST,
			Some(100),
			metadata,
			false,
			3
		));

		let collection_id = NftCollection::OFFICESTEST.value();
		let item_id = pallet_nft::ItemsCount::<Test>::get()[collection_id as usize] - 1;

		assert_noop!(
			FinalizerModule::reject_transaction_asset(Origin::signed(DAN), collection_id, item_id,),
			Error::<Test>::HouseHasNotFinalisingStatus
		);
	});
}

#[test]
fn reject_transaction_asset_should_succeed() {
	new_test_ext().execute_with(|| {
		let mut block_number = System::block_number();
		let amount = 100;

		for account_id in 1..6 {
			assert_ok!(RoleModule::set_role(
				Origin::signed(account_id),
				account_id,
				crate::Onboarding::HousingFund::ROLES::Accounts::INVESTOR
			));

			// test contribute with sufficient contribution and free balance
			assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(account_id), amount));

			let contribution = HousingFundModule::contributions(account_id).unwrap();

			assert_eq!(contribution.block_number, block_number);

			block_number = block_number.saturating_add(1);
			System::set_block_number(block_number);
		}

		assert_ok!(RoleModule::set_role(
			Origin::signed(KEZIA),
			KEZIA,
			crate::Onboarding::HousingFund::ROLES::Accounts::SERVICER
		));
		assert_ok!(RoleModule::account_approval(Origin::signed(ALICE), KEZIA));
		assert_ok!(RoleModule::set_role(
			Origin::signed(AMANI),
			AMANI,
			crate::Onboarding::HousingFund::ROLES::Accounts::SELLER
		));
		assert_ok!(RoleModule::account_approval(Origin::signed(ALICE), AMANI));

		let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata0".to_vec().try_into().unwrap();

		assert_ok!(NftModule::create_collection(
			Origin::signed(KEZIA),
			NftCollection::OFFICESTEST,
			metadata.clone()
		));

		assert_ok!(OnboardingModule::create_and_submit_proposal(
			Origin::signed(AMANI),
			NftCollection::OFFICESTEST,
			Some(100),
			metadata,
			false,
			3
		));

		let collection_id = NftCollection::OFFICESTEST.value();
		let item_id = pallet_nft::ItemsCount::<Test>::get()[collection_id as usize] - 1;

		assert_ok!(OnboardingModule::change_status(
			Origin::signed(AMANI),
			NftCollection::OFFICESTEST,
			item_id,
			crate::Onboarding::AssetStatus::ONBOARDED
		));

		assert_ok!(BiddingModule::process_onboarded_assets());

		assert_eq!(
			pallet_onboarding::Houses::<Test>::get(collection_id, item_id).unwrap().status,
			crate::Onboarding::AssetStatus::FINALISING
		);

		assert_ok!(RoleModule::set_role(
			Origin::signed(DAN),
			DAN,
			crate::Onboarding::HousingFund::ROLES::Accounts::NOTARY
		));
		assert_ok!(RoleModule::account_approval(Origin::signed(ALICE), DAN));

		assert_ok!(FinalizerModule::reject_transaction_asset(
			Origin::signed(DAN),
			collection_id,
			item_id,
		));

		let house = OnboardingModule::houses(collection_id, item_id).unwrap();
		assert_eq!(house.status, crate::Onboarding::AssetStatus::REJECTED);

		assert_eq!(
			HousingFundModule::fund_balance(),
			crate::HousingFund::FundInfo {
				total: HousingFundModule::u64_to_balance_option(500).unwrap(),
				transferable: HousingFundModule::u64_to_balance_option(500).unwrap(),
				reserved: HousingFundModule::u64_to_balance_option(0).unwrap(),
			}
		);

		let event = <frame_system::Pallet<Test>>::events()
			.pop()
			.expect("Expected at least one EventRecord to be found")
			.event;

		// check that the event has been raised
		assert_eq!(
			event,
			mock::Event::FinalizerModule(crate::Event::NotaryRejectedAssetTransaction(
				DAN,
				collection_id,
				item_id
			))
		);
	});
}

#[test]
fn cancel_transaction_asset_no_seller_role_should_fail() {
	new_test_ext().execute_with(|| {
		assert_ok!(RoleModule::set_role(
			Origin::signed(KEZIA),
			KEZIA,
			crate::Onboarding::HousingFund::ROLES::Accounts::SERVICER
		));
		assert_ok!(RoleModule::account_approval(Origin::signed(ALICE), KEZIA));

		assert_ok!(RoleModule::set_role(
			Origin::signed(AMANI),
			AMANI,
			crate::Onboarding::HousingFund::ROLES::Accounts::SELLER
		));
		assert_ok!(RoleModule::account_approval(Origin::signed(ALICE), AMANI));

		let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata0".to_vec().try_into().unwrap();

		assert_ok!(NftModule::create_collection(
			Origin::signed(KEZIA),
			NftCollection::OFFICESTEST,
			metadata.clone()
		));

		assert_ok!(OnboardingModule::create_and_submit_proposal(
			Origin::signed(AMANI),
			NftCollection::OFFICESTEST,
			Some(100),
			metadata,
			false,
			3
		));

		let collection_id = NftCollection::OFFICESTEST.value();
		let item_id = pallet_nft::ItemsCount::<Test>::get()[collection_id as usize] - 1;

		assert_noop!(
			FinalizerModule::cancel_transaction_asset(
				Origin::signed(KEZIA),
				collection_id,
				item_id,
			),
			Error::<Test>::NotASeller
		);
	});
}

#[test]
fn cancel_transaction_asset_no_existing_house_should_fail() {
	new_test_ext().execute_with(|| {
		assert_ok!(RoleModule::set_role(
			Origin::signed(AMANI),
			AMANI,
			crate::Onboarding::HousingFund::ROLES::Accounts::SELLER
		));
		assert_ok!(RoleModule::account_approval(Origin::signed(ALICE), AMANI));

		let collection_id = NftCollection::OFFICESTEST.value();

		assert_noop!(
			FinalizerModule::cancel_transaction_asset(Origin::signed(AMANI), collection_id, 1,),
			Error::<Test>::AssetDoesNotExist
		);
	});
}

#[test]
fn cancel_transaction_asset_no_finalised_status_should_fail() {
	new_test_ext().execute_with(|| {
		assert_ok!(RoleModule::set_role(
			Origin::signed(KEZIA),
			KEZIA,
			crate::Onboarding::HousingFund::ROLES::Accounts::SERVICER
		));
		assert_ok!(RoleModule::account_approval(Origin::signed(ALICE), KEZIA));

		assert_ok!(RoleModule::set_role(
			Origin::signed(AMANI),
			AMANI,
			crate::Onboarding::HousingFund::ROLES::Accounts::SELLER
		));
		assert_ok!(RoleModule::account_approval(Origin::signed(ALICE), AMANI));

		let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata0".to_vec().try_into().unwrap();

		assert_ok!(NftModule::create_collection(
			Origin::signed(KEZIA),
			NftCollection::OFFICESTEST,
			metadata.clone()
		));

		assert_ok!(OnboardingModule::create_and_submit_proposal(
			Origin::signed(AMANI),
			NftCollection::OFFICESTEST,
			Some(100),
			metadata,
			false,
			3
		));

		let collection_id = NftCollection::OFFICESTEST.value();
		let item_id = pallet_nft::ItemsCount::<Test>::get()[collection_id as usize] - 1;

		assert_noop!(
			FinalizerModule::cancel_transaction_asset(
				Origin::signed(AMANI),
				collection_id,
				item_id,
			),
			Error::<Test>::HouseHasNotFinalisedStatus
		);
	});
}

#[test]
fn cancel_transaction_asset_should_succeed() {
	new_test_ext().execute_with(|| {
		let mut block_number = System::block_number();
		let amount = 100;

		for account_id in 1..6 {
			assert_ok!(RoleModule::set_role(
				Origin::signed(account_id),
				account_id,
				crate::Onboarding::HousingFund::ROLES::Accounts::INVESTOR
			));

			// test contribute with sufficient contribution and free balance
			assert_ok!(HousingFundModule::contribute_to_fund(Origin::signed(account_id), amount));

			let contribution = HousingFundModule::contributions(account_id).unwrap();

			assert_eq!(contribution.block_number, block_number);

			block_number = block_number.saturating_add(1);
			System::set_block_number(block_number);
		}

		assert_ok!(RoleModule::set_role(
			Origin::signed(KEZIA),
			KEZIA,
			crate::Onboarding::HousingFund::ROLES::Accounts::SERVICER
		));
		assert_ok!(RoleModule::account_approval(Origin::signed(ALICE), KEZIA));
		assert_ok!(RoleModule::set_role(
			Origin::signed(AMANI),
			AMANI,
			crate::Onboarding::HousingFund::ROLES::Accounts::SELLER
		));
		assert_ok!(RoleModule::account_approval(Origin::signed(ALICE), AMANI));

		let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata0".to_vec().try_into().unwrap();

		assert_ok!(NftModule::create_collection(
			Origin::signed(KEZIA),
			NftCollection::OFFICESTEST,
			metadata.clone()
		));

		assert_ok!(OnboardingModule::create_and_submit_proposal(
			Origin::signed(AMANI),
			NftCollection::OFFICESTEST,
			Some(100),
			metadata,
			false,
			3
		));

		let collection_id = NftCollection::OFFICESTEST.value();
		let item_id = pallet_nft::ItemsCount::<Test>::get()[collection_id as usize] - 1;

		assert_ok!(OnboardingModule::change_status(
			Origin::signed(AMANI),
			NftCollection::OFFICESTEST,
			item_id,
			crate::Onboarding::AssetStatus::ONBOARDED
		));

		assert_ok!(BiddingModule::process_onboarded_assets());

		assert_eq!(
			pallet_onboarding::Houses::<Test>::get(collection_id, item_id).unwrap().status,
			crate::Onboarding::AssetStatus::FINALISING
		);

		assert_ok!(RoleModule::set_role(
			Origin::signed(DAN),
			DAN,
			crate::Onboarding::HousingFund::ROLES::Accounts::NOTARY
		));
		assert_ok!(RoleModule::account_approval(Origin::signed(ALICE), DAN));

		assert_ok!(FinalizerModule::validate_transaction_asset(
			Origin::signed(DAN),
			collection_id,
			item_id,
		));

		assert_ok!(FinalizerModule::cancel_transaction_asset(
			Origin::signed(AMANI),
			collection_id,
			item_id,
		));

		let house = OnboardingModule::houses(collection_id, item_id).unwrap();
		assert_eq!(house.status, crate::Onboarding::AssetStatus::CANCELLED);

		assert_eq!(
			HousingFundModule::fund_balance(),
			crate::HousingFund::FundInfo {
				total: HousingFundModule::u64_to_balance_option(500).unwrap(),
				transferable: HousingFundModule::u64_to_balance_option(500).unwrap(),
				reserved: HousingFundModule::u64_to_balance_option(0).unwrap(),
			}
		);

		let event = <frame_system::Pallet<Test>>::events()
			.pop()
			.expect("Expected at least one EventRecord to be found")
			.event;

		// check that the event has been raised
		assert_eq!(
			event,
			mock::Event::FinalizerModule(crate::Event::SellerCancelledAssetTransaction(
				AMANI,
				collection_id,
				item_id
			))
		);
	});
}
