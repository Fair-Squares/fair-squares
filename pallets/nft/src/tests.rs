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
        RoleModule::set_role(Origin::signed(CHARLIE).clone(), Acc::SELLER).ok();
        RoleModule::account_approval(Origin::signed(ALICE),CHARLIE).ok();
        assert_ok!(NFTPallet::create_collection(
            Origin::signed(CHARLIE),
            COLLECTION_ID_0,
            metadata.clone()
        ));
        assert_eq!(
            NFTPallet::collections(COLLECTION_ID_0).unwrap(),
            CollectionInfo {
                created_by: Acc::SELLER,
                metadata: metadata.clone()
            }
        );

        expect_events(vec![crate::Event::CollectionCreated {
            owner: CHARLIE,
            collection_id: COLLECTION_ID_0,
            created_by: Acc::SELLER,
        }
        .into()]);

        // not allowed in Permissions
        RoleModule::set_role(Origin::signed(BOB).clone(), Acc::INVESTOR).ok();
        assert_noop!(
            NFTPallet::create_collection(Origin::signed(BOB), COLLECTION_ID_2, metadata.clone()),
            Error::<Test>::NotPermitted
        );

        // existing collection ID
        assert_noop!(
            NFTPallet::create_collection(
                Origin::signed(CHARLIE),
                COLLECTION_ID_0,
                metadata.clone()
            ),
            pallet_uniques::Error::<Test>::InUse
        );

        // reserved collection ID
        assert_noop!(
            NFTPallet::create_collection(
                Origin::signed(CHARLIE),
                COLLECTION_ID_RESERVED,
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
        RoleModule::set_role(Origin::signed(CHARLIE).clone(), Acc::SERVICER).ok();
        RoleModule::account_approval(Origin::signed(ALICE),CHARLIE).ok();
        RoleModule::set_role(Origin::signed(BOB).clone(), Acc::SELLER).ok();
        RoleModule::account_approval(Origin::signed(ALICE),BOB).ok();
        RoleModule::set_role(Origin::signed(DAVE).clone(), Acc::INVESTOR).ok();

        assert_ok!(NFTPallet::create_collection(
            Origin::signed(CHARLIE),
            COLLECTION_ID_0,
            metadata.clone()
        ));
        assert_ok!(NFTPallet::create_collection(
            Origin::signed(CHARLIE),
            COLLECTION_ID_1,
            metadata.clone()
        ));

        assert_ok!(NFTPallet::mint(
            Origin::signed(CHARLIE),
            COLLECTION_ID_0,
            ITEM_ID_0,
            metadata.clone()
        ));
        assert_eq!(
            NFTPallet::items(COLLECTION_ID_0, ITEM_ID_0).unwrap(),
            ItemInfo {
                metadata: metadata.clone()
            }
        );

        expect_events(vec![crate::Event::ItemMinted {
            owner: CHARLIE,
            collection_id: COLLECTION_ID_0,
            item_id: ITEM_ID_0,
        }
        .into()]);

        // duplicate item
        assert_noop!(
            NFTPallet::mint(Origin::signed(CHARLIE), COLLECTION_ID_0, ITEM_ID_0, metadata.clone()),
            pallet_uniques::Error::<Test>::AlreadyExists
        );

        // not allowed in Permissions
        assert_noop!(
            NFTPallet::mint(Origin::signed(DAVE), COLLECTION_ID_1, ITEM_ID_0, metadata.clone()),
            Error::<Test>::NotPermitted
        );


        // invalid collection ID
        assert_noop!(
            NFTPallet::mint(Origin::signed(BOB), NON_EXISTING_COLLECTION_ID, ITEM_ID_0, metadata),
            Error::<Test>::CollectionUnknown
        );
    });
}

#[test]
fn transfer_works() {
    ExtBuilder::default().build().execute_with(|| {
        let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
            b"metadata".to_vec().try_into().unwrap();
            RoleModule::set_role(Origin::signed(CHARLIE).clone(), Acc::SERVICER).ok();
            RoleModule::account_approval(Origin::signed(ALICE),CHARLIE).ok();
            RoleModule::set_role(Origin::signed(EVE).clone(), Acc::SERVICER).ok();
            RoleModule::account_approval(Origin::signed(ALICE),EVE).ok();
            RoleModule::set_role(Origin::signed(BOB).clone(), Acc::SELLER).ok();
            RoleModule::account_approval(Origin::signed(ALICE),BOB).ok();
            RoleModule::set_role(Origin::signed(DAVE).clone(), Acc::INVESTOR).ok();
        assert_ok!(NFTPallet::create_collection(
            Origin::signed(CHARLIE),
            COLLECTION_ID_0,
            metadata.clone()
        ));
        assert_ok!(NFTPallet::create_collection(
            Origin::signed(CHARLIE),
            COLLECTION_ID_1,
            metadata.clone()
        ));
        assert_ok!(NFTPallet::mint(
            Origin::signed(CHARLIE),
            COLLECTION_ID_0,
            ITEM_ID_0,
            metadata.clone()
        ));
        assert_ok!(NFTPallet::mint(
            Origin::signed(CHARLIE),
            COLLECTION_ID_1,
            ITEM_ID_0,
            metadata
        ));

        // not existing
        //assert_noop!(
        //    NFTPallet::transfer(Origin::signed(CHARLIE), COLLECTION_ID_2, ITEM_ID_0, BOB),
        //    Error::<Test>::CollectionUnknown
        //);

        // not owner
        assert_noop!(
            NFTPallet::transfer(Origin::signed(BOB), COLLECTION_ID_0, ITEM_ID_0, DAVE),
            Error::<Test>::NotPermitted
        );

        // not allowed in Permissions
        assert_noop!(
            NFTPallet::transfer(Origin::signed(BOB), COLLECTION_ID_1, ITEM_ID_0, DAVE),
            Error::<Test>::NotPermitted
        );

        assert_ok!(NFTPallet::transfer(
            Origin::signed(CHARLIE),
            COLLECTION_ID_0,
            ITEM_ID_0,
            EVE
        ));
        assert_eq!(NFTPallet::owner(COLLECTION_ID_0, ITEM_ID_0).unwrap(), EVE);

        assert_ok!(NFTPallet::transfer(
            Origin::signed(EVE),
            COLLECTION_ID_0,
            ITEM_ID_0,
            BOB
        ));
        assert_eq!(NFTPallet::owner(COLLECTION_ID_0, ITEM_ID_0).unwrap(), BOB);

        expect_events(vec![crate::Event::ItemTransferred {
            from: EVE,
            to: BOB,
            collection_id: COLLECTION_ID_0,
            item_id: ITEM_ID_0,
        }
        .into()]);
    });
}



