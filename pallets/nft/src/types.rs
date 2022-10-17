pub use super::*;
use enum_iterator::Sequence;
pub use frame_support::inherent::Vec;
use frame_support::pallet_prelude::*;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

pub use scale_info::{prelude::vec, TypeInfo};

/// NFT Collection ID
pub type CollectionId = u32;
#[derive(Clone, Encode, Decode, PartialEq, Eq, TypeInfo, Copy, Sequence)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub enum PossibleCollections {
	HOUSES,
	OFFICES,
	APPARTMENTS,
	HOUSESTEST,
	OFFICESTEST,
	APPARTMENTSTEST,
	NONEXISTING,
}

impl PossibleCollections {
	pub fn value(&self) -> CollectionId {
		match *self {
			PossibleCollections::HOUSES => 0,
			PossibleCollections::OFFICES => 1,
			PossibleCollections::APPARTMENTS => 2,
			PossibleCollections::HOUSESTEST => 4,
			PossibleCollections::OFFICESTEST => 5,
			PossibleCollections::APPARTMENTSTEST => 6,
			PossibleCollections::NONEXISTING => 3,
		}
	}
}

/// NFT Item ID
pub type ItemId = u32;

#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct CollectionInfo<BoundedVec> {
	pub created_by: Acc,
	/// Arbitrary data about a collection, e.g. IPFS hash
	pub metadata: BoundedVec,
}

#[derive(Encode, Decode, Eq, Copy, PartialEq, Clone, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ItemInfo<BoundedVec> {
	pub metadata: BoundedVec,
}

pub trait NftPermission<Acc> {
	fn can_create(created_by: &Acc) -> bool;
	fn can_mint(created_by: &Acc) -> bool;
	fn can_burn(created_by: &Acc) -> bool;
	fn can_destroy(created_by: &Acc) -> bool;
	fn has_deposit(created_by: &Acc) -> bool;
}

#[derive(Encode, Decode, Eq, Copy, PartialEq, Clone, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct NftPermissions;

impl NftPermission<Acc> for NftPermissions {
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
