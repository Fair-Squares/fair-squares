use frame_support::pallet_prelude::*;
pub use super::*;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use scale_info::TypeInfo;

pub type NftCollectionOf = Nft::PossibleCollections;
pub use Nft::ItemInfoOf;


#[derive(Clone, Encode, Decode, PartialEq, Eq, TypeInfo,Copy)]
#[cfg_attr(feature = "std", derive(Debug,Serialize, Deserialize))]
pub enum AssetStatus {
	EDITING,
    REVIEWING,
	VOTING,
    FINALIZING,
	APPROVED,
    REJECTEDIT,
    REJECTBURN,
}

#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Asset<T:Config> {
	/// Asset status
	pub(super) status: AssetStatus,
	/// Asset creation block
	pub(super) created: BlockNumberOf<T>,
	/// nft infos
	pub(super) infos: ItemInfoOf<T>,
}

impl<T: Config> Asset<T>
	{
	pub fn new(collection:T::NftCollectionId,item:T::NftItemId,infos:ItemInfoOf<T>) -> DispatchResult{
		let status = AssetStatus::EDITING;
		let created = <frame_system::Pallet<T>>::block_number();
		let house = Asset::<T>{status: status, created: created,infos: infos};
		Houses::<T>::insert(collection,item,house);

		Ok(())
		
	}
}
