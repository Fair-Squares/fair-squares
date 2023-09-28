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
pub use pallet::*;

use codec::HasCompact;
use frame_support::{
	dispatch::DispatchResult,
	ensure,
	traits::{tokens::nonfungibles::*, Get},
	transactional, BoundedVec,
};
pub use frame_system::{ensure_root, ensure_signed};
use pallet_nfts::DestroyWitness;

pub use functions::*;
pub use pallet_roles::Config as Roles;
pub use pallet_nfts::Config as Nfts;
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

//#[cfg(test)]
//mod tests;

/*pub type BoundedVecOfNfts<T> = BoundedVec<u8, <T as pallet_uniques::Config>::StringLimit>;
type CollectionInfoOf<T> = CollectionInfo<BoundedVecOfNfts<T>>;
pub type ItemInfoOf<T> = ItemInfo<BoundedVec<u8, <T as pallet_uniques::Config>::StringLimit>>;*/

pub type BoundedVecOfNfts<T> = BoundedVec<u8, <T as pallet_nfts::Config>::StringLimit>;
type CollectionInfoOf<T> = CollectionInfo<BoundedVecOfNfts<T>>;
pub type ItemInfoOf<T> = ItemInfo<BoundedVec<u8, <T as pallet_nfts::Config>::StringLimit>>;
pub type Acc = pallet_roles::Accounts;


// Re-export pallet items so that they can be accessed from the crate namespace.


#[frame_support::pallet]
pub mod pallet {

	use super::*;
	//use frame_system::WeightInfo;
	use frame_support::{pallet_prelude::*, traits::EnsureOrigin};
	use frame_system::pallet_prelude::OriginFor;


	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + Nfts+Roles{
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;		
		type ProtocolOrigin: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;	
		type Permissions: NftPermission<Acc>;
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

		}

	#[pallet::storage]
	#[pallet::getter(fn collections)]
	/// Stores Collection info
	pub type Collections<T: Config> =
		StorageMap<_, Twox64Concat, T::NftCollectionId , CollectionInfoOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn items)]
	/// Stores Item info
	pub type Items<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::NftCollectionId ,
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

	#[derive(frame_support::DefaultNoBound)]
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub owner: Option<T::AccountId>,
		pub collection_id: Option<u32>,
		pub created_by: Option<Acc>,
		pub metadata: Option<BoundedVecOfNfts<T>>,
	}
	
	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			let index = self.collection_id.unwrap();
			if index>0{
				for n in 0..index {
					let n0:T::NftCollectionId=n.into();
					crate::Pallet::<T>::do_create_collection(
						self.owner.clone().unwrap(),
						n0,
						self.created_by.unwrap(),
						self.metadata.clone().unwrap(),
					)
					.ok();
				}
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
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		#[transactional]
		pub fn create_collection(
			origin: OriginFor<T>,
			collection_id: PossibleCollections,
			metadata: BoundedVecOfNfts<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			let coll_id: T::NftCollectionId = collection_id.value().into();

			let created_by = pallet_roles::Pallet::<T>::get_roles(&sender)[0];
			//ensure!(T::Permissions::can_create(&created_by), Error::<T>::NotPermitted);

			Self::do_create_collection(sender, coll_id.clone(), created_by, metadata.clone())?;
			
		pallet_nfts::Pallet::<T>::set_collection_metadata(origin,coll_id.into(),metadata)?;
			Ok(())
		}

		/// Mints a NFT in the specified Collection
		/// and sets its metadata
		///
		/// Parameters:
		/// - `collection_id`: The Collection of the asset to be minted.
		/// - `item_id`: The Collection of the asset to be minted.
		/// - `metadata`: Arbitrary data about an Item, e.g. IPFS hash or symbol
		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		#[transactional]
		pub fn mint(
			origin: OriginFor<T>,
			collection_id: PossibleCollections,
			metadata: BoundedVecOfNfts<T>,
		) -> DispatchResult {
			let sender0 = ensure_signed(origin.clone())?;
			let mut sender = sender0.clone(); 
			let coll_owners =pallet_nfts::CollectionAccount::<T>::iter_keys();
			let coll_id: T::NftCollectionId = collection_id.clone().value().into();
			let created_by = pallet_roles::Pallet::<T>::get_roles(&sender0)[0];
			let idx = collection_id.value() as usize;			
			let item_id:T::NftItemId = Self::itemid()[idx].into();
			let dest =  T::Lookup::unlookup(sender0.clone());
			let item_config= pallet_nfts::ItemConfig{
				settings: pallet_nfts::ItemSettings::default()
			};
			ensure!(T::Permissions::can_mint(&created_by), Error::<T>::NotPermitted);
			for val in coll_owners{
				if val.1==coll_id.into(){
					sender = val.0;
				}
				

			}
			
			ensure!(pallet_nfts::CollectionAccount::<T>::contains_key(sender.clone(),coll_id.into()), Error::<T>::CollectionUnknown);



			let origin_collection= RawOrigin::Signed(sender.clone());
			
			pallet_nfts::Pallet::<T>::force_mint(origin_collection.into(), coll_id.clone().into(), item_id.clone().into(), dest.clone().into(),item_config)?;

			Items::<T>::set(coll_id,item_id,Some(ItemInfo{metadata:metadata.clone()}));
			Self::set_metadata(sender,coll_id.clone(), item_id.clone(),metadata.clone()).ok();
			debug_assert!(Items::<T>::get(coll_id,item_id).is_some());
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
		#[pallet::call_index(2)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		#[transactional]
		pub fn transfer(
			origin: OriginFor<T>,
			collection_id: PossibleCollections,
			item_id: T::NftItemId ,
			dest: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			//the transaction is triggered by Root
			let sender = ensure_signed(origin)?;

			//Nft transfered from old to new owner
			let coll_id: T::NftCollectionId = collection_id.value().into();
			let owner = Self::owner(coll_id, item_id).ok_or(Error::<T>::ItemUnknown)?;
			ensure!(sender==owner,Error::<T>::NotPermitted);
			let origin0 = RawOrigin::Signed(owner);

			pallet_nfts::Pallet::<T>::transfer(origin0.into(),coll_id.into(), item_id.into(), dest)?;

			Ok(())
		}

		/// Triggered by a servicer (`origin`) this removes a token from existence
		///
		/// Parameters:
		/// - `collection_id`: The Collection of the asset to be burned.
		/// - `item_id`: The Item of the asset to be burned.
		#[pallet::call_index(3)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		#[transactional]
		pub fn burn(
			origin: OriginFor<T>,
			collection_id: PossibleCollections,
			item_id: T::NftItemId ,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			let triggered_by = pallet_roles::Pallet::<T>::get_roles(&sender)[0];
			ensure!(T::Permissions::can_burn(&triggered_by), Error::<T>::NotPermitted);

			let coll_id: T::NftCollectionId = collection_id.value().into();
			let owner = Self::owner(coll_id, item_id).ok_or(Error::<T>::ItemUnknown)?;
			let origin0 = RawOrigin::Signed(owner);
			let idx = collection_id.value() as usize;
			pallet_nfts::Pallet::<T>::burn(origin0.into(), coll_id.into(), item_id.into())?;
			ItemsCount::<T>::mutate(|x| {
				x[idx] -= 1;
			});
			Ok(())
		}

		/// Removes a Collection from existence
		///
		/// Parameters:
		/// - `collection_id`: The identifier of the asset Collection to be destroyed.
		#[pallet::call_index(4)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		#[transactional]
		pub fn destroy_collection(
			origin: OriginFor<T>,
			collection_id: PossibleCollections,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let coll_id: T::NftCollectionId = collection_id.value().into();

			let created_by = pallet_roles::Pallet::<T>::get_roles(&sender)[0];

			ensure!(T::Permissions::can_destroy(&created_by), Error::<T>::NotPermitted);

			Self::do_destroy_collection(sender, coll_id)?;

			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		SomethingStored(T::AccountId),
		/// A Collection was created
		CollectionCreated {
			owner: T::AccountId,
			collection_id: T::NftCollectionId ,
			created_by: Acc,
		},
		/// An Item was minted
		ItemMinted { owner: T::AccountId, collection_id: T::NftCollectionId , item_id: T::NftItemId  },
		/// An Item was transferred
		ItemTransferred {
			from: T::AccountId,
			to: T::AccountId,
			collection_id: T::NftCollectionId ,
			item_id: T::NftItemId ,
		},
		/// An Item was burned
		ItemBurned { owner: T::AccountId, collection_id: T::NftCollectionId , item_id: T::NftItemId  },
		/// A Collection was destroyed
		CollectionDestroyed { owner: T::AccountId, collection_id: T::NftCollectionId  },
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