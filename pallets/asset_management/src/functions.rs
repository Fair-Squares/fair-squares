pub use super::*;
pub use frame_support::pallet_prelude::*;
#[allow(unused_imports)]
use num_traits::float::FloatCore;
pub use scale_info::prelude::boxed::Box;
pub use sp_core::H256;
use sp_runtime::traits::{StaticLookup, Zero};
impl<T: Config> Pallet<T> {
	pub fn approve_representative(origin: OriginFor<T>, who: T::AccountId) -> DispatchResult {
		let caller = ensure_signed(origin.clone())?;
		let mut representative = Roles::Pallet::<T>::get_pending_representatives(&who).unwrap();
		representative.activated = true;
		representative.assets_accounts.clear();
		representative.assets_accounts.push(caller);
		//get Rep number
		let mut index = Roles::Pallet::<T>::rep_num();
		//Update Rep index
		representative.index = index;

		Roles::RepresentativeLog::<T>::insert(&who, representative);
		Roles::RepApprovalList::<T>::remove(&who);
		Roles::AccountsRolesLog::<T>::insert(&who, Roles::Accounts::REPRESENTATIVE);
		let who2 = T::Lookup::unlookup(who.clone());

		//Check that the Representative is not already a Registrar
		//If a Representative is revoked from a given asset, and approved
		//for another asset, we don't want to repeat the registrar settings

		let mut check0 = false;
		let v = Ident::Pallet::<T>::registrars();
		for i in v {
			let reg = i.unwrap();
			if reg.account == who.clone() {
				check0 = true;
			}
		}

		if check0 == false {
			//Set the representative as a registrar
			Ident::Pallet::<T>::add_registrar(origin, who2).ok();

			//Set registrar fields
			let origin2: OriginFor<T> = RawOrigin::Signed(who).into();
			Ident::Pallet::<T>::set_fields(origin2.clone(), index, Default::default()).ok();

			//Set registrar fees
			let fee0 = Self::manage_bal_to_u128(T::RepFees::get()).unwrap();
			let bals0 = BalanceType::<T>::convert_to_balance(fee0);
			let fees = bals0.ident_bal;
			Ident::Pallet::<T>::set_fee(origin2, index, fees).ok();

			//Update Rep number
			index += 1;
			Roles::RepNumber::<T>::put(index);
		}

		Ok(())
	}

	pub fn revoke_representative(who: T::AccountId) -> DispatchResult {
		Roles::RepresentativeLog::<T>::mutate(&who, |val| {
			let mut val0 = val.clone().unwrap();
			val0.activated = false;
			*val = Some(val0);
		});
		Roles::AccountsRolesLog::<T>::remove(&who);

		Ok(())
	}

	pub fn calculate_guaranty(collection: T::NftCollectionId, item: T::NftItemId) -> u128 {
		let coeff = T::Guaranty::get() as u128;
		let ror = T::RoR::get();
		let price0 = Onboarding::Pallet::<T>::houses(collection, item).unwrap().price.unwrap();
		let price1 = Self::onboarding_bal_to_u128(ror.mul_floor(price0)).unwrap();
		let time = <T as Config>::Lease::get();
		let rent = ((price1 as f64) / time as f64).round();
		let amount: u128 = coeff * (rent as u128);
		amount
	}

	pub fn guaranty_payment(
		origin: OriginFor<T>,
		from: T::AccountId,
		collection: T::NftCollectionId,
		item: T::NftItemId,
	) -> DispatchResult {
		let creator = ensure_signed(origin.clone())?;

		//Calculate guaranty deposit using Return On Rent and guaranty coefficients found in runtime
		let amount = Self::calculate_guaranty(collection, item);

		//convert amount to payment_pallet compatible balance
		let bals0 = BalanceType::<T>::convert_to_balance(amount);
		let amount1 = bals0.payment_bal;

		//create payment_request
		Payment::Pallet::<T>::request_payment(origin, from.clone(), amount1).ok();

		//Store payment details
		let details = Payment::Pallet::<T>::get_payment_details(&from, &creator).unwrap();
		GuarantyPayment::<T>::insert(from.clone(), creator.clone(), details);

		Ok(())
	}

	pub fn owners_infos(asset_account: T::AccountId) -> Option<Share::Ownership<T>> {
		//Find the asset in Share Distributor using asset account
		let assets = Share::Virtual::<T>::iter_keys();
		let mut infos = None;
		for (i, j) in assets {
			let ownership = Share::Pallet::<T>::virtual_acc(i, j).unwrap();
			if asset_account.clone() == ownership.virtual_account {
				//Get the owners
				infos = Some(ownership);
			}
		}
		infos
	}

	pub fn tenant_link_asset(
		tenant: T::AccountId,
		collection: T::NftCollectionId,
		item: T::NftItemId,
		asset_account: T::AccountId,
	) -> DispatchResult {
		// Update tenant info
		//We first get the Return on Rent coeffient
		let ror = T::RoR::get();
		Roles::TenantLog::<T>::mutate(&tenant, |val| {
			let mut val0 = val.clone().unwrap();
			// get asset price
			let price0 = Onboarding::Pallet::<T>::houses(collection, item).unwrap().price.unwrap();
			let price1 = Self::onboarding_bal_to_u128(ror.mul_floor(price0)).unwrap();

			//Update rent in tenant infos added.
			let time = <T as Config>::Lease::get();
			let rent0 = ((price1 as f64) / time as f64).round();
			let rent1 = (rent0 as u128) * time as u128;
			let now = <frame_system::Pallet<T>>::block_number();
			let mut bals = BalanceType::<T>::convert_to_balance(rent0 as u128);
			let rent = bals.roles_bal;
			bals = BalanceType::<T>::convert_to_balance(rent1);
			let year_rent = bals.roles_bal;
			val0.rent = rent.into();
			val0.asset_account = Some(asset_account);
			val0.remaining_rent = year_rent;
			val0.remaining_payments = time as u8;
			val0.contract_start = now;
			*val = Some(val0);
		});

		// Update asset info
		Onboarding::Houses::<T>::mutate(collection, item, |house| {
			let mut house0 = house.clone().unwrap();
			house0.tenants.push(tenant);
			*house = Some(house0);
		});

		Ok(())
	}

	pub fn tenant_unlink_asset(
		tenant: T::AccountId,
		collection: T::NftCollectionId,
		item: T::NftItemId,
	) -> DispatchResult {
		// Update tenant info
		Roles::TenantLog::<T>::mutate(&tenant, |val| {
			let mut val0 = val.clone().unwrap();
			val0.asset_account = None;
			*val = Some(val0);
		});

		// Update asset info
		Onboarding::Houses::<T>::mutate(collection, item, |house| {
			let mut house0 = house.clone().unwrap();
			house0.tenants.retain(|t| *t != tenant);
			*house = Some(house0);
		});

		Ok(())
	}

	pub fn create_proposal_hash_and_note(
		caller: T::AccountId,
		proposal_call: pallet::Call<T>,
	) -> T::Hash {
		let origin: <T as frame_system::Config>::Origin = RawOrigin::Signed(caller.clone()).into();
		let proposal = Box::new(Self::get_formatted_call(proposal_call.into()));

		let call = Call::<T>::execute_call_dispatch { account_id: caller, proposal };
		let call_formatted = Self::get_formatted_call(call.into());
		let call_dispatch = Box::new(call_formatted);

		let proposal_hash = T::Hashing::hash_of(&call_dispatch);
		let proposal_encoded: Vec<u8> = call_dispatch.encode();
		match Dem::Pallet::<T>::note_preimage(origin, proposal_encoded) {
			Ok(_) => (),
			Err(x) if x == Error::<T>::DuplicatePreimage.into() => (),
			Err(x) => panic!("{:?}", x),
		}
		proposal_hash
	}

	pub fn caller_can_vote(caller: &T::AccountId, ownership: Share::Ownership<T>) -> bool {
		let owners = ownership.owners;
		owners.contains(caller)
	}

	pub fn manage_bal_to_u128(input: BalanceOf<T>) -> Option<u128> {
		input.try_into().ok()
	}
	pub fn assets_bal_to_u128(input: <T as Assetss::Config>::Balance) -> Option<u128> {
		input.try_into().ok()
	}
	pub fn roles_bal_to_u128(input: Roles::BalanceOf<T>) -> Option<u128> {
		input.try_into().ok()
	}
	pub fn onboarding_bal_to_u128(input: Onboarding::BalanceOf<T>) -> Option<u128> {
		input.try_into().ok()
	}

	// to be deleted//

	pub fn blocknumber_to_u128(input: BlockNumberFor<T>) -> Option<u128> {
		input.try_into().ok()
	}
	// to be deleted//

	pub fn get_formatted_call(call: <T as Config>::Call) -> <T as Config>::Call {
		call
	}

	///The function below is monitoring ongoing referendums
	///in order to update the status of corresponding Proposal Logs
	pub fn begin_block(now: T::BlockNumber) -> Weight {
		let max_block_weight = Weight::from_ref_time(1000_u64);
		if (now % <T as Config>::CheckPeriod::get()).is_zero() {
			let indexes = ProposalsIndexes::<T>::iter();
			for index in indexes {
				//check if the status is Finished
				let ref_infos: RefInfos<T> = Dem::Pallet::<T>::referendum_info(index.1).unwrap();
				let b = match ref_infos {
					pallet_democracy::ReferendumInfo::Finished { approved, end: _ } => {
						(1, approved)
					},
					_ => (0, false),
				};
				if b.0 == 1 {
					//get the local prop_infos and update vote result if referendum ended
					ProposalsLog::<T>::mutate(index.1, |val| {
						let mut val0 = val.clone().unwrap();
						if b.1 {
							val0.vote_result = VoteResult::ACCEPTED
						} else {
							val0.vote_result = VoteResult::REJECTED
						}
						*val = Some(val0)
					});
				}
			}
		}
		max_block_weight
	}

	///The function below regularly checks (every 15 days) for active Tenants on the blockchain
	///when a tenant is fund, his specific Rent-per-block is first calculated.
	///Next, based on the number of blocks ellapsed since the day of its activation,
	///the amount that should have been paid up to this point is calculated, and compared
	///with the amount that has been actually paid.
	///If the balance of the Tenant is negative, an event is emitted to notify him of his debt,
	///If not, nothing happens.
	///It will also distribute payed rent to the owners, according to their share.
	pub fn finish_block(now: T::BlockNumber) -> Weight {
		if (now % <T as Config>::CheckPeriod::get()).is_zero() {
			//get list of tenants
			let tenants = Roles::Pallet::<T>::tenant_list();
			for i in tenants {
				let tenant = Roles::Pallet::<T>::tenants(i).unwrap();
				if !tenant.asset_account.is_none() {
					let time = <T as Config>::Lease::get();
					let remaining_p = tenant.remaining_payments;
					let contract_begin = tenant.contract_start;

					let rent = Self::roles_bal_to_u128(tenant.rent).unwrap() * time as u128;
					let rent_float = rent as f64;
					let rent0 = Self::roles_bal_to_u128(tenant.rent).unwrap();

					//Calculate rent per block
					let total_blocks = <T as Config>::ContractLength::get();
					let mut rpb = Self::blocknumber_to_u128(total_blocks.clone()).unwrap();
					let mut rpb_float = rpb as f64;
					rpb_float = (rent_float / rpb_float).round();
					rpb = rpb_float as u128;

					//number of blocks from the start of the contract
					let blocks = Self::blocknumber_to_u128(now - contract_begin).unwrap();
					let amount_due = blocks.saturating_mul(rpb);

					//check how many rents were payed
					let payed = (time as u128 - remaining_p as u128) * rent.clone();
					let asset_account = tenant.asset_account.clone().unwrap();
					let asset_account_free_balance =
						<T as Config>::Currency::free_balance(&asset_account);

					let infos = Self::owners_infos(asset_account.clone()).unwrap();

					//Distribute rent to owners if number of rents
					//awaiting for distribution is greater than 0
					if infos.rent_nbr > 0 {
						//Get owners list

						let owners = infos.owners;
						let bals0 = BalanceType::<T>::convert_to_balance(rent0);
						let rent1 = bals0.manage_bal;

						//Get Asset_tokens infos
						let token_id = infos.token_id;
						let total_issuance =
							Assetss::Pallet::<T>::total_supply(token_id.clone().into());
						let total_issuance_float =
							Self::assets_bal_to_u128(total_issuance).unwrap() as f64;

						//Remove maintenance fees from rent and convert it to f64
						let maintenance = T::Maintenance::get() * rent1.clone();
						let distribute = rent1.saturating_sub(maintenance.clone());

						//Get the total amount to distribute
						let distribute_float = (Self::manage_bal_to_u128(distribute.clone())
							.unwrap() * infos.rent_nbr as u128) as f64;

						debug_assert!(distribute.clone() > Zero::zero());
						debug_assert!(distribute.clone() < rent1.clone());
						debug_assert!(maintenance.clone() < asset_account_free_balance);

						//Reserve maintenance fees
						let reservation =
							<T as Config>::Currency::reserve(&asset_account, maintenance.into());
						
							//Emmit maintenance fee payment event
						Self::deposit_event(Event::MaintenanceFeesPayment {
							tenant: tenant.account_id.clone(),
							when: now,
							asset_account: tenant.asset_account.unwrap(),
							amount: maintenance.clone(),
							
						});

						debug_assert!(reservation.is_ok());

						//Now distribute rent between owners according to their share
						for i in owners.clone() {
							//Get owner's share: we divide
							//the owner's tokens by the total token issuance, and multiply the result by
							//the total amount to be distributed.
							let share = Assetss::Pallet::<T>::balance(token_id.clone().into(), &i);
							let share_float = Self::assets_bal_to_u128(share).unwrap() as f64
								/ total_issuance_float;
							let amount_float = share_float * distribute_float.clone();
							let bals0 = BalanceType::<T>::convert_to_balance(amount_float as u128);
							let amount = bals0.manage_bal;
							<T as Config>::Currency::transfer(
								&asset_account,
								&i,
								amount,
								ExistenceRequirement::AllowDeath,
							)
							.ok();
						}

						//Emmit rent distribution event
						Self::deposit_event(Event::RentDistributed {
							owners,
							amount: distribute,
							when: now,
						});

						//Now return the awaiting payment number to 0
						let ownership_infos = Share::Virtual::<T>::iter_keys();
						for (i, j) in ownership_infos {
							let infos = Share::Pallet::<T>::virtual_acc(&i, &j).unwrap();
							if infos.virtual_account == asset_account {
								Share::Virtual::<T>::mutate(i.clone(), j.clone(), |val| {
									let mut val0 = val.clone().unwrap();
									val0.rent_nbr = 0;
									*val = Some(val0);
								});
							}
						}
					}

					//Calculate the debt if negative balance
					if payed < amount_due && (now % <T as Config>::RentCheck::get()).is_zero() {
						let tenant_debt0 = amount_due - payed;
						let bals0 = BalanceType::<T>::convert_to_balance(tenant_debt0);
						let debt = bals0.manage_bal;

						//Event to inform the tenant of the amount of his debt
						Self::deposit_event(Event::TenantDebt {
							tenant: tenant.account_id,
							debt,
							when: now,
						});
					}
				}
			}
		}
		Weight::zero()
	}
}
