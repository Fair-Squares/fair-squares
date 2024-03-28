pub use super::*;
pub use frame_support::{
	assert_ok,
	dispatch::{DispatchResult},
	pallet_prelude::*,
	sp_runtime::{
		traits::{AccountIdConversion, Hash, One, Saturating, StaticLookup, Zero},
		PerThing, Percent,
	},
	storage::{child,bounded_vec::BoundedVec},
	traits::{
		Currency, ExistenceRequirement, Get, LockableCurrency, ReservableCurrency, WithdrawReasons,
	},
	PalletId,
};

pub use frame_system::{ensure_signed, pallet_prelude::*, RawOrigin};
pub use scale_info::{
	prelude::{format, vec::Vec},
	TypeInfo
};
pub use serde::{Deserialize, Serialize};
pub type BlockNumberOf<T> = BlockNumberFor<T>;
pub type BalanceOf<T> =
<<T as Roles::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Clone, Encode, Decode, PartialEq, Eq, TypeInfo,MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Owners<T: Config> {
	pub owners: BoundedVec<(T::AccountId, <T as Assets::Config>::Balance),T::MaxOwners>,
	///Creation Blocknumber
	pub created_at_block: BlockNumberOf<T>,
	///TokenId
	pub token_id: <T as pallet::Config>::AssetId,
	///Total supply of tokens
	pub supply: <T as Assets::Config>::Balance,
}

impl<T: Config> Owners<T> {
	pub fn new(virtual_account: T::AccountId) -> DispatchResult {
		let own = Vec::new();
		let owners = BoundedVec::truncate_from(own);
		let created_at_block = <frame_system::Pallet<T>>::block_number();
		let token_id: <T as pallet::Config>::AssetId = TokenId::<T>::get().into();
		let supply = Zero::zero();
		let tokens = Owners::<T> { owners, created_at_block, token_id, supply };

		Tokens::<T>::insert(virtual_account, tokens);

		Ok(())
	}
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, TypeInfo,MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Ownership<T: Config> {
	/// Virtual account
	pub virtual_account: T::AccountId,
	/// NFT owners accounts list
	pub owners: BoundedVec<T::AccountId,T::MaxOwners>,
	///Creation Blocknumber
	pub created: BlockNumberOf<T>,
	///TokenId
	pub token_id: <T as pallet::Config>::AssetId,
	///Number of rents awaiting distribution
	pub rent_nbr: u32,
}

impl<T: Config> Ownership<T> {
	pub fn new(
		collection: T::NftCollectionId,
		item: T::NftItemId,
		virtual_account: T::AccountId,
	) -> DispatchResult {
		let own = Vec::new();
		let owners = BoundedVec::truncate_from(own);
		let created = <frame_system::Pallet<T>>::block_number();
		let token_id: <T as pallet::Config>::AssetId = TokenId::<T>::get().into();
		let rent_nbr = 0;
		let ownership = Ownership::<T> { virtual_account, owners, created, token_id, rent_nbr };

		Virtual::<T>::insert(collection, item, ownership);

		Ok(())
	}
}