use frame_support::{assert_noop, assert_ok};

use primitives::nft::ClassType;

use super::*;
use mock::*;
use std::convert::TryInto;

type NFTPallet = Pallet<Test>;

#[test]
fn create_class_works() {
	ExtBuilder::default().build().execute_with(|| {
		let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata".to_vec().try_into().unwrap();

		assert_ok!(NFTPallet::create_class(
			Origin::signed(ALICE),
			CLASS_ID_0,
			Default::default(),
			metadata.clone()
		));
		assert_ok!(NFTPallet::create_class(
			Origin::signed(ALICE),
			CLASS_ID_1,
			ClassType::Marketplace,
			metadata.clone()
		));
		assert_noop!(
			NFTPallet::create_class(
				Origin::signed(ALICE),
				CLASS_ID_2,
				ClassType::LiquidityMining,
				metadata.clone()
			),
			Error::<Test>::NotPermitted
		);
		assert_ok!(NFTPallet::do_create_class(
			ALICE,
			CLASS_ID_2,
			ClassType::LiquidityMining,
			metadata.clone()
		));
		assert_noop!(
			NFTPallet::create_class(
				Origin::signed(ALICE),
				CLASS_ID_RESERVED,
				ClassType::Marketplace,
				metadata
			),
			Error::<Test>::IdReserved
		);
	})
}

#[test]
fn mint_works() {
	ExtBuilder::default().build().execute_with(|| {
		let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata".to_vec().try_into().unwrap();

		assert_ok!(NFTPallet::create_class(
			Origin::signed(ALICE),
			CLASS_ID_0,
			Default::default(),
			metadata.clone()
		));
		assert_ok!(NFTPallet::do_create_class(
			ALICE,
			CLASS_ID_1,
			ClassType::LiquidityMining,
			metadata.clone()
		));
		assert_ok!(NFTPallet::mint(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			metadata.clone()
		));
		assert_ok!(NFTPallet::mint(
			Origin::signed(BOB),
			CLASS_ID_0,
			INSTANCE_ID_1,
			metadata.clone()
		));
		assert_noop!(
			NFTPallet::mint(Origin::signed(ALICE), CLASS_ID_1, INSTANCE_ID_2, metadata.clone()),
			Error::<Test>::NotPermitted
		);
		assert_ok!(NFTPallet::do_mint(ALICE, CLASS_ID_1, INSTANCE_ID_2, metadata.clone()));

		assert_ok!(NFTPallet::create_class(
			Origin::signed(ALICE),
			CLASS_ID_2,
			Default::default(),
			metadata.clone()
		));
		assert_noop!(
			NFTPallet::mint(Origin::signed(ALICE), NON_EXISTING_CLASS_ID, INSTANCE_ID_0, metadata),
			Error::<Test>::ClassUnknown
		);

		assert_noop!(
			NFTPallet::destroy_class(Origin::signed(ALICE), NON_EXISTING_CLASS_ID),
			Error::<Test>::ClassUnknown
		);
	});
}

#[test]
fn transfer_works() {
	ExtBuilder::default().build().execute_with(|| {
		let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata".to_vec().try_into().unwrap();

		assert_ok!(NFTPallet::create_class(
			Origin::signed(ALICE),
			CLASS_ID_0,
			Default::default(),
			metadata.clone()
		));
		assert_ok!(NFTPallet::do_create_class(
			ALICE,
			CLASS_ID_1,
			ClassType::LiquidityMining,
			metadata.clone()
		));
		assert_eq!(Balances::free_balance(ALICE), 190_000 * BSX);
		assert_ok!(NFTPallet::mint(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			metadata.clone()
		));
		assert_ok!(NFTPallet::do_mint(ALICE, CLASS_ID_1, INSTANCE_ID_0, metadata));
		assert_eq!(Balances::free_balance(ALICE), 189_900 * BSX);
		assert_ok!(NFTPallet::transfer(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			BOB
		));
		assert_noop!(
			NFTPallet::transfer(Origin::signed(CHARLIE), CLASS_ID_0, INSTANCE_ID_0, ALICE),
			Error::<Test>::NotPermitted
		);
		assert_ok!(NFTPallet::transfer(
			Origin::signed(ALICE),
			CLASS_ID_1,
			INSTANCE_ID_0,
			BOB
		));
		assert_ok!(NFTPallet::do_transfer(CLASS_ID_1, INSTANCE_ID_0, BOB, CHARLIE));
		assert_eq!(Balances::free_balance(BOB), 150_000 * BSX);
		assert_ok!(NFTPallet::transfer(Origin::signed(BOB), CLASS_ID_0, INSTANCE_ID_0, BOB));
		assert_eq!(Balances::free_balance(BOB), 150_000 * BSX);
		assert_ok!(NFTPallet::transfer(
			Origin::signed(BOB),
			CLASS_ID_0,
			INSTANCE_ID_0,
			CHARLIE
		));
		assert_eq!(Balances::free_balance(ALICE), 189_900 * BSX);
		assert_eq!(Balances::free_balance(BOB), 150_000 * BSX);
		assert_eq!(Balances::free_balance(CHARLIE), 15_000 * BSX);
	});
}

#[test]
fn burn_works() {
	ExtBuilder::default().build().execute_with(|| {
		let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata".to_vec().try_into().unwrap();

		assert_ok!(NFTPallet::create_class(
			Origin::signed(ALICE),
			CLASS_ID_0,
			Default::default(),
			metadata.clone()
		));
		assert_ok!(NFTPallet::do_create_class(
			ALICE,
			CLASS_ID_1,
			ClassType::LiquidityMining,
			metadata.clone()
		));
		assert_ok!(NFTPallet::mint(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			metadata.clone()
		));
		assert_ok!(NFTPallet::do_mint(BOB, CLASS_ID_1, INSTANCE_ID_0, metadata));

		assert_noop!(
			NFTPallet::burn(Origin::signed(BOB), CLASS_ID_0, INSTANCE_ID_0),
			Error::<Test>::NotPermitted
		);
		assert_noop!(
			NFTPallet::burn(Origin::signed(BOB), CLASS_ID_1, INSTANCE_ID_0),
			Error::<Test>::NotPermitted
		);

		assert_ok!(NFTPallet::burn(Origin::signed(ALICE), CLASS_ID_0, INSTANCE_ID_0));
	});
}

#[test]
fn destroy_class_works() {
	ExtBuilder::default().build().execute_with(|| {
		let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata".to_vec().try_into().unwrap();

		assert_ok!(NFTPallet::create_class(
			Origin::signed(ALICE),
			CLASS_ID_0,
			Default::default(),
			metadata.clone()
		));
		assert_ok!(NFTPallet::do_create_class(
			ALICE,
			CLASS_ID_1,
			ClassType::LiquidityMining,
			metadata.clone()
		));
		assert_ok!(NFTPallet::mint(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			metadata.clone()
		));
		assert_ok!(NFTPallet::do_mint(BOB, CLASS_ID_1, INSTANCE_ID_0, metadata));

		assert_noop!(
			NFTPallet::destroy_class(Origin::signed(ALICE), CLASS_ID_0),
			Error::<Test>::TokenClassNotEmpty
		);

		assert_ok!(NFTPallet::burn(Origin::signed(ALICE), CLASS_ID_0, INSTANCE_ID_0));
		assert_ok!(NFTPallet::destroy_class(Origin::signed(ALICE), CLASS_ID_0));
		assert_noop!(
			NFTPallet::destroy_class(Origin::signed(ALICE), CLASS_ID_1),
			Error::<Test>::NotPermitted
		);
		assert_ok!(NFTPallet::do_burn(BOB, CLASS_ID_1, INSTANCE_ID_0));
		assert_ok!(NFTPallet::do_destroy_class(ALICE, CLASS_ID_1));
		assert_noop!(
			NFTPallet::destroy_class(Origin::signed(ALICE), CLASS_ID_0),
			Error::<Test>::ClassUnknown
		);
	});
}