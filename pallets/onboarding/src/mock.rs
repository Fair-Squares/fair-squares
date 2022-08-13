use super::*;
use crate as pallet_onboarding;
use frame_support::traits::{ConstU16, ConstU64, ConstU32, AsEnsureOriginWithArg, EqualPrivilegeOnly};
use frame_support::{parameter_types, weights::Weight};
use frame_system;

use pallet_roles::GenesisBuild;
use pallet_collective::{Instance1, PrimeDefaultVote};
use frame_system::{EnsureRoot, EnsureSigned};
use sp_core::{crypto::AccountId32, H256};
use crate::Nft::{NftPermissions};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};
use pallet_collective;
use pallet_democracy;

type AccountId = AccountId32;
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type Balance = u64;
//helper types
pub type Acc = pallet_roles::Accounts;
pub type MaxProposals = u32;
pub type BlockNumber = u64;
pub type CollectionId = u32;
pub type ItemId = u32;
pub type NftColl = Nft::PossibleCollections;


// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		OnboardingModule: pallet_onboarding::{Pallet, Call, Storage, Event<T>},
		VotingModule: pallet_voting::{Pallet, Call, Storage, Event<T>},
		RoleModule: pallet_roles::{Pallet, Call, Storage, Event<T>},
		Sudo: pallet_sudo::{Pallet, Call, Storage, Event<T>},
		NftModule: pallet_nft::{Pallet, Call, Storage, Event<T>},
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Uniques: pallet_uniques::{Pallet, Call, Storage, Event<T>},
		Collective: pallet_collective::<Instance1>::{Pallet, Call, Event<T>, Origin<T>, Config<T>},
		Democracy: pallet_democracy::{Pallet, Call, Storage, Config<T>, Event<T>},
	}
);

parameter_types! {
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(1024);
}

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = BlockNumber;
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

parameter_types! {
    pub const CollectionDeposit: Balance = 10_000 ; // 1 UNIT deposit to create asset class
    pub const ItemDeposit: Balance = 100 ; // 1/100 UNIT deposit to create asset instance
    pub const KeyLimit: u32 = 32;	// Max 32 bytes per key
    pub const ValueLimit: u32 = 64;	// Max 64 bytes per value
    pub const UniquesMetadataDepositBase: Balance = 1000 ;
    pub const AttributeDepositBase: Balance = 100 ;
    pub const DepositPerByte: Balance = 10 ;
    pub const UniquesStringLimit: u32 = 32;

}

impl pallet_uniques::Config for Test {
    type Event = Event;
    type CollectionId = CollectionId;
    type ItemId = ItemId;
    type Currency = Balances;
    type ForceOrigin = EnsureRoot<AccountId>;
    type Locker = ();
    type CollectionDeposit = CollectionDeposit;
    type ItemDeposit = ItemDeposit;
    type MetadataDepositBase = UniquesMetadataDepositBase;
    type AttributeDepositBase = AttributeDepositBase;
    type DepositPerByte = DepositPerByte;
    type StringLimit = UniquesStringLimit;
    type KeyLimit = KeyLimit;
    type ValueLimit = ValueLimit;
    type WeightInfo = ();
    type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<AccountId>>;
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
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type MaxScheduledPerBlock = ();
	type WeightInfo = ();
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type PreimageProvider = ();
	type NoPreimagePostponement = ();
}

parameter_types!{
	pub const MaxProposal:MaxProposals = 7;
}
impl pallet_collective::Config<Instance1> for Test {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = ConstU64<3>;
	type MaxProposals = MaxProposal;
	type MaxMembers = MaxMembers;
	type DefaultVote = PrimeDefaultVote;
	type WeightInfo = ();
}

parameter_types! {
	pub const LaunchPeriod: BlockNumber = 5; //ok
	pub const VotingPeriod: BlockNumber = 5; //ok
	pub const FastTrackVotingPeriod: BlockNumber = 2; //ok
	pub const InstantAllowed: bool = true; //ok
	pub const MinimumDeposit: Balance = 100; //ok
	pub const EnactmentPeriod: BlockNumber = 5; //ok
	pub const CooloffPeriod: BlockNumber = 5; //ok
	pub const PreimageByteDeposit: Balance = 1; //ok
	pub const MaxVotes: u32 = 100;
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
    type MaxProposals = MaxProposal;
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


parameter_types! {
	pub ReserveCollectionIdUpTo: u32 = 3;
}
impl pallet_nft::Config for Test {
	type Event = Event;
	type WeightInfo = ();
	type NftCollectionId = CollectionId;
	type NftItemId = ItemId;
	type ProtocolOrigin = EnsureRoot<AccountId>;
	type Permissions = NftPermissions;
	type ReserveCollectionIdUpTo = ReserveCollectionIdUpTo;
}


impl pallet_voting::Config for Test {
	type Event = Event;
	type WeightInfo = ();
}



parameter_types! {
	pub const ProposalFee:u64 = 100;	
}

impl pallet_onboarding::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type Prop = Call;
	type ProposalFee = ProposalFee;
	type WeightInfo = ();
}

//---implementing pallet sudo---------
impl pallet_sudo::Config for Test {
	type Event = Event;
	type Call = Call;
}

parameter_types! {
	pub const MaxMembers:u32 =7;
}
impl pallet_roles::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type WeightInfo = ();
	type MaxMembers = MaxMembers;
}


pub const ALICE: AccountId = AccountId::new([1u8; 32]);
pub const BOB: AccountId = AccountId::new([2u8; 32]);
pub const CHARLIE: AccountId = AccountId::new([3u8; 32]);
pub const DAVE: AccountId = AccountId::new([6u8; 32]);
pub const EVE: AccountId = AccountId::new([5u8; 32]);
pub const ACCOUNT_WITH_NO_BALANCE0: AccountId = AccountId::new([4u8; 32]);

pub struct ExtBuilder;
impl Default for ExtBuilder {
    fn default() -> Self {
        ExtBuilder
    }
}

impl ExtBuilder {
    pub fn build(self) -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

        pallet_balances::GenesisConfig::<Test> {
            balances: vec![(ALICE, 200_000 ), (BOB, 200_000_000 ), (CHARLIE, 200_000_000 ), (DAVE, 150_000), (EVE, 150_000 )],
        }
        .assimilate_storage(&mut t)
        .unwrap();
        pallet_sudo::GenesisConfig::<Test> { key: Some(ALICE) }
		.assimilate_storage(&mut t)
		.unwrap();

        let mut ext = sp_io::TestExternalities::new(t);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }
}

pub fn expect_events(e: Vec<Event>) {
    e.into_iter().for_each(frame_system::Pallet::<Test>::assert_has_event);
}
