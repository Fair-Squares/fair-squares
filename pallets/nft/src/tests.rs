use frame_support::{assert_noop, assert_ok};

use super::*;
use mock::*;
use std::convert::TryInto;

type NFTPallet = Pallet<Test>;
pub fn prep_roles() {
	RoleModule::set_role(Origin::signed(CHARLIE), CHARLIE, Acc::SERVICER).ok();
	RoleModule::account_approval(Origin::signed(ALICE), CHARLIE).ok();
	RoleModule::set_role(Origin::signed(EVE), EVE, Acc::SERVICER).ok();
	RoleModule::account_approval(Origin::signed(ALICE), EVE).ok();
	RoleModule::set_role(Origin::signed(BOB), BOB, Acc::SELLER).ok();
	RoleModule::account_approval(Origin::signed(ALICE), BOB).ok();
	RoleModule::set_role(Origin::signed(DAVE), DAVE, Acc::INVESTOR).ok();
	RoleModule::set_role(
		Origin::signed(ACCOUNT_WITH_NO_BALANCE0),
		ACCOUNT_WITH_NO_BALANCE0,
		Acc::SERVICER,
	)
	.ok();
	RoleModule::account_approval(Origin::signed(ALICE), ACCOUNT_WITH_NO_BALANCE0).ok();
	RoleModule::set_role(
		Origin::signed(ACCOUNT_WITH_NO_BALANCE1),
		ACCOUNT_WITH_NO_BALANCE1,
		Acc::SELLER,
	)
	.ok();
	RoleModule::account_approval(Origin::signed(ALICE), ACCOUNT_WITH_NO_BALANCE1).ok();
}

#[test]
fn create_collection_works() {
	ExtBuilder::default().build().execute_with(|| {
		let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata".to_vec().try_into().unwrap();
		prep_roles();
		assert_ok!(NFTPallet::create_collection(
			Origin::signed(CHARLIE),
			PossibleCollections::HOUSESTEST,
			metadata.clone()
		));
		assert_eq!(
			NFTPallet::collections(HOUSESTEST).unwrap(),
			CollectionInfo { created_by: Acc::SERVICER, metadata: metadata.clone() }
		);

		expect_events(vec![crate::Event::CollectionCreated {
			owner: CHARLIE,
			collection_id: HOUSESTEST,
			created_by: Acc::SERVICER,
		}
		.into()]);

		// not allowed in Permissions
		assert_noop!(
			NFTPallet::create_collection(
				Origin::signed(BOB),
				PossibleCollections::OFFICESTEST,
				metadata.clone()
			),
			Error::<Test>::NotPermitted
		);

		// existing collection ID
		assert_noop!(
			NFTPallet::create_collection(
				Origin::signed(CHARLIE),
				PossibleCollections::HOUSESTEST,
				metadata
			),
			pallet_uniques::Error::<Test>::InUse
		);
	})
}

#[test]
fn mint_works() {
	ExtBuilder::default().build().execute_with(|| {
		let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata".to_vec().try_into().unwrap();
		prep_roles();

		assert_ok!(NFTPallet::create_collection(
			Origin::signed(CHARLIE),
			PossibleCollections::HOUSESTEST,
			metadata.clone()
		));
		assert_ok!(NFTPallet::create_collection(
			Origin::signed(CHARLIE),
			PossibleCollections::OFFICESTEST,
			metadata.clone()
		));

		assert_ok!(NFTPallet::mint(
			Origin::signed(BOB),
			PossibleCollections::HOUSESTEST,
			metadata.clone()
		));
		assert_eq!(
			NFTPallet::items(HOUSESTEST, ITEM_ID_0).unwrap(),
			ItemInfo { metadata: metadata.clone() }
		);

		expect_events(vec![crate::Event::ItemMinted {
			owner: BOB,
			collection_id: HOUSESTEST,
			item_id: ITEM_ID_0,
		}
		.into()]);

		// not allowed in Permissions
		assert_noop!(
			NFTPallet::mint(
				Origin::signed(DAVE),
				PossibleCollections::OFFICESTEST,
				metadata.clone()
			),
			Error::<Test>::NotPermitted
		);

		// invalid collection ID
		assert_noop!(
			NFTPallet::mint(Origin::signed(BOB), PossibleCollections::NONEXISTING, metadata),
			Error::<Test>::CollectionUnknown
		);
	});
}

#[test]
fn transfer_works() {
	ExtBuilder::default().build().execute_with(|| {
		let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata".to_vec().try_into().unwrap();
		prep_roles();
		assert_ok!(NFTPallet::create_collection(
			Origin::signed(CHARLIE),
			PossibleCollections::HOUSESTEST,
			metadata.clone()
		));
		assert_ok!(NFTPallet::create_collection(
			Origin::signed(CHARLIE),
			PossibleCollections::OFFICESTEST,
			metadata.clone()
		));
		assert_ok!(NFTPallet::mint(
			Origin::signed(BOB),
			PossibleCollections::HOUSESTEST,
			metadata.clone()
		));
		assert_ok!(NFTPallet::mint(
			Origin::signed(BOB),
			PossibleCollections::OFFICESTEST,
			metadata
		));

		let origin: Origin = frame_system::RawOrigin::Root.into();

		// not existing
		assert_noop!(
			NFTPallet::transfer(
				origin.clone(),
				PossibleCollections::APPARTMENTSTEST,
				ITEM_ID_0,
				BOB
			),
			Error::<Test>::ItemUnknown
		);

		assert_ok!(NFTPallet::transfer(
			origin.clone(),
			PossibleCollections::HOUSESTEST,
			ITEM_ID_0,
			DAVE
		));
		assert_eq!(NFTPallet::owner(HOUSESTEST, ITEM_ID_0).unwrap(), DAVE);

		assert_ok!(NFTPallet::transfer(origin, PossibleCollections::HOUSESTEST, ITEM_ID_0, BOB));
		assert_eq!(NFTPallet::owner(HOUSESTEST, ITEM_ID_0).unwrap(), BOB);

		expect_events(vec![crate::Event::ItemTransferred {
			from: DAVE,
			to: BOB,
			collection_id: HOUSESTEST,
			item_id: ITEM_ID_0,
		}
		.into()]);
	});
}

#[test]
fn burn_works() {
	ExtBuilder::default().build().execute_with(|| {
		let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata".to_vec().try_into().unwrap();
		prep_roles();

		assert_ok!(NFTPallet::create_collection(
			Origin::signed(CHARLIE),
			PossibleCollections::HOUSESTEST,
			metadata.clone()
		));
		assert_ok!(NFTPallet::create_collection(
			Origin::signed(CHARLIE),
			PossibleCollections::OFFICESTEST,
			metadata.clone()
		));
		assert_ok!(NFTPallet::mint(
			Origin::signed(BOB),
			PossibleCollections::HOUSESTEST,
			metadata.clone()
		));
		assert_ok!(NFTPallet::mint(
			Origin::signed(BOB),
			PossibleCollections::HOUSESTEST,
			metadata.clone()
		));
		assert_ok!(NFTPallet::mint(
			Origin::signed(BOB),
			PossibleCollections::OFFICESTEST,
			metadata
		));

		// not allowed in Permissions
		assert_noop!(
			NFTPallet::burn(Origin::signed(BOB), PossibleCollections::OFFICESTEST, ITEM_ID_0),
			Error::<Test>::NotPermitted
		);

		assert_ok!(NFTPallet::burn(
			Origin::signed(CHARLIE),
			PossibleCollections::HOUSESTEST,
			ITEM_ID_0
		));
		assert!(!<Items<Test>>::contains_key(HOUSESTEST, ITEM_ID_0));

		expect_events(vec![crate::Event::ItemBurned {
			owner: BOB,
			collection_id: HOUSESTEST,
			item_id: ITEM_ID_0,
		}
		.into()]);

		// not existing
		assert_noop!(
			NFTPallet::burn(Origin::signed(CHARLIE), PossibleCollections::HOUSESTEST, ITEM_ID_0),
			Error::<Test>::ItemUnknown
		);
	});
}

#[test]
fn destroy_collection_works() {
	ExtBuilder::default().build().execute_with(|| {
		let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata".to_vec().try_into().unwrap();
		prep_roles();
		assert_ok!(NFTPallet::create_collection(
			Origin::signed(CHARLIE),
			PossibleCollections::HOUSESTEST,
			metadata.clone()
		));
		assert_ok!(NFTPallet::create_collection(
			Origin::signed(CHARLIE),
			PossibleCollections::OFFICESTEST,
			metadata.clone()
		));
		assert_ok!(NFTPallet::mint(Origin::signed(BOB), PossibleCollections::HOUSESTEST, metadata));

		// existing item
		assert_noop!(
			NFTPallet::destroy_collection(Origin::signed(CHARLIE), PossibleCollections::HOUSESTEST),
			Error::<Test>::TokenCollectionNotEmpty
		);
		assert_ok!(NFTPallet::burn(
			Origin::signed(CHARLIE),
			PossibleCollections::HOUSESTEST,
			ITEM_ID_0
		));

		// not allowed in Permissions
		assert_noop!(
			NFTPallet::destroy_collection(Origin::signed(BOB), PossibleCollections::OFFICESTEST),
			Error::<Test>::NotPermitted
		);

		assert_ok!(NFTPallet::destroy_collection(
			Origin::signed(CHARLIE),
			PossibleCollections::HOUSESTEST
		));
		assert_eq!(NFTPallet::collections(HOUSESTEST), None);

		expect_events(vec![crate::Event::CollectionDestroyed {
			owner: CHARLIE,
			collection_id: HOUSESTEST,
		}
		.into()]);

		// not existing
		assert_noop!(
			NFTPallet::destroy_collection(Origin::signed(CHARLIE), PossibleCollections::HOUSESTEST),
			Error::<Test>::CollectionUnknown
		);
	});
}

#[test]
fn deposit_works() {
	ExtBuilder::default().build().execute_with(|| {
		let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata".to_vec().try_into().unwrap();

		prep_roles();
		let collection_deposit = <Test as pallet_uniques::Config>::CollectionDeposit::get();
		let initial_balance = <Test as pallet_uniques::Config>::Currency::free_balance(&CHARLIE);
		// has deposit
		assert_eq!(<Test as pallet_uniques::Config>::Currency::reserved_balance(&CHARLIE), 0);

		assert_ok!(NFTPallet::create_collection(
			Origin::signed(CHARLIE),
			PossibleCollections::HOUSESTEST,
			metadata.clone()
		));
		assert_eq!(
			<Test as pallet_uniques::Config>::Currency::free_balance(&CHARLIE),
			initial_balance - collection_deposit
		);
		assert_eq!(
			<Test as pallet_uniques::Config>::Currency::reserved_balance(&CHARLIE),
			collection_deposit
		);

		assert_ok!(NFTPallet::destroy_collection(
			Origin::signed(CHARLIE),
			PossibleCollections::HOUSESTEST
		));
		assert_eq!(
			<Test as pallet_uniques::Config>::Currency::free_balance(&CHARLIE),
			initial_balance
		);
		assert_eq!(<Test as pallet_uniques::Config>::Currency::reserved_balance(&CHARLIE), 0);

		// no deposit
		assert_ok!(NFTPallet::create_collection(
			Origin::signed(CHARLIE),
			PossibleCollections::HOUSESTEST,
			metadata.clone()
		));
		assert_ok!(NFTPallet::mint(Origin::signed(BOB), PossibleCollections::HOUSESTEST, metadata));
		assert_eq!(<Test as pallet_uniques::Config>::Currency::free_balance(&BOB), initial_balance);
		assert_eq!(<Test as pallet_uniques::Config>::Currency::reserved_balance(&BOB), 0);

		assert_ok!(NFTPallet::burn(
			Origin::signed(CHARLIE),
			PossibleCollections::HOUSESTEST,
			ITEM_ID_0
		));
		assert_eq!(<Test as pallet_uniques::Config>::Currency::free_balance(&BOB), initial_balance);
		assert_eq!(<Test as pallet_uniques::Config>::Currency::reserved_balance(&BOB), 0);
	})
}

#[test]
fn create_typed_collection_should_not_work_without_deposit_when_deposit_is_required() {
	ExtBuilder::default().build().execute_with(|| {
		prep_roles();
		assert_noop!(
			NFTPallet::create_typed_collection(ACCOUNT_WITH_NO_BALANCE0, HOUSESTEST),
			pallet_balances::Error::<Test>::InsufficientBalance
		);
	});
}

#[test]
fn create_typed_collection_should_not_work_when_not_permitted() {
	ExtBuilder::default().build().execute_with(|| {
		prep_roles();
		assert_noop!(
			NFTPallet::create_typed_collection(DAVE, HOUSESTEST),
			Error::<Test>::NotPermitted
		);
		assert_noop!(
			NFTPallet::create_typed_collection(ACCOUNT_WITH_NO_BALANCE1, HOUSESTEST),
			Error::<Test>::NotPermitted
		);
	});
}

#[test]
fn is_id_reserved_should_return_false_when_id_is_not_from_reserved_range() {
	assert!(
		!NFTPallet::is_id_reserved(mock::ReserveCollectionIdUpTo::get() + 1),
		"(ReserveCollectionIdUpTo + 1) should not be part of reserved CollectionId range"
	);

	assert!(
		!NFTPallet::is_id_reserved(mock::ReserveCollectionIdUpTo::get() + 50_000_000),
		"num > ReserveCollectionIdUpTo should not be part of reserved CollectionId range"
	);
}

#[test]
fn is_id_reserved_should_return_true_when_id_is_from_reserved_range() {
	assert!(
		NFTPallet::is_id_reserved(mock::ReserveCollectionIdUpTo::get()),
		"num == ReserveCollectionIdUpTo should be part of reserved CollectionId range"
	);
}
