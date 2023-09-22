pub use super::*;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
pub use codec::HasCompact;
pub use frame_support::{
    pallet_prelude::*,PalletId,
	codec::{Decode, Encode},
	dispatch::{DispatchResult, Dispatchable, EncodeLike,GetDispatchInfo},
	ensure,
	traits::{
		defensive_prelude::*,
		schedule::{v3::Named as ScheduleNamed, DispatchTime},
		Bounded, Currency, EnsureOrigin, Get, Hash as PreimageHash, LockIdentifier,
		LockableCurrency, OnUnbalanced, QueryPreimage, ReservableCurrency, StorePreimage,
		WithdrawReasons,tokens::nonfungibles::*, BalanceStatus, ExistenceRequirement,
		UnfilteredDispatchable,
	},
	transactional, BoundedVec,
};
pub use frame_system::{ensure_signed, pallet_prelude::*, RawOrigin};

pub use sp_runtime::{
	traits::{AccountIdConversion, AtLeast32BitUnsigned, Saturating, StaticLookup, Zero},
	DispatchError, Percent,
};
pub use sp_std::boxed::Box;
pub use sp_std::prelude::*;
pub use scale_info::prelude::vec::Vec;
pub type BlockNumberOf<T> = BlockNumberFor<T>;
pub type NftCollectionOf = Nft::PossibleCollections;
pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type CallOf<T> = <T as frame_system::Config>::RuntimeCall;
pub type BoundedCallOf<T> = Bounded<CallOf<T>>;
type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;

pub use Nft::ItemInfoOf;

#[derive(Clone, Encode, Decode, PartialEq, Eq, TypeInfo, Copy)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub enum AssetStatus {
	EDITING,
	REVIEWING,
	VOTING,
	ONBOARDED,
	FINALISING,
	FINALISED,
	PURCHASED,
	REJECTED,
	SLASH,
	CANCELLED,
}

#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebugNoBound, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Asset<T: Config> {
	/// Asset status
	pub status: AssetStatus,
	/// Asset creation block
	pub(super) created: BlockNumberOf<T> ,
	/// NFT infos
	pub(super) infos: ItemInfoOf<T>,
	/// NFT Price
	pub price: Option<BalanceOf<T>>,
	/// Representative
	pub representative: Option<T::AccountId>,
	/// Tenants
	pub tenants: Vec<T::AccountId>,
	/// Proposal hash
	pub proposal_hash: T::Hash,
	/// Maximum number of tenants for this asset
	pub max_tenants: u8,
}

impl<T: Config> Asset<T> {
	pub fn new(
		collection: T::NftCollectionId,
		item: T::NftItemId,
		infos: ItemInfoOf<T>,
		price: Option<BalanceOf<T>>,
		max_tenants: u8
	) -> DispatchResult {
		let status = AssetStatus::EDITING;
		let created = <frame_system::Pallet<T>>::block_number();
		let house = Asset::<T> {
			status,
			created,
			infos,
			price,
			representative: None,
			tenants: Default::default(),
			proposal_hash: Default::default(),
			max_tenants,
		};
		Houses::<T>::insert(collection, item, house);

		Ok(())
	}
}

#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebugNoBound, TypeInfo)]
#[scale_info(skip_type_params(T))]
//#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct VotingCalls<T: Config> {
	/// Asset creation block
	pub(super) reject_edit: Box<T::Prop>,
	/// NFT infos
	pub(super) reject_destroy: Box<T::Prop>,
	/// NFT Price
	pub(super) democracy_status: Box<T::Prop>,
	///After positive Investor vote status
	pub(super) after_vote_status: Box<T::Prop>,
}

impl<T: Config> VotingCalls<T> {
	pub fn new(collection: T::NftCollectionId, item: T::NftItemId) -> DispatchResult {
		let nbr: u32 = 0;
		let call: T::Prop = Call::<T>::do_something { something: nbr }.into();

		let calls = VotingCalls::<T> {
			reject_edit: Box::new(call.clone()),
			reject_destroy: Box::new(call.clone()),
			democracy_status: Box::new(call.clone()),
			after_vote_status: Box::new(call),
		};
		Vcalls::<T>::insert(collection, item, calls);
		Ok(())
	}
}