use frame_support::{assert_noop, assert_ok};

use super::*;
use mock::*;
use mock::Acc;
use std::convert::TryInto;

type NFTPallet = Pallet<Test>;


macro_rules! bvec {
	($( $x:tt )*) => {
		vec![$( $x )*].try_into().unwrap()
	}
}

#[test]
fn mint_works() {
	new_test_ext().execute_with(|| {
		
		//prep_roles();
		RolesModule::set_role(RuntimeOrigin::signed(CHARLIE), CHARLIE, Acc::INVESTOR).ok();

		assert_ok!(NFTPallet::create_collection(
			RuntimeOrigin::signed(CHARLIE),
			PossibleCollections::HOUSESTEST,
			bvec![0,0,3]
		));
		assert_ok!(NFTPallet::create_collection(
			RuntimeOrigin::signed(CHARLIE),
			PossibleCollections::OFFICESTEST,
			bvec![0,0,1]
		));

		assert_ok!(NFTPallet::mint(
			RuntimeOrigin::signed(BOB),
			PossibleCollections::HOUSESTEST,
			bvec![0,0,4]
		));
		/*assert_eq!(
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
		);*/
	});
}
