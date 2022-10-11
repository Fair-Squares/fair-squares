use super::*;
use crate as pallet_voting;
use frame_support::{
	parameter_types,
	traits::{ConstU32, ConstU64, EqualPrivilegeOnly},
};

use frame_support::pallet_prelude::Weight;
use frame_system::{EnsureRoot, EnsureSigned};
use pallet_collective::PrimeDefaultVote;
use pallet_roles::GenesisBuild;

use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};

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
		VotingModule: pallet_voting::{Pallet, Call, Storage, Event<T>},
		Collective: pallet_collective::<Instance1>::{Pallet, Call, Event<T>, Origin<T>, Config<T>},
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>},
		Democracy: pallet_democracy::{Pallet, Call, Storage, Config<T>, Event<T>},
		RoleModule: pallet_roles::{Pallet, Call, Storage, Event<T>},
		Sudo:pallet_sudo::{Pallet, Call, Storage, Config<T>, Event<T>},
	}
);

pub type MaxMembers = ConstU32<100>;
pub type BlockNumber = u64;
pub type Balance = u128;

parameter_types! {
	pub const MotionDuration: u64 = 2;
	pub const MaxProposals: u32 = 100;
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(1024);
}

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Call = Call;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
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
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

impl pallet_sudo::Config for Test {
	type Event = Event;
	type Call = Call;
}

impl pallet_roles::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type WeightInfo = ();
	type MaxMembers = MaxMembers;
}

parameter_types! {
	pub const Delay: BlockNumber = 0;//3 * MINUTES;
	pub const CheckDelay: BlockNumber = 1;//3 * MINUTES;
	pub const InvestorVoteAmount: u128 = 1;
	pub const CheckPeriod: BlockNumber = 1;
}

impl pallet_voting::Config for Test {
	type Event = Event;
	type Call = Call;
	type WeightInfo = ();
	type Delay = Delay;
	type InvestorVoteAmount = InvestorVoteAmount;
	type LocalCurrency = Balances;
	type CheckDelay = CheckDelay;
	type HouseCouncilOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountIdOf<Test>, CouncilCollective, 1, 2>;
	type MinimumDepositVote = MinimumDeposit;
	type CheckPeriod = CheckPeriod;
}

parameter_types! {
	pub const CouncilMotionDuration: BlockNumber = 2;
}

type CouncilCollective = pallet_collective::Instance1;
impl COLL::Config<Instance1> for Test {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = MaxProposals;
	type MaxMembers = MaxMembers;
	type DefaultVote = PrimeDefaultVote;
	type WeightInfo = ();
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) * BlockWeights::get().max_block;
}
impl pallet_scheduler::Config for Test {
	type Event = Event;
	type Origin = Origin;
	type PalletsOrigin = OriginCaller;
	type Call = Call;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<u64>;
	type MaxScheduledPerBlock = ();
	type WeightInfo = ();
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type PreimageProvider = ();
	type NoPreimagePostponement = ();
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 1;
}
impl pallet_balances::Config for Test {
	type MaxReserves = ();
	type ReserveIdentifier = ();
	type MaxLocks = ();
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

parameter_types! {
	pub const LaunchPeriod: BlockNumber = 1; //ok
	pub const VotingPeriod: BlockNumber = 5; //ok
	pub const FastTrackVotingPeriod: BlockNumber = 20; //ok
	pub const InstantAllowed: bool = true; //ok
	pub const MinimumDeposit: Balance = 1; //ok
	pub const EnactmentPeriod: BlockNumber = 200; //ok
	pub const CooloffPeriod: BlockNumber = 200; //ok
	pub const PreimageByteDeposit: Balance = 10; //ok
	pub const MaxVotes: u32 = 4;
}

impl pallet_democracy::Config for Test {
	type Proposal = Call;
	type Event = Event;
	type Currency = Balances;
	type EnactmentPeriod = EnactmentPeriod; //ok
	type LaunchPeriod = LaunchPeriod; //ok
	type VotingPeriod = VotingPeriod; //ok
	type VoteLockingPeriod = EnactmentPeriod; //ok
	type MinimumDeposit = MinimumDeposit; //ok
	type ExternalOrigin = EnsureRoot<Self::AccountId>;
	type ExternalMajorityOrigin = EnsureRoot<Self::AccountId>;
	type ExternalDefaultOrigin = EnsureRoot<Self::AccountId>;
	type FastTrackOrigin = EnsureRoot<Self::AccountId>;
	type InstantOrigin = EnsureRoot<Self::AccountId>;
	type InstantAllowed = InstantAllowed; //ok
	type FastTrackVotingPeriod = FastTrackVotingPeriod; //ok
	type CancellationOrigin = EnsureRoot<Self::AccountId>;
	type BlacklistOrigin = EnsureRoot<Self::AccountId>;
	type CancelProposalOrigin = EnsureRoot<Self::AccountId>;
	type VetoOrigin = EnsureSigned<Self::AccountId>;
	type CooloffPeriod = CooloffPeriod; //ok
	type PreimageByteDeposit = PreimageByteDeposit; //ok
	type OperationalPreimageOrigin = EnsureSigned<Self::AccountId>;
	type Slash = ();
	type Scheduler = Scheduler;
	type PalletsOrigin = OriginCaller;
	type MaxVotes = MaxVotes; //ok
	type WeightInfo = ();
	type MaxProposals = MaxProposals;
}

pub const ALICE: u64 = 1;
pub const BOB: u64 = 2;
pub const CHARLIE: u64 = 3;
pub const DAVE: u64 = 4;
pub const EVE: u64 = 5;

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	// frame_system::GenesisConfig::default().build_storage::<Test>().unwrap().into()

	let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	// Initialize balances
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![
			(ALICE, 200_000),
			(BOB, 200_000_000),
			(CHARLIE, 200_000_000),
			(DAVE, 150_000),
			(EVE, 150_000),
		],
	}
	.assimilate_storage(&mut storage)
	.unwrap();

	pallet_collective::GenesisConfig::<Test, pallet_collective::Instance1> {
		members: vec![ALICE, BOB, CHARLIE, DAVE],
		phantom: Default::default(),
	}
	.assimilate_storage(&mut storage)
	.unwrap();

	pallet_sudo::GenesisConfig::<Test> { key: Some(ALICE) }
		.assimilate_storage(&mut storage)
		.unwrap();

	let mut externalities = sp_io::TestExternalities::new(storage);
	externalities.execute_with(|| System::set_block_number(1));
	externalities
}
