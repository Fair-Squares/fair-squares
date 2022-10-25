use super::*;
use crate as pallet_nft;
#[cfg(feature = "std")]
use frame_support::traits::{AsEnsureOriginWithArg, Everything};
use frame_support::{parameter_types, weights::Weight};
use frame_system::EnsureRoot;
use pallet_roles::GenesisBuild;
use sp_core::{crypto::AccountId32, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};

mod nfc {
	// Re-export needed for `impl_outer_event!`.
	pub use super::super::*;
}

type AccountId = AccountId32;
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type Balance = u128;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Uniques: pallet_uniques::{Pallet, Storage, Event<T>},
		NFT: pallet_nft::{Pallet, Call, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Sudo:pallet_sudo::{Pallet, Call, Storage, Config<T>, Event<T>},
		RoleModule: pallet_roles::{Pallet, Call, Storage, Event<T>},
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
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
}

impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
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
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
	pub const MaxReserves: u32 = 50;
}
impl pallet_balances::Config for Test {
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = frame_system::Pallet<Test>;
	type MaxLocks = ();
	type WeightInfo = ();
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = ();
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

pub const ALICE: AccountId = AccountId::new([1u8; 32]);
pub const BOB: AccountId = AccountId::new([2u8; 32]);
pub const CHARLIE: AccountId = AccountId::new([3u8; 32]);
pub const DAVE: AccountId = AccountId::new([6u8; 32]);
pub const EVE: AccountId = AccountId::new([5u8; 32]);
pub const ACCOUNT_WITH_NO_BALANCE0: AccountId = AccountId::new([4u8; 32]);
pub const ACCOUNT_WITH_NO_BALANCE1: AccountId = AccountId::new([7u8; 32]);
pub const BSX: Balance = 100_000_000_000;
pub const HOUSES: <Test as pallet_uniques::Config>::CollectionId = 0;
pub const HOUSESTEST: <Test as pallet_uniques::Config>::CollectionId = 4;
pub const HOUSESRES: <Test as pallet_uniques::Config>::CollectionId = 3;
pub const COLLECTION_ID_RESERVED: <Test as pallet_uniques::Config>::CollectionId = 42;
pub const ITEM_ID_0: <Test as pallet_uniques::Config>::ItemId = 0;
pub const ITEM_ID_1: <Test as pallet_uniques::Config>::ItemId = 1;
pub const ITEM_ID_2: <Test as pallet_uniques::Config>::ItemId = 2;
pub const NON_EXISTING_COLLECTION_ID: <Test as pallet_uniques::Config>::CollectionId = 999;

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
