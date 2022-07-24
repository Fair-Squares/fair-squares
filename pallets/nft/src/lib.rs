

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
use frame_system::ensure_signed;
use pallet_uniques::DestroyWitness;

use sp_runtime::{
    traits::{AtLeast32BitUnsigned, StaticLookup, Zero},
    DispatchError,
};
use sp_std::boxed::Box;
pub use types::*;
pub use functions::*;
pub use pallet_roles as Roles;
use weights::WeightInfo;

mod benchmarking;
pub mod types;
pub mod functions;
pub mod weights;

#[cfg(test)]
pub mod mock;

//#[cfg(test)]
//mod tests;

pub type BoundedVecOfUnq<T> = BoundedVec<u8, <T as pallet_uniques::Config>::StringLimit>;
type CollectionInfoOf<T> = CollectionInfo<BoundedVecOfUnq<T>>;
pub type ItemInfoOf<T> = ItemInfo<BoundedVec<u8, <T as pallet_uniques::Config>::StringLimit>>;
pub type Acc = Roles::Accounts;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

    use super::*;
    use frame_support::{pallet_prelude::*, traits::EnsureOrigin};
    use frame_system::pallet_prelude::OriginFor;

	#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
	#[derive(TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct TokenByOwnerData<T:Config> {
		pub percent_owned: u32,
		pub item: ItemInfoOf<T>,
	}

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_uniques::Config + pallet_roles::Config{
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
    pub type Collections<T: Config> = StorageMap<_, Twox64Concat, T::NftCollectionId, CollectionInfoOf<T>>;

    #[pallet::storage]
    #[pallet::getter(fn items)]
    /// Stores Item info
    pub type Items<T: Config> =
        StorageDoubleMap<_, Twox64Concat, T::NftCollectionId, Twox64Concat, T::NftItemId, ItemInfoOf<T>>;

	
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
        #[pallet::weight(<T as pallet::Config>::WeightInfo::create_collection())]
        #[transactional]
        pub fn create_collection(
            origin: OriginFor<T>,
            collection_id: T::NftCollectionId,
            metadata: BoundedVecOfUnq<T>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            //ensure!(T::ReserveCollectionIdUpTo::get() < collection_id, Error::<T>::IdReserved);
            ensure!(!Self::is_id_reserved(collection_id), Error::<T>::IdReserved);
            let role_type = Roles::Pallet::<T>::get_roles(&sender).unwrap();
            ensure!(T::Permissions::can_create(&role_type), Error::<T>::NotPermitted);

            Self::do_create_collection(sender, collection_id,role_type, metadata)?;

            Ok(())
        }

        /// Mints an NFT in the specified Collection
        /// and sets its metadata
        ///
        /// Parameters:
        /// - `collection_id`: The Collection of the asset to be minted.
        /// - `item_id`: The Collection of the asset to be minted.
        /// - `metadata`: Arbitrary data about an Item, e.g. IPFS hash or symbol
        #[pallet::weight(<T as pallet::Config>::WeightInfo::mint())]
        #[transactional]
        pub fn mint(
            origin: OriginFor<T>,
            collection_id: T::NftCollectionId,
            item_id: T::NftItemId,
            metadata: BoundedVecOfUnq<T>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let role_type = Self::collections(collection_id)
            .map(|c| c.role_type)
            .ok_or(Error::<T>::CollectionUnknown)?;            
        
            ensure!(T::Permissions::can_mint(&role_type), Error::<T>::NotPermitted);

            Self::do_mint(sender, collection_id, item_id, metadata)?;

            Ok(())
        }

        /// Transfers NFT from account A to account B
        /// Only the ProtocolOrigin can send NFT to another account
        /// This is to prevent creating deposit burden for others
        ///
        /// Parameters:
        /// - `collection_id`: The Collection of the asset to be transferred.
        /// - `item_id`: The Item of the asset to be transferred.
        /// - `dest`: The account to receive ownership of the asset.
        #[pallet::weight(<T as pallet::Config>::WeightInfo::transfer())]
        #[transactional]
        pub fn transfer(
            origin: OriginFor<T>,
            collection_id: T::NftCollectionId,
            item_id: T::NftItemId,
            dest: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let dest = T::Lookup::lookup(dest)?;
            let role_type = Self::collections(collection_id)
            .map(|c| c.role_type)
            .ok_or(Error::<T>::CollectionUnknown)?;

            ensure!(T::Permissions::can_transfer(&role_type), Error::<T>::NotPermitted);

            Self::do_transfer(collection_id, item_id, sender, dest)?;

            Ok(())
        }

        /// Removes a token from existence
        ///
        /// Parameters:
        /// - `collection_id`: The Collection of the asset to be burned.
        /// - `item_id`: The Item of the asset to be burned.
        #[pallet::weight(<T as pallet::Config>::WeightInfo::burn())]
        #[transactional]
        pub fn burn(origin: OriginFor<T>, collection_id: T::NftCollectionId, item_id: T::NftItemId) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let role_type = Self::collections(collection_id)
            .map(|c| c.role_type)
            .ok_or(Error::<T>::CollectionUnknown)?;

            ensure!(T::Permissions::can_burn(&role_type), Error::<T>::NotPermitted);

            Self::do_burn(sender, collection_id, item_id)?;

            Ok(())
        }

        /// Removes a Collection from existence
        ///
        /// Parameters:
        /// - `collection_id`: The identifier of the asset Collection to be destroyed.
        #[pallet::weight(<T as pallet::Config>::WeightInfo::destroy_collection())]
        #[transactional]
        pub fn destroy_collection(origin: OriginFor<T>, collection_id: T::NftCollectionId) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let role_type = Self::collections(collection_id)
            .map(|c| c.role_type)
            .ok_or(Error::<T>::CollectionUnknown)?;

            ensure!(T::Permissions::can_destroy(&role_type), Error::<T>::NotPermitted);

            Self::do_destroy_collection(sender, collection_id)?;

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
            role_type: Acc,
        },
        /// An Item was minted
        ItemMinted {
            owner: T::AccountId,
            collection_id: T::NftCollectionId,
            item_id: T::NftItemId,
        },
        /// An Item was transferred
        ItemTransferred {
            from: T::AccountId,
            to: T::AccountId,
            collection_id: T::NftCollectionId,
            item_id: T::NftItemId,
        },
        /// An Item was burned
        ItemBurned {
            owner: T::AccountId,
            collection_id: T::NftCollectionId,
            item_id: T::NftItemId,
        },
        /// A Collection was destroyed
        CollectionDestroyed {
            owner: T::AccountId,
            collection_id: T::NftCollectionId,
        },
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


