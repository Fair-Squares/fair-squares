use super::*;

pub use frame_system::pallet_prelude::*;
pub use codec::HasCompact;
pub use frame_support::{
    dispatch::{DispatchResult, EncodeLike,Dispatchable},
    ensure,
    traits::{Currency,ReservableCurrency,BalanceStatus ,ExistenceRequirement,tokens::nonfungibles::*, Get},
    transactional, BoundedVec,
};
pub use frame_system::{ensure_signed, RawOrigin};

pub use sp_runtime::{
    traits::{AtLeast32BitUnsigned, StaticLookup, Zero,Saturating,AccountIdConversion},
    DispatchError,
};
pub use sp_std::boxed::Box;


impl<T: Config> Pallet<T> {
		pub fn create_asset(
			origin: OriginFor<T>,
			collection: NftCollectionOf,
			metadata: Nft::BoundedVecOfUnq<T>,
            new_price: Option<BalanceOf<T>>,
            item_id: T::NftItemId,
		) -> DispatchResult {
			
			let coll_id: T::NftCollectionId = collection.clone().value().into();
			// Mint nft
			Nft::Pallet::<T>::mint(origin.clone(),collection.clone(),metadata).ok();
			
			let infos = Nft::Items::<T>::get(coll_id.clone(),item_id.clone()).unwrap();
            // Set asset price
            Self::price(origin,collection,item_id.clone(),new_price.clone()).ok();
			// Create Asset
			Asset::<T>::new(coll_id,item_id,infos,new_price).ok();

			Ok(())
		}

        pub fn status(collection: NftCollectionOf, item_id: T::NftItemId, status:AssetStatus ){
			let collection_id: T::NftCollectionId = collection.clone().value().into();
			Houses::<T>::mutate(collection_id,item_id.clone(),|val|{
				let mut v0=val.clone().unwrap();
				v0.status = status;
				*val = Some(v0);

			});

		}
		
		
		

        pub fn price(
			origin: OriginFor<T>,
			collection: NftCollectionOf,
			item_id: T::NftItemId,
			new_price: Option<BalanceOf<T>>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let collection_id: T::NftCollectionId = collection.value().into();

			ensure!(
				pallet_nft::Pallet::<T>::owner(collection_id, item_id) == Some(sender.clone()),
				Error::<T>::NotTheTokenOwner
			);

			Prices::<T>::mutate_exists(collection_id, item_id, |price| *price = new_price);

			Self::deposit_event(Event::TokenPriceUpdated {
				who: sender,
				collection: collection_id,
				item: item_id,
				price: new_price,
			});

			Ok(())
		}

		// Conversion of u64 to BalanxceOf<T>
	pub fn u64_to_balance_option(input: u64) -> Option<BalanceOf<T>> {
		input.try_into().ok()
	}

	// Conversion of BalanceOf<T> to u32
	pub fn balance_to_u64_option(input: BalanceOf<T>) -> Option<u64> {
		input.try_into().ok()
	}

    
    }

