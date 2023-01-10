pub use super::*;
pub use frame_support::{assert_err, assert_ok};
use frame_system::pallet_prelude::OriginFor;
use mock::*;

pub type Bvec<Test> = BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit>;

pub fn prep_roles() {
	RoleModule::set_role(Origin::signed(CHARLIE), CHARLIE, Acc::SERVICER).ok();
	RoleModule::account_approval(Origin::signed(ALICE), CHARLIE).ok();
	RoleModule::set_role(Origin::signed(BOB), BOB, Acc::SELLER).ok();
	RoleModule::account_approval(Origin::signed(ALICE), BOB).ok();
	RoleModule::set_role(Origin::signed(DAVE), DAVE, Acc::INVESTOR).ok();
	RoleModule::set_role(Origin::signed(EVE), EVE, Acc::INVESTOR).ok();
	RoleModule::set_role(Origin::signed(FERDIE), FERDIE, Acc::REPRESENTATIVE).ok(); //FERDIE approval will be tested
	RoleModule::set_role(Origin::signed(GERARD), GERARD, Acc::TENANT).ok();
	RoleModule::set_role(Origin::signed(HUNTER), HUNTER, Acc::TENANT).ok();
}

fn next_block() {
	System::set_block_number(System::block_number() + 1);
	Scheduler::on_initialize(System::block_number());
	Democracy::on_initialize(System::block_number());
	AssetManagement::begin_block(System::block_number());
}

fn fast_forward_to(n: u64) {
	while System::block_number() < n {
		next_block();
	}
}

#[test]
fn representative() {
	ExtBuilder::default().build().execute_with(|| {
		//submit a request for representative role
		RoleModule::set_role(Origin::signed(CHARLIE), CHARLIE, Acc::REPRESENTATIVE).ok();
		//approve request
		//assert_ok!(AssetManagement::)
	});
}

pub fn prep_test(
	price1: u64,
	price2: u64,
	metadata0: Bvec<Test>,
	metadata1: Bvec<Test>,
	metadata2: Bvec<Test>,
) {
	prep_roles();

	//Dave and EVE contribute to the fund
	assert_ok!(HousingFund::contribute_to_fund(Origin::signed(DAVE), 50_000));
	assert_ok!(HousingFund::contribute_to_fund(Origin::signed(EVE), 50_000));

	//Charlie creates a collection
	assert_ok!(NftModule::create_collection(
		Origin::signed(CHARLIE),
		NftColl::OFFICESTEST,
		metadata0.clone()
	));
	//Charlie creates a second collection
	assert_ok!(NftModule::create_collection(
		Origin::signed(CHARLIE),
		NftColl::APPARTMENTSTEST,
		metadata0
	));
	// Bob creates a proposal without submiting for review

	assert_ok!(OnboardingModule::create_and_submit_proposal(
		Origin::signed(BOB),
		NftColl::OFFICESTEST,
		Some(price1),
		metadata1,
		false
	));

	assert_ok!(OnboardingModule::create_and_submit_proposal(
		Origin::signed(BOB),
		NftColl::APPARTMENTSTEST,
		Some(price2),
		metadata2,
		false
	));
}

#[test]
fn share_distributor0() {
	ExtBuilder::default().build().execute_with(|| {
		let metadata0 = b"metadata0".to_vec().try_into().unwrap();
		let metadata1 = b"metadata1".to_vec().try_into().unwrap();
		let metadata2 = b"metadata2".to_vec().try_into().unwrap();
		//put some funds in FairSquare SlashFees account
		let fees_account = OnboardingModule::account_id();
		<Test as pallet::Config>::Currency::make_free_balance_be(&fees_account, 150_000u32.into());

		let price1 = 40_000;
		let price2 = 30_000;
		prep_test(price1, price2, metadata0, metadata1, metadata2);
		let coll_id0 = NftColl::OFFICESTEST.value();
		let item_id0 = pallet_nft::ItemsCount::<Test>::get()[coll_id0 as usize] - 1;
		let origin: OriginFor<Test> = frame_system::RawOrigin::Root.into();
		let origin_bob = Origin::signed(BOB);

		//Change first asset status to FINALISED
		OnboardingModule::change_status(
			origin_bob.clone(),
			NftColl::OFFICESTEST,
			item_id0,
			Onboarding::AssetStatus::FINALISED,
		)
		.ok();

		//Store initial owner
		let old_owner0 = pallet_nft::Pallet::<Test>::owner(coll_id0, item_id0).unwrap();

		//Execute virtual account transactions
		assert_ok!(ShareDistributor::virtual_account(coll_id0, item_id0));
		//Store new owner
		let new_owner0 = ShareDistributor::virtual_acc(coll_id0, item_id0).unwrap().virtual_account;

		//Execute nft transaction
		assert_ok!(ShareDistributor::nft_transaction(coll_id0, item_id0, new_owner0.clone()));

		//Compare new & old owner
		assert_ne!(old_owner0, new_owner0);

		//Create a FundOperation struct for this asset
		let fund_op = HFund::FundOperation {
			nft_collection_id: coll_id0,
			nft_item_id: item_id0,
			amount: price1,
			block_number: 1,
			contributions: vec![(EVE, 25_000), (DAVE, 15_000)],
		};
		let id = ShareDistributor::virtual_acc(coll_id0, item_id0).unwrap().token_id;
		//Add new owners and asset to housing fund
		HFund::Reservations::<Test>::insert((coll_id0, item_id0), fund_op);
		println!("Reservations {:?}", HFund::Reservations::<Test>::get((coll_id0, item_id0)));
		println!("Virtual Account {:?}", ShareDistributor::virtual_acc(coll_id0, item_id0));

		//Create token
		assert_ok!(ShareDistributor::create_tokens(origin, coll_id0, item_id0, new_owner0.clone()));
		assert_eq!(1, ShareDistributor::token_id());
		assert_eq!(0, ShareDistributor::virtual_acc(coll_id0, item_id0).unwrap().token_id);
		assert_eq!(1000, Assets::total_supply(id));
		//Check that new_owner0 is in possession of 1000 tokens
		assert_eq!(1000, Assets::balance(id, new_owner0.clone()));
		//Distribute token
		assert_ok!(ShareDistributor::distribute_tokens(new_owner0.clone(), coll_id0, item_id0));
		let balance0 = Assets::balance(id, DAVE);
		let balance1 = Assets::balance(id, EVE);

		let _infos = ShareDistributor::tokens_infos(new_owner0.clone()).unwrap().owners;
		println!("Tokens own by DAVE:{:?}\nTokens own by Eve:{:?}", balance0, balance1);
		println!("Total supply {:?}", Assets::total_supply(id));

		// Bob creates a second proposal without submiting for review
		let coll_id1 = NftColl::APPARTMENTSTEST.value();
		let item_id1 = pallet_nft::ItemsCount::<Test>::get()[coll_id1 as usize] - 1;

		//Store initial owner
		let old_owner1 = pallet_nft::Pallet::<Test>::owner(coll_id1, item_id1).unwrap();

		//Change first asset status to FINALISED
		OnboardingModule::change_status(
			origin_bob.clone(),
			NftColl::APPARTMENTSTEST,
			item_id1,
			Onboarding::AssetStatus::FINALISED,
		)
		.ok();

		//Execute virtual account transactions
		assert_ok!(ShareDistributor::virtual_account(coll_id1, item_id1));

		//Store new owner
		let new_owner1 = ShareDistributor::virtual_acc(coll_id1, item_id1).unwrap().virtual_account;

		//Execute nft transaction
		assert_ok!(ShareDistributor::nft_transaction(coll_id1, item_id1, new_owner1.clone()));

		//Compare new & old owner
		assert_ne!(old_owner1, new_owner1);

		//Get the virtual accounts
		let virtual0 = Share::Virtual::<Test>::get(coll_id0, item_id0).unwrap();
		let virtual1 = Share::Virtual::<Test>::get(coll_id1, item_id1).unwrap();

		//Check that virtual accounts are different
		println!("Virtual account nbr1:{:?}\nVirtual account nbr2:{:?}", virtual0, virtual1);
		assert_ne!(virtual0.virtual_account, virtual1.virtual_account);
		//Check that virtual accounts are the new owners
		assert_eq!(new_owner0, virtual0.virtual_account);
		assert_eq!(new_owner1, virtual1.virtual_account);
		Balances::set_balance(
			frame_system::RawOrigin::Root.into(),
			virtual0.virtual_account,
			5_000_000_000,
			1_000_000_000,
		)
		.ok();
		Balances::set_balance(
			frame_system::RawOrigin::Root.into(),
			virtual1.virtual_account,
			5_000_000_000,
			1_000_000_000,
		)
		.ok();

		//Representative Role status before Approval
		assert!(!RoleModule::get_pending_representatives(FERDIE).unwrap().activated);

		let origin_eve = Origin::signed(EVE);
		let origin_dave = Origin::signed(DAVE);

		//////////////////////////////////////////////////////////////////////////////////////////
		/////							TEST representative_approval						//////
		//////////////////////////////////////////////////////////////////////////////////////////

		//Create voting session, aka Referendum to elect FERDIE as a representative.
		assert_ok!(AssetManagement::launch_representative_session(
			origin_eve.clone(),
			NftColl::OFFICESTEST,
			item_id0,
			FERDIE,
			VoteProposals::Election
		));
		let mut ref_index = 0;
		//Get Referendum status before vote
		let mut ref_infos = Democracy::referendum_info(0).unwrap();
		println!(
			"\n\nReferendum status before vote is: {:?}\n present block is: {:?}\n\n",
			&ref_infos,
			System::block_number()
		);

		//Investors vote
		assert_ok!(AssetManagement::owners_vote(origin_eve.clone(), ref_index, true));
		assert_ok!(AssetManagement::owners_vote(origin_dave.clone(), ref_index, true));

		//Voting events emmited
		expect_events(vec![
			mock::Event::AssetManagement(crate::Event::InvestorVoted {
				caller: EVE,
				session_number: ref_index,
				when: System::block_number(),
			}),
			mock::Event::AssetManagement(crate::Event::InvestorVoted {
				caller: DAVE,
				session_number: ref_index,
				when: System::block_number(),
			}),
		]);

		let initial_block_number = System::block_number();
		let end_block_number = initial_block_number
			.saturating_add(<Test as pallet_democracy::Config>::VotingPeriod::get());

		fast_forward_to(end_block_number);
		ref_infos = Democracy::referendum_info(0).unwrap();

		let b = match ref_infos {
			pallet_democracy::ReferendumInfo::Finished { approved, end: _ } => approved,
			_ => false,
		};

		println!(
			"\n\nReferendum status after vote is: {:?}\n present block is: {:?}\n\n",
			&ref_infos,
			System::block_number()
		);
		println!("\n\nvote result is:{:?}", b);
		let prop0 = AssetManagement::proposals(0).unwrap().vote_result;
		println!("\n\nVote results:{:?}\n\n", prop0);

		//Proposal enactement should happen 2 blocks later
		fast_forward_to(end_block_number.saturating_add(<Test as crate::Config>::Delay::get()));

		//The line below evaluate the results of TEST_0, TEST_1, & TEST_2 by looking for the result
		// of a correctly executed call.
		assert!(Roles::RepresentativeLog::<Test>::contains_key(FERDIE));
		assert!(Roles::AccountsRolesLog::<Test>::contains_key(FERDIE));

		//////////////////////////////////////////////////////////////////////////////////////////
		/////							TEST launch_tenant_session							//////
		//////////////////////////////////////////////////////////////////////////////////////////
		println!("\n\nTest start: launch_tenant_session");
		// Bob(Not a representative) tries proposing a tenant(GERARD)
		assert_err!(
			AssetManagement::launch_tenant_session(
				origin_bob,
				NftColl::OFFICESTEST,
				item_id0,
				GERARD,
				VoteProposals::Election,
				Ident::Judgement::Reasonable,
			),
			Error::<Test>::NotARepresentative
		);

		println!("\n\nlaunch_tenant_session - : NOT A REPRESENTATIVE");

		let origin_ferdie = Origin::signed(FERDIE);

		assert_err!(
			AssetManagement::launch_tenant_session(
				origin_ferdie.clone(),
				NftColl::OFFICES,
				10,
				GERARD,
				VoteProposals::Election,
				Ident::Judgement::Reasonable,
			),
			Error::<Test>::NotAnAsset
		);
		println!("\n\nlaunch_tenant_session - : NOT AN ASSET");

		let rep_ferdie = RoleModule::reps(FERDIE).unwrap();
		println!("\nRepresentative Ferdie: {:?}", rep_ferdie);

		// Ferdie(representative) proposes GERARD(tenant) for a house but the house is not under
		// control of Ferdie
		assert_err!(
			AssetManagement::launch_tenant_session(
				origin_ferdie.clone(),
				NftColl::APPARTMENTSTEST,
				item_id1,
				GERARD,
				VoteProposals::Election,
				Ident::Judgement::Reasonable,
			),
			Error::<Test>::AssetOutOfControl
		);
		println!("\n\nlaunch_tenant_session - : ASSEST NOT LINKED TO THE REPRESENTATIVE");

		// Ferdie(Representative) proposes BOB(Not a tenant) as a tenant
		assert_err!(
			AssetManagement::launch_tenant_session(
				origin_ferdie.clone(),
				NftColl::OFFICESTEST,
				item_id0,
				BOB,
				VoteProposals::Election,
				Ident::Judgement::Reasonable,
			),
			Error::<Test>::NotATenant
		);

		println!("\n\nlaunch_tenant_session - : NOT A TENANT");

		// Ferdie(Representative) proposes a tenant(GERARD)
		// Check if GERARD has the tenant role
		assert!(Roles::TenantLog::<Test>::contains_key(GERARD));

		// Check the tenants of the house
		let house0 = OnboardingModule::houses(coll_id0, item_id0).unwrap();
		assert!(house0.tenants.is_empty());

		// Check the asset_account of the tenant
		let tenant0 = RoleModule::tenants(GERARD).unwrap();
		assert!(tenant0.asset_account.is_none());

		/***	START: Successful scenario of proposing a tenant    ** */
		// Create a voting session, aka referendum to propose GERARD as a tenant for the first house
		assert_ok!(AssetManagement::launch_tenant_session(
			origin_ferdie.clone(),
			NftColl::OFFICESTEST,
			item_id0,
			GERARD,
			VoteProposals::Election,
			Ident::Judgement::Reasonable,
		));

		println!("\n\nlaunch_tenant_session - : A SUCCESSFUL SCENARIO");

		// Investors vote
		ref_index += 1;
		assert_ok!(AssetManagement::owners_vote(origin_eve.clone(), ref_index, true));
		assert_ok!(AssetManagement::owners_vote(origin_dave.clone(), ref_index, true));

		// Voting events emitted
		expect_events(vec![
			mock::Event::AssetManagement(crate::Event::InvestorVoted {
				caller: EVE,
				session_number: ref_index,
				when: System::block_number(),
			}),
			mock::Event::AssetManagement(crate::Event::InvestorVoted {
				caller: DAVE,
				session_number: ref_index,
				when: System::block_number(),
			}),
		]);

		let initial_block_number = System::block_number();
		let end_block_number = initial_block_number
			.saturating_add(<Test as pallet_democracy::Config>::VotingPeriod::get());

		fast_forward_to(end_block_number);

		//Proposal enactement should happen 2 blocks later
		fast_forward_to(end_block_number.saturating_add(<Test as crate::Config>::Delay::get()));

		// Check the tenants of the house
		let house0 = OnboardingModule::houses(coll_id0, item_id0).unwrap();
		assert_eq!(house0.tenants, vec![GERARD]);

		// Check the asset_account of the tenant
		let tenant0 = RoleModule::tenants(GERARD).unwrap();
		assert_eq!(
			tenant0.asset_account.unwrap(),
			ShareDistributor::virtual_acc(coll_id0, item_id0).unwrap().virtual_account
		);

		/***	END: Successful scenario of proposing a tenant    ** */

		// Ferdie(Representative) again proposes GERARD for the second house.
		assert_err!(
			AssetManagement::launch_tenant_session(
				origin_ferdie.clone(),
				NftColl::OFFICESTEST,
				item_id0,
				GERARD,
				VoteProposals::Election,
				Ident::Judgement::Reasonable,
			),
			Error::<Test>::AlreadyLinkedWithAsset
		);
		println!("\n\nlaunch_tenant_session - : THE TENANT IS ALREADY LINKED WITH AN ASSET");

		// demote a tenant
		assert_err!(
			AssetManagement::launch_tenant_session(
				origin_ferdie.clone(),
				NftColl::OFFICESTEST,
				item_id0,
				HUNTER,
				VoteProposals::Demotion,
				Ident::Judgement::Reasonable,
			),
			Error::<Test>::TenantAssetNotLinked
		);
		println!("\n\nlaunch_tenant_session - : DEMOTE A TENANT NOT LINKED WITH AN ASSET");

		// Multiple tenants for an asset
		assert_ok!(AssetManagement::launch_tenant_session(
			origin_ferdie.clone(),
			NftColl::OFFICESTEST,
			item_id0,
			HUNTER,
			VoteProposals::Election,
			Ident::Judgement::Reasonable,
		));

		ref_index += 1;
		assert_ok!(AssetManagement::owners_vote(origin_eve.clone(), ref_index, true));
		assert_ok!(AssetManagement::owners_vote(origin_dave.clone(), ref_index, true));

		// Voting events emitted
		expect_events(vec![
			mock::Event::AssetManagement(crate::Event::InvestorVoted {
				caller: EVE,
				session_number: ref_index,
				when: System::block_number(),
			}),
			mock::Event::AssetManagement(crate::Event::InvestorVoted {
				caller: DAVE,
				session_number: ref_index,
				when: System::block_number(),
			}),
		]);

		let initial_block_number = System::block_number();
		let end_block_number = initial_block_number
			.saturating_add(<Test as pallet_democracy::Config>::VotingPeriod::get());

		fast_forward_to(end_block_number);

		//Proposal enactement should happen 2 blocks later
		fast_forward_to(end_block_number.saturating_add(<Test as crate::Config>::Delay::get()));

		// Check the tenants of the house
		let house0 = OnboardingModule::houses(coll_id0, item_id0).unwrap();
		assert_eq!(house0.tenants, vec![GERARD, HUNTER]);

		// Check the asset_account of the tenant
		let tenant1 = RoleModule::tenants(HUNTER).unwrap();
		assert_eq!(
			tenant1.asset_account.unwrap(),
			ShareDistributor::virtual_acc(coll_id0, item_id0).unwrap().virtual_account
		);

		println!("\n\nlaunch_tenant_session - : MULTIPLE TENANTS FOR AN ASSET");

		assert_ok!(AssetManagement::launch_tenant_session(
			origin_ferdie,
			NftColl::OFFICESTEST,
			item_id0,
			HUNTER,
			VoteProposals::Demotion,
			Ident::Judgement::Reasonable,
		));

		ref_index += 1;
		assert_ok!(AssetManagement::owners_vote(origin_eve.clone(), ref_index, true));
		assert_ok!(AssetManagement::owners_vote(origin_dave.clone(), ref_index, true));

		// Voting events emitted
		expect_events(vec![
			mock::Event::AssetManagement(crate::Event::InvestorVoted {
				caller: EVE,
				session_number: ref_index,
				when: System::block_number(),
			}),
			mock::Event::AssetManagement(crate::Event::InvestorVoted {
				caller: DAVE,
				session_number: ref_index,
				when: System::block_number(),
			}),
		]);

		let initial_block_number = System::block_number();
		let end_block_number = initial_block_number
			.saturating_add(<Test as pallet_democracy::Config>::VotingPeriod::get());

		fast_forward_to(end_block_number);

		//Proposal enactement should happen 2 blocks later
		fast_forward_to(end_block_number.saturating_add(<Test as crate::Config>::Delay::get()));

		// Check the tenants of the house
		let house0 = OnboardingModule::houses(coll_id0, item_id0).unwrap();
		assert_eq!(house0.tenants, vec![GERARD]);

		// Check the asset_account of the tenant
		assert!(RoleModule::tenants(HUNTER).unwrap().asset_account.is_none());
		assert_eq!(
			RoleModule::tenants(GERARD).unwrap().asset_account.unwrap(),
			ShareDistributor::virtual_acc(coll_id0, item_id0).unwrap().virtual_account
		);

		//////////////////////////////////////////////////////////////////////////////////////////
		/////								TEST demote_representative						//////
		//////////////////////////////////////////////////////////////////////////////////////////
		//Create voting session, aka Referendum to demote FERDIE from her/his representative role.
		assert_ok!(AssetManagement::launch_representative_session(
			origin_eve.clone(),
			NftColl::OFFICESTEST,
			item_id0,
			FERDIE,
			VoteProposals::Demotion
		));

		ref_index += 1;

		//Investors vote
		assert_ok!(AssetManagement::owners_vote(origin_eve, ref_index, true));
		assert_ok!(AssetManagement::owners_vote(origin_dave, ref_index, true));

		//Voting events emmited
		expect_events(vec![
			mock::Event::AssetManagement(crate::Event::InvestorVoted {
				caller: EVE,
				session_number: ref_index,
				when: System::block_number(),
			}),
			mock::Event::AssetManagement(crate::Event::InvestorVoted {
				caller: DAVE,
				session_number: ref_index,
				when: System::block_number(),
			}),
		]);

		let initial_block_number = System::block_number();
		let end_block_number = initial_block_number
			.saturating_add(<Test as pallet_democracy::Config>::VotingPeriod::get());

		fast_forward_to(end_block_number);

		//Proposal enactement should happen 2 blocks later
		fast_forward_to(end_block_number.saturating_add(<Test as crate::Config>::Delay::get()));

		//The line below evaluate the results of TEST_0, TEST_1, & TEST_2 by looking for the result
		// of a correctly executed call.
		assert!(!Roles::AccountsRolesLog::<Test>::contains_key(FERDIE));
	});
}
