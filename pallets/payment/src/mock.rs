use crate as pallet_payment;
use crate::PaymentDetail;
use frame_support::{
	weights::DispatchClass,
	parameter_types,
	traits::{ConstU16, ConstU64,ConstU32, Contains, Everything, GenesisBuild, Hooks, OnFinalize}
};
use frame_system as system;
use sp_core::{crypto::AccountId32, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	Percent,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
pub type Balance = u64;

pub type AccountId = AccountId32;
pub const PAYMENT_CREATOR: AccountId = AccountId::new([10u8; 32]);
pub const PAYMENT_RECIPENT: AccountId = AccountId::new([11u8; 32]);
pub const PAYMENT_CREATOR_TWO: AccountId = AccountId::new([30u8; 32]);
pub const PAYMENT_RECIPENT_TWO: AccountId = AccountId::new([31u8; 32]);
pub const RESOLVER_ACCOUNT: AccountId = AccountId::new([12u8; 32]);
pub const FEE_RECIPIENT_ACCOUNT: AccountId = AccountId::new([20u8; 32]);
pub const PAYMENT_RECIPENT_FEE_CHARGED: AccountId = AccountId::new([21u8; 32]);
pub const INCENTIVE_PERCENTAGE: u8 = 10;
pub const MARKETPLACE_FEE_PERCENTAGE: u8 = 10;
pub const CANCEL_BLOCK_BUFFER: u64 = 600;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Storage, Config,Event<T>},
		PaymentModule: pallet_payment::{Pallet, Call, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
	}
);

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}


pub struct MockDisputeResolver;
impl crate::types::DisputeResolver<AccountId> for MockDisputeResolver {
	fn get_resolver_account() -> AccountId {
		RESOLVER_ACCOUNT
	}
}

pub struct MockFeeHandler;
impl crate::types::FeeHandler<Test> for MockFeeHandler {
	fn apply_fees(
		_from: &AccountId,
		to: &AccountId,
		_detail: &PaymentDetail<Test>,
		_remark: Option<&[u8]>,
	) -> (AccountId, Percent) {
		match to {
			&PAYMENT_RECIPENT_FEE_CHARGED => (FEE_RECIPIENT_ACCOUNT, Percent::from_percent(MARKETPLACE_FEE_PERCENTAGE)),
			_ => (FEE_RECIPIENT_ACCOUNT, Percent::from_percent(0)),
		}
	}
}

parameter_types! {
	pub const IncentivePercentage: Percent = Percent::from_percent(INCENTIVE_PERCENTAGE);
	pub const MaxRemarkLength: u32 = 50;
	pub const CancelBufferBlockLength: u64 = CANCEL_BLOCK_BUFFER;
	pub const MaxScheduledTaskListLength : u32 = 5;
}

impl pallet_payment::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type DisputeResolver = MockDisputeResolver;
	type IncentivePercentage = IncentivePercentage;
	type FeeHandler = MockFeeHandler;
	type MaxRemarkLength = MaxRemarkLength;
	type CancelBufferBlockLength = CancelBufferBlockLength;
	type MaxScheduledTaskListLength = MaxScheduledTaskListLength;
	type WeightInfo = ();
}

//----implememting pallet_balances-----
impl pallet_balances::Config for Test {
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type MaxLocks = ConstU32<10>;
	type Balance = u64;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore = System;
	type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

		pallet_balances::GenesisConfig::<Test> {
			balances: vec![
				(PAYMENT_CREATOR, 200_000),
				(PAYMENT_CREATOR_TWO, 200_000_000),
			],
		}
		.assimilate_storage(&mut t)
		.unwrap();
		let mut ext: sp_io::TestExternalities = t.into();
	// need to set block number to 1 to test events
	ext.execute_with(|| System::set_block_number(1));
	ext
}

pub fn run_n_blocks(n: u64) -> u64 {
	const IDLE_WEIGHT: u64 = 10_000_000_000;
	const BUSY_WEIGHT: u64 = IDLE_WEIGHT / 1000;

	let start_block = System::block_number();

	for block_number in (0..=n).map(|n| n + start_block) {
		System::set_block_number(block_number);

		// Odd blocks gets busy
		let idle_weight = if block_number % 2 == 0 {
			IDLE_WEIGHT
		} else {
			BUSY_WEIGHT
		};
		// ensure the on_idle is executed
		<frame_system::Pallet<Test>>::register_extra_weight_unchecked(
			PaymentModule::on_idle(block_number, frame_support::weights::Weight::from_ref_time(idle_weight)),
			DispatchClass::Mandatory,
		);

		<frame_system::Pallet<Test> as OnFinalize<u64>>::on_finalize(block_number);
	}
	System::block_number()
}