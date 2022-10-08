pub use super::*;
pub use crate::mock::*;
pub use frame_support::{assert_noop, assert_ok};
use frame_system::pallet_prelude::OriginFor;

pub type Bvec<Test> = BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit>;

pub fn prep_roles() {
	RoleModule::set_role(Origin::signed(CHARLIE), CHARLIE, Acc::SERVICER).ok();
	RoleModule::account_approval(Origin::signed(ALICE), CHARLIE).ok();
	RoleModule::set_role(Origin::signed(BOB), BOB, Acc::SELLER).ok();
	RoleModule::account_approval(Origin::signed(ALICE), BOB).ok();
	RoleModule::set_role(Origin::signed(DAVE), DAVE, Acc::INVESTOR).ok();
	RoleModule::set_role(Origin::signed(EVE), EVE, Acc::INVESTOR).ok();
	RoleModule::set_role(
		Origin::signed(ACCOUNT_WITH_NO_BALANCE0),
		ACCOUNT_WITH_NO_BALANCE0,
		Acc::SERVICER,
	)
	.ok();
	RoleModule::account_approval(Origin::signed(ALICE), ACCOUNT_WITH_NO_BALANCE0).ok();
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
	assert_ok!(HousingFund::Pallet::<Test>::contribute_to_fund(Origin::signed(DAVE), 50_000));
	assert_ok!(HousingFund::Pallet::<Test>::contribute_to_fund(Origin::signed(EVE), 50_000));

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
		let fees_account = Onboarding::Pallet::<Test>::account_id();
		<Test as pallet::Config>::Currency::make_free_balance_be(&fees_account, 150_000u32.into());

		let price1 = 40_000;
		let price2 = 30_000;
		prep_test(price1, price2, metadata0, metadata1, metadata2);
		let coll_id0 = NftColl::OFFICESTEST.value();
		let item_id0 = pallet_nft::ItemsCount::<Test>::get()[coll_id0 as usize] - 1;
		let origin: OriginFor<Test> = frame_system::RawOrigin::Root.into();
		let origin2 = Origin::signed(BOB);

		//Change first asset status to FINALISED
		Onboarding::Pallet::<Test>::change_status(
			origin2.clone(),
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
		let fund_op = HousingFund::FundOperation {
			nft_collection_id: coll_id0,
			nft_item_id: item_id0,
			amount: price1,
			block_number: 1,
			contributions: vec![(EVE, 25_000), (DAVE, 15_000)],
		};
		let id = ShareDistributor::virtual_acc(coll_id0, item_id0).unwrap().token_id;
		//Add new owners and asset to housing fund
		HousingFund::Reservations::<Test>::insert((coll_id0, item_id0), fund_op);
		println!("Reservations {:?}", HousingFund::Reservations::<Test>::get((coll_id0, item_id0)));
		println!("Virtual Account {:?}", ShareDistributor::virtual_acc(coll_id0, item_id0));

		//Create token
		assert_ok!(ShareDistributor::create_tokens(origin, coll_id0, item_id0, new_owner0.clone()));
		assert_eq!(1, ShareDistributor::token_id());
		assert_eq!(0, ShareDistributor::virtual_acc(coll_id0, item_id0).unwrap().token_id);
		assert_eq!(1000, Assets::Pallet::<Test>::total_supply(id));
		//Check that new_owner0 is in possession of 1000 tokens
		assert_eq!(1000, Assets::Pallet::<Test>::balance(id, new_owner0.clone()));
		//Distribute token
		assert_ok!(ShareDistributor::distribute_tokens(new_owner0.clone(), coll_id0, item_id0));
		let balance0 = Assets::Pallet::<Test>::balance(id, DAVE);
		let balance1 = Assets::Pallet::<Test>::balance(id, EVE);

		let _infos = ShareDistributor::tokens_infos(new_owner0.clone()).unwrap().owners;
		println!("Tokens own by DAVE:{:?}\nTokens own by Eve:{:?}", balance0, balance1);
		println!("Total supply {:?}", Assets::Pallet::<Test>::total_supply(id));

		// Bob creates a second proposal without submiting for review
		let coll_id1 = NftColl::APPARTMENTSTEST.value();
		let item_id1 = pallet_nft::ItemsCount::<Test>::get()[coll_id1 as usize] - 1;

		//Store initial owner
		let old_owner1 = pallet_nft::Pallet::<Test>::owner(coll_id1, item_id1).unwrap();

		//Change first asset status to FINALISED
		Onboarding::Pallet::<Test>::change_status(
			origin2,
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
		let virtual0 = Virtual::<Test>::get(coll_id0, item_id0).unwrap();
		let virtual1 = Virtual::<Test>::get(coll_id1, item_id1).unwrap();

		//Check that virtual accounts are different
		println!("Virtual account nbr1:{:?}\nVirtual account nbr2:{:?}", virtual0, virtual1);
		assert_ne!(virtual0.virtual_account, virtual1.virtual_account);
		//Check that virtual accounts are the new owners
		assert_eq!(new_owner0, virtual0.virtual_account);
		assert_eq!(new_owner1, virtual1.virtual_account);
	});
}

#[test]
fn share_distributor1() {
	ExtBuilder::default().build().execute_with(|| {
		let metadata0 = b"metadata0".to_vec().try_into().unwrap();
		let metadata1 = b"metadata1".to_vec().try_into().unwrap();
		let metadata2 = b"metadata2".to_vec().try_into().unwrap();
		//put some funds in FairSquare SlashFees account
		let fees_account = Onboarding::Pallet::<Test>::account_id();
		<Test as pallet::Config>::Currency::make_free_balance_be(&fees_account, 150_000u32.into());

		let price1 = 40_000;
		let price2 = 30_000;
		prep_test(price1, price2, metadata0, metadata1, metadata2);
		let coll_id0 = NftColl::OFFICESTEST.value();
		let item_id0 = pallet_nft::ItemsCount::<Test>::get()[coll_id0 as usize] - 1;
		let origin: OriginFor<Test> = frame_system::RawOrigin::Root.into();
		let origin2 = Origin::signed(BOB);

		let contribution_eve = HousingFund::Contribution {
			account_id: EVE,
			available_balance: HousingFund::Pallet::<Test>::u64_to_balance_option(10_000).unwrap(),
			reserved_balance: HousingFund::Pallet::<Test>::u64_to_balance_option(25_000).unwrap(),
			contributed_balance: HousingFund::Pallet::<Test>::u64_to_balance_option(0).unwrap(),
			has_withdrawn: false,
			block_number: 1,
			contributions: vec![HousingFund::ContributionLog {
				amount: HousingFund::Pallet::<Test>::u64_to_balance_option(15_000).unwrap(),
				block_number: 1,
			}],
			withdraws: Vec::new(),
		};

		let contribution_dave = HousingFund::Contribution {
			account_id: DAVE,
			available_balance: HousingFund::Pallet::<Test>::u64_to_balance_option(10_000).unwrap(),
			reserved_balance: HousingFund::Pallet::<Test>::u64_to_balance_option(15_000).unwrap(),
			contributed_balance: HousingFund::Pallet::<Test>::u64_to_balance_option(0).unwrap(),
			has_withdrawn: false,
			block_number: 1,
			contributions: vec![HousingFund::ContributionLog {
				amount: HousingFund::Pallet::<Test>::u64_to_balance_option(25_000).unwrap(),
				block_number: 1,
			}],
			withdraws: Vec::new(),
		};

		// Add contributions to storage
		HousingFund::Contributions::<Test>::insert(EVE, contribution_eve);
		HousingFund::Contributions::<Test>::insert(DAVE, contribution_dave);

		//Create a FundOperation struct for this asset
		let fund_op = HousingFund::FundOperation {
			nft_collection_id: coll_id0,
			nft_item_id: item_id0,
			amount: price1,
			block_number: 1,
			contributions: vec![(EVE, 25_000), (DAVE, 15_000)],
		};

		//Add new owners and asset to housing fund
		HousingFund::Reservations::<Test>::insert((coll_id0, item_id0), fund_op);

		// Update the Housing fund to fit with the contributions
		HousingFund::FundBalance::<Test>::mutate(|val| {
			*val = HousingFund::FundInfo { total: 60_000, transferable: 20_000, reserved: 40_000 };
		});

		//Change first asset status to FINALISED
		Onboarding::Pallet::<Test>::change_status(
			origin2,
			NftColl::OFFICESTEST,
			item_id0,
			Onboarding::AssetStatus::FINALISED,
		)
		.ok();

		//Store initial owner
		let old_owner0 = pallet_nft::Pallet::<Test>::owner(coll_id0, item_id0).unwrap();

		assert_ok!(ShareDistributor::create_virtual(origin, coll_id0, item_id0));
		let when = <frame_system::Pallet<Test>>::block_number();
		let new_owner0 = ShareDistributor::virtual_acc(coll_id0, item_id0).unwrap().virtual_account;
		let owners = ShareDistributor::virtual_acc(coll_id0, item_id0).unwrap().owners;
		let id = ShareDistributor::virtual_acc(coll_id0, item_id0).unwrap().token_id;
		println!("The token id is:{:?}", id.clone());
		assert_eq!(1000, Assets::Pallet::<Test>::total_supply(id));

		let balance0 = Assets::Pallet::<Test>::balance(id, DAVE);
		let balance1 = Assets::Pallet::<Test>::balance(id, EVE);
		let balance2 = Assets::Pallet::<Test>::balance(id, new_owner0.clone());

		println!(
			"Tokens own by DAVE:{:?}\nTokens own by Eve:{:?}\nTokens own by Virtual_account:{:?}",
			balance0, balance1, balance2
		);
		let infos = ShareDistributor::tokens_infos(new_owner0.clone()).unwrap().owners;

		println!("Again, new owners are:\n{:?}", infos);

		assert_eq!(owners.len() > 1, true);
		expect_events(vec![
			crate::Event::VirtualCreated {
				account: new_owner0.clone(),
				collection: coll_id0,
				item: item_id0,
				when,
			}
			.into(),
			crate::Event::NftTransactionExecuted {
				nft_transfer_to: new_owner0,
				nft_transfer_from: old_owner0,
				when,
			}
			.into(),
		]);
	})
}
