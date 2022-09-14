pub use super::*;
pub use frame_support::{
	assert_ok,
	dispatch::{DispatchResult, EncodeLike},
	inherent::Vec,
	pallet_prelude::*,
	sp_runtime::traits::{AccountIdConversion, Hash, Saturating, StaticLookup, Zero},
	storage::child,
	traits::{
		Currency, ExistenceRequirement, Get, LockableCurrency, ReservableCurrency, WithdrawReasons,
	},
	PalletId,
};
pub use frame_system::{ensure_signed, pallet_prelude::*, RawOrigin};
pub use scale_info::{prelude::{vec,format}, TypeInfo};
pub use serde::{Deserialize, Serialize};



#[derive(Clone, Encode, Decode, Default, PartialEq, Eq, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Ownership<T:Config> {
    /// Virtual account 
	pub virtual_account: T::AccountId,
	/// NFT owners accounts list
	pub owners: Vec<T::AccountId>,
}

impl<T: Config> Ownership<T> {
	pub fn new(
		collection: T::NftCollectionId,
		item: T::NftItemId,
		virtual_account: T::AccountId
	) -> DispatchResult {
		let owners = Vec::new();
		let ownership = Ownership::<T>{ virtual_account,owners};
		Virtual::<T>::insert(collection,item,ownership);

		Ok(())		
	}
}