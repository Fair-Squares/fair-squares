
use frame_support::pallet_prelude::*;
pub use super::*;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use scale_info::TypeInfo;

/// NFT Collection ID
pub type CollectionId = u32;

/// NFT Instance ID
pub type ItemId = u32;


#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct CollectionInfo<BoundedVec> {    
    /// Arbitrary data about a collection, e.g. IPFS hash
    pub metadata: BoundedVec,
}

#[derive(Encode, Decode, Eq, Copy, PartialEq, Clone, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ItemInfo<BoundedVec> {
    pub metadata: BoundedVec,
}



pub trait NftPermission<Acc> {
    fn can_create(role_type: &Acc) -> bool;
    fn can_mint(role_type: &Acc) -> bool;
    fn can_transfer(role_type: &Acc) -> bool;
    fn can_burn(role_type: &Acc) -> bool;
    fn can_destroy(role_type: &Acc) -> bool;
    fn has_deposit(role_type: &Acc) -> bool;
}

#[derive(Encode, Decode, Eq, Copy, PartialEq, Clone, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct NftPermissions;

impl NftPermission<Acc> for NftPermissions {
    fn can_create(role_type: &Acc) -> bool {
        matches!(*role_type, Acc::SELLER)
    }

    fn can_mint(role_type: &Acc) -> bool {
        matches!(*role_type, Acc::SERVICER)
    }

    fn can_transfer(role_type: &Acc) -> bool {
        matches!(*role_type, Acc::SERVICER)
    }

    fn can_burn(role_type: &Acc) -> bool {
        matches!(*role_type, Acc::SERVICER)
    }

    fn can_destroy(role_type: &Acc) -> bool {
        matches!(*role_type, Acc::SERVICER)
    }

    fn has_deposit(role_type: &Acc) -> bool {
        matches!(*role_type, Acc::SERVICER)
    }
}
