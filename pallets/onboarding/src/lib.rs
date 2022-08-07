#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]


mod types;
mod functions;

pub use types::*;
pub use functions::*;

pub use pallet_roles as Roles;
pub use pallet_nft as Nft;
pub use pallet_sudo as Sudo;

pub use pallet::*;

//#[cfg(test)]
//mod mock;

//#[cfg(test)]
//mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::WeightInfo;


type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;


#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;	

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + Roles::Config + Nft::Config +Sudo::Config{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: ReservableCurrency<Self::AccountId>;
		type Prop: Parameter + Dispatchable<Origin = Self::Origin> + From<Call<Self>>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::storage]
	#[pallet::getter(fn prices)]
	/// Stores token info
	pub(super) type Prices<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::NftCollectionId,
		Blake2_128Concat,
		T::NftItemId,
		BalanceOf<T>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn houses)]
	/// Stores token info
	pub(super) type Houses<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::NftCollectionId,
		Blake2_128Concat,
		T::NftItemId,
		Asset<T>,
		OptionQuery,
	>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),

		/// The price for a token was updated
		TokenPriceUpdated {
			who: T::AccountId,
			collection: T::NftCollectionId,
			item: T::NftItemId,
			price: Option<BalanceOf<T>>,
		},

		/// Token was sold to a new owner
		TokenSold {
			owner: T::AccountId,
			buyer: T::AccountId,
			collection: T::NftCollectionId,
			item: T::NftItemId,
			price: BalanceOf<T>,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		/// The acting account does not correspond to the token owner
		NotTheTokenOwner,
		/// Class or instance does not exist
		CollectionOrItemUnknown,
		/// Cannot buy from yourself
		BuyFromSelf,
		/// Item is not for sale 
		NotForSale,
		///NFT Item cannot be edited
		CannotEditItem,
		///Nft Item has not been approved for sell
		ItemNotApproved,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		//#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::do_something(100))]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored(something, who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		#[transactional]
		pub fn set_price(
			origin: OriginFor<T>,
			collection: NftCollectionOf,
			item_id: T::NftItemId,
			new_price: Option<BalanceOf<T>>,
		) -> DispatchResult {
			let collection_id: T::NftCollectionId = collection.clone().value().into();
			ensure!(Houses::<T>::contains_key(collection_id.clone(),item_id.clone()),Error::<T>::CollectionOrItemUnknown);
			
			let asset = Self::houses(collection_id,item_id.clone()).unwrap();
			let status = asset.status;
			ensure!(status == AssetStatus::EDITING || status == AssetStatus::REJECTEDIT,Error::<T>::CannotEditItem);

			Self::price(origin,collection,item_id,new_price).ok();

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn do_buy(
            origin: OriginFor<T>,
            collection: NftCollectionOf,
            item_id: T::NftItemId,
			_infos: ItemInfoOf<T>,
        ) -> DispatchResult {
			let buyer = ensure_signed(origin.clone()).unwrap();
            let collection_id: T::NftCollectionId = collection.clone().value().into();

			//Chaeck that the house item exists and has the correct status
			ensure!(Houses::<T>::contains_key(collection_id.clone(),item_id.clone()),Error::<T>::CollectionOrItemUnknown);			
			let asset = Self::houses(collection_id,item_id.clone()).unwrap();
			let status = asset.status;
			ensure!(status == AssetStatus::APPROVED,Error::<T>::ItemNotApproved);

			//Check that the owner is not the buyer 
            let owner = Nft::Pallet::<T>::owner(collection_id, item_id).ok_or(Error::<T>::CollectionOrItemUnknown)?;
            ensure!(buyer != owner, Error::<T>::BuyFromSelf);
			
			//Execute transaction
            let owner_origin = T::Origin::from(RawOrigin::Signed(owner.clone()));
            let price = Prices::<T>::get(collection_id, item_id).unwrap();
            <T as Config>::Currency::transfer(&buyer, &owner, price, ExistenceRequirement::KeepAlive)?;
            let to = T::Lookup::unlookup(buyer.clone());
            Nft::Pallet::<T>::transfer(owner_origin, collection, item_id, to)?;
            Self::deposit_event(Event::TokenSold {
                owner,
                buyer,
                collection: collection_id,
                item: item_id,
                price,
            });
            Ok(())
        }
        
		

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		#[transactional]
		pub fn submit_proposal(
            origin: OriginFor<T>,
            collection: NftCollectionOf,
            price: Option<BalanceOf<T>>,
			metadata: Nft::BoundedVecOfUnq<T>,
            )-> DispatchResult {
				let _caller = ensure_signed(origin.clone()).unwrap();
				let idx = collection.clone().value() as usize;
				
				// Get itemId and infos from minted nft
				let item_id: T::NftItemId = Nft::ItemsCount::<T>::get()[idx].into();

				//Create asset
				Self::create_asset(origin,collection.clone(),metadata,price,item_id.clone()).ok();

				//Change asset status to REVIEWING
				let collection_id: T::NftCollectionId = collection.clone().value().into();
				Houses::<T>::mutate(collection_id.clone(),item_id.clone(),|val|{
					let mut v0 =val.clone().unwrap();
					v0.status = AssetStatus::REVIEWING;

					*val = Some(v0);
				});
				let house = Self::houses(collection_id,item_id.clone()).unwrap();
				let infos = house.infos;
			
				//Create Call for the sell/buy transaction
				let _call:T::Prop = Call::<T>::do_buy{collection: collection,item_id: item_id,infos:infos}.into();
				

				Ok(())
                
            }

			

		

		




		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => return Err(Error::<T>::NoneValue.into()),
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				},
			}
		}
	}
}
