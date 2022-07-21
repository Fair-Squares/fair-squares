
pub use super::*;


impl<T: Config> Pallet<T> {
    pub fn collection_owner(collection_id: T::NftCollectionId) -> Option<T::AccountId> {
        pallet_uniques::Pallet::<T>::collection_owner(collection_id.clone().into())
    }

    pub fn owner(collection_id: T::NftCollectionId, item_id: T::NftItemId) -> Option<T::AccountId> {
        pallet_uniques::Pallet::<T>::owner(collection_id.into(), item_id.into())
    }

    pub fn do_create_collection(
        owner: T::AccountId,
        collection_id: T::NftCollectionId,
        role_type: Acc,
        metadata: BoundedVecOfUnq<T>,
    ) -> DispatchResult {
        let deposit_info = match T::Permissions::has_deposit(&role_type) {
            false => (Zero::zero(), true),
            true => (T::CollectionDeposit::get(), false),
        };
        pallet_uniques::Pallet::<T>::do_create_collection(
            collection_id.into(),
            owner.clone(),
            owner.clone(),
            deposit_info.0,
            deposit_info.1,
            pallet_uniques::Event::Created {
                collection: collection_id.into(),
                creator: owner.clone(),
                owner: owner.clone(),
            },
        )?;

        Collections::<T>::insert(collection_id, CollectionInfo { metadata });

        Self::deposit_event(Event::CollectionCreated {
            owner,
            collection_id,
        });

        Ok(())
    }

    pub fn do_mint(
        owner: T::AccountId,
        collection_id: T::NftCollectionId,
        item_id: T::NftItemId,
        metadata: BoundedVecOfUnq<T>,
    ) -> DispatchResult {
        
        pallet_uniques::Pallet::<T>::do_mint(collection_id.into(), item_id.into(), owner.clone(), |_details| Ok(()))?;

		let share = TokenByOwnerData::<T>{
			percent_owned: 100000,
			item: ItemInfo {
				metadata: metadata.clone(),
			},
		};
		let key_exists = TokenByOwner::<T>::contains_key(&owner,(&collection_id, &item_id));
		if key_exists == false{
			TokenByOwner::<T>::insert(&owner, (&collection_id, &item_id), share);
		}

        Items::<T>::insert(collection_id, item_id, ItemInfo { metadata });

        Self::deposit_event(Event::ItemMinted {
            owner,
            collection_id,
            item_id,
        });

        Ok(())
    }

    pub fn do_transfer(
        collection_id: T::NftCollectionId,
        item_id: T::NftItemId,
        from: T::AccountId,
        to: T::AccountId,
		share: u32,
    ) -> DispatchResult {
        
        if from == to {
            return Ok(());
        }

        
		let mut owner_data =  TokenByOwner::<T>::get(&from,(&collection_id,&item_id)).unwrap();
		let oldshare = owner_data.percent_owned;
		ensure!(oldshare>=share, Error::<T>::NotPermitted);
		let newshare = oldshare-&share;
		owner_data.percent_owned = newshare;
		TokenByOwner::<T>::mutate(&from,(&collection_id,&item_id),|val|{
			*val = Some(owner_data);
		});

		
		if TokenByOwner::<T>::contains_key(&to,(&collection_id,&item_id)){
			let mut owner_data1 =  TokenByOwner::<T>::get(&to,(&collection_id,&item_id)).unwrap();
			let oldshare1 = owner_data1.percent_owned;
			let new = &share+oldshare1;
			ensure!(new <= 100000, Error::<T>::NotPermitted);
			owner_data1.percent_owned = new;
			TokenByOwner::<T>::mutate(&to,(&collection_id,&item_id),|val|{
				*val = Some(owner_data1);
			});
		} else{
			let owner_data1 = TokenByOwnerData::<T>{
				percent_owned: share,
				item: Items::<T>::get(&collection_id,&item_id).unwrap(),
			};
			TokenByOwner::<T>::insert(&to,(&collection_id,&item_id),owner_data1);
		}
		

        pallet_uniques::Pallet::<T>::do_transfer(
            collection_id.into(),
            item_id.into(),
            to.clone(),
            |_collection_details, _item_details| {
                let owner = Self::owner(collection_id, item_id).ok_or(Error::<T>::ItemUnknown)?;
                ensure!(owner == from, Error::<T>::NotPermitted);
                Self::deposit_event(Event::ItemTransferred {
                    from,
                    to,
                    collection_id,
                    item_id,
                });
                Ok(())
            },
        )
    }

    pub fn do_burn(owner: T::AccountId, collection_id: T::NftCollectionId, item_id: T::NftItemId) -> DispatchResult {
        
        pallet_uniques::Pallet::<T>::do_burn(
            collection_id.into(),
            item_id.into(),
            |_collection_details, _item_details| {
                let iowner = Self::owner(collection_id, item_id).ok_or(Error::<T>::ItemUnknown)?;
                ensure!(owner == iowner, Error::<T>::NotPermitted);
                Ok(())
            },
        )?;

        Items::<T>::remove(collection_id, item_id);

        Self::deposit_event(Event::ItemBurned {
            owner,
            collection_id,
            item_id,
        });

        Ok(())
    }

    pub fn do_destroy_collection(owner: T::AccountId, collection_id: T::NftCollectionId) -> DispatchResult {
        
        let witness =
            pallet_uniques::Pallet::<T>::get_destroy_witness(&collection_id.into()).ok_or(Error::<T>::CollectionUnknown)?;

        // witness struct is empty because we don't allow destroying a Collection with existing items
        ensure!(witness.items == 0u32, Error::<T>::TokenCollectionNotEmpty);

        pallet_uniques::Pallet::<T>::do_destroy_collection(collection_id.into(), witness, Some(owner.clone()))?;
        Collections::<T>::remove(collection_id);

        Self::deposit_event(Event::CollectionDestroyed { owner, collection_id });
        Ok(())
    }
}

impl<T: Config> Inspect<T::AccountId> for Pallet<T> {
    type ItemId = T::NftItemId;
    type CollectionId = T::NftCollectionId;

    fn owner(collection: &Self::CollectionId, item: &Self::ItemId) -> Option<T::AccountId> {
        Self::owner(*collection, *item)
    }

    fn collection_owner(collection: &Self::CollectionId) -> Option<T::AccountId> {
        Self::collection_owner(*collection)
    }

}

impl<T: Config> InspectEnumerable<T::AccountId> for Pallet<T> {
    fn collections() -> Box<dyn Iterator<Item = Self::CollectionId>> {
        Box::new(Collections::<T>::iter_keys())
    }

    fn items(collection: &Self::CollectionId) -> Box<dyn Iterator<Item = Self::ItemId>> {
        Box::new(Items::<T>::iter_key_prefix(collection))
    }

    fn owned(who: &T::AccountId) -> Box<dyn Iterator<Item = (Self::CollectionId, Self::ItemId)>> {
        Box::new(
            pallet_uniques::Pallet::<T>::owned(who)
                .map(|(collection_id, item_id)| (collection_id.into(), item_id.into())),
        )
    }

    fn owned_in_collection(collection: &Self::CollectionId, who: &T::AccountId) -> Box<dyn Iterator<Item = Self::ItemId>> {
        Box::new(
            pallet_uniques::Pallet::<T>::owned_in_collection(
                &(Into::<<T as pallet_uniques::Config>::CollectionId>::into(*collection)),
                who,
            )
            .map(|i| i.into()),
        )
    }
}

impl<T: Config> Create<T::AccountId> for Pallet<T> {
    fn create_collection(collection: &Self::CollectionId, who: &T::AccountId, _admin: &T::AccountId) -> DispatchResult {
        Self::do_create_collection(who.clone(), *collection, Default::default(), BoundedVec::default())?;

        Ok(())
    }
}

impl<T: Config> Destroy<T::AccountId> for Pallet<T> {
    type DestroyWitness = pallet_uniques::DestroyWitness;

    fn get_destroy_witness(collection: &Self::CollectionId) -> Option<Self::DestroyWitness> {
        pallet_uniques::Pallet::<T>::get_destroy_witness(
            &(Into::<<T as pallet_uniques::Config>::CollectionId>::into(*collection)),
        )
    }

    fn destroy(
        collection: Self::CollectionId,
        _witness: Self::DestroyWitness,
        _maybe_check_owner: Option<T::AccountId>,
    ) -> Result<Self::DestroyWitness, DispatchError> {
        let owner = Self::collection_owner(collection).ok_or(Error::<T>::CollectionUnknown)?;

        Self::do_destroy_collection(owner, collection)?;

        // We can return empty struct here because we don't allow destroying a Collection with existing items
        Ok(DestroyWitness {
            items: 0,
            item_metadatas: 0,
            attributes: 0,
        })
    }
}

impl<T: Config> Mutate<T::AccountId> for Pallet<T> {
    fn mint_into(collection: &Self::CollectionId, item: &Self::ItemId, who: &T::AccountId) -> DispatchResult {
        Self::do_mint(who.clone(), *collection, *item, BoundedVec::default())?;

        Ok(())
    }

    fn burn(collection: &Self::CollectionId, item: &Self::ItemId,maybe_check_owner: Option<&T::AccountId>) -> DispatchResult {
        let owner = Self::owner(*collection, *item).ok_or(Error::<T>::ItemUnknown)?;

        Self::do_burn(owner, *collection, *item)?;

        Ok(())
    }
}



