use super::*;

pub use frame_system::pallet_prelude::*;
pub use codec::HasCompact;
pub use frame_support::{
    dispatch::{DispatchResult, EncodeLike},
    ensure,
    traits::{Currency,ReservableCurrency,ExistenceRequirement,tokens::nonfungibles::*, Get},
    transactional, BoundedVec,
};
pub use frame_system::{ensure_signed, RawOrigin};

pub use sp_runtime::{
    traits::{AtLeast32BitUnsigned, StaticLookup, Zero,Saturating},
    DispatchError,
};
pub use sp_std::boxed::Box;

impl<T: Config> Pallet<T> {
		pub fn create_asset(
			origin: OriginFor<T>,
			collection: NftCollectionOf,
			metadata: Nft::BoundedVecOfUnq<T>,
		) -> DispatchResult {
			
			let coll_id: T::NftCollectionId = collection.clone().value().into();
			let idx = collection.value() as usize;
			let _caller = ensure_signed(origin.clone()).unwrap();
			// Mint nft
			Nft::Pallet::<T>::mint(origin,collection.clone(),metadata).ok();
			// Get itemId and infos from minted nft
			let item_id: T::NftItemId = Nft::ItemsCount::<T>::get()[idx].into();
			let infos = Nft::Items::<T>::get(coll_id.clone(),item_id.clone()).unwrap();
			//Create Asset
			Asset::<T>::new(coll_id,item_id,infos).ok();

			Ok(())
		}

        pub fn do_buy(
            buyer: T::AccountId,
            collection: NftCollectionOf,
            item_id: T::NftItemId,
        ) -> DispatchResult {
            let collection_id: T::NftCollectionId = collection.clone().value().into();
            let owner = Nft::Pallet::<T>::owner(collection_id, item_id).ok_or(Error::<T>::CollectionOrItemUnknown)?;
            ensure!(buyer != owner, Error::<T>::BuyFromSelf);
    
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
    
    }

