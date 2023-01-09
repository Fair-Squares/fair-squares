pub use super::*;

impl<T: Config> Pallet<T> {
	/// A tenant requests access for an asset and asks judgement to a representative
	/// The origin must be a tenant account
	/// - info: profile information of the tenant
	/// - asset_type: type of the asset
	/// - asset_id: item_id of the asset
	/// - representative: account id of the representative
	pub fn request_asset(
		origin: OriginFor<T>,
		info: Box<IdentityInfo<T::MaxAdditionalFields>>,
		asset_type: Nft::PossibleCollections,
		asset_id: T::NftItemId,
		//representative: T::AccountId,
	) -> DispatchResult {
		let caller = ensure_signed(origin.clone())?;

		// Ensure that the caller has the tenancy role
		ensure!(Roles::TenantLog::<T>::contains_key(caller), Error::<T>::NotATenant);

		// Ensure that the asset is valid
		let collection_id: T::NftCollectionId = asset_type.value().into();
		let ownership = Share::Pallet::<T>::virtual_acc(collection_id, asset_id);
		ensure!(ownership.is_some(), Error::<T>::NotAnAsset);
		let virtual_account = ownership.unwrap().virtual_account;


		let reps = Roles::RepresentativeLog::<T>::iter_keys();
		for i in reps{
			let rep = Roles::Pallet::<T>::reps(&i).unwrap();
			if rep.assets_accounts.contains(&virtual_account){
				Ident::Pallet::<T>::set_identity(origin.clone(), info.clone()).ok();
				Ident::Pallet::<T>::request_judgement(origin.clone(), rep.index, 50u32.into()).ok();

			}
		}


		// Ensure that the account specified by `representative` has representative role
		//let rep = Roles::Pallet::<T>::reps(representative);
		//ensure!(rep.is_some(), Error::<T>::NotARepresentative);
		//let rep = rep.unwrap();

		// Ensure that the asset is linked with the representative
		//ensure!(rep.assets_accounts.contains(&virtual_account), Error::<T>::AssetNotLinked);

		//Ident::Pallet::<T>::set_identity(origin.clone(), info).ok();
		//Ident::Pallet::<T>::request_judgement(origin, rep.index, 50u32.into()).ok();

		Ok(())
	}
}
