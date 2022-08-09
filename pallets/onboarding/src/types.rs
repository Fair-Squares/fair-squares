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

#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebugNoBound, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Asset<T:Config> {
	/// Asset status
	pub(super) status: AssetStatus,
	/// Asset creation block
	pub(super) created: BlockNumberOf<T>,
	/// NFT infos
	pub(super) infos: ItemInfoOf<T>,
	/// NFT Price
	pub(super) price: Option<BalanceOf<T>>,
}

impl<T: Config> Asset<T>
	{
	pub fn new(collection:T::NftCollectionId,item:T::NftItemId,infos:ItemInfoOf<T>, price: Option<BalanceOf<T>>) -> DispatchResult{
		let status = AssetStatus::EDITING;
		let created = <frame_system::Pallet<T>>::block_number();
		let house = Asset::<T>{status: status, created: created,infos: infos, price: price};
		Houses::<T>::insert(collection,item,house);

		Ok(())
		
	}
}

#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebugNoBound, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct VotingCalls<T:Config> {
	/// Asset status
	pub(super) buy: T::Prop,
	/// Asset creation block
	pub(super) reject_edit: T::Prop,
	/// NFT infos
	pub(super) reject_destroy: T::Prop,
	/// NFT Price
	pub(super) democracy_status: T::Prop,
}

impl<T: Config> VotingCalls<T>{
	pub fn new(collection:T::NftCollectionId,item:T::NftItemId) -> DispatchResult{
		let nbr:u32 = 0;
		let call:T::Prop = Call::<T>::do_something{something: nbr}.into();
		let calls = VotingCalls::<T>{buy:call.clone(),reject_edit:call.clone(),reject_destroy:call.clone(),democracy_status:call.clone()};
		Vcalls::<T>::insert(collection,item,calls);
		Ok(())
	}
}
