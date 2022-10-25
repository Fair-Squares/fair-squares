use super::*;
use crate as pallet_housing_fund;
use frame_support::{
	parameter_types,
	traits::{AsEnsureOriginWithArg, ConstU16, ConstU64},
	PalletId,
};
use frame_system::EnsureRoot;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

use crate::NFT::NftPermissions;
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
pub(crate) type Balance = u128;
type AccountId = u64;
pub type CollectionId = u32;
pub type ItemId = u32;
pub type NftCollection = crate::NFT::PossibleCollections;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
		{
			System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
			HousingFundModule: pallet_housing_fund::{Pallet, Call, Storage, Event<T>},
			Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
			RoleModule: pallet_roles::{Pallet, Call, Storage, Event<T>},
			Uniques: pallet_uniques::{Pallet, Call, Storage, Event<T>},
			NftModule: pallet_nft::{Pallet, Call, Storage, Event<T>},
			Sudo:pallet_sudo::{Pallet, Call, Storage, Config<T>, Event<T>},
		}
);

impl frame_system::Config for Test {
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
	type AccountData = pallet_balances::AccountData<u128>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_sudo::Config for Test {
	type Event = Event;
	type Call = Call;
}
parameter_types! {
	pub const MaxMembers:u32 =200;
}
impl pallet_roles::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type WeightInfo = ();
	type MaxMembers = MaxMembers;
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
	pub const MinContribution: u128 = 10;
	pub const FundThreshold: u128 = 2;
	pub const MaxFundContribution: u128 = 200;
	pub const HousingFundPalletId: PalletId = PalletId(*b"housfund");
	pub const MaxInvestorPerHouse: u32 = 2;
}

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

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	let fund_account_id = HousingFundModule::fund_account_id();

	// Initialize balance for account_id (1, 2)
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![(1, 100), (2, 100), (fund_account_id, 10)],
	}
	.assimilate_storage(&mut storage)
	.unwrap();

	let mut externalities = sp_io::TestExternalities::new(storage);
	externalities.execute_with(|| System::set_block_number(1));
	externalities
}
