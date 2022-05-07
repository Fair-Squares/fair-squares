use super::*;
use crate as nft;
use frame_support::{
	assert_noop, assert_ok, parameter_types,
	traits::{OnFinalize, OnInitialize},
};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	generic,
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32,
};
use sp_std::any::{Any, TypeId};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		OrmlNFT: orml_nft::{Pallet, Storage, Config<T>},
		Nft: nft::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u32 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
	type BaseCallFilter = ();
	type Origin = Origin;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Call = Call;
	type Index = u32;
	type BlockNumber = u32;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId32;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = generic::Header<Self::BlockNumber, BlakeTwo256>;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
	pub const MaxLocks: u32 = 10;
}

impl pallet_balances::Config for Test {
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type MaxLocks = MaxLocks;
	type Balance = u64;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

parameter_types! {
	pub const ClassCreationFee: u32 = CREATION_FEE;
	pub const Pot: AccountId32 = AccountId32::new([9u8; 32]);
}

impl nft::Config for Test {
	type Currency = Balances;
	type Event = Event;
	type WeightInfo = ();
	type ClassCreationFee = ClassCreationFee;
	type Pot = Pot;
}

parameter_types! {
	pub const MaxClassMetadata: u32 = 1024;
	pub const MaxTokenMetadata: u32 = 1024;
}

impl orml_nft::Config for Test {
	type ClassId = u32;
	type TokenId = u64;
	type ClassData = ClassData<BlockNumberOf<Self>, ClassIdOf<Self>>;
	type TokenData = TokenData;
	type MaxClassMetadata = MaxClassMetadata;
	type MaxTokenMetadata = MaxTokenMetadata;
}

pub type NftError = nft::Error<Test>;

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}

pub fn run_to_block(n: u32) {
	while System::block_number() < n {
		<Nft as OnFinalize<u32>>::on_finalize(System::block_number());
		<System as OnFinalize<u32>>::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		<System as OnInitialize<u32>>::on_initialize(System::block_number());
		<Nft as OnInitialize<u32>>::on_initialize(System::block_number());
	}
}

/// Put Event type as T, this method filters the system events storage accordingly
///
/// Type Parameters:
/// - `T`: Event/ pallet Event type
/// - Example: TypeId::of::<Event> : global event, it will not reject anything and always return true.
/// 		   Event::System : Event of frame_system
/// 		   Event::Balances : Event of pallet_balances
///            Event::Nft: Event ofself crate
/// 		   _ : return empty vector
pub fn events_filter<T: 'static>() -> Vec<Event> {
	let mut evt = System::events();

	evt.retain(|evt| if_right_events::<T>(&evt.event));
	return evt.into_iter().map(|evt| evt.event).collect::<Vec<_>>();
}

/// return true if Event is an instance of T
///
/// Parameters:
/// - `evt`: Event
/// Type Parameters:
/// - `T`: Event/ pallet Event type
/// - Example: TypeId::of::<Event> : global event, it will not reject anything and always return true.
/// 		   Event::System : Event of frame_system
/// 		   Event::Balances : Event of pallet_balances
///            Event::Nft: Event ofself crate
/// 		   
/// Ormal_NFT is also tested but no imported Event so far.
pub fn if_right_events<T: 'static>(evt: &Event) -> bool {
	if TypeId::of::<T>() == TypeId::of::<Event>() {
		return true;
	} else {
		match evt {
			Event::System(i) => return if_right_raw_events::<T>(i),
			Event::Balances(i) => return if_right_raw_events::<T>(i),
			Event::Nft(i) => return if_right_raw_events::<T>(i),
		}
	}
}

/// return true if s is an instance of T
///
/// Parameters:
/// - `s`: Any
/// Type Parameters:
/// - `T`: type
pub fn if_right_raw_events<T: 'static>(s: &dyn Any) -> bool {
	if let Some(_) = s.downcast_ref::<T>() {
		true
	} else {
		false
	}
}

#[test]
fn check_test_helper() {
	let evt = Event::System(frame_system::Event::NewAccount(AccountId32::from([0u8; 32])));
	assert_eq!(if_right_events::<frame_system::Event::<Test>>(&evt), true);

	let evt = Event::Balances(pallet_balances::Event::<Test>::Transfer(
		AccountId32::from([0u8; 32]),
		AccountId32::from([1u8; 32]),
		CREATION_FEE.into(),
	));
	assert_eq!(if_right_events::<pallet_balances::Event::<Test>>(&evt), true);

	let evt = Event::Nft(crate::Event::CreatedClass(AccountId32::from([0u8; 32]), 0));
	assert_eq!(if_right_events::<crate::Event::<Test>>(&evt), true);
}

#[test]
fn demostration_of_event_filter() {
	new_test_ext().execute_with(|| {
		let alice_account: AccountId32 = AccountId32::from([
			0xd4, 0x35, 0x93, 0xc7, 0x15, 0xfd, 0xd3, 0x1c, 0x61, 0x14, 0x1a, 0xbd, 0x04, 0xa9,
			0x9f, 0xd6, 0x82, 0x2c, 0x85, 0x58, 0x85, 0x4c, 0xcd, 0xe3, 0x9a, 0x56, 0x84, 0xe7,
			0xa5, 0x6d, 0xa2, 0x7d,
		]);

		run_to_block(1);
		assert_eq!(System::block_number(), 1);

		// give balance to Alice
		let _ = Balances::deposit_creating(&alice_account, (CREATION_FEE + 10).into());
		// issue a simple class
		assert_ok!(Nft::create_class(
			Origin::signed(alice_account.clone()),
			CID::default(),
			Properties::default(),
			None,
			None,
			ClassType::Simple(100),
		));

		// <frame_system::Event::<Test>> type argument: give the events belong to frame_system only
		assert_eq!(
			events_filter::<frame_system::Event::<Test>>(),
			[
				Event::System(frame_system::Event::NewAccount(alice_account.clone())),
				Event::System(frame_system::Event::NewAccount(Pot::get())),
			]
		);

		// <pallet_balances::Event::<Test>> type argument: give the events belong to pallet_balances only
		assert_eq!(
			events_filter::<pallet_balances::Event::<Test>>(),
			[
				Event::Balances(pallet_balances::Event::<Test>::Endowed(
					alice_account.clone(),
					(CREATION_FEE + 10).into()
				)),
				Event::Balances(pallet_balances::Event::<Test>::Endowed(
					Pot::get(),
					CREATION_FEE.into()
				)),
				Event::Balances(pallet_balances::Event::<Test>::Transfer(
					alice_account.clone(),
					Pot::get(),
					CREATION_FEE.into()
				)),
			]
		);

		// <crate::Event::<Test>> type argument: which in our case, crate is our nft crate, give the events belong to self-design events only
		assert_eq!(
			events_filter::<crate::Event::<Test>>(),
			[Event::Nft(crate::Event::CreatedClass(alice_account.clone(), 0)),]
		);

		// <Event> argument: Event is the general type, give all events
		assert_eq!(
			events_filter::<Event>(),
			[
				Event::System(frame_system::Event::NewAccount(alice_account.clone())),
				Event::Balances(pallet_balances::Event::<Test>::Endowed(
					alice_account.clone(),
					(CREATION_FEE + 10).into()
				)),
				Event::System(frame_system::Event::NewAccount(Pot::get())),
				Event::Balances(pallet_balances::Event::<Test>::Endowed(
					Pot::get(),
					CREATION_FEE.into()
				)),
				Event::Balances(pallet_balances::Event::<Test>::Transfer(
					alice_account.clone(),
					Pot::get(),
					CREATION_FEE.into()
				)),
				Event::Nft(crate::Event::CreatedClass(alice_account.clone(), 0)),
			]
		);

		// get_vector provide event display on index level. negative index will display reversed order element's reference.
		assert_eq!(
			events_filter::<Event>()[4],
			Event::Balances(pallet_balances::Event::<Test>::Transfer(
				alice_account.clone(),
				Pot::get(),
				CREATION_FEE.into()
			)),
		);
	})
}
