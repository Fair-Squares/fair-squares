pub use super::*;
pub use frame_support::{
	assert_ok,
	dispatch::{DispatchResult, EncodeLike},
	inherent::Vec,
	pallet_prelude::*,
	sp_runtime::{
		traits::{AccountIdConversion, Hash, One, Saturating, StaticLookup, Zero},
		PerThing, Percent,
	},
	storage::child,
	traits::{
		Currency, ExistenceRequirement, Get, LockableCurrency, ReservableCurrency, WithdrawReasons,
	},
	PalletId,
};

pub use frame_system::{ensure_signed, pallet_prelude::*, RawOrigin};
pub use scale_info::{
	prelude::{format, vec},
	TypeInfo,
};
pub use serde::{Deserialize, Serialize};
pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;
pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Clone, Encode, Decode, PartialEq, Eq, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Owners<T: Config> {
	pub owners: Vec<(T::AccountId, <T as Assets::Config>::Balance)>,
	///Creation Blocknumber
	pub created: BlockNumberOf<T>,
	///TokenId
	pub token_id: <T as pallet::Config>::AssetId,
	///Total supply of tokens
	pub supply: <T as Assets::Config>::Balance,
}

impl<T: Config> Owners<T> {
	pub fn new(virtual_account: T::AccountId) -> DispatchResult {
		let owners = Vec::new();
		let created = <frame_system::Pallet<T>>::block_number();
		let token_id: <T as pallet::Config>::AssetId = TokenId::<T>::get().into();
		let supply = Zero::zero();
		let tokens = Owners::<T> { owners, created, token_id, supply };

		Tokens::<T>::insert(virtual_account, tokens);

		Ok(())
	}
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Ownership<T: Config> {
	/// Virtual account
	pub virtual_account: T::AccountId,
	/// NFT owners accounts list
	pub owners: Vec<T::AccountId>,
	///Creation Blocknumber
	pub created: BlockNumberOf<T>,
	///TokenId
	pub token_id: <T as pallet::Config>::AssetId,
}

impl<T: Config> Ownership<T> {
	pub fn new(
		collection: T::NftCollectionId,
		item: T::NftItemId,
		virtual_account: T::AccountId,
	) -> DispatchResult {
		let owners = Vec::new();
		let created = <frame_system::Pallet<T>>::block_number();
		let token_id: <T as pallet::Config>::AssetId = TokenId::<T>::get().into();
		let ownership = Ownership::<T> { virtual_account, owners, created, token_id };

		Virtual::<T>::insert(collection, item, ownership);

		Ok(())
	}
}
