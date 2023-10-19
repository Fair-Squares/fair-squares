use crate as pallet_onboarding;
use super::*;
use frame_support::{
	parameter_types,assert_noop, assert_ok, ord_parameter_types,
	traits::{
		AsEnsureOriginWithArg,ConstU32,ConstU16, ConstU64, Contains, EqualPrivilegeOnly, OnInitialize, SortedMembers,
		StorePreimage,
	},
	weights::Weight,PalletId
};
use pallet_roles::BuildGenesisConfig;
use sp_core::{crypto::AccountId32, H256};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup,Verify,IdentifyAccount},
	BuildStorage,Percent,Perbill,MultiSignature
};
use frame_system::{EnsureRoot, EnsureSigned, EnsureSignedBy};

type Block = frame_system::mocking::MockBlock<Test>;
type AccountId = <AccountPublic as IdentifyAccount>::AccountId;
type AccountIdOf<Test> = <Test as frame_system::Config>::AccountId;
type Balance = u128;
pub type BlockNumber = u64;
pub type Signature = MultiSignature;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test 
	{
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
		RolesModule: pallet_roles::{Pallet, Call, Storage, Event<T>, Config<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Sudo:pallet_sudo::{Pallet, Call, Storage, Config<T>, Event<T>},
		OnboardingModule: pallet_onboarding::{Pallet, Call, Storage, Event<T>,Config<T>},
		NftModule: pallet_nft::{Pallet, Call, Storage, Event<T>, Config<T>},
		Collective: pallet_collective::<Instance2>::{Pallet, Call, Event<T>, Origin<T>, Config<T>},
		Nfts: pallet_nfts::{Pallet, Call, Storage, Event<T>},
		Utility: pallet_utility::{Pallet, Call, Event},
		Democracy: pallet_democracy::{Pallet, Call, Storage, Config<T>, Event<T>},
		Housing: pallet_housing_fund::{Pallet, Call, Storage, Event<T>},
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>},

	}
);

//helper types
pub type Acc = pallet_roles::Accounts;
#[derive(Eq,Copy,PartialEq,Clone)]
pub struct NftTestPermissions;


impl pallet_nft::NftPermission<Acc> for NftTestPermissions {
	fn can_create(created_by: &Acc) -> bool {
		matches!(*created_by, Acc::SERVICER)
	}

	fn can_mint(created_by: &Acc) -> bool {
		matches!(*created_by, Acc::SELLER)
	}

	fn can_burn(created_by: &Acc) -> bool {
		matches!(*created_by, Acc::SERVICER)
	}

	fn can_destroy(created_by: &Acc) -> bool {
		matches!(*created_by, Acc::SERVICER)
	}

	fn has_deposit(created_by: &Acc) -> bool {
		matches!(*created_by, Acc::SERVICER)
	}
}


pub type AccountPublic = <Signature as Verify>::Signer;

parameter_types! {
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(Weight::from_parts(1024_u64, 0));
}
impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
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

impl pallet_utility::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = ();
}

parameter_types! {
	pub const MaxMembers:u32 = 5;
	pub const MaxRoles:u32 = 3;
	pub const CheckPeriod: BlockNumber = 5;
}
impl pallet_roles::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type MaxMembers = MaxMembers;
	type MaxRoles= MaxRoles;
	type CheckPeriod = CheckPeriod;
	type BackgroundCouncilOrigin =
		pallet_collective::EnsureProportionAtLeast<Self::AccountId, BackgroundCollective, 1, 2>;
}

parameter_types! {
	pub const ProposalFee: Percent= Percent::from_percent(15);
	pub const SlashedFee: Percent = Percent::from_percent(10);
	pub const FeesAccount: PalletId = PalletId(*b"feeslash");
	pub const Delay:BlockNumber =5;
	pub const CheckDelay:BlockNumber =5;

}

impl pallet_onboarding::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Prop = RuntimeCall;
	type ProposalFee = ProposalFee;
	type Slash = SlashedFee;
	type WeightInfo = ();
	type FeesAccount = FeesAccount;
	type Delay = Delay;
	type CheckDelay = CheckDelay;
	type MinimumDeposit = MinimumDeposit;
}

parameter_types! {
	pub const MinContribution: u128 = 10;
	pub const FundThreshold: u128 = 2;
	pub const MaxFundContribution: u128 = 200;
	pub const HousingFundPalletId: PalletId = PalletId(*b"housfund");
	pub const MaxInvestorPerHouse: u32 = 2;
}

impl pallet_housing_fund::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type MinContribution = MinContribution;
	type FundThreshold = FundThreshold;
	type MaxFundContribution = MaxFundContribution;
	type PalletId = HousingFundPalletId;
	type MaxInvestorPerHouse = MaxInvestorPerHouse;
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) * BlockWeights::get().max_block;
}

impl pallet_scheduler::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type PalletsOrigin = OriginCaller;
	type RuntimeCall = RuntimeCall;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<Self::AccountId>;
	type MaxScheduledPerBlock = ConstU32<100>;
	type WeightInfo = ();
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type Preimages = ();
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
	type RuntimeEvent = RuntimeEvent;
	type Currency = pallet_balances::Pallet<Self>;
	type EnactmentPeriod = ConstU64<2>;
	type LaunchPeriod = ConstU64<2>;
	type VotingPeriod = ConstU64<2>;
	type VoteLockingPeriod = ConstU64<3>;
	type FastTrackVotingPeriod = ConstU64<2>;
	type MinimumDeposit = MinimumDeposit;
	type MaxDeposits = ConstU32<1000>;
	type MaxBlacklisted = ConstU32<5>;
	type SubmitOrigin = EnsureSigned<Self::AccountId>;
	type ExternalOrigin = EnsureRoot<Self::AccountId>;
	type ExternalMajorityOrigin = EnsureRoot<Self::AccountId>;
	type ExternalDefaultOrigin = EnsureRoot<Self::AccountId>;
	type FastTrackOrigin = EnsureRoot<Self::AccountId>;
	type CancellationOrigin = EnsureRoot<Self::AccountId>;
	type BlacklistOrigin = EnsureRoot<Self::AccountId>;
	type CancelProposalOrigin = EnsureRoot<Self::AccountId>;
	type VetoOrigin =  EnsureSigned<Self::AccountId>;
	type CooloffPeriod = ConstU64<2>;
	type Slash = ();
	type InstantOrigin = EnsureRoot<Self::AccountId>;
	type InstantAllowed = InstantAllowed;
	type Scheduler = Scheduler;
	type MaxVotes = ConstU32<100>;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = ();
	type MaxProposals = ConstU32<100>;
	type Preimages = ();
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = frame_system::Pallet<Test>;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
	type RuntimeHoldReason = ();
	type FreezeIdentifier = ();
	type MaxHolds = ();
	type MaxFreezes = ();
}

//---implementing pallet sudo---------
impl pallet_sudo::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type WeightInfo =();
}





parameter_types! {
	pub const BackgroundMotionDuration: BlockNumber = 5;
	pub const BackgroundMaxProposals: u32 = 100;
	pub const BackgroundMaxMembers: u32 = 100;
}

type BackgroundCollective = pallet_collective::Instance2;
impl pallet_collective::Config<BackgroundCollective> for Test {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = BackgroundMotionDuration;
	type MaxProposals = BackgroundMaxProposals;
	type MaxMembers = BackgroundMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = ();
	type SetMembersOrigin = EnsureRoot<Self::AccountId>;
	type MaxProposalWeight =();
}

parameter_types! {
	pub ReserveCollectionIdUpTo: u32 = 500;
}
impl pallet_nft::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	//type WeightInfo = pallet_nft::weights::SubstrateWeight<Runtime>;
	type NftCollectionId = u32;
	type NftItemId = u32;
	type ProtocolOrigin = EnsureRoot<AccountId>;
	type Permissions = NftTestPermissions;
	//type ReserveCollectionIdUpTo = ReserveCollectionIdUpTo;
}

use pallet_nfts::PalletFeatures;
parameter_types! {
	pub storage Features: PalletFeatures = PalletFeatures::all_enabled();
}

impl pallet_nfts::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type CollectionId = u32;
	type ItemId = u32;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<Self::AccountId>>;
	type ForceOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type Locker = ();
	type CollectionDeposit = ExistentialDeposit;
	type ItemDeposit = ExistentialDeposit;
	type MetadataDepositBase = ExistentialDeposit;
	type AttributeDepositBase = ExistentialDeposit;
	type DepositPerByte =ExistentialDeposit;
	type StringLimit = ConstU32<50>;
	type KeyLimit = ConstU32<50>;
	type ValueLimit = ConstU32<50>;
	type ApprovalsLimit = ConstU32<10>;
	type ItemAttributesApprovalsLimit = ConstU32<2>;
	type MaxTips = ConstU32<10>;
	type MaxDeadlineDuration = ConstU64<10000>;
	type MaxAttributesPerCall = ConstU32<2>;
	type Features = Features;
	/// Off-chain = signature On-chain - therefore no conversion needed.
	/// It needs to be From<MultiSignature> for benchmarking.
	type OffchainSignature = Signature;
	/// Using `AccountPublic` here makes it trivial to convert to `AccountId` via `into_account()`.
	type OffchainPublic = AccountPublic;
	type WeightInfo = ();
	#[cfg(feature = "runtime-benchmarks")]
	type Helper = ();
}

pub const ALICE:  AccountId = AccountId::new([1u8; 32]);
pub const BOB: AccountId = AccountId::new([2u8; 32]);
pub const CHARLIE: AccountId = AccountId::new([3u8; 32]);
pub const DAVE: AccountId = AccountId::new([6u8; 32]);
pub const EVE: AccountId = AccountId::new([5u8; 32]);
pub const BSX: Balance = 100_000_000_000;


pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

	pallet_balances::GenesisConfig::<Test> {
		balances: vec![
			(ALICE, 200_000_000),
			(BOB, 200_000_000),
			(CHARLIE, 200_000_000),
			(DAVE, 200_000_000),
			(EVE, 200_000_000),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	pallet_sudo::GenesisConfig::<Test> { key: Some(ALICE) }
		.assimilate_storage(&mut t)
		.unwrap();

	pallet_collective::GenesisConfig::<Test, pallet_collective::Instance2> {
		members: vec![ALICE, BOB, CHARLIE, DAVE],
		phantom: Default::default(),
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext

}



