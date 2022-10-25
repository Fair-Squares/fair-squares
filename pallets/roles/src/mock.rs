use crate as pallet_roles;
use frame_support::{
	parameter_types,
	traits::{ConstU32, ConstU64, GenesisBuild},
};
use frame_system as system;
pub use frame_system::RawOrigin;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

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
		RoleModule: pallet_roles::{Pallet, Call, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Sudo:pallet_sudo::{Pallet, Call, Storage, Config<T>, Event<T>},
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

//---implementing pallet sudo---------
impl pallet_sudo::Config for Test {
	type Event = Event;
	type Call = Call;
}

parameter_types! {
	pub const MaxMembers:u32 =5;
}
impl pallet_roles::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type WeightInfo = ();
	type MaxMembers = MaxMembers;
}

//helper types
pub type Acc = pallet_roles::Accounts;

// Build genesis storage according to the mock runtime.
pub fn new_test_ext(root_key: u64) -> sp_io::TestExternalities {
	let mut t = system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_sudo::GenesisConfig::<Test> { key: Some(root_key) }
		.assimilate_storage(&mut t)
		.unwrap();
	t.into()
}
