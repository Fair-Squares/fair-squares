use super::*;
use crate as pallet_nft;
use frame_support::{
	parameter_types,
	traits::{ConstU16,ConstU32,ConstU64,AsEnsureOriginWithArg},
	weights::Weight,
};
use sp_core:: H256;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup,IdentifyAccount,Verify},
	BuildStorage, MultiSignature,
};
use frame_system::{EnsureRoot,};
use pallet_nfts::PalletFeatures;

type Block = frame_system::mocking::MockBlock<Test>;
pub type Signature = MultiSignature;
pub type AccountPublic = <Signature as Verify>::Signer;
pub type AccountId = <AccountPublic as IdentifyAccount>::AccountId;
type Balance = u128;
pub type BlockNumber = u64;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test 
	{
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
		RolesModule: pallet_roles::{Pallet, Call, Storage, Event<T>, Config<T>},
        Nft: pallet_nft::{Pallet, Call, Storage, Event<T>, Config<T>},
		Nfts: pallet_nfts::{Pallet, Call, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Sudo:pallet_sudo::{Pallet, Call, Storage, Config<T>, Event<T>},
		Collective: pallet_collective::<Instance2>::{Pallet, Call, Event<T>, Origin<T>, Config<T>},
	}
);

//helper types
pub type Acc = pallet_roles::Accounts;

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
	type RuntimeEvent = RuntimeEvent;
	//type WeightInfo = ();
	type NftCollectionId = CollectionId;
	type NftItemId = ItemId;
	type ProtocolOrigin = EnsureRoot<Self::AccountId>;
	type Permissions = NftTestPermissions;
}

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
	type CollectionDeposit = CollectionDeposit;
	type ItemDeposit = ItemDeposit;
	type MetadataDepositBase = AttributeDepositBase;
	type AttributeDepositBase = AttributeDepositBase;
	type DepositPerByte = AttributeDepositBase;
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
pub const HOUSES: <Test as pallet_nfts::Config>::CollectionId = 0;
pub const HOUSESTEST: <Test as pallet_nfts::Config>::CollectionId = 4;
pub const HOUSESRES: <Test as pallet_nfts::Config>::CollectionId = 3;
pub const COLLECTION_ID_RESERVED: <Test as pallet_nfts::Config>::CollectionId = 42;
pub const ITEM_ID_0: <Test as pallet_nfts::Config>::ItemId = 0;
pub const ITEM_ID_1: <Test as pallet_nfts::Config>::ItemId = 1;
pub const ITEM_ID_2: <Test as pallet_nfts::Config>::ItemId = 2;
pub const NON_EXISTING_COLLECTION_ID: <Test as pallet_nfts::Config>::CollectionId = 999;


// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t= frame_system::GenesisConfig::<Test>::default().build_storage().unwrap().into();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![
			(ALICE, 200_000 * BSX),
			(BOB, 200_000 * BSX),
			(CHARLIE, 200_000 * BSX),
			(DAVE, 150_000 * BSX),
			(EVE, 150_000 * BSX),
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
