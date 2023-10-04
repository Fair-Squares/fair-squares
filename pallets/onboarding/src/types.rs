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
	pub type BalanceOf1<T> =
	<<T as DEM::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type CallOf<T> = <T as frame_system::Config>::RuntimeCall;
pub type BoundedCallOf<T> = Bounded<CallOf<T>>;
type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;

pub use Nft::ItemInfoOf;

pub type Status = pallet_roles::AssetStatus;

#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebugNoBound, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Asset<T: Config> {
	/// Asset status
	pub status: Status,
	/// Asset creation block
	pub created: BlockNumberOf<T> ,
	/// NFT infos
	pub(super) infos: ItemInfoOf<T>,
	/// NFT Price
	pub price: Option<BalanceOf<T>>,
	/// Representative
	pub representative: Option<T::AccountId>,
	/// Tenants
	pub tenants: Vec<T::AccountId>,
	/// Proposal hash
	pub ref_index: u32,
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
	) -> Self {
		let status = Status::EDITING;
		let created = <frame_system::Pallet<T>>::block_number();
		let house = Asset::<T> {
			status,
			created,
			infos,
			price,
			representative: None,
			tenants: Default::default(),
			ref_index: 0,
			max_tenants,
		};
		Houses::<T>::insert(collection, item, house.clone());
		house
		//Ok(())
	}
}

#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebugNoBound, TypeInfo)]
#[scale_info(skip_type_params(T))]
//#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct VotingCalls<T: Config> {
	/// Asset creation block
	pub(super) reject_edit: Call<T>,
	/// NFT infos
	pub(super) reject_destroy: Call<T>,
	/// NFT Price
	pub(super) democracy_status: Call<T>,
	///After positive Investor vote status
	pub(super) after_vote_status: Call<T>,
}

impl<T: Config> VotingCalls<T> {
	pub fn new(collection: T::NftCollectionId, item: T::NftItemId) -> DispatchResult {
		let nbr: u32 = 0;
		let call:Call<T> = Call::<T>::do_something { something: nbr }.into();

		let calls = VotingCalls::<T> {
			reject_edit: call.clone(),
			reject_destroy: call.clone(),
			democracy_status: call.clone(),
			after_vote_status: call,
		};
		Vcalls::<T>::insert(collection, item, calls);
		Ok(())
	}
}