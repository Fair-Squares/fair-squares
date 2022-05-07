use super::*;
use crate::mock::{Event, *};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::AccountId32;

fn initial_accounts() -> (AccountId32, AccountId32) {
	let alice_account: AccountId32 = AccountId32::from([
		0xd4, 0x35, 0x93, 0xc7, 0x15, 0xfd, 0xd3, 0x1c, 0x61, 0x14, 0x1a, 0xbd, 0x04, 0xa9, 0x9f,
		0xd6, 0x82, 0x2c, 0x85, 0x58, 0x85, 0x4c, 0xcd, 0xe3, 0x9a, 0x56, 0x84, 0xe7, 0xa5, 0x6d,
		0xa2, 0x7d,
	]);

	let bob_account: AccountId32 = AccountId32::from([
		0x51u8, 0x82u8, 0xa7u8, 0x3eu8, 0x48u8, 0xbdu8, 0x6eu8, 0x81u8, 0x4du8, 0x0cu8, 0x2bu8,
		0x41u8, 0x67u8, 0x2du8, 0x9cu8, 0xb8u8, 0xc8u8, 0x7cu8, 0x42u8, 0x21u8, 0xb5u8, 0x5bu8,
		0xc0u8, 0x8eu8, 0x09u8, 0x43u8, 0x19u8, 0x8eu8, 0x90u8, 0xcau8, 0xadu8, 0x1fu8,
	]);

	let start_wealth: u64 = (10 * CREATION_FEE + 10).into();
	let _ = Balances::deposit_creating(&alice_account, start_wealth);
	let _ = Balances::deposit_creating(&bob_account, start_wealth);

	run_to_block(1);
	assert_eq!(System::block_number(), 1);

	let _ = Balances::deposit_creating(&Pot::get(), 1);

	return (alice_account.clone(), bob_account.clone());
}

#[test]
// Test for general Event:
// CreatedClass(T::AccountId, ClassIdOf<T>)
// Test for Error:
// WrongClassType
// CreationFeeNotPaid
fn test_general_process() {
	new_test_ext().execute_with(|| {
		let (alice_account, _bob_account) = initial_accounts();

		let merkle_root = [
			0x0cu8, 0x67u8, 0xcau8, 0xf4u8, 0x61u8, 0x29u8, 0x0cu8, 0xd4u8, 0x63u8, 0xe5u8, 0x35u8,
			0x21u8, 0x3fu8, 0x99u8, 0x6eu8, 0x32u8, 0x73u8, 0x6eu8, 0x65u8, 0xa2u8, 0x06u8, 0x37u8,
			0x83u8, 0xfdu8, 0xe5u8, 0x03u8, 0x6bu8, 0x71u8, 0x39u8, 0x6du8, 0xfbu8, 0x0cu8,
		];

		// Simple: CreatedClass Event
		assert_ok!(Nft::create_class(
			Origin::signed(alice_account.clone()),
			CID::default(),
			Properties::default(),
			None,
			None,
			ClassType::Simple(100),
		));
		assert_eq!(
			events_filter::<crate::Event::<Test>>()[0],
			Event::Nft(crate::Event::CreatedClass(alice_account.clone(), 0)),
		);

		// Claim: CreatedClass Event
		assert_ok!(Nft::create_class(
			Origin::signed(alice_account.clone()),
			CID::default(),
			Properties(ClassProperty::Transferable | ClassProperty::Burnable),
			None,
			None,
			ClassType::Claim(merkle_root),
		));
		assert_eq!(
			events_filter::<crate::Event::<Test>>()[1],
			Event::Nft(crate::Event::CreatedClass(alice_account.clone(), 1)),
		);

		// Merge: CreatedClass Event
		assert_ok!(Nft::create_class(
			Origin::signed(alice_account.clone()),
			CID::default(),
			Properties(ClassProperty::Transferable | ClassProperty::Burnable),
			None,
			None,
			ClassType::Merge(0, 1, false),
		));
		assert_eq!(
			events_filter::<crate::Event::<Test>>()[2],
			Event::Nft(crate::Event::CreatedClass(alice_account.clone(), 2)),
		);

		//mint 5 instance of merge type token
		assert_noop!(
			Nft::mint(
				Origin::signed(alice_account.clone()),
				alice_account.clone(),
				2,
				CID::default(),
				5
			),
			NftError::WrongClassType
		);

		//CreationFeeNotPaid
		let random_account: AccountId32 = AccountId32::from([0u8; 32]);
		assert_noop!(
			Nft::create_class(
				Origin::signed(random_account.clone()),
				CID::default(),
				Properties(ClassProperty::Transferable | ClassProperty::Burnable),
				None,
				None,
				ClassType::Merge(0, 1, false),
			),
			NftError::CreationFeeNotPaid
		);
	})
}

#[test]
// Test for burn function:
// BurnedToken(T::AccountId, ClassIdOf<T>, TokenIdOf<T>)
// Test for Error:
// ClassIdNotFound
// TokenIdNotFound
// NonBurnable
// NoPermission
fn test_burn_process() {
	new_test_ext().execute_with(|| {
		let (alice_account, bob_account) = initial_accounts();

		// create Unburnable and Transferable class without start/end restrcition class id 0
		assert_ok!(Nft::create_class(
			Origin::signed(alice_account.clone()),
			CID::default(),
			Properties(ClassProperty::Transferable.into()),
			None,
			None,
			ClassType::Simple(100),
		));

		// create Burnable and Transferable class without start/end restrcition class id 1
		assert_ok!(Nft::create_class(
			Origin::signed(alice_account.clone()),
			CID::default(),
			Properties(ClassProperty::Transferable | ClassProperty::Burnable),
			None,
			None,
			ClassType::Simple(100),
		));

		//mint 5 instance class id 0, 1
		assert_ok!(Nft::mint(
			Origin::signed(alice_account.clone()),
			bob_account.clone(),
			0,
			CID::default(),
			5,
		));
		assert_ok!(Nft::mint(
			Origin::signed(alice_account.clone()),
			bob_account.clone(),
			1,
			CID::default(),
			5,
		));

		// burn non-exist class token
		assert_noop!(
			Nft::burn(Origin::signed(bob_account.clone()), (9, 0),),
			NftError::ClassIdNotFound
		);

		// burn unburnable class token
		assert_noop!(
			Nft::burn(Origin::signed(bob_account.clone()), (0, 0),),
			NftError::NonBurnable
		);

		//  burn exist class but non-exist burnable token
		assert_noop!(
			Nft::burn(Origin::signed(bob_account.clone()), (1, 10),),
			NftError::TokenNotFound
		);

		// someone else burn no-owned token
		assert_noop!(
			Nft::burn(Origin::signed(alice_account.clone()), (1, 0),),
			NftError::NoPermission
		);

		// normal burned
		assert_ok!(Nft::burn(Origin::signed(bob_account.clone()), (1, 0),));
		assert_eq!(
			events_filter::<crate::Event::<Test>>()[4],
			Event::Nft(crate::Event::BurnedToken(bob_account.clone(), 1, 0)),
		);
	});
}

#[test]
// Test for transfer function:
// TransferredToken(T::AccountId, ClassIdOf<T>, TokenIdOf<T>)
// Test for Error:
// ClassIdNotFound
// TokenNotFound
// NonBurnable
// NoPermission
fn test_transfer_process() {
	new_test_ext().execute_with(|| {
		let (alice_account, bob_account) = initial_accounts();

		// create Burnable and unTransferable class without start/end restrcition class id 0
		assert_ok!(Nft::create_class(
			Origin::signed(alice_account.clone()),
			CID::default(),
			Properties(ClassProperty::Burnable.into()),
			None,
			None,
			ClassType::Simple(100),
		));

		// create Burnable and Transferable class without start/end restrcition class id 1
		assert_ok!(Nft::create_class(
			Origin::signed(alice_account.clone()),
			CID::default(),
			Properties(ClassProperty::Transferable | ClassProperty::Burnable),
			None,
			None,
			ClassType::Simple(100),
		));

		//mint 5 instance class id 0, 1
		assert_ok!(Nft::mint(
			Origin::signed(alice_account.clone()),
			bob_account.clone(),
			0,
			CID::default(),
			5,
		));
		assert_ok!(Nft::mint(
			Origin::signed(alice_account.clone()),
			bob_account.clone(),
			1,
			CID::default(),
			5,
		));

		// transfer non-exist class token
		assert_noop!(
			Nft::transfer(Origin::signed(bob_account.clone()), alice_account.clone(), (9, 0),),
			NftError::ClassIdNotFound
		);

		// transfer untransferable class token
		assert_noop!(
			Nft::transfer(Origin::signed(bob_account.clone()), alice_account.clone(), (0, 0),),
			NftError::NonTransferable
		);

		//  transfer exist class but non-exist token
		assert_noop!(
			Nft::transfer(Origin::signed(bob_account.clone()), alice_account.clone(), (1, 10),),
			// If transfer an non-exist token id, the error will be from orml_nft, who's Event or Error is not defined
			orml_nft::Error::<Test>::TokenNotFound
		);

		// someone else transfer no-owned token
		assert_noop!(
			Nft::transfer(Origin::signed(alice_account.clone()), bob_account.clone(), (1, 0),),
			// If transfer an no-permission token id, the error will be from orml_nft, who's Event or Error is not defined
			orml_nft::Error::<Test>::NoPermission
		);

		// normal transfer
		assert_ok!(Nft::transfer(
			Origin::signed(bob_account.clone()),
			alice_account.clone(),
			(1, 0),
		));
		assert_eq!(
			events_filter::<crate::Event::<Test>>()[4],
			Event::Nft(crate::Event::TransferredToken(
				bob_account.clone(),
				alice_account.clone(),
				1,
				0
			)),
		);
	});
}

#[test]
// Test for Simple type Event:
// MintedToken(T::AccountId, T::AccountId, ClassIdOf<T>, u32)
// Test for Error:
// ClassIdNotFound
// NoPermission,
// InvalidQuantity,
// QuantityOverflow
// OutOfCampaignPeriod
fn test_minted_token_process() {
	new_test_ext().execute_with(|| {
		let (alice_account, bob_account) = initial_accounts();
		// create Transferable Unburnable class without start/end restrcition
		assert_ok!(Nft::create_class(
			Origin::signed(alice_account.clone()),
			CID::default(),
			Properties(ClassProperty::Transferable.into()),
			None,
			None,
			ClassType::Simple(10),
		));

		//mint class Id=1 non-exist class
		assert_noop!(
			Nft::mint(
				Origin::signed(alice_account.clone()),
				bob_account.clone(),
				1,
				CID::default(),
				1,
			),
			NftError::ClassIdNotFound
		);

		//mint 0 invalid instance quantity
		assert_noop!(
			Nft::mint(
				Origin::signed(alice_account.clone()),
				bob_account.clone(),
				0,
				CID::default(),
				0,
			),
			NftError::InvalidQuantity
		);

		//mint 11 exceed the maximum instance limit
		assert_noop!(
			Nft::mint(
				Origin::signed(alice_account.clone()),
				bob_account.clone(),
				0,
				CID::default(),
				11,
			),
			NftError::QuantityOverflow
		);

		//mint 5 instance with right ClassInfo owner
		assert_ok!(Nft::mint(
			Origin::signed(alice_account.clone()),
			bob_account.clone(),
			0,
			CID::default(),
			5,
		));
		assert_eq!(
			events_filter::<crate::Event::<Test>>()[1],
			Event::Nft(crate::Event::MintedToken(
				alice_account.clone(),
				bob_account.clone(),
				0,
				0,
				5
			))
		);

		//mint 5 instance with wrong ClassInfo owner
		assert_noop!(
			Nft::mint(
				Origin::signed(bob_account.clone()),
				bob_account.clone(),
				0,
				CID::default(),
				5,
			),
			NftError::NoPermission
		);

		// create Transferable Unburnable class with start/end restrcition
		assert_ok!(Nft::create_class(
			Origin::signed(alice_account.clone()),
			CID::default(),
			Properties(ClassProperty::Transferable | ClassProperty::Burnable),
			10.into(),
			100.into(),
			ClassType::Simple(100),
		));

		run_to_block(2);
		assert_eq!(System::block_number(), 2);

		//mint 5 instance out of time
		assert_noop!(
			Nft::mint(
				Origin::signed(alice_account.clone()),
				bob_account.clone(),
				1,
				CID::default(),
				5,
			),
			NftError::OutOfCampaignPeriod
		);

		run_to_block(11);
		assert_eq!(System::block_number(), 11);
		//mint 5 instance within time
		assert_ok!(Nft::mint(
			Origin::signed(alice_account.clone()),
			bob_account.clone(),
			1,
			CID::default(),
			5,
		));
		assert_eq!(
			events_filter::<crate::Event::<Test>>()[3],
			Event::Nft(crate::Event::MintedToken(
				alice_account.clone(),
				bob_account.clone(),
				1,
				0,
				5
			))
		);
	});
}

#[test]
// Test for Claim type Event:
// ClaimedToken(T::AccountId, ClassIdOf<T>)
// Test for Error:
// OutOfCampaignPeriod
// NonTransferable,
// ClassClaimedListNotFound
// UserNotInClaimList
// TokenAlreadyClaimed
fn test_claimed_token_process() {
	new_test_ext().execute_with(|| {
		let (alice_account, bob_account) = initial_accounts();

		// root is 0x0c67caf461290cd463e535213f996e32736e65a2063783fde5036b71396dfb0c
		let merkle_root = [
			0x0cu8, 0x67u8, 0xcau8, 0xf4u8, 0x61u8, 0x29u8, 0x0cu8, 0xd4u8, 0x63u8, 0xe5u8, 0x35u8,
			0x21u8, 0x3fu8, 0x99u8, 0x6eu8, 0x32u8, 0x73u8, 0x6eu8, 0x65u8, 0xa2u8, 0x06u8, 0x37u8,
			0x83u8, 0xfdu8, 0xe5u8, 0x03u8, 0x6bu8, 0x71u8, 0x39u8, 0x6du8, 0xfbu8, 0x0cu8,
		];

		// proof of alice is 0xd8b63c7168eef1bc3b00cdf73d1636429a26ab607b52f1de073b1f53edd9302d
		let alice_proof = vec![[
			0xd8u8, 0xb6u8, 0x3cu8, 0x71u8, 0x68u8, 0xeeu8, 0xf1u8, 0xbcu8, 0x3bu8, 0x00u8, 0xcdu8,
			0xf7u8, 0x3du8, 0x16u8, 0x36u8, 0x42u8, 0x9au8, 0x26u8, 0xabu8, 0x60u8, 0x7bu8, 0x52u8,
			0xf1u8, 0xdeu8, 0x07u8, 0x3bu8, 0x1fu8, 0x53u8, 0xedu8, 0xd9u8, 0x30u8, 0x2du8,
		]];

		// issue a claim class : class id, 0
		assert_ok!(Nft::create_class(
			Origin::signed(alice_account.clone()),
			CID::default(),
			Properties(ClassProperty::Burnable.into()),
			10.into(),
			100.into(),
			ClassType::Claim(merkle_root),
		));

		// fake simple NFT : class id, 1
		assert_ok!(Nft::create_class(
			Origin::signed(alice_account.clone()),
			CID::default(),
			Properties(ClassProperty::Transferable | ClassProperty::Burnable),
			10.into(),
			100.into(),
			ClassType::Simple(100),
		));

		run_to_block(2);
		assert_eq!(System::block_number(), 2);

		// alice claim out of time
		assert_noop!(
			Nft::claim(Origin::signed(alice_account.clone()), 0, 0, alice_proof.clone(),),
			NftError::OutOfCampaignPeriod
		);

		run_to_block(11);
		assert_eq!(System::block_number(), 11);

		// alice claims with random proof
		assert_noop!(
			Nft::claim(Origin::signed(alice_account.clone()), 0, 0, vec![[0u8; 32]],),
			NftError::UserNotInClaimList
		);

		// Claim non-existed type
		assert_noop!(
			Nft::claim(Origin::signed(alice_account.clone()), 0, 1, alice_proof.clone(),),
			NftError::ClassClaimedListNotFound //WrongClassType // should we raise this error first??? This can never be triggered.
		);

		// alice claims with alice's proof
		assert_ok!(Nft::claim(Origin::signed(alice_account.clone()), 0, 0, alice_proof.clone(),));
		assert_eq!(
			events_filter::<crate::Event::<Test>>()[2],
			Event::Nft(crate::Event::ClaimedToken(alice_account.clone(), 0, 0))
		);

		// alice claims again
		assert_noop!(
			Nft::claim(Origin::signed(alice_account.clone()), 0, 0, alice_proof,),
			NftError::TokenAlreadyClaimed
		);

		// alice transfer non-transferable token class id 0
		assert_noop!(
			Nft::transfer(Origin::signed(alice_account.clone()), bob_account.clone(), (0, 1)),
			NftError::NonTransferable
		);
	})
}

#[test]
// Test for Merge type Event:
// MergedToken(T::AccountId, ClassIdOf<T>)
// TransferredToken(T::AccountId, T::AccountId, ClassIdOf<T>, TokenIdOf<T>)
// Test for Error:
// WrongMergeBase
// NonBurnable
// OutOfCampaignPeriod
// TokenNotFound
// TokenUsed
fn test_merged_token_process() {
	new_test_ext().execute_with(|| {
		let (alice_account, bob_account) = initial_accounts();

		// issue basic unburnable NFTs : class id 0, 1
		assert_ok!(Nft::create_class(
			Origin::signed(alice_account.clone()),
			CID::default(),
			Properties(ClassProperty::Transferable.into()),
			None,
			None,
			ClassType::Simple(10),
		));

		assert_ok!(Nft::create_class(
			Origin::signed(alice_account.clone()),
			CID::default(),
			Properties(ClassProperty::Transferable.into()),
			None,
			None,
			ClassType::Simple(10),
		));

		// issue basic burnable NFTs : class id 2, 3
		assert_ok!(Nft::create_class(
			Origin::signed(alice_account.clone()),
			CID::default(),
			Properties(ClassProperty::Transferable | ClassProperty::Burnable),
			None,
			None,
			ClassType::Simple(10),
		));

		assert_ok!(Nft::create_class(
			Origin::signed(alice_account.clone()),
			CID::default(),
			Properties(ClassProperty::Transferable | ClassProperty::Burnable),
			None,
			None,
			ClassType::Simple(10),
		));

		// mint unburnable NFTs  : class id 0
		assert_ok!(Nft::mint(
			Origin::signed(alice_account.clone()),
			bob_account.clone(),
			0,
			CID::default(),
			10,
		));
		assert_eq!(
			events_filter::<crate::Event::<Test>>()[4],
			Event::Nft(crate::Event::MintedToken(
				alice_account.clone(),
				bob_account.clone(),
				0,
				0,
				10
			))
		);

		// mint unburnable NFTs  : class id 1
		assert_ok!(Nft::mint(
			Origin::signed(alice_account.clone()),
			bob_account.clone(),
			1,
			CID::default(),
			10,
		));
		assert_eq!(
			events_filter::<crate::Event::<Test>>()[5],
			Event::Nft(crate::Event::MintedToken(
				alice_account.clone(),
				bob_account.clone(),
				1,
				0,
				10
			))
		);

		// mint burnable NFTs  : class id 2, 3
		assert_ok!(Nft::mint(
			Origin::signed(alice_account.clone()),
			bob_account.clone(),
			2,
			CID::default(),
			10,
		));
		assert_ok!(Nft::mint(
			Origin::signed(alice_account.clone()),
			bob_account.clone(),
			3,
			CID::default(),
			10,
		));

		// issue advanced NFTs
		assert_noop!(
			Nft::create_class(
				Origin::signed(alice_account.clone()),
				CID::default(),
				Properties(ClassProperty::Transferable | ClassProperty::Burnable),
				10.into(),
				100.into(),
				ClassType::Merge(0, 1, true),
			),
			NftError::NonBurnable
		);

		// issue base unburn merge NFTs : class id 4
		assert_ok!(Nft::create_class(
			Origin::signed(alice_account.clone()),
			CID::default(),
			Properties(ClassProperty::Transferable | ClassProperty::Burnable),
			10.into(),
			100.into(),
			ClassType::Merge(2, 3, false),
		));

		// issue base burn merge NFTs: class id 5
		assert_ok!(Nft::create_class(
			Origin::signed(alice_account.clone()),
			CID::default(),
			Properties(ClassProperty::Transferable | ClassProperty::Burnable),
			10.into(),
			100.into(),
			ClassType::Merge(2, 3, true),
		));

		//---------------------------------------------//
		//---unburn merge NFT--------------------------//
		// merge out of time
		assert_noop!(
			Nft::merge(Origin::signed(bob_account.clone()), 4, (2, 9), (3, 9),),
			NftError::OutOfCampaignPeriod
		);

		run_to_block(11);
		assert_eq!(System::block_number(), 11);

		// merge existed but wrong base class type
		assert_noop!(
			Nft::merge(Origin::signed(bob_account.clone()), 4, (0, 11), (1, 11),),
			NftError::WrongMergeBase
		);

		// merge existed class but non-existed token
		assert_noop!(
			Nft::merge(Origin::signed(bob_account.clone()), 4, (2, 11), (3, 11),),
			NftError::TokenNotFound
		);

		// merge existed class and existed token
		assert_ok!(Nft::merge(Origin::signed(bob_account.clone()), 4, (2, 9), (3, 9),));
		assert_eq!(
			events_filter::<crate::Event::<Test>>()[10],
			Event::Nft(crate::Event::MergedToken(bob_account.clone(), 4, 0))
		);

		// merge existed class and existed token again for used token
		assert_noop!(
			Nft::merge(Origin::signed(bob_account.clone()), 4, (2, 9), (3, 9),),
			NftError::TokenUsed
		);

		//---------------------//
		//---burn merge NFT--------------------------//
		assert_ok!(Nft::merge(Origin::signed(bob_account.clone()), 5, (2, 9), (3, 9),));

		// merge will generate burn event
		assert_eq!(
			events_filter::<crate::Event::<Test>>()[11],
			Event::Nft(crate::Event::BurnedToken(bob_account.clone(), 2, 9)),
		);
		assert_eq!(
			events_filter::<crate::Event::<Test>>()[12],
			Event::Nft(crate::Event::BurnedToken(bob_account.clone(), 3, 9)),
		);
		// merge event
		assert_eq!(
			events_filter::<crate::Event::<Test>>()[13],
			Event::Nft(crate::Event::MergedToken(bob_account.clone(), 5, 0))
		);

		// check the owner of burned token is account #0
		let random_account: AccountId32 = AccountId32::from([0u8; 32]);
		assert_eq!(Nft::owner((2, 9)).unwrap_or(random_account.clone()), random_account);

		// transfer class id=2 token id=8 to account #0
		assert_ok!(Nft::transfer(
			Origin::signed(bob_account.clone()),
			random_account.clone(),
			(2, 8)
		));
		assert_eq!(
			events_filter::<crate::Event::<Test>>()[14],
			Event::Nft(crate::Event::TransferredToken(
				bob_account.clone(),
				random_account.clone(),
				2,
				8
			))
		);
		assert_eq!(Nft::owner((2, 8)).unwrap_or(random_account.clone()), random_account);
	});
}
