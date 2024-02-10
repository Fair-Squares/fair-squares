use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
pub use super::*;

fn next_block() {
	System::set_block_number(System::block_number() + 1);
	RolesModule::on_initialize(System::block_number());
	
}

macro_rules! bvec {
	($( $x:tt )*) => {
		vec![$( $x )*].try_into().unwrap()
	}
}

fn fast_forward_to(n: u64) {
	while System::block_number() < n {
		next_block();
	}
}

#[test]
fn bidding_roles(){
	new_test_ext().execute_with(||{

		//let inv_list = vec![ALICE,DAVE,BOB,CHARLIE,EVE];
		assert_eq!(RolesModule::get_pending_servicers().len(), 0);
		assert_eq!(RolesModule::get_pending_house_sellers().len(), 0);
		assert_eq!(RolesModule::get_pending_notaries().len(), 0);
		let council = Collective::members();
		assert_eq!(council.len(),3);

		//Investor & Tenant roles
		assert_ok!(RolesModule::set_role(RuntimeOrigin::signed(DAVE), DAVE, Acc::INVESTOR));
		assert_ok!(RolesModule::set_role(RuntimeOrigin::signed(EVE), EVE, Acc::TENANT));
		assert!(pallet_roles::InvestorLog::<Test>::contains_key(DAVE));
		assert!(pallet_roles::TenantLog::<Test>::contains_key(EVE));

		assert_ok!(RolesModule::set_role(RuntimeOrigin::signed(ALICE), ALICE, Acc::INVESTOR));
		assert_ok!(RolesModule::set_role(RuntimeOrigin::signed(ACCOUNT_WITH_BALANCE0), ACCOUNT_WITH_BALANCE0, Acc::INVESTOR));
		assert_ok!(RolesModule::set_role(RuntimeOrigin::signed(ACCOUNT_WITH_BALANCE1), ACCOUNT_WITH_BALANCE1, Acc::INVESTOR));
		assert_ok!(RolesModule::set_role(RuntimeOrigin::signed(ACCOUNT_WITH_BALANCE2), ACCOUNT_WITH_BALANCE2, Acc::INVESTOR));
		assert_ok!(RolesModule::set_role(RuntimeOrigin::signed(ACCOUNT_WITH_BALANCE3), ACCOUNT_WITH_BALANCE3, Acc::INVESTOR));

		assert!(pallet_roles::InvestorLog::<Test>::contains_key(ALICE));
		assert!(pallet_roles::InvestorLog::<Test>::contains_key(ACCOUNT_WITH_BALANCE0));
		assert!(pallet_roles::InvestorLog::<Test>::contains_key(ACCOUNT_WITH_BALANCE1));
		assert!(pallet_roles::InvestorLog::<Test>::contains_key(ACCOUNT_WITH_BALANCE2));
		assert!(pallet_roles::InvestorLog::<Test>::contains_key(ACCOUNT_WITH_BALANCE3));

		//Seller,Servicer, and Notary roles
		assert_ok!(RolesModule::set_role(RuntimeOrigin::signed(BOB),BOB,Acc::SELLER));
		assert_ok!(RolesModule::set_role(RuntimeOrigin::signed(EVE),EVE,Acc::SERVICER));
		assert_ok!(RolesModule::set_role(RuntimeOrigin::signed(CHARLIE),CHARLIE,Acc::NOTARY));

		assert_eq!(RolesModule::get_pending_house_sellers().len(),1);
		let account =RolesModule::get_pending_house_sellers()[0].account_id.clone();
		assert_eq!(account,BOB);
		assert_eq!(RolesModule::get_requested_role(BOB).unwrap().role.unwrap(),Acc::SELLER);

		assert_ok!(RolesModule::council_vote(RuntimeOrigin::signed(council[2].clone()),EVE,true));
		assert_ok!(RolesModule::council_vote(RuntimeOrigin::signed(council[1].clone()),EVE,true));
		assert_ok!(RolesModule::council_vote(RuntimeOrigin::signed(council[0].clone()),EVE,true));

		assert_ok!(RolesModule::council_vote(RuntimeOrigin::signed(council[2].clone()),BOB,true));
		assert_ok!(RolesModule::council_vote(RuntimeOrigin::signed(council[1].clone()),BOB,true));
		assert_ok!(RolesModule::council_vote(RuntimeOrigin::signed(council[0].clone()),BOB,true));

		assert_ok!(RolesModule::council_vote(RuntimeOrigin::signed(council[2].clone()),CHARLIE,true));
		assert_ok!(RolesModule::council_vote(RuntimeOrigin::signed(council[1].clone()),CHARLIE,true));
		assert_ok!(RolesModule::council_vote(RuntimeOrigin::signed(council[0].clone()),CHARLIE,true));

		assert_eq!(RolesModule::sellers(BOB).is_some(),false);

		assert_ok!(RolesModule::council_close(RuntimeOrigin::signed(council[2].clone()),BOB));
		assert_ok!(RolesModule::council_close(RuntimeOrigin::signed(council[2].clone()),EVE));
		assert_ok!(RolesModule::council_close(RuntimeOrigin::signed(council[2].clone()),CHARLIE));

		

		


		assert_eq!(RolesModule::get_pending_house_sellers().len(), 0);

		assert_eq!(RolesModule::sellers(BOB).is_some(),true);
		assert_eq!(RolesModule::notaries(CHARLIE).is_some(),true);
		assert_eq!(RolesModule::servicers(EVE).is_some(),true);


		assert_ok!(HousingFund::contribute_to_fund(RuntimeOrigin::signed(ALICE), 350_000*BSX));
		assert_ok!(HousingFund::contribute_to_fund(RuntimeOrigin::signed(DAVE), 160_000*BSX));
		assert_ok!(HousingFund::contribute_to_fund(RuntimeOrigin::signed(ACCOUNT_WITH_BALANCE0), 150_000*BSX));
		assert_ok!(HousingFund::contribute_to_fund(RuntimeOrigin::signed(ACCOUNT_WITH_BALANCE1), 70_000*BSX));
		assert_ok!(HousingFund::contribute_to_fund(RuntimeOrigin::signed(ACCOUNT_WITH_BALANCE2), 220_000*BSX));
		assert_ok!(HousingFund::contribute_to_fund(RuntimeOrigin::signed(ACCOUNT_WITH_BALANCE3), 530_000*BSX));

		assert_ok!(NftModule::create_collection(RuntimeOrigin::signed(EVE),pallet_nft::PossibleCollections::HOUSES,bvec![0,0,3]));

		
		assert_ok!(OnboardingModule::create_and_submit_proposal(RuntimeOrigin::signed(BOB),
																pallet_nft::PossibleCollections::HOUSES,
																 Some(2_605_000_000_000),
																 bvec![0u8; 20],
																   true,
																    3));
																	
	let council1 = Collective1::members();
	assert_eq!(council1.len(),3);

	assert_ok!(CouncilModule::seller_proposal_evaluation(
		RuntimeOrigin::signed(council1[0].clone()).into(), 
		pallet_nft::PossibleCollections::HOUSES, 
		0));


	let proposal = CouncilModule::get_submitted_proposal(BOB);
	assert_eq!(proposal.is_some(),true);

	let status = pallet_roles::AssetStatus::REVIEWING;
	let coll_id0 = pallet_nft::PossibleCollections::HOUSES;
	let coll_id = coll_id0.value();
	assert_eq!(OnboardingModule::houses(coll_id,0).unwrap().status,status);

	assert_ok!(CouncilModule::housing_council_vote(RuntimeOrigin::signed(council1[0].clone()), BOB, true));
	assert_ok!(CouncilModule::housing_council_vote(RuntimeOrigin::signed(council1[1].clone()), BOB, true));
	assert_ok!(CouncilModule::housing_council_vote(RuntimeOrigin::signed(council1[2].clone()), BOB, true));	
	assert_ok!(CouncilModule::housing_council_close(RuntimeOrigin::signed(council1[1].clone()), BOB));

	let now = System::block_number();
	expect_events(vec![
		RuntimeEvent::CouncilModule(pallet_council::Event::HousingCouncilSessionClosed{ 
			who: council1[1].clone(), 
			proposal_index: 0, 
			when: now.clone()
		})
	]);
	//let end_block = now.saturating_add(<Test as pallet_council::Config>::CheckPeriod::get());
	//fast_forward_to(end_block);
	
	

	loop{
		let  event_ref = 
		record(RuntimeEvent::CouncilModule(pallet_council::Event::ProposalApproved(System::block_number(), BOB)))
		;
		if System::events().contains(&event_ref){
			break
		}

		next_block();
			
		
	}


	let houses = OnboardingModule::houses(coll_id, 0).unwrap();
	assert_eq!(houses.status,pallet_roles::AssetStatus::REVIEWING);
	

	/*loop{
		let  event_ref0 = 
		record(RuntimeEvent::OnboardingModule(pallet_onboarding::Event::ReferendumStarted { index: 0 }));
		
		if System::events().contains(&event_ref0){
			break
		}

		next_block();

	}

	//let status = pallet_roles::AssetStatus::VOTING;
	//assert_eq!(OnboardingModule::houses(coll_id,0).unwrap().status,status);
	
	assert_ok!(OnboardingModule::investor_vote(RuntimeOrigin::signed(ALICE), 0, true));
	assert_ok!(OnboardingModule::investor_vote(RuntimeOrigin::signed(DAVE), 0, true));
	assert_ok!(OnboardingModule::investor_vote(RuntimeOrigin::signed(ACCOUNT_WITH_BALANCE0), 0, true));
	assert_ok!(OnboardingModule::investor_vote(RuntimeOrigin::signed(ACCOUNT_WITH_BALANCE1), 0, true));
	assert_ok!(OnboardingModule::investor_vote(RuntimeOrigin::signed(ACCOUNT_WITH_BALANCE2), 0, true));
	assert_ok!(OnboardingModule::investor_vote(RuntimeOrigin::signed(ACCOUNT_WITH_BALANCE3), 0, true));

*/
	

	})

	
}
