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
		for i in reps {
			let rep = Roles::Pallet::<T>::reps(&i).unwrap();
			if rep.assets_accounts.contains(&virtual_account) {
				Ident::Pallet::<T>::set_identity(origin.clone(), info.clone()).ok();
				Ident::Pallet::<T>::request_judgement(origin.clone(), rep.index, 50u32.into()).ok();
			}
		}

		Ok(())
	}

	pub fn rent_helper(tenant_account: T::AccountId) -> DispatchResult {
		let tenant = Roles::Pallet::<T>::tenants(tenant_account.clone()).unwrap();
		let total_rent = tenant.remaining_rent;
		let remaining_p = tenant.remaining_payments;
		let rent0: u128 = Roles::Pallet::<T>::balance_to_u128_option(tenant.rent).unwrap();
		let rent = Self::u128_to_balance_option(rent0).unwrap();
		let asset_account = tenant.asset_account.unwrap();
		<T as Config>::Currency::transfer(
			&tenant_account,
			&asset_account,
			rent,
			ExistenceRequirement::AllowDeath,
		)
		.ok();

		Roles::TenantLog::<T>::mutate(tenant_account, |val| {
			let mut val0 = val.clone().unwrap();
			val0.remaining_rent = total_rent.saturating_sub(tenant.rent);
			val0.remaining_payments = remaining_p - 1;
			*val = Some(val0);
		});

		Ok(())
	}

	pub fn payment_helper(
		from: OriginFor<T>,
		virtual_account: T::AccountId,
		collection: T::NftCollectionId,
		item: T::NftItemId,
	) -> DispatchResult {
		let tenant = ensure_signed(from.clone())?;

		//Accept and pay the guaranty
		Payment::Pallet::<T>::accept_and_pay(from.clone(), virtual_account.clone()).ok();
		let origin2 = frame_system::RawOrigin::Signed(virtual_account.clone());

		//Change payment state in Asset_Management storage
		Assets::GuarantyPayment::<T>::mutate(tenant.clone(), virtual_account, |val| {
			let mut infos = val.clone().unwrap();
			infos.state = Payment::PaymentState::PaymentCompleted;
			*val = Some(infos);
		});

		//Connect tenant with asset
		Assets::Pallet::<T>::link_tenant_to_asset(origin2.into(), tenant, collection, item).ok();

		Ok(())
	}

	pub fn balance_to_u128_option(input: BalanceOf<T>) -> Option<u128> {
		input.try_into().ok()
	}
	pub fn u128_to_balance_option(input: u128) -> Option<BalanceOf<T>> {
		input.try_into().ok()
	}
}
