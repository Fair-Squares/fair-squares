use crate as pallet_bidding;
use frame_support::{
	parameter_types,
	traits::{AsEnsureOriginWithArg, ConstU16, ConstU64, EqualPrivilegeOnly},
	weights::Weight,
	PalletId,
};
use frame_system as system;
use frame_system::{EnsureRoot, EnsureSigned};
use pallet_collective::{Instance1, PrimeDefaultVote};
use pallet_nft::NftPermissions;
use pallet_roles::GenesisBuild;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type CouncilCollective = pallet_collective::Instance1;
type Block = frame_system::mocking::MockBlock<Test>;
pub(crate) type Balance = u128;
pub type CollectionId = u32;
pub type ItemId = u32;
pub type NftCollection = pallet_nft::PossibleCollections;
type AccountId = u64;
pub type MaxProposals = u32;
pub type BlockNumber = u64;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		BiddingModule: pallet_bidding::{Pallet, Call, Storage, Event<T>},
		OnboardingModule: pallet_onboarding::{Pallet, Call, Storage, Event<T>},
		NftModule: pallet_nft::{Pallet, Call, Storage, Event<T>},
		Uniques: pallet_uniques::{Pallet, Call, Storage, Event<T>},
		HousingFund: pallet_housing_fund::{Pallet, Call, Storage, Event<T>},
		Sudo: pallet_sudo::{Pallet, Call, Storage, Event<T>},
		RoleModule: pallet_roles::{Pallet, Call, Storage, Event<T>},
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>},
		Collective: pallet_collective::<Instance1>::{Pallet, Call, Event<T>, Origin<T>, Config<T>},
		Democracy: pallet_democracy::{Pallet, Call, Storage, Config<T>, Event<T>},
		VotingModule: pallet_voting::{Pallet, Call, Storage, Event<T>},
		Assets: pallet_assets::{Pallet, Storage, Config<T>, Event<T>},
		ShareDistributorModule: pallet_share_distributor::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(1024);
}

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
	type AccountData = pallet_balances::AccountData<u128>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub static ExistentialDeposit: Balance = 1;
}
impl pallet_balances::Config for Test {
	type MaxLocks = frame_support::traits::ConstU32<1024>;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
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

parameter_types! {
	pub const ProposalFee:u64 = 5;
	pub const FeesAccount: PalletId = PalletId(*b"feeslash");
}

impl pallet_onboarding::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type Prop = Call;
	type ProposalFee = ProposalFee;
	type WeightInfo = ();
	type FeesAccount = FeesAccount;
}

parameter_types! {
	pub const MinContribution: u64 = 5;
	pub const FundThreshold: u64 = 100;
	pub const MaxFundContribution: u64 = 20;
	pub const MaxInvestorPerHouse: u32 = 10;
	pub const HousingFundPalletId: PalletId = PalletId(*b"housfund");
}

/// Configure the pallet-housing_fund in pallets/housing_fund.
impl pallet_housing_fund::Config for Test {
	type Event = Event;
	type LocalCurrency = Balances;
	type MinContribution = MinContribution;
	type FundThreshold = FundThreshold;
	type MaxFundContribution = MaxFundContribution;
	type WeightInfo = pallet_housing_fund::weights::SubstrateWeight<Test>;
	type PalletId = HousingFundPalletId;
	type MaxInvestorPerHouse = MaxInvestorPerHouse;
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

parameter_types! {
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
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 2>;
	type MinimumDepositVote = MinimumDeposit;
	type CheckPeriod = CheckPeriod;
}

parameter_types! {
	pub const AssetDeposit: u64 = 100 ;
	pub const ApprovalDeposit: u64 = 1 ;
	pub const MetadataDepositPerByte: u64 = 1 ;
	pub const StringLimit: u32 = 50;
	pub const MetadataDepositBase: u64 = 1000 ;
	pub const AssetAccountDeposit: u64 = 1;
}

impl pallet_assets::Config for Test {
	type Event = Event;
	type Balance = u32;
	type AssetId = u32;
	type Currency = Balances;
	type ForceOrigin = EnsureRoot<AccountId>;
	type AssetDeposit = AssetDeposit;
	type AssetAccountDeposit = AssetAccountDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = StringLimit;
	type Freezer = ();
	type Extra = ();
	type WeightInfo = ();
}

parameter_types! {
	pub const AssetsFees: Balance = 20000;
}
impl pallet_share_distributor::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type AssetId = u32;
	type Fees = AssetsFees;
}

parameter_types! {
	pub const SimultaneousAssetBidder: u64 = 1;
	pub const MaxTriesBid: u64 = 3;
	pub const MaxTriesAseemblingInvestor: u64 = 3;
	pub const MaximumSharePerInvestor: u64 = 20;
	pub const MinimumSharePerInvestor: u64 = 10;
	pub const NewAssetScanPeriod: u64 = 20;
}

impl pallet_bidding::Config for Test {
	type Event = Event;
	type WeightInfo = ();
	type Currency = Balances;
	type SimultaneousAssetBidder = SimultaneousAssetBidder;
	type MaxTriesBid = MaxTriesBid;
	type MaxTriesAseemblingInvestor = MaxTriesAseemblingInvestor;
	type MaximumSharePerInvestor = MaximumSharePerInvestor;
	type MinimumSharePerInvestor = MinimumSharePerInvestor;
	type NewAssetScanPeriod = NewAssetScanPeriod;
}

pub const ALICE: u64 = 1;
pub const BOB: u64 = 2;
pub const CHARLIE: u64 = 3;
pub const DAVE: u64 = 4;
pub const EVE: u64 = 5;
pub const AMANI: u64 = 6;
pub const KEZIA: u64 = 7;
pub const DAN: u64 = 8;

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	pallet_balances::GenesisConfig::<Test> {
		balances: vec![
			(ALICE, 200_000),
			(BOB, 200_000_000),
			(CHARLIE, 200_000_000),
			(DAVE, 150_000),
			(EVE, 150_000),
			(AMANI, 150_000),
			(KEZIA, 150_000),
			(DAN, 150_000),
		],
	}
	.assimilate_storage(&mut storage)
	.unwrap();

	pallet_sudo::GenesisConfig::<Test> { key: Some(ALICE) }
		.assimilate_storage(&mut storage)
		.unwrap();

	pallet_collective::GenesisConfig::<Test, pallet_collective::Instance1> {
		members: vec![ALICE, BOB, CHARLIE, DAVE],
		phantom: Default::default(),
	}
	.assimilate_storage(&mut storage)
	.unwrap();

	let mut externalities = sp_io::TestExternalities::new(storage);
	externalities.execute_with(|| System::set_block_number(1));
	externalities
}
