pub use super::*;
pub use frame_system::{pallet_prelude::OriginFor, RawOrigin};



impl<T: Config> Pallet<T> {
	pub fn collection_owner(collection_id: T::NftCollectionId) -> Option<T::AccountId> {
		pallet_nfts::Pallet::<T>::collection_owner(collection_id.into())
	}

	pub fn owner(collection_id: T::NftCollectionId, item_id: T::NftItemId ) -> Option<T::AccountId> {
		pallet_nfts::Pallet::<T>::owner(collection_id.into(), item_id.into())
	}

	pub fn do_create_collection(
		owner: T::AccountId,
		collection_id: T::NftCollectionId,
		created_by: Acc,
		metadata: BoundedVecOfNfts<T>,
	) -> DispatchResult {
		let deposit_info = match T::Permissions::has_deposit(&created_by) {
			false => (Zero::zero(), true),
			true => (T::CollectionDeposit::get(), false),
		};
		pallet_nfts::Pallet::<T>::do_create_collection(
			collection_id.into(),
			owner.clone(),
			owner.clone(),
			pallet_nfts::CollectionConfig{
				settings: pallet_nfts::CollectionSettings::all_enabled(),
		max_supply: None,
		mint_settings: pallet_nfts::MintSettings::default(),
			},
			deposit_info.0,
			pallet_nfts::Event::Created {
				collection: collection_id.into(),
				creator: owner.clone(),
				owner: owner.clone(),
			},
		)?;

		Collections::<T>::insert(collection_id, CollectionInfo { created_by, metadata });

		Self::deposit_event(Event::CollectionCreated { owner, collection_id, created_by });

		Ok(())
	}

	
	pub fn set_metadata(
		owner: T::AccountId,
		collection_id: T::NftCollectionId,
		item_id: T::NftItemId ,
		metadata: BoundedVecOfNfts<T>,
	) -> DispatchResult {
		ensure!(Collections::<T>::contains_key(collection_id), Error::<T>::CollectionUnknown);
		let origin = RawOrigin::Signed(owner.clone());

		let res0 = pallet_nfts::Pallet::<T>::set_metadata(
			origin.into(),
			collection_id.into(),
			item_id.into(),
			metadata.clone(),
		);
		debug_assert!(res0.is_ok());

		Items::<T>::mutate(collection_id, item_id, |val| {
			if val.is_some(){
				let mut val0 = val.clone().unwrap();
			val0.metadata = metadata;
			*val = Some(val0);
			}			
		});

		Self::deposit_event(Event::ItemMinted { owner, collection_id, item_id });

		Ok(())
	}


	

	pub fn do_destroy_collection(
		owner: T::AccountId,
		collection_id: T::NftCollectionId,
	) -> DispatchResult {
		let coll_id0:<T as Nfts>::CollectionId=collection_id.into();
		let collection_details = pallet_nfts::Collection::<T>::get(coll_id0).unwrap();
		let witness = collection_details.destroy_witness();

		// witness struct is empty because we don't allow destroying a Collection with existing
		// items
		ensure!(witness.item_metadatas == 0u32, Error::<T>::TokenCollectionNotEmpty);

		pallet_nfts::Pallet::<T>::do_destroy_collection(
			collection_id.into(),
			witness,
			Some(owner.clone()),
		)?;
		Collections::<T>::remove(collection_id);

		Self::deposit_event(Event::CollectionDestroyed { owner, collection_id });
		Ok(())
	}
	pub fn get_origin(account_id: T::AccountId) -> <T as frame_system::Config>::RuntimeOrigin {
		frame_system::RawOrigin::Signed(account_id).into()
	}
}

impl<T: Config> Inspect<T::AccountId> for Pallet<T> {
	type ItemId = T::NftItemId ;
	type CollectionId = T::NftCollectionId;

	fn owner(collection: &Self::CollectionId, item: &Self::ItemId) -> Option<T::AccountId> {
		Self::owner(*collection, *item)
	}

	fn collection_owner(collection: &Self::CollectionId) -> Option<T::AccountId> {
		Self::collection_owner(*collection)
	}
	
}





