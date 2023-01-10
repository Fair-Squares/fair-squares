pub use super::*;

impl<T: Config> Pallet<T> {
	/// A tenant requests access for an asset and asks judgement to a representative
	/// The origin must be a tenant account
	/// - info: profile information of the tenant
	/// - asset_type: type of the asset
	/// - asset_id: item_id of the asset
	/// - representative: account id of the representative
	pub fn request_helper(
		origin: OriginFor<T>,
		virtual_account: T::AccountId,
		info: Box<IdentityInfo<T::MaxAdditionalFields>>,		
	) -> DispatchResult {	


		let reps = Roles::RepresentativeLog::<T>::iter_keys();
		for i in reps{
			let rep = Roles::Pallet::<T>::reps(&i).unwrap();
			if rep.assets_accounts.contains(&virtual_account){
				Ident::Pallet::<T>::set_identity(origin.clone(), info.clone()).ok();
				Ident::Pallet::<T>::request_judgement(origin.clone(), rep.index, 50u32.into()).ok();

			}
		}

		Ok(())
	}
}
