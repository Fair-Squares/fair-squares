pub use super::*;
pub use frame_support::{assert_err, assert_ok};
use mock::*;

pub type Bvec<Test> = BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit>;

pub fn prep_roles() {
	RoleModule::set_role(Origin::signed(CHARLIE), CHARLIE, Acc::SERVICER).ok();
	RoleModule::account_approval(Origin::signed(ALICE), CHARLIE).ok();
	RoleModule::set_role(Origin::signed(BOB), BOB, Acc::SELLER).ok();
	RoleModule::account_approval(Origin::signed(ALICE), BOB).ok();
	assert_ok!(RoleModule::set_role(Origin::signed(NOTARY), NOTARY, Acc::NOTARY));
	assert_ok!(RoleModule::account_approval(Origin::signed(ALICE), NOTARY));
	assert_ok!(RoleModule::set_role(
		Origin::signed(REPRESENTATIVE),
		REPRESENTATIVE,
		Acc::REPRESENTATIVE
	));
	RoleModule::set_role(Origin::signed(DAVE), DAVE, Acc::INVESTOR).ok();
	RoleModule::set_role(Origin::signed(EVE), EVE, Acc::INVESTOR).ok();
	RoleModule::set_role(Origin::signed(GERARD), GERARD, Acc::INVESTOR).ok();
	RoleModule::set_role(Origin::signed(FERDIE), FERDIE, Acc::INVESTOR).ok();
	RoleModule::set_role(Origin::signed(HUNTER), HUNTER, Acc::INVESTOR).ok();
	RoleModule::set_role(Origin::signed(FRED), FRED, Acc::INVESTOR).ok();
	RoleModule::set_role(Origin::signed(SALIM), SALIM, Acc::INVESTOR).ok();
	RoleModule::set_role(Origin::signed(TENANT0), TENANT0, Acc::TENANT).ok();
}

pub fn prep_test(price1: u64, metadata0: Bvec<Test>, metadata1: Bvec<Test>) {
	prep_roles();

	//Dave and EVE contribute to the fund
	assert_ok!(HousingFund::contribute_to_fund(Origin::signed(DAVE), 1_000_000));
	assert_ok!(HousingFund::contribute_to_fund(Origin::signed(EVE), 800_000));
	assert_ok!(HousingFund::contribute_to_fund(Origin::signed(GERARD), 400_000));
	assert_ok!(HousingFund::contribute_to_fund(Origin::signed(FERDIE), 300_000));
	assert_ok!(HousingFund::contribute_to_fund(Origin::signed(HUNTER), 100_000));
	assert_ok!(HousingFund::contribute_to_fund(Origin::signed(FRED), 1_500_000));
	assert_ok!(HousingFund::contribute_to_fund(Origin::signed(SALIM), 2_000_000));

	//---ASSET PURCHASE STEP---

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
	// Bob creates and submit a proposal

	assert_ok!(OnboardingModule::create_and_submit_proposal(
		Origin::signed(BOB),
		NftColl::OFFICESTEST,
		Some(price1),
		metadata1,
		true
	));

	//Get the proposal hash
	let mut proposal = pallet_voting::VotingProposals::<Test>::iter();
	let prop = proposal.next().unwrap();
	let hash0 = prop.0;
	let infos = prop.1;
	assert_eq!(infos.proposal_hash, hash0);

	let coll_id0 = NftColl::OFFICESTEST.value();
	let item_id0 = pallet_nft::ItemsCount::<Test>::get()[coll_id0 as usize] - 1;
	let mut house = OnboardingModule::houses(coll_id0, item_id0).unwrap();
	assert_eq!(house.status, pallet_onboarding::AssetStatus::REVIEWING);

	//Council vote
	assert_ok!(VotingModule::council_vote(Origin::signed(ALICE), hash0, true,));
	assert_ok!(VotingModule::council_vote(Origin::signed(CHARLIE), hash0, true,));
	assert_ok!(VotingModule::council_vote(Origin::signed(BOB), hash0, true,));

	let initial_block_number = System::block_number();
	let end_block_number = initial_block_number
		.saturating_add(<Test as pallet_voting::Config>::Delay::get())
		.saturating_add(
			<Test as pallet_collective::Config<pallet_collective::Instance1>>::MotionDuration::get(
			),
		);

	assert_eq!(VotingModule::collective_proposals(hash0), Some(end_block_number));
	fast_forward_to(end_block_number);

	assert_ok!(VotingModule::council_close_vote(Origin::signed(ALICE), hash0,));

	let voting_proposal = VotingModule::voting_proposals(hash0).unwrap();

	assert!(voting_proposal.collective_closed);
	assert!(voting_proposal.collective_step);

	//fast_forward_to(end_block_number+1);
	next_block();

	house = OnboardingModule::houses(coll_id0, item_id0).unwrap();
	assert_eq!(house.status, pallet_onboarding::AssetStatus::VOTING);

	//Investors Democracy vote

	//Check proposal content
	let voting_proposal = VotingModule::voting_proposals(hash0).unwrap();
	assert_eq!(voting_proposal.account_id, BOB);

	// Start vote, and check events emitted after first voter.
	// Also output referendum status after each vote.
	assert_ok!(VotingModule::investor_vote(Origin::signed(DAVE), hash0, true,));

	let mut ref_infos =
		Democracy::referendum_info(voting_proposal.democracy_referendum_index).unwrap();
	println!(
		"\n\nReferendum status after vote is: {:?}\n present block is: {:?}\n\n",
		&ref_infos,
		System::block_number()
	);

	let event = <frame_system::Pallet<Test>>::events()
		.pop()
		.expect("Expected at least one EventRecord to be found")
		.event;

	// check that the event has been raised
	assert_eq!(
		event,
		crate::mock::Event::VotingModule(pallet_voting::Event::InvestorVoted(
			DAVE,
			hash0,
			System::block_number()
		)),
	);

	assert_ok!(VotingModule::investor_vote(Origin::signed(EVE), hash0, false,));
	ref_infos = Democracy::referendum_info(voting_proposal.democracy_referendum_index).unwrap();
	println!(
		"\n\nReferendum status after vote is: {:?}\n present block is: {:?}\n\n",
		&ref_infos,
		System::block_number()
	);

	assert_ok!(VotingModule::investor_vote(Origin::signed(GERARD), hash0, false,));
	ref_infos = Democracy::referendum_info(voting_proposal.democracy_referendum_index).unwrap();
	println!(
		"\n\nReferendum status after vote is: {:?}\n present block is: {:?}\n\n",
		&ref_infos,
		System::block_number()
	);

	assert_ok!(VotingModule::investor_vote(Origin::signed(FERDIE), hash0, true,));
	ref_infos = Democracy::referendum_info(voting_proposal.democracy_referendum_index).unwrap();
	println!(
		"\n\nReferendum status after vote is: {:?}\n present block is: {:?}\n\n",
		&ref_infos,
		System::block_number()
	);

	assert_ok!(VotingModule::investor_vote(Origin::signed(HUNTER), hash0, true,));
	ref_infos = Democracy::referendum_info(voting_proposal.democracy_referendum_index).unwrap();
	println!(
		"\n\nReferendum status after vote is: {:?}\n present block is: {:?}\n\n",
		&ref_infos,
		System::block_number()
	);

	assert_ok!(VotingModule::investor_vote(Origin::signed(FRED), hash0, true,));
	ref_infos = Democracy::referendum_info(voting_proposal.democracy_referendum_index).unwrap();
	println!(
		"\n\nReferendum status after vote is: {:?}\n present block is: {:?}\n\n",
		&ref_infos,
		System::block_number()
	);

	assert_ok!(VotingModule::investor_vote(Origin::signed(SALIM), hash0, true,));
	ref_infos = Democracy::referendum_info(voting_proposal.democracy_referendum_index).unwrap();
	println!(
		"\n\nReferendum status after vote is: {:?}\n present block is: {:?}\n\n",
		&ref_infos,
		System::block_number()
	);

	let end_democracy_vote = end_block_number
		.saturating_add(<Test as pallet_voting::Config>::Delay::get())
		.saturating_add(<Test as pallet_democracy::Config>::VotingPeriod::get());

	assert_eq!(Some(end_democracy_vote), VotingModule::democracy_proposals(hash0));

	fast_forward_to(end_democracy_vote + 2);

	ref_infos = Democracy::referendum_info(voting_proposal.democracy_referendum_index).unwrap();
	println!(
		"\n\nReferendum status after vote is: {:?}\n present block is: {:?}\n\n",
		&ref_infos,
		System::block_number()
	);

	//Asset Status should now be `ONBOARDED`
	house = OnboardingModule::houses(coll_id0, item_id0).unwrap();
	assert_eq!(house.status, pallet_onboarding::AssetStatus::ONBOARDED);

	//Move to next block until asset status is changed by pallet_bidding
	while house.status == pallet_onboarding::AssetStatus::ONBOARDED {
		next_block();
		house = OnboardingModule::houses(coll_id0, item_id0).unwrap();
	}

	//Asset status should now be `FINALISING`
	assert_eq!(house.status, pallet_onboarding::AssetStatus::FINALISING);
	println!("\n\nAsset status is:{:?}\n\n", house.status);

	//The Notary will now Finalize the asset
	assert_ok!(Finalise::validate_transaction_asset(Origin::signed(NOTARY), coll_id0, item_id0,));
	house = OnboardingModule::houses(coll_id0, item_id0).unwrap();

	//Asset status should now be `FINALISED`
	assert_eq!(house.status, pallet_onboarding::AssetStatus::FINALISED);

	//Move to next block until asset status is changed by pallet_bidding
	while house.status == pallet_onboarding::AssetStatus::FINALISED {
		next_block();
		house = OnboardingModule::houses(coll_id0, item_id0).unwrap();
	}

	//Asset status should now be `PURCHASED`
	assert_eq!(house.status, pallet_onboarding::AssetStatus::PURCHASED);
	println!("\n\nAsset status is:{:?}\n\n", house.status);

	//---ASSET MANAGEMENT STEP---

	//Let's get the asset virtual Account
	let asset_ownership = ShareDistributor::virtual_acc(coll_id0, item_id0).unwrap();
	let asset_account = asset_ownership.virtual_account;

	// The new owners need a Representative for their asset. Salim starts
	// a referendum for the representative candidate.

	assert_ok!(AssetManagement::launch_representative_session(
		Origin::signed(SALIM),
		NftColl::OFFICESTEST,
		item_id0,
		REPRESENTATIVE,
		pallet_asset_management::VoteProposals::Election,
	));

	//Get the referendum infos
	let mut ref0 = pallet_asset_management::ProposalsLog::<Test>::iter();
	let ref1 = ref0.next().unwrap();
	//Let's make sure that we have the right referendum
	let proposal_rec = ref1.1;
	assert_eq!(proposal_rec.caller_account, SALIM);
	assert_eq!(proposal_rec.candidate_account, REPRESENTATIVE);
	assert_eq!(proposal_rec.virtual_account, asset_account.clone());
	//Get the referendum index and start voting
	let ref_index = ref1.0;

	assert_ok!(AssetManagement::owners_vote(Origin::signed(SALIM), ref_index, true));

	assert_ok!(AssetManagement::owners_vote(Origin::signed(DAVE), ref_index, true));

	assert_ok!(AssetManagement::owners_vote(Origin::signed(EVE), ref_index, true));

	assert_ok!(AssetManagement::owners_vote(Origin::signed(GERARD), ref_index, true));

	assert_ok!(AssetManagement::owners_vote(Origin::signed(FERDIE), ref_index, true));

	assert_ok!(AssetManagement::owners_vote(Origin::signed(HUNTER), ref_index, true));

	assert_ok!(AssetManagement::owners_vote(Origin::signed(FRED), ref_index, true));

	//End REPRESENTATIVE referendum
	let initial_block_number = System::block_number();
	let end_block_number = initial_block_number
		.saturating_add(<Test as pallet_democracy::Config>::VotingPeriod::get());

	fast_forward_to(end_block_number);
	ref_infos = Democracy::referendum_info(0).unwrap();

	println!(
		"\n\nREPRESENTATIVE Referendum status after vote is: {:?}\n present block is: {:?}\n\n",
		&ref_infos,
		System::block_number()
	);

	//Enact Proposal
	fast_forward_to(
		end_block_number.saturating_add(<Test as pallet_asset_management::Config>::Delay::get()),
	);

	//Check the results of the enacted proposal
	assert!(Roles::RepresentativeLog::<Test>::contains_key(REPRESENTATIVE));
	assert!(Roles::AccountsRolesLog::<Test>::contains_key(REPRESENTATIVE));

	//Now that we have a Tenant/Representative/Asset. Let the Tenant ask for an asset
	let tenant_bal_init = Balances::free_balance(TENANT0);
	assert_ok!(crate::Pallet::<Test>::request_asset(
		Origin::signed(TENANT0),
		Box::new(ten()),
		NftColl::OFFICESTEST,
		item_id0,
	));

	let tenant_bal = Balances::free_balance(TENANT0);

	let paid_fees = tenant_bal_init.saturating_sub(tenant_bal);
	println!("\n\nThe tenant paid {:?}units for asset request\n\n", paid_fees);

	//Check that the identity was correctly created
	assert_eq!(pallet_identity::Pallet::<Test>::identity(TENANT0).unwrap().info, ten());

	//Representative gives a positive Judgement and start a referendum for the tenant
	assert_ok!(AssetManagement::launch_tenant_session(
		Origin::signed(REPRESENTATIVE),
		NftColl::OFFICESTEST,
		item_id0,
		TENANT0,
		pallet_asset_management::VoteProposals::Election,
		Ident::Judgement::Reasonable,
	));

	//Get the referendum infos
	let ref0 = pallet_asset_management::ProposalsLog::<Test>::iter();

	for i in ref0 {
		let ref_index = match i.1.caller_account {
			REPRESENTATIVE => i.0,
			_ => 5,
		};
		if ref_index != 5 {
			//get vector of owners
			let house = ShareDistributor::virtual_acc(coll_id0, item_id0).unwrap();
			let owners = house.owners;
			for owner in owners {
				//each owner vote
				assert_ok!(AssetManagement::owners_vote(Origin::signed(owner), ref_index, true));
			}
		}
	}

	//End Tenant referendum
	let initial_block_number = System::block_number();
	let end_block_number = initial_block_number
		.saturating_add(<Test as pallet_democracy::Config>::VotingPeriod::get());

	fast_forward_to(end_block_number);
	ref_infos = Democracy::referendum_info(0).unwrap();

	println!(
		"\n\nTenant Referendum status after vote is: {:?}\n present block is: {:?}\n\n",
		&ref_infos,
		System::block_number()
	);

	//Enact Proposal
	fast_forward_to(
		end_block_number.saturating_add(<Test as pallet_asset_management::Config>::Delay::get()),
	);

	//Check that a guaranty_payment request was sent to the tenant
	let payment_info = AssetManagement::guaranty(TENANT0, asset_account.clone()).unwrap();
	assert_eq!(payment_info.state, pallet_payment::PaymentState::PaymentRequested);

	//Tenant pays the Guaranty Deposit
	let tenant_init_balance = Balances::free_balance(TENANT0);
	let asset_initial = Balances::free_balance(asset_account.clone());

	assert_ok!(crate::Pallet::<Test>::pay_guaranty_deposit(
		Origin::signed(TENANT0),
		NftColl::OFFICESTEST,
		item_id0,
	));
	let payed_amount = tenant_init_balance.saturating_sub(Balances::free_balance(TENANT0));
	let payed_amount1 = Balances::free_balance(asset_account.clone()).saturating_sub(asset_initial);
	println!("Payed amount is {:?}", payed_amount);
	println!("Received amount is {:?}", payed_amount1);

	//Check that the Tenant is connected to the asset
	let asset = OnboardingModule::houses(coll_id0, item_id0).unwrap();
	let tenant_inf = pallet_roles::Pallet::<Test>::tenants(TENANT0).unwrap();

	assert_eq!(asset.tenants[0], TENANT0);
	assert_eq!(asset_account, tenant_inf.asset_account.unwrap());
	println!("the beginning of the contract is at block: {:?}", tenant_inf.contract_start)
}

fn next_block() {
	System::set_block_number(System::block_number() + 1);
	Scheduler::on_initialize(System::block_number());
	Democracy::on_initialize(System::block_number());
	VotingModule::on_initialize(System::block_number());
	Bidding::on_initialize(System::block_number());
	AssetManagement::on_initialize(System::block_number());
	AssetManagement::on_idle(System::block_number(), Weight::MAX);
}

fn fast_forward_to(n: u64) {
	while System::block_number() < n {
		next_block();
	}
}

//Helper for tenant infos
fn ten() -> Ident::IdentityInfo<MaxAdditionalFields> {
	IdentityInfo {
		additional: Default::default(),
		display: Ident::Data::Raw(b"ten".to_vec().try_into().unwrap()),
		legal: Ident::Data::Raw(b"The Right Ordinal Ten, Esq.".to_vec().try_into().unwrap()),
		web: Ident::Data::Raw(b"www.mystery.com".to_vec().try_into().unwrap()),
		riot: Default::default(),
		email: Default::default(),
		pgp_fingerprint: Default::default(),
		image: Default::default(),
		twitter: Default::default(),
	}
}

#[test]
fn test_00() {
	new_test_ext().execute_with(|| {
		let metadata0 = b"metadata0".to_vec().try_into().unwrap();
		let metadata1 = b"metadata1".to_vec().try_into().unwrap();

		//put some funds in FairSquare SlashFees account
		let fees_account = OnboardingModule::account_id();
		<Test as pallet::Config>::Currency::make_free_balance_be(&fees_account, 150_000u32.into());

		//Execute workflow up to TENANT0 connection to an asset
		let price1 = 450_000;
		prep_test(price1, metadata0, metadata1);
		let tenant_inf = pallet_roles::Pallet::<Test>::tenants(TENANT0).unwrap();

		//TENANT0 is now connected to an asset. let's check rent payment status
		let end_block = tenant_inf
			.clone()
			.contract_start
			.saturating_add(<Test as pallet_asset_management::Config>::RentCheck::get());
		fast_forward_to(end_block);
		println!("tenant_rent is: {:?}", tenant_inf.rent);
		let event = <frame_system::Pallet<Test>>::events()
			.pop()
			.expect("Expected at least one EventRecord to be found")
			.event;

		println!("\n\nrecent events:\n{:?}", event);
		next_block();

		let asset = tenant_inf.asset_account.clone();
		let virtual_initial_balance = Balances::free_balance(asset.unwrap());

		//TENANT0 pays the first rent
		assert_ok!(crate::Pallet::<Test>::pay_rent(Origin::signed(TENANT0)));
		let event = <frame_system::Pallet<Test>>::events()
			.pop()
			.expect("Expected at least one EventRecord to be found")
			.event;

		println!("\n\nrecent events:\n{:?}", event);

		//Let's check that rent transfer toward virtual account occured
		let virtual_balance = Balances::free_balance(&tenant_inf.asset_account.unwrap());
		let coll_id0 = NftColl::OFFICESTEST.value();
		let item_id0 = pallet_nft::ItemsCount::<Test>::get()[coll_id0 as usize] - 1;
		let asset_ownership = ShareDistributor::virtual_acc(coll_id0, item_id0).unwrap();
		let owners = asset_ownership.owners;
		let owner0 = &owners[0];
		let owner0_initial_balance = Balances::free_balance(owner0);

		assert_ne!(virtual_initial_balance, virtual_balance);
		assert_eq!(virtual_balance, virtual_initial_balance.saturating_add(tenant_inf.rent));

		//Let's check that owner0 got a transfer from the rent after block change
		let initial_block_number = System::block_number();
		let end_block = initial_block_number
			.saturating_add(<Test as pallet_asset_management::Config>::RentCheck::get());

		fast_forward_to(end_block);
		let owner0_balance = Balances::free_balance(owner0);

		assert_ne!(owner0_initial_balance, owner0_balance);
		println!(
			"\n\nOwner0 received:{:?}\nThe rent is:{:?}",
			owner0_balance.saturating_sub(owner0_initial_balance),
			tenant_inf.rent
		);
	})
}
