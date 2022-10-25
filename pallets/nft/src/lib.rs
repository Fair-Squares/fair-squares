//! # NFT Pallet
//! This NFT pallet is directly inspired from the Galactic Council nft pallet.
//! Url: `https://github.com/galacticcouncil/warehouse`
//!
//!
//! The NFT Pallet is used to manage & perform diverse actions on NFTs
//! in the FairSquares framework.
//!
//! ## Overview
//!
//! Management of NFTs assets is made possible through the following actions:
//! - NFT Collection creation
//! - NFT asset creation or Minting
//! - NFT asset transfer
//! - NFT asset burning
//! - NFT collection destruction
//! Access to each action is restricted to a specific role.
//!
//! ### Dispatchable Functions

//! * `create_collection` - Restricted to Servicer role, this function
//! creates an NFT Collection of the given collection and sets its metadata

//! * `mint` - Restricted to Seller role, this function mints a NFT in the
//! specified collection, and sets its metadata

//! * `transfer` - Restricted to Servicer role, this function called by A(servicer)
//!  transfers NFT from account B(seller) to account C.

//! * `burn` - Restricted to Servicer role, this function Removes a NFT item from existence

//! * `destroy_collection` - Restricted to Servicer role, this function Removes a Collection from
//!   existence

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]

use codec::HasCompact;
use frame_support::{
	dispatch::DispatchResult,
	ensure,
	traits::{tokens::nonfungibles::*, Get},
	transactional, BoundedVec,
};
use frame_system::{ensure_root, ensure_signed};
use pallet_uniques::DestroyWitness;

pub use functions::*;
pub use pallet_roles as Roles;
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, StaticLookup, Zero},
	DispatchError,
};
use sp_std::boxed::Box;
pub use types::*;
//use weights::WeightInfo;

mod benchmarking;
pub mod functions;
pub mod types;
//pub mod weights;

#[cfg(test)]
pub mod mock;

#[cfg(test)]
mod tests;

pub type BoundedVecOfUnq<T> = BoundedVec<u8, <T as pallet_uniques::Config>::StringLimit>;
type CollectionInfoOf<T> = CollectionInfo<BoundedVecOfUnq<T>>;
pub type ItemInfoOf<T> = ItemInfo<BoundedVec<u8, <T as pallet_uniques::Config>::StringLimit>>;
pub type Acc = Roles::Accounts;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use super::*;
	use frame_system::WeightInfo;
	use frame_support::{pallet_prelude::*, traits::EnsureOrigin};
	use frame_system::pallet_prelude::OriginFor;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_uniques::Config + pallet_roles::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type WeightInfo: WeightInfo;
		type ProtocolOrigin: EnsureOrigin<Self::Origin>;
		type NftCollectionId: Member
			+ Parameter
			+ Default
			+ Copy
			+ HasCompact
			+ AtLeast32BitUnsigned
			+ Into<Self::CollectionId>
			+ From<Self::CollectionId>;
		type NftItemId: Member
			+ Parameter
			+ Default
			+ Copy
			+ HasCompact
			+ AtLeast32BitUnsigned
			+ Into<Self::ItemId>
			+ From<Self::ItemId>;
		type Permissions: NftPermission<Acc>;

		/// Collection IDs reserved for runtime up to the following constant
		#[pallet::constant]
		type ReserveCollectionIdUpTo: Get<Self::NftCollectionId>;
	}

	#[pallet::storage]
	#[pallet::getter(fn collections)]
	/// Stores Collection info
	pub type Collections<T: Config> =
		StorageMap<_, Twox64Concat, T::NftCollectionId, CollectionInfoOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn items)]
	/// Stores Item info
	pub type Items<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::NftCollectionId,
		Twox64Concat,
		T::NftItemId,
		ItemInfoOf<T>,
	>;

	#[pallet::type_value]
	///Initializing function for the approval waiting list
	pub fn InitDefault<T: Config>() -> Vec<u32> {
		vec![0, 0, 0, 0, 0, 0, 0]
	}

	#[pallet::storage]
	#[pallet::getter(fn itemid)]
	/// Update Item ID
	pub type ItemsCount<T: Config> = StorageValue<_, Vec<u32>, ValueQuery, InitDefault<T>>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub owner: Option<T::AccountId>,
		pub collection_id: Option<u32>,
		pub created_by: Option<Acc>,
		pub metadata: Option<BoundedVecOfUnq<T>>,
	}
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				owner: Default::default(),
				collection_id: Default::default(),
				created_by: Default::default(),
				metadata: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			let index = self.collection_id.unwrap();
			for n in 0..index {
				crate::Pallet::<T>::do_create_collection(
					self.owner.clone().unwrap(),
					n.into(),
					self.created_by.unwrap(),
					self.metadata.clone().unwrap(),
				)
				.ok();
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Creates an NFT Collection of the given Collection
		/// and sets its metadata
		///
		/// Parameters:
		/// - `collection_id`: Identifier of a Collection
		/// - `metadata`: Arbitrary data about a Collection, e.g. IPFS hash or name
		///
		/// Emits CollectionCreated event
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn create_collection(
			origin: OriginFor<T>,
			collection_id: PossibleCollections,
			metadata: BoundedVecOfUnq<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let coll_id: CollectionId = collection_id.value();

			//ensure!(T::ReserveCollectionIdUpTo::get() != coll_id.clone().into(),
			// Error::<T>::IdReserved);
			ensure!(!Self::is_id_reserved(coll_id.into()), Error::<T>::IdReserved);
			let created_by = Roles::Pallet::<T>::get_roles(&sender).unwrap();
			ensure!(T::Permissions::can_create(&created_by), Error::<T>::NotPermitted);

			Self::do_create_collection(sender, coll_id.into(), created_by, metadata)?;

			Ok(())
		}

		/// Mints a NFT in the specified Collection
		/// and sets its metadata
		///
		/// Parameters:
		/// - `collection_id`: The Collection of the asset to be minted.
		/// - `item_id`: The Collection of the asset to be minted.
		/// - `metadata`: Arbitrary data about an Item, e.g. IPFS hash or symbol
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn mint(
			origin: OriginFor<T>,
			collection_id: PossibleCollections,
			metadata: BoundedVecOfUnq<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let coll_id: CollectionId = collection_id.clone().value();
			let idx = collection_id.value() as usize;
			let created_by = Roles::Pallet::<T>::get_roles(&sender).unwrap();
			let item_id = Self::itemid()[idx];

			ensure!(T::Permissions::can_mint(&created_by), Error::<T>::NotPermitted);

			Self::do_mint(sender, coll_id.into(), item_id.into(), metadata)?;
			ItemsCount::<T>::mutate(|x| {
				x[idx] += 1;
			});

			Ok(())
		}

		/// Triggered by Root(`origin`), this transfers NFT from owner account to `dest` account
		///
		/// Parameters:
		/// - `collection_id`: The Collection of the asset to be transferred.
		/// - `item_id`: The Item of the asset to be transferred.
		/// - `dest`: The account to receive ownership of the asset.
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn transfer(
			origin: OriginFor<T>,
			collection_id: PossibleCollections,
			item_id: T::NftItemId,
			dest: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			//the transaction is triggered by Root
			ensure_root(origin)?;

			//Nft transfered from old to new owner
			let coll_id: CollectionId = collection_id.value();
			let dest = T::Lookup::lookup(dest)?;
			let owner = Self::owner(coll_id.into(), item_id).ok_or(Error::<T>::ItemUnknown)?;

			Self::do_transfer(coll_id.into(), item_id, owner, dest)?;

			Ok(())
		}

		/// Triggered by a servicer (`origin`) this removes a token from existence
		///
		/// Parameters:
		/// - `collection_id`: The Collection of the asset to be burned.
		/// - `item_id`: The Item of the asset to be burned.
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn burn(
			origin: OriginFor<T>,
			collection_id: PossibleCollections,
			item_id: T::NftItemId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let triggered_by = Roles::Pallet::<T>::get_roles(&sender).unwrap();
			ensure!(T::Permissions::can_burn(&triggered_by), Error::<T>::NotPermitted);

			let coll_id: CollectionId = collection_id.value();
			let owner = Self::owner(coll_id.into(), item_id).ok_or(Error::<T>::ItemUnknown)?;

			Self::do_burn(owner, coll_id.into(), item_id)?;

			Ok(())
		}

		/// Removes a Collection from existence
		///
		/// Parameters:
		/// - `collection_id`: The identifier of the asset Collection to be destroyed.
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn destroy_collection(
			origin: OriginFor<T>,
			collection_id: PossibleCollections,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let coll_id: CollectionId = collection_id.value();

			let created_by = Roles::Pallet::<T>::get_roles(&sender).unwrap();

			ensure!(T::Permissions::can_destroy(&created_by), Error::<T>::NotPermitted);

			Self::do_destroy_collection(sender, coll_id.into())?;

			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A Collection was created
		CollectionCreated {
			owner: T::AccountId,
			collection_id: T::NftCollectionId,
			created_by: Acc,
		},
		/// An Item was minted
		ItemMinted { owner: T::AccountId, collection_id: T::NftCollectionId, item_id: T::NftItemId },
		/// An Item was transferred
		ItemTransferred {
			from: T::AccountId,
			to: T::AccountId,
			collection_id: T::NftCollectionId,
			item_id: T::NftItemId,
		},
		/// An Item was burned
		ItemBurned { owner: T::AccountId, collection_id: T::NftCollectionId, item_id: T::NftItemId },
		/// A Collection was destroyed
		CollectionDestroyed { owner: T::AccountId, collection_id: T::NftCollectionId },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Count of items overflown
		NoAvailableItemId,
		/// Count of collections overflown
		NoAvailableCollectionId,
		/// Collection still contains minted tokens
		TokenCollectionNotEmpty,
		/// Collection does not exist
		CollectionUnknown,
		/// Item does not exist
		ItemUnknown,
		/// Operation not permitted
		NotPermitted,
		/// ID reserved for runtime
		IdReserved,
	}
}
