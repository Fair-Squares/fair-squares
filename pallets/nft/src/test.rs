
use frame_support::{assert_noop, assert_ok, traits::tokens::nonfungibles::*};

use super::*;
use mock::*;
use std::convert::TryInto;
type NFTPallet = Pallet<Test>;

#[test]
fn create_collection_works() {
    ExtBuilder::default().build().execute_with(|| {
        let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
            b"metadata".to_vec().try_into().unwrap();

        assert_ok!(NFTPallet::create_collection(
            Origin::signed(ALICE),
            COLLECTION_ID_0,
            metadata.clone()
        ));
        assert_eq!(
            NFTPallet::collections(COLLECTION_ID_0).unwrap(),
            CollectionInfo {
                collection_type: RoleType::Marketplace,
                metadata: metadata.clone()
            }
        );

        expect_events(vec![crate::Event::CollectionCreated {
            owner: ALICE,
            collection_id: COLLECTION_ID_0,
            collection_type: RoleType::Marketplace,
        }
        .into()]);

        // not allowed in Permissions
        assert_noop!(
            NFTPallet::create_collection(Origin::signed(ALICE), COLLECTION_ID_2, RoleType::Auction, metadata.clone()),
            Error::<Test>::NotPermitted
        );

        // existing collection ID
        assert_noop!(
            NFTPallet::create_collection(
                Origin::signed(ALICE),
                COLLECTION_ID_0,
                RoleType::LiquidityMining,
                metadata.clone()
            ),
            pallet_uniques::Error::<Test>::InUse
        );

        // reserved collection ID
        assert_noop!(
            NFTPallet::create_collection(
                Origin::signed(ALICE),
                COLLECTION_ID_RESERVED,
                RoleType::Marketplace,
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

        assert_ok!(NFTPallet::create_collection(
            Origin::signed(ALICE),
            COLLECTION_ID_0,
            Default::default(), // Marketplace
            metadata.clone()
        ));
        assert_ok!(NFTPallet::create_collection(
            Origin::signed(ALICE),
            COLLECTION_ID_1,
            RoleType::Redeemable,
            metadata.clone()
        ));

        assert_ok!(NFTPallet::mint(
            Origin::signed(ALICE),
            COLLECTION_ID_0,
            ITEM_ID_0,
            metadata.clone()
        ));
        assert_eq!(
            NFTPallet::Items(COLLECTION_ID_0, ITEM_ID_0).unwrap(),
            ItemInfo {
                metadata: metadata.clone()
            }
        );

        expect_events(vec![crate::Event::ItemMinted {
            owner: ALICE,
            collection_id: COLLECTION_ID_0,
            item_id: ITEM_ID_0,
        }
        .into()]);

        // duplicate Item
        assert_noop!(
            NFTPallet::mint(Origin::signed(ALICE), COLLECTION_ID_0, ITEM_ID_0, metadata.clone()),
            pallet_uniques::Error::<Test>::AlreadyExists
        );

        // not allowed in Permissions
        assert_noop!(
            NFTPallet::mint(Origin::signed(ALICE), COLLECTION_ID_1, ITEM_ID_0, metadata.clone()),
            Error::<Test>::NotPermitted
        );

        // not owner
        assert_noop!(
            NFTPallet::mint(Origin::signed(BOB), COLLECTION_ID_1, ITEM_ID_0, metadata.clone()),
            Error::<Test>::NotPermitted
        );

        // invalid collection ID
        assert_noop!(
            NFTPallet::mint(Origin::signed(ALICE), NON_EXISTING_COLLECTION_ID, ITEM_ID_0, metadata),
            Error::<Test>::CollectionUnknown
        );
    });
}

#[test]
fn transfer_works() {
    ExtBuilder::default().build().execute_with(|| {
        let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
            b"metadata".to_vec().try_into().unwrap();

        assert_ok!(NFTPallet::create_collection(
            Origin::signed(ALICE),
            COLLECTION_ID_0,
            Default::default(),
            metadata.clone()
        ));
        assert_ok!(NFTPallet::create_collection(
            Origin::signed(ALICE),
            COLLECTION_ID_1,
            RoleType::LiquidityMining,
            metadata.clone()
        ));
        assert_ok!(NFTPallet::mint(
            Origin::signed(ALICE),
            COLLECTION_ID_0,
            ITEM_ID_0,
            metadata.clone()
        ));
        assert_ok!(NFTPallet::mint(
            Origin::signed(ALICE),
            COLLECTION_ID_1,
            ITEM_ID_0,
            metadata
        ));

        // not existing
        assert_noop!(
            NFTPallet::transfer(Origin::signed(CHARLIE), COLLECTION_ID_2, ITEM_ID_0, ALICE),
            Error::<Test>::CollectionUnknown
        );

        // not owner
        assert_noop!(
            NFTPallet::transfer(Origin::signed(CHARLIE), COLLECTION_ID_0, ITEM_ID_0, ALICE),
            Error::<Test>::NotPermitted
        );

        // not allowed in Permissions
        assert_noop!(
            NFTPallet::transfer(Origin::signed(ALICE), COLLECTION_ID_1, ITEM_ID_0, BOB),
            Error::<Test>::NotPermitted
        );

        assert_ok!(NFTPallet::transfer(
            Origin::signed(ALICE),
            COLLECTION_ID_0,
            ITEM_ID_0,
            ALICE
        ));
        assert_eq!(NFTPallet::owner(COLLECTION_ID_0, ITEM_ID_0).unwrap(), ALICE);

        assert_ok!(NFTPallet::transfer(
            Origin::signed(ALICE),
            COLLECTION_ID_0,
            ITEM_ID_0,
            BOB
        ));
        assert_eq!(NFTPallet::owner(COLLECTION_ID_0, ITEM_ID_0).unwrap(), BOB);

        expect_events(vec![crate::Event::ItemTransferred {
            from: ALICE,
            to: BOB,
            collection_id: COLLECTION_ID_0,
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

        assert_ok!(NFTPallet::create_collection(
            Origin::signed(ALICE),
            COLLECTION_ID_0,
            Default::default(),
            metadata.clone()
        ));
        assert_ok!(NFTPallet::create_collection(
            Origin::signed(ALICE),
            COLLECTION_ID_1,
            RoleType::LiquidityMining,
            metadata.clone()
        ));
        assert_ok!(NFTPallet::mint(
            Origin::signed(ALICE),
            COLLECTION_ID_0,
            ITEM_ID_0,
            metadata.clone()
        ));
        assert_ok!(NFTPallet::mint(
            Origin::signed(ALICE),
            COLLECTION_ID_0,
            ITEM_ID_1,
            metadata.clone()
        ));
        assert_ok!(NFTPallet::mint(
            Origin::signed(ALICE),
            COLLECTION_ID_1,
            ITEM_ID_0,
            metadata
        ));

        // not owner
        assert_noop!(
            NFTPallet::burn(Origin::signed(BOB), COLLECTION_ID_0, ITEM_ID_0),
            Error::<Test>::NotPermitted
        );

        // not allowed in Permissions
        assert_noop!(
            NFTPallet::burn(Origin::signed(ALICE), COLLECTION_ID_1, ITEM_ID_0),
            Error::<Test>::NotPermitted
        );

        assert_ok!(NFTPallet::burn(Origin::signed(ALICE), COLLECTION_ID_0, ITEM_ID_0));
        assert!(!<Items<Test>>::contains_key(COLLECTION_ID_0, ITEM_ID_0));

        expect_events(vec![crate::Event::ItemBurned {
            owner: ALICE,
            collection_id: COLLECTION_ID_0,
            item_id: ITEM_ID_0,
        }
        .into()]);

        // not existing
        assert_noop!(
            NFTPallet::burn(Origin::signed(ALICE), COLLECTION_ID_0, ITEM_ID_0),
            pallet_uniques::Error::<Test>::Unknown
        );
    });
}

#[test]
fn destroy_collection_works() {
    ExtBuilder::default().build().execute_with(|| {
        let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
            b"metadata".to_vec().try_into().unwrap();

        assert_ok!(NFTPallet::create_collection(
            Origin::signed(ALICE),
            COLLECTION_ID_0,
            Default::default(), // Marketplace
            metadata.clone()
        ));
        assert_ok!(NFTPallet::create_collection(
            Origin::signed(ALICE),
            COLLECTION_ID_1,
            RoleType::Redeemable,
            metadata.clone()
        ));
        assert_ok!(NFTPallet::mint(
            Origin::signed(ALICE),
            COLLECTION_ID_0,
            ITEM_ID_0,
            metadata
        ));

        // existing Item
        assert_noop!(
            NFTPallet::destroy_collection(Origin::signed(ALICE), COLLECTION_ID_0),
            Error::<Test>::TokenCollectionNotEmpty
        );
        assert_ok!(NFTPallet::burn(Origin::signed(ALICE), COLLECTION_ID_0, ITEM_ID_0));

        // not allowed in Permissions
        assert_noop!(
            NFTPallet::destroy_collection(Origin::signed(ALICE), COLLECTION_ID_1),
            Error::<Test>::NotPermitted
        );

        assert_ok!(NFTPallet::destroy_collection(Origin::signed(ALICE), COLLECTION_ID_0));
        assert_eq!(NFTPallet::collections(COLLECTION_ID_0), None);

        expect_events(vec![crate::Event::CollectionDestroyed {
            owner: ALICE,
            collection_id: COLLECTION_ID_0,
        }
        .into()]);

        // not existing
        assert_noop!(
            NFTPallet::destroy_collection(Origin::signed(ALICE), COLLECTION_ID_0),
            Error::<Test>::CollectionUnknown
        );
    });
}

#[test]
fn deposit_works() {
    ExtBuilder::default().build().execute_with(|| {
        let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
            b"metadata".to_vec().try_into().unwrap();

        let collection_deposit = <Test as pallet_uniques::Config>::CollectionDeposit::get();
        let initial_balance = <Test as pallet_uniques::Config>::Currency::free_balance(&ALICE);

        // has deposit
        assert_eq!(<Test as pallet_uniques::Config>::Currency::reserved_balance(&ALICE), 0);
        assert_ok!(NFTPallet::create_collection(
            Origin::signed(ALICE),
            COLLECTION_ID_0,
            RoleType::Marketplace,
            metadata.clone()
        ));
        assert_eq!(
            <Test as pallet_uniques::Config>::Currency::free_balance(&ALICE),
            initial_balance - collection_deposit
        );
        assert_eq!(
            <Test as pallet_uniques::Config>::Currency::reserved_balance(&ALICE),
            collection_deposit
        );

        assert_ok!(NFTPallet::destroy_collection(Origin::signed(ALICE), COLLECTION_ID_0));
        assert_eq!(
            <Test as pallet_uniques::Config>::Currency::free_balance(&ALICE),
            initial_balance
        );
        assert_eq!(<Test as pallet_uniques::Config>::Currency::reserved_balance(&ALICE), 0);

        // no deposit
        assert_ok!(NFTPallet::create_collection(
            Origin::signed(ALICE),
            COLLECTION_ID_0,
            RoleType::LiquidityMining,
            metadata
        ));
        assert_eq!(
            <Test as pallet_uniques::Config>::Currency::free_balance(&ALICE),
            initial_balance
        );
        assert_eq!(<Test as pallet_uniques::Config>::Currency::reserved_balance(&ALICE), 0);

        assert_ok!(NFTPallet::destroy_collection(Origin::signed(ALICE), COLLECTION_ID_0));
        assert_eq!(
            <Test as pallet_uniques::Config>::Currency::free_balance(&ALICE),
            initial_balance
        );
        assert_eq!(<Test as pallet_uniques::Config>::Currency::reserved_balance(&ALICE), 0);
    })
}

#[test]
fn nonfungible_traits_work() {
    ExtBuilder::default().build().execute_with(|| {
        let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
            b"metadata".to_vec().try_into().unwrap();

        assert_ok!(NFTPallet::create_collection(
            Origin::signed(ALICE),
            COLLECTION_ID_0,
            Default::default(),
            metadata.clone()
        ));

        assert_ok!(NFTPallet::mint(
            Origin::signed(BOB),
            COLLECTION_ID_0,
            ITEM_ID_0,
            metadata.clone()
        ));

        // `Inspect` trait
        assert_eq!(
            <NFTPallet as Inspect<<Test as frame_system::Config>::AccountId>>::owner(&COLLECTION_ID_0, &ITEM_ID_0),
            Some(BOB)
        );
        assert_eq!(
            <NFTPallet as Inspect<<Test as frame_system::Config>::AccountId>>::owner(&COLLECTION_ID_1, &ITEM_ID_0),
            None
        );
        assert_eq!(
            <NFTPallet as Inspect<<Test as frame_system::Config>::AccountId>>::owner(&COLLECTION_ID_0, &ITEM_ID_1),
            None
        );

        assert_eq!(
            <NFTPallet as Inspect<<Test as frame_system::Config>::AccountId>>::collection_owner(&COLLECTION_ID_0),
            Some(ALICE)
        );
        assert_eq!(
            <NFTPallet as Inspect<<Test as frame_system::Config>::AccountId>>::collection_owner(&COLLECTION_ID_1),
            None
        );

        assert!(
            <NFTPallet as Inspect<<Test as frame_system::Config>::AccountId>>::can_transfer(
                &COLLECTION_ID_0,
                &ITEM_ID_0
            )
        );
        assert!(
            !<NFTPallet as Inspect<<Test as frame_system::Config>::AccountId>>::can_transfer(
                &COLLECTION_ID_1,
                &ITEM_ID_1
            )
        );

        // `InspectEnumerable` trait
        assert_eq!(
            *<NFTPallet as InspectEnumerable<<Test as frame_system::Config>::AccountId>>::collections()
                .collect::<Vec<CollectionId>>(),
            vec![COLLECTION_ID_0]
        );
        assert_eq!(
            *<NFTPallet as InspectEnumerable<<Test as frame_system::Config>::AccountId>>::Items(&COLLECTION_ID_0)
                .collect::<Vec<ItemId>>(),
            vec![ITEM_ID_0]
        );
        assert_eq!(
            *<NFTPallet as InspectEnumerable<<Test as frame_system::Config>::AccountId>>::owned(&BOB)
                .collect::<Vec<(CollectionId, ItemId)>>(),
            vec![(COLLECTION_ID_0, ITEM_ID_0)]
        );
        assert_eq!(
            *<NFTPallet as InspectEnumerable<<Test as frame_system::Config>::AccountId>>::owned_in_collection(
                &COLLECTION_ID_0,
                &BOB
            )
            .collect::<Vec<ItemId>>(),
            vec![ITEM_ID_0]
        );

        // `Create` trait
        assert_noop!(
            <NFTPallet as Create<<Test as frame_system::Config>::AccountId>>::create_collection(&COLLECTION_ID_0, &BOB, &ALICE),
            pallet_uniques::Error::<Test>::InUse
        );
        assert_ok!(
            <NFTPallet as Create<<Test as frame_system::Config>::AccountId>>::create_collection(&COLLECTION_ID_1, &BOB, &ALICE)
        );

        // `Destroy` trait
        let witness =
            <NFTPallet as Destroy<<Test as frame_system::Config>::AccountId>>::get_destroy_witness(&COLLECTION_ID_0)
                .unwrap();

        assert_eq!(
            witness,
            pallet_uniques::DestroyWitness {
                Items: 1,
                item_metadatas: 0,
                attributes: 0
            }
        );
        assert_noop!(
            <NFTPallet as Destroy<<Test as frame_system::Config>::AccountId>>::destroy(
                COLLECTION_ID_0,
                witness,
                Some(ALICE)
            ),
            Error::<Test>::TokenCollectionNotEmpty
        );

        let empty_witness = pallet_uniques::DestroyWitness {
            Items: 0,
            item_metadatas: 0,
            attributes: 0,
        };

        assert_ok!(NFTPallet::create_collection(
            Origin::signed(ALICE),
            COLLECTION_ID_2,
            Default::default(),
            metadata,
        ));

        assert_ok!(
            <NFTPallet as Destroy<<Test as frame_system::Config>::AccountId>>::destroy(
                COLLECTION_ID_2,
                empty_witness,
                Some(ALICE)
            ),
            empty_witness
        );

        // `Mutate` trait
        assert_noop!(
            <NFTPallet as Mutate<<Test as frame_system::Config>::AccountId>>::mint_into(
                &COLLECTION_ID_2,
                &ITEM_ID_1,
                &BOB
            ),
            Error::<Test>::CollectionUnknown
        );
        assert_ok!(
            <NFTPallet as Mutate<<Test as frame_system::Config>::AccountId>>::mint_into(
                &COLLECTION_ID_0,
                &ITEM_ID_1,
                &BOB
            )
        );

        assert_ok!(
            <NFTPallet as Mutate<<Test as frame_system::Config>::AccountId>>::burn_from(&COLLECTION_ID_0, &ITEM_ID_1)
        );
        assert!(!<Items<Test>>::contains_key(COLLECTION_ID_0, ITEM_ID_1));

        // `Transfer` trait
        assert_ok!(
            <NFTPallet as Transfer<<Test as frame_system::Config>::AccountId>>::transfer(
                &COLLECTION_ID_0,
                &ITEM_ID_0,
                &ALICE
            )
        );
        assert_eq!(NFTPallet::owner(COLLECTION_ID_0, ITEM_ID_0), Some(ALICE));
    });
}

#[test]
fn is_id_reserved_should_return_true_when_id_is_from_reserved_range() {
    assert!(
        NFTPallet::is_id_reserved(0),
        "0 should be part of reserved CollectionId range"
    );

    assert!(
        NFTPallet::is_id_reserved(13),
        "num <= ReserveCollectionIdUpTo should be part of reserved CollectionId range"
    );

    assert!(
        NFTPallet::is_id_reserved(mock::ReserveCollectionIdUpTo::get()),
        "num == ReserveCollectionIdUpTo should be part of reserved CollectionId range"
    );
}

#[test]
fn is_id_reserved_should_return_false_when_id_is_not_from_reserved_range() {
    assert!(
        !NFTPallet::is_id_reserved(mock::ReserveCollectionIdUpTo::get() + 1),
        "(ReserveCollectionIdUpTo + 1) should not be part of reserved CollectionId range"
    );

    assert!(
        !NFTPallet::is_id_reserved(mock::ReserveCollectionIdUpTo::get() + 500_000_000_000),
        "num > ReserveCollectionIdUpTo should not be part of reserved CollectionId range"
    );
}

#[test]
fn create_typed_collection_should_work_without_deposit_when_deposit_is_not_required() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(NFTPallet::create_typed_collection(
            ACCOUNT_WITH_NO_BALANCE,
            COLLECTION_ID_0,
            RoleType::LiquidityMining
        ));

        assert_eq!(
            NFTPallet::collections(COLLECTION_ID_0).unwrap(),
            CollectionInfoOf::<Test> {
                collection_type: RoleType::LiquidityMining,
                metadata: Default::default()
            }
        )
    });
}

#[test]
fn create_typed_collection_should_not_work_without_deposit_when_deposit_is_required() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            NFTPallet::create_typed_collection(ACCOUNT_WITH_NO_BALANCE, COLLECTION_ID_0, RoleType::Marketplace),
            pallet_balances::Error::<Test>::InsufficientBalance
        );
    });
}

#[test]
fn create_typed_collection_should_not_work_when_not_permitted() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            NFTPallet::create_typed_collection(ALICE, COLLECTION_ID_0, RoleType::Auction),
            Error::<Test>::NotPermitted
        );
    });
}