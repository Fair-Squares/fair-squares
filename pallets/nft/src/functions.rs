pub use super::*;
pub use frame_system::{pallet_prelude::OriginFor, RawOrigin};

pub trait CreateTypedCollection<AccountId, CollectionId>: Create<AccountId> {
	/// This function create an NFT collection of `created_by` type.
	fn create_typed_collection(owner: AccountId, collection_id: CollectionId) -> DispatchResult;
}

pub trait ReserveCollectionId<CollectionId> {
	/// This function returns `true` if collection id is from the reserved range, `false` otherwise.
	fn is_id_reserved(id: CollectionId) -> bool;
}

impl<T: Config> Pallet<T> {
	pub fn collection_owner(collection_id: T::NftCollectionId) -> Option<T::AccountId> {
		pallet_uniques::Pallet::<T>::collection_owner(collection_id.into())
	}

	pub fn owner(collection_id: T::NftCollectionId, item_id: T::NftItemId) -> Option<T::AccountId> {
		pallet_uniques::Pallet::<T>::owner(collection_id.into(), item_id.into())
	}

	pub fn do_create_collection(
		owner: T::AccountId,
		collection_id: T::NftCollectionId,
		created_by: Acc,
		metadata: BoundedVecOfUnq<T>,
	) -> DispatchResult {
		let deposit_info = match T::Permissions::has_deposit(&created_by) {
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

		Collections::<T>::insert(collection_id, CollectionInfo { created_by, metadata });

		Self::deposit_event(Event::CollectionCreated { owner, collection_id, created_by });

		Ok(())
	}

	pub fn do_mint(
		owner: T::AccountId,
		collection_id: T::NftCollectionId,
		item_id: T::NftItemId,
		metadata: BoundedVecOfUnq<T>,
	) -> DispatchResult {
		ensure!(Collections::<T>::contains_key(collection_id), Error::<T>::CollectionUnknown);
		pallet_uniques::Pallet::<T>::do_mint(
			collection_id.into(),
			item_id.into(),
			owner.clone(),
			|_details| Ok(()),
		)?;

		Items::<T>::insert(collection_id, item_id, ItemInfo { metadata });

		Self::deposit_event(Event::ItemMinted { owner, collection_id, item_id });

		Ok(())
	}

	pub fn set_metadata(
		owner: T::AccountId,
		collection_id: T::NftCollectionId,
		item_id: T::NftItemId,
		metadata: BoundedVecOfUnq<T>,
	) -> DispatchResult {
		ensure!(Collections::<T>::contains_key(collection_id), Error::<T>::CollectionUnknown);
		let origin = RawOrigin::Signed(owner);

		let res0 = pallet_uniques::Pallet::<T>::set_metadata(
			origin.into(),
			collection_id.into(),
			item_id.into(),
			metadata.clone(),
			false,
		);
		debug_assert!(res0.is_ok());

		Items::<T>::mutate(collection_id, item_id, |val| {
			let mut val0 = val.clone().unwrap();
			val0.metadata = metadata;
			*val = Some(val0);
		});

		//Self::deposit_event(Event::ItemMinted { owner, collection_id, item_id });

		Ok(())
	}

	pub fn do_transfer(
		collection_id: T::NftCollectionId,
		item_id: T::NftItemId,
		from: T::AccountId,
		to: T::AccountId,
	) -> DispatchResult {
		if from == to {
			return Ok(())
		}

		pallet_uniques::Pallet::<T>::do_transfer(
			collection_id.into(),
			item_id.into(),
			to.clone(),
			|_collection_details, _item_details| {
				Self::deposit_event(Event::ItemTransferred { from, to, collection_id, item_id });
				Ok(())
			},
		)
	}

	pub fn do_burn(
		owner: T::AccountId,
		collection_id: T::NftCollectionId,
		item_id: T::NftItemId,
	) -> DispatchResult {
		pallet_uniques::Pallet::<T>::do_burn(
			collection_id.into(),
			item_id.into(),
			|_collection_details, _item_details| Ok(()),
		)?;

		Items::<T>::remove(collection_id, item_id);

		Self::deposit_event(Event::ItemBurned { owner, collection_id, item_id });

		Ok(())
	}

	pub fn do_destroy_collection(
		owner: T::AccountId,
		collection_id: T::NftCollectionId,
	) -> DispatchResult {
		let witness = pallet_uniques::Pallet::<T>::get_destroy_witness(&collection_id.into())
			.ok_or(Error::<T>::CollectionUnknown)?;

		// witness struct is empty because we don't allow destroying a Collection with existing
		// items
		ensure!(witness.items == 0u32, Error::<T>::TokenCollectionNotEmpty);

		pallet_uniques::Pallet::<T>::do_destroy_collection(
			collection_id.into(),
			witness,
			Some(owner.clone()),
		)?;
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

	fn owned_in_collection(
		collection: &Self::CollectionId,
		who: &T::AccountId,
	) -> Box<dyn Iterator<Item = Self::ItemId>> {
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
	fn create_collection(
		collection: &Self::CollectionId,
		who: &T::AccountId,
		_admin: &T::AccountId,
	) -> DispatchResult {
		Self::do_create_collection(
			who.clone(),
			*collection,
			Default::default(),
			BoundedVec::default(),
		)?;

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

		// We can return empty struct here because we don't allow destroying a Collection with
		// existing items
		Ok(DestroyWitness { items: 0, item_metadatas: 0, attributes: 0 })
	}
}

impl<T: Config> Mutate<T::AccountId> for Pallet<T> {
	fn mint_into(
		collection: &Self::CollectionId,
		item: &Self::ItemId,
		who: &T::AccountId,
	) -> DispatchResult {
		Self::do_mint(who.clone(), *collection, *item, BoundedVec::default())?;

		Ok(())
	}

	fn burn(
		collection: &Self::CollectionId,
		item: &Self::ItemId,
		_maybe_check_owner: Option<&T::AccountId>,
	) -> DispatchResult {
		let owner = Self::owner(*collection, *item).ok_or(Error::<T>::ItemUnknown)?;

		Self::do_burn(owner, *collection, *item)?;

		Ok(())
	}
}

impl<T: Config> CreateTypedCollection<T::AccountId, T::NftCollectionId> for Pallet<T> {
	fn create_typed_collection(
		owner: T::AccountId,
		collection_id: T::NftCollectionId,
	) -> DispatchResult {
		let created_by = Roles::Pallet::<T>::get_roles(&owner).unwrap();
		ensure!(T::Permissions::can_create(&created_by), Error::<T>::NotPermitted);
		Self::do_create_collection(owner, collection_id, created_by, Default::default())
	}
}

impl<T: Config> ReserveCollectionId<T::NftCollectionId> for Pallet<T> {
	fn is_id_reserved(id: T::NftCollectionId) -> bool {
		id == T::ReserveCollectionIdUpTo::get()
	}
}
