use crate as pallet_template;
use frame_support::traits::{ConstU16,ConstU32,ConstU8,ConstU64};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};
use primitives::constants::currency::UNITS;
pub use sp_runtime::{Perbill, Permill};
use frame_system::{EnsureSignedBy, EnsureRoot, EnsureSigned};
use primitives::{
	nft::{ClassType, NftPermissions},
	ClassId, InstanceId,
};
use common_runtime::{wnft};
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
		TemplateModule: pallet_template::{Pallet, Call, Storage, Event<T>},
		Scheduler: pallet_scheduler,
		Balances: pallet_balances,
		Democracy: pallet_democracy,
		Uniques: pallet_uniques::{Pallet, Call, Storage, Event<T>},
		NFT: pallet_nft::{Pallet, Call, Event<T>, Storage},
	}
);

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u64;
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
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}
//---pallet-uniques implementation----//
frame_support::parameter_types! {
	pub const ClassDeposit: u64 = 100 * 1_000_000; // 100 UNITS deposit to create asset class
	pub const InstanceDeposit: u64 = 100 * 1_000_000; // 100 UNITS deposit to create asset instance
	pub const KeyLimit: u32 = 256;	// Max 256 bytes per key
	pub const ValueLimit: u32 = 1024;	// Max 1024 bytes per value
	pub const UniquesMetadataDepositBase: u64 = 100 * 1_000_000;
	pub const AttributeDepositBase: u64 = 10 * 1_000_000;
	pub const DepositPerByte: u64 = 1_000_000;
	pub const UniquesStringLimit: u32 = 60;
}

impl pallet_uniques::Config for Test {
	type Event = Event;
	type ClassId = ClassId;
	type InstanceId = InstanceId;
	type Currency = Balances;
	type ForceOrigin = EnsureRoot<u64>;
	type ClassDeposit = ClassDeposit;
	type InstanceDeposit = InstanceDeposit;
	type MetadataDepositBase = UniquesMetadataDepositBase;
	type AttributeDepositBase = AttributeDepositBase;
	type DepositPerByte = DepositPerByte;
	type StringLimit = UniquesStringLimit;
	type KeyLimit = KeyLimit;
	type ValueLimit = ValueLimit;
	type CreateOrigin = frame_support::traits::AsEnsureOriginWithArg<EnsureSigned<u64>>;
	type WeightInfo = ();
}
//---END-------//

//-----pallet-nft implementation------//
frame_support::parameter_types! {
	pub ReserveClassIdUpTo: u128 = 999_999;
}

impl pallet_nft::Config for Test {
	type Currency = Balances;
	type Event = Event;
	type WeightInfo = wnft::BasiliskWeight<Test>;
	type NftClassId = ClassId;
	type NftInstanceId = InstanceId;
	type ProtocolOrigin = EnsureRoot<u64>;
	type ClassType = ClassType;
	type Permissions = NftPermissions;
	type ReserveClassIdUpTo = ReserveClassIdUpTo;
}
//-----END----//

//--pallet-balances implementation-----//
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

//pallet-scheduler is need bcause of pallet-democracy
frame_support::parameter_types! {
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(1_000_000);
}
frame_support::parameter_types! {
	pub MaximumSchedulerWeight: frame_support::weights::Weight = Perbill::from_percent(80) * BlockWeights::get().max_block;
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
	type OriginPrivilegeCmp = frame_support::traits::EqualPrivilegeOnly;
	type PreimageProvider = ();
	type NoPreimagePostponement = ();
}
//-------END-----------//

//---pallet-democracy------//
pub struct OneToFive;
impl frame_support::traits::SortedMembers<u64> for OneToFive {
	fn sorted_members() -> Vec<u64> {
		vec![1, 2, 3, 4, 5]
	}
	#[cfg(feature = "runtime-benchmarks")]
	fn add(_m: &u64) {}
}
frame_support::ord_parameter_types! {
	pub const One: u64 = 1;
	pub const Two: u64 = 2;
	pub const Three: u64 = 3;
	pub const Four: u64 = 4;
	pub const Five: u64 = 5;
	pub const Six: u64 = 6;
}

frame_support::parameter_types! {
	pub static PreimageByteDeposit: u64 = 0;
	pub static InstantAllowed: bool = false;
}


impl pallet_democracy::Config for Test {
	type Proposal = Call;
	type Event = Event;
	type Currency = pallet_balances::Pallet<Self>;
	type EnactmentPeriod = ConstU64<2>;
	type LaunchPeriod = ConstU64<2>;
	type VotingPeriod = ConstU64<2>;
	type VoteLockingPeriod = ConstU64<3>;
	type FastTrackVotingPeriod = ConstU64<2>;
	type MinimumDeposit = ConstU64<1>;
	type ExternalOrigin = EnsureSignedBy<Two, u64>;
	type ExternalMajorityOrigin = EnsureSignedBy<Three, u64>;
	type ExternalDefaultOrigin = EnsureSignedBy<One, u64>;
	type FastTrackOrigin = EnsureSignedBy<Five, u64>;
	type CancellationOrigin = EnsureSignedBy<Four, u64>;
	type BlacklistOrigin = EnsureRoot<u64>;
	type CancelProposalOrigin = EnsureRoot<u64>;
	type VetoOrigin = EnsureSignedBy<OneToFive, u64>;
	type CooloffPeriod = ConstU64<2>;
	type PreimageByteDeposit = PreimageByteDeposit;
	type Slash = ();
	type InstantOrigin = EnsureSignedBy<Six, u64>;
	type InstantAllowed = InstantAllowed;
	type Scheduler = Scheduler;
	type MaxVotes = ConstU32<100>;
	type OperationalPreimageOrigin = EnsureSignedBy<Six, u64>;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = ();
	type MaxProposals = ConstU32<100>;
}
//----END-----//

//---pallet-template--//
frame_support::parameter_types! {
	pub const SubmissionDeposit: u64 = 10;
	pub const MinContribution: u64 = 10;

}

impl pallet_template::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type MinContribution = MinContribution;
	type SubmissionDeposit = SubmissionDeposit;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}
