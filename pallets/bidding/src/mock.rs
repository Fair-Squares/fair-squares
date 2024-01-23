use super::*;
use crate as pallet_bidding;
use frame_support::{
	parameter_types,ord_parameter_types,
	traits::{ConstU16,ConstU32,ConstU64,EqualPrivilegeOnly,SortedMembers,AsEnsureOriginWithArg},
	weights::Weight,
};
use sp_core:: H256;
use sp_io::storage;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup,IdentifyAccount,Verify},
	BuildStorage, MultiSignature,
};
use sp_runtime::Perbill;
use pallet_nfts::PalletFeatures;
use frame_system::{EnsureRoot,EnsureSigned,EnsureSignedBy};
use pallet_assets::AssetsCallback;
type Block = frame_system::mocking::MockBlock<Test>;
pub type Signature = MultiSignature;
pub type AccountPublic = <Signature as Verify>::Signer;
pub type AccountId = <AccountPublic as IdentifyAccount>::AccountId;
type Balance = u64;
pub type BlockNumber = u64;
type AssetId = u32;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test 
	{
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
		RolesModule: pallet_roles::{Pallet, Call, Storage, Event<T>, Config<T>},
		BiddingModule: pallet_bidding::{Pallet, Call, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Sudo:pallet_sudo::{Pallet, Call, Storage, Config<T>, Event<T>},
		CollectiveFlip: pallet_insecure_randomness_collective_flip::{Pallet, Storage},
		ShareDistributor: pallet_share_distributor::{Pallet, Call, Storage,  Event<T>},
		NftModule: pallet_nft::{Pallet, Call, Storage, Event<T>, Config<T>},
		Nfts: pallet_nfts::{Pallet, Call, Storage, Event<T>},
		HousingFund: pallet_housing_fund::{Pallet, Call, Storage, Event<T>},
		Democracy: pallet_democracy::{Pallet, Call, Storage, Config<T>, Event<T>},
		OnboardingModule: pallet_onboarding::{Pallet, Call, Storage, Event<T>, Config<T>},
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>},
		Utility: pallet_utility,
		Assets: pallet_assets::{Pallet, Storage, Config<T>, Event<T>},
		Collective: pallet_collective::<Instance2>::{Pallet, Call, Event<T>, Origin<T>, Config<T>},
		Preimage: pallet_preimage,
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

parameter_types! {
	pub ReserveCollectionIdUpTo: u32 = 45;
}

parameter_types!{
	pub const MaxGenerateRandom:u32 =60;
	pub const MinContributionper: Percent= Percent::from_percent(5);
	pub const MaxContributionper: Percent= Percent::from_percent(30);
	pub const NewAssetScanPeriod: u32 = 5;
}

impl pallet_bidding::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type MaxGenerateRandom = MaxGenerateRandom;
	type Randomness = CollectiveFlip;
	type MinContributionper = MinContributionper;
	type MaxContributionper = MaxContributionper;
	type NewAssetScanPeriod = NewAssetScanPeriod;
}

impl pallet_utility::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = ();
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

impl pallet_preimage::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type Currency = Balances;
	type ManagerOrigin = EnsureRoot<AccountId>;
	type Consideration = ();
}

impl pallet_scheduler::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type PalletsOrigin = OriginCaller;
	type RuntimeCall = RuntimeCall;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type MaxScheduledPerBlock = ();
	type WeightInfo = ();
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type Preimages = ();
}

parameter_types! {
	pub storage Features: PalletFeatures = PalletFeatures::all_enabled();
}

parameter_types! {
	pub static PreimageByteDeposit: u64 = 0;
	pub static InstantAllowed: bool = false;
}
ord_parameter_types! {
	pub const One: u64 = 1;
	pub const Two: u64 = 2;
	pub const Three: u64 = 3;
	pub const Four: u64 = 4;
	pub const Five: u64 = 5;
	pub const Six: u64 = 6;
}


impl pallet_democracy::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = pallet_balances::Pallet<Self>;
	type EnactmentPeriod = ConstU64<2>;
	type LaunchPeriod = ConstU64<2>;
	type VotingPeriod = ConstU64<2>;
	type VoteLockingPeriod = ConstU64<3>;
	type FastTrackVotingPeriod = ConstU64<2>;
	type MinimumDeposit = ConstU64<1>;
	type MaxDeposits = ConstU32<1000>;
	type MaxBlacklisted = ConstU32<5>;
	type SubmitOrigin = EnsureSigned<Self::AccountId>;
	type ExternalOrigin = EnsureRoot<AccountId>;
	type ExternalMajorityOrigin = EnsureRoot<AccountId>;
	type ExternalDefaultOrigin = EnsureRoot<AccountId>;
	type FastTrackOrigin = EnsureRoot<AccountId>;
	type CancellationOrigin = EnsureRoot<AccountId>;
	type BlacklistOrigin = EnsureRoot<AccountId>;
	type CancelProposalOrigin = EnsureRoot<AccountId>;
	type VetoOrigin = EnsureSigned<Self::AccountId>;
	type CooloffPeriod = ConstU64<2>;
	type Slash = ();
	type InstantOrigin = EnsureRoot<AccountId>;
	type InstantAllowed = InstantAllowed;
	type Scheduler = Scheduler;
	type MaxVotes = ConstU32<100>;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = ();
	type MaxProposals = ConstU32<100>;
	type Preimages = Preimage;
}

impl pallet_nfts::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type CollectionId = u32;
	type ItemId = u32;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<Self::AccountId>>;
	type ForceOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type Locker = ();
	type CollectionDeposit = ConstU64<2>;
	type ItemDeposit = ConstU64<1>;
	type MetadataDepositBase = ConstU64<1>;
	type AttributeDepositBase = ConstU64<1>;
	type DepositPerByte = ConstU64<1>;
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


impl pallet_nft::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	//type WeightInfo = pallet_nft::weights::SubstrateWeight<Runtime>;
	type NftCollectionId = u32;
	type NftItemId = u32;
	type ProtocolOrigin = EnsureRoot<AccountId>;
	type Permissions = NftTestPermissions;
	type MaxItems = ConstU32<7>;
	//type ReserveCollectionIdUpTo = ReserveCollectionIdUpTo;
}



parameter_types! {
	pub const ProposalFee: Percent= Percent::from_percent(15);
	pub const SlashedFee: Percent = Percent::from_percent(10);
	pub const FeesAccount: PalletId = PalletId(*b"feeslash");
	pub const Delay: BlockNumber = 0;
	pub const CheckDelay: BlockNumber = 0;
	pub const MinimumDeposit: Balance = 100; //ok
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

impl pallet_insecure_randomness_collective_flip::Config for Test {}

parameter_types! {
	pub const CollectionDeposit: Balance = 10_000 * BSX; // 1 UNIT deposit to create asset class
	pub const ItemDeposit: Balance = 100 * BSX; // 1/100 UNIT deposit to create asset instance
	pub const KeyLimit: u32 = 32;	// Max 32 bytes per key
	pub const ValueLimit: u32 = 64;	// Max 64 bytes per value
	pub const UniquesMetadataDepositBase: Balance = 1000 * BSX;
	pub const AttributeDepositBase: Balance = 100 * BSX;
	pub const DepositPerByte: Balance = 10 * BSX;
	pub const UniquesStringLimit: u32 = 32;
}



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

parameter_types! {
	pub const MaxMembers:u32 = 5;
	#[derive(Clone)]
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



parameter_types!{
	pub const AssetsFees: Balance = 25 ;
	pub const MaxOwners:u32 = 20; 
}

impl pallet_share_distributor::Config for Test{
	type RuntimeEvent = RuntimeEvent;
	type AssetId = AssetId;
	type Fees = AssetsFees;
	type MaxOwners = MaxOwners;
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
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
	type RuntimeHoldReason = ();
	type FreezeIdentifier = ();
	type MaxHolds = ();
	type MaxFreezes = ();
	type RuntimeFreezeReason = ();
}
parameter_types! {
	pub const AssetDeposit: u64 = 100 ;
	pub const ApprovalDeposit: u64 = 1 ;
	pub const MetadataDepositPerByte: u64 = 1 ;
	pub const StringLimit: u32 = 50;
	pub const MetadataDepositBase: u64 = 1000 ;
	pub const AssetAccountDeposit: u64 = 1;
}
pub struct AssetsCallbackHandle;
impl AssetsCallback<AssetId, AccountId> for AssetsCallbackHandle {
	fn created(_id: &AssetId, _owner: &AccountId) -> Result<(), ()> {
		if Self::should_err() {
			Err(())
		} else {
			storage::set(Self::CREATED.as_bytes(), &().encode());
			Ok(())
		}
	}

	fn destroyed(_id: &AssetId) -> Result<(), ()> {
		if Self::should_err() {
			Err(())
		} else {
			storage::set(Self::DESTROYED.as_bytes(), &().encode());
			Ok(())
		}
	}
}

impl AssetsCallbackHandle {
	pub const CREATED: &'static str = "asset_created";
	pub const DESTROYED: &'static str = "asset_destroyed";

	const RETURN_ERROR: &'static str = "return_error";

	// Configures `Self` to return `Ok` when callbacks are invoked
	pub fn set_return_ok() {
		storage::clear(Self::RETURN_ERROR.as_bytes());
	}

	// Configures `Self` to return `Err` when callbacks are invoked
	pub fn set_return_error() {
		storage::set(Self::RETURN_ERROR.as_bytes(), &().encode());
	}

	// If `true`, callback should return `Err`, `Ok` otherwise.
	fn should_err() -> bool {
		storage::exists(Self::RETURN_ERROR.as_bytes())
	}
}


impl pallet_assets::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Balance = u64;
	type AssetId = u32;
	type AssetIdParameter = u32;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<AccountId>>;
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type AssetDeposit = ConstU64<1>;
	type AssetAccountDeposit = ConstU64<10>;
	type MetadataDepositBase = ConstU64<1>;
	type MetadataDepositPerByte = ConstU64<1>;
	type ApprovalDeposit = ConstU64<1>;
	type StringLimit = ConstU32<50>;
	type Freezer = ();
	type WeightInfo = ();
	type CallbackHandle = AssetsCallbackHandle;
	type Extra = ();
	type RemoveItemsLimit = ConstU32<5>;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
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


pub const ALICE: AccountId = AccountId::new([1u8; 32]);
pub const BOB: AccountId = AccountId::new([2u8; 32]);
pub const CHARLIE: AccountId = AccountId::new([3u8; 32]);
pub const DAVE: AccountId = AccountId::new([6u8; 32]);
pub const EVE: AccountId = AccountId::new([5u8; 32]);
pub const ACCOUNT_WITH_NO_BALANCE0: AccountId = AccountId::new([4u8; 32]);
pub const ACCOUNT_WITH_NO_BALANCE1: AccountId = AccountId::new([7u8; 32]);
pub const BSX: Balance = 100_000_000_000;



// Build genesis storage according to the mock runtime.
pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
	let mut t= frame_system::GenesisConfig::<Test>::default().build_storage().unwrap().into();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![
			(ALICE, 300_000 * BSX),
			(BOB, 400_000 * BSX),
			(CHARLIE, 250_000 * BSX),
			(DAVE, 450_000 * BSX),
			(EVE, 150_000 * BSX),
			(ACCOUNT_WITH_NO_BALANCE0, 150_000 * BSX),
			(ACCOUNT_WITH_NO_BALANCE1, 150_000 * BSX),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	pallet_collective::GenesisConfig::<Test, pallet_collective::Instance2> {
		members: vec![ALICE, BOB, CHARLIE],
		phantom: Default::default(),
	}
	.assimilate_storage(&mut t)
	.unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
}
