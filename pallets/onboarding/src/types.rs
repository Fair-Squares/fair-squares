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

#[derive(Encode, Decode, Eq, Copy, PartialEq, Clone, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Asset<AssetStatus, BlockNumber,Balance,ItemInfoOf> {
	/// Asset status
	pub(super) status: AssetStatus,
	/// Asset creation block
	pub(super) created: BlockNumber,
    /// Asset price
    pub(super) value: Balance,
	/// nft infos
	pub(super) infos: ItemInfoOf,
}
