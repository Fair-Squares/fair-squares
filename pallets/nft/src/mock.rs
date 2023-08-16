use crate as pallet_nft;
use frame_support::{
	parameter_types,
	traits::{ConstU16,ConstU64,AsEnsureOriginWithArg, Everything},
	weights::Weight,
};
use sp_core::{crypto::AccountId32, H256};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,	
};
use frame_system::{EnsureRoot,};

type Block = frame_system::mocking::MockBlock<Test>;

pub type BlockNumber = u64;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test 
	{
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
        Uniques: pallet_uniques::{Pallet, Storage, Event<T>},
		RolesModule: pallet_roles::{Pallet, Call, Storage, Event<T>, Config<T>},
        NFT: pallet_nft::{Pallet, Call, Storage, Event<T>, Config<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Sudo:pallet_sudo::{Pallet, Call, Storage, Config<T>, Event<T>},
	}
);

parameter_types! {
	pub ReserveCollectionIdUpTo: u32 = 45;
}

#[derive(Eq, Copy, PartialEq, Clone)]
pub struct NftTestPermissions;

impl NftPermission<Acc> for NftTestPermissions {
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

impl Config for Test {
	type Event = Event;
	type WeightInfo = ();
	type NftCollectionId = CollectionId;
	type NftItemId = ItemId;
	type ProtocolOrigin = EnsureRoot<AccountId>;
	type Permissions = NftTestPermissions;
	type ReserveCollectionIdUpTo = ReserveCollectionIdUpTo;
}

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
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
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

impl pallet_balances::Config for Test {
	type Balance = u64;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ();
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

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::<Test>::default().build_storage().unwrap().into()
}
