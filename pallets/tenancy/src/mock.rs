use super::*;
pub use crate as pallet_tenancy;
use frame_support::{
	parameter_types,
	traits::{AsEnsureOriginWithArg, ConstU16, ConstU32, ConstU64, EqualPrivilegeOnly},
	weights::Weight,
	PalletId,
};

use crate::Nft::NftPermissions;
use frame_system::{EnsureRoot, EnsureSigned};
use pallet_roles::GenesisBuild;
use sp_core::{crypto::AccountId32, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};

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
		System: frame_system::{Pallet, Call, Storage, Event<T>},
		TenancyModule: pallet_tenancy::{Pallet, Call, Storage, Event<T>},
		AssetManagement: pallet_asset_management::{Pallet, Call, Storage, Event<T>},
		Ident: pallet_identity::{Pallet, Call, Storage, Event<T>},
		ShareDistributor: pallet_share_distributor::{Pallet, Call, Storage, Event<T>},
		RoleModule: pallet_roles::{Pallet, Call, Storage, Event<T>},
		NftModule: pallet_nft::{Pallet, Call, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Uniques: pallet_uniques::{Pallet, Call, Storage, Event<T>},
		Payment: pallet_payment::{Pallet, Call, Storage, Event<T>}
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
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_tenancy::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type WeightInfo = ();
}

pub struct MockDisputeResolver;
impl pallet_payment::DisputeResolver<AccountId> for MockDisputeResolver {
	fn get_resolver_account() -> AccountId {
		RESOLVER_ACCOUNT
	}
}

pub struct MockFeeHandler;
impl pallet_payment::FeeHandler<Test> for MockFeeHandler {
	fn apply_fees(
		_from: &AccountId,
		to: &AccountId,
		_detail: &pallet_payment::PaymentDetail<Test>,
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
	pub const BasicDeposit: Balance = 10 ;       // 258 bytes on-chain
	pub const FieldDeposit: Balance = 250 ;        // 66 bytes on-chain
	pub const SubAccountDeposit: Balance = 2 ;   // 53 bytes on-chain
	pub const MaxAdditionalFields: u32 = 100;
	pub const MaxRegistrars: u32 = 1;
	pub const MaxSubAccounts: u32 = 100;
}

impl pallet_identity::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type BasicDeposit = BasicDeposit;
	type FieldDeposit = FieldDeposit;
	type MaxRegistrars = MaxRegistrars;
	type ForceOrigin = EnsureRoot<AccountId>;
	type RegistrarOrigin = EnsureRoot<AccountId>;
	type MaxAdditionalFields = MaxAdditionalFields;
	type MaxSubAccounts = MaxSubAccounts;
	type Slashed = ();
	type SubAccountDeposit = SubAccountDeposit;
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
	pub const MaxMembers:u32 =7;
}
impl pallet_roles::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type WeightInfo = ();
	type MaxMembers = MaxMembers;
}

parameter_types! {
	pub const AssetsFees: Balance = 15000;
}
impl pallet_share_distributor::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type AssetId = u32;
	type Fees = AssetsFees;
}

parameter_types! {
	pub const JudgementFee: u64= 2;
	pub const GuarantyCoefficient: u32 = 3;
	pub const RoR:u32 = 3;
}

impl pallet_asset_management::Config for Test {
	type Event = Event;
	type Call = Call;
	type Delay = Delay;
	type CheckDelay = CheckDelay;
	type InvestorVoteAmount = InvestorVoteAmount;
	type CheckPeriod = CheckPeriod;
	type MinimumDepositVote = MinimumDeposit;
	type RepFees = JudgementFee;
	type Currency = Balances;
	type Guaranty = GuarantyCoefficient;
	type RoR = RoR;
	type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}
