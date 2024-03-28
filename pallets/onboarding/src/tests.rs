use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};
pub use super::*;
use std::convert::TryInto;

macro_rules! bvec{
    ($($x:tt)*)=>{
        vec![$($x)*].try_into().unwrap()
    }
}

#[test]
fn setting_roles(){
	new_test_ext().execute_with(||{

		assert_eq!(RolesModule::get_pending_servicers().len(), 0);
		assert_eq!(RolesModule::get_pending_house_sellers().len(), 0);
		assert_eq!(RolesModule::get_pending_notaries().len(), 0);
		let council = Collective::members();
		assert_eq!(council.len(),4);


		//Investor & Tenant roles
		assert_ok!(RolesModule::set_role(RuntimeOrigin::signed(DAVE), DAVE, Acc::INVESTOR));
		assert_ok!(RolesModule::set_role(RuntimeOrigin::signed(EVE), EVE, Acc::INVESTOR));
		assert!(pallet_roles::InvestorLog::<Test>::contains_key(DAVE));
        assert!(pallet_roles::InvestorLog::<Test>::contains_key(EVE));
        assert_ok!(pallet_nft::Pallet::<Test>::create_collection(
            RuntimeOrigin::signed(DAVE),
            pallet_nft::PossibleCollections::HOUSESTEST,
            bvec![0,0,1]
        ));
        assert_ok!(Housing::contribute_to_fund(
            RuntimeOrigin::signed(EVE).into(),
            20_000_000
        ));

        assert_ok!(OnboardingModule::create_and_submit_proposal(
            RuntimeOrigin::signed(EVE),
            pallet_nft::PossibleCollections::HOUSESTEST,
            Some(200),
            bvec![0,0,1],
            false,
            2
        ));

        
        println!("Number of items:{:?}",NftModule::itemid());
		assert!(NftModule::itemid()[4]>0);
        assert_ok!(NftModule::transfer(
            RuntimeOrigin::signed(EVE),
            pallet_nft::PossibleCollections::HOUSESTEST,
            0,
            DAVE
        ));
        assert_ok!(
            NftModule::burn(
                RuntimeOrigin::signed(DAVE),
                pallet_nft::PossibleCollections::HOUSESTEST,
                0,
            )
        );

		
	})
}
