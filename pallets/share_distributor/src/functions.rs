//1) create VA from nft collection & item Id's --> Done
//2) create tokens
//3) Use onboarding do_buy
//4) transfer tokens to owners
use super::*;
use enum_iterator::all;
use num_traits::float::FloatCore;
use sp_runtime::{traits::SaturatedConversion, FixedPointNumber, FixedU128};

impl<T: Config> Pallet<T> {
	///The function below create a virtual account from the NFT collection and item id's
	pub fn virtual_account(
		collection_id: T::NftCollectionId,
		item_id: T::NftItemId,
	) -> DispatchResult {
		//Create virtual account
		let text0 = format!("{:?}_{:?}_account", collection_id, item_id.clone());
		let bytes = text0.as_bytes();
		let array: &[u8; 8] = &bytes[0..8].try_into().unwrap();
		let account: T::AccountId = PalletId(*array).into_account_truncating();

		//Store account inside storage
		Ownership::<T>::new(collection_id, item_id, account.clone()).ok();
		Owners::<T>::new(account.clone()).ok();

		//The virtual account needs some initial funds to pay for asset creation fees
		//These funds could be provided by the FairSquare FeesAccount maintained in the
		//Onboarding pallet.
		let fees_account = Onboarding::Pallet::<T>::account_id();
		let fees = T::Fees::get();
		let res = <T as pallet::Config>::Currency::transfer(
			&fees_account,
			&account,
			fees,
			ExistenceRequirement::AllowDeath,
		);
		debug_assert!(res.is_ok());

		Ok(())
	}

	///This function executes all actions relatives to nft transfer from the seller to the virtual
	/// account
	pub fn nft_transaction(
		collection_id: T::NftCollectionId,
		item_id: T::NftItemId,
		virtual_id: T::AccountId,
	) -> DispatchResult {
		//Get collection
		let collection_vec = all::<Nft::PossibleCollections>().collect::<Vec<_>>();
		let _infos = Onboarding::Houses::<T>::get(collection_id, item_id).unwrap();
		let mut coll_id = Nft::PossibleCollections::HOUSES;
		for i in collection_vec.iter() {
			let val: T::NftCollectionId = i.value().into();
			if val == collection_id {
				coll_id = *i;
			}
		}
		//Execute NFT and money transfer
		Onboarding::Pallet::do_buy(coll_id, item_id, virtual_id, _infos).ok();

		Ok(())
	}

	///Collect contributors to the bid, and their shares
	pub fn owner_and_shares(
		collection_id: T::NftCollectionId,
		item_id: T::NftItemId,
		_total_tokens: <T as Assets::Config>::Balance,
	) -> Vec<(T::AccountId, u128)> {
		//Get owners and their reserved contribution to the bid

		let reservation_infos =
			HousingFund::Reservations::<T>::get((collection_id, item_id)).unwrap();
		let vec0 = reservation_infos.contributions;
		let price = reservation_infos.amount;
		let virtual_acc = Self::virtual_acc(collection_id, item_id).unwrap().virtual_account;

		let mut vec = Vec::new();
		for i in vec0.iter() {
			let price0 = Self::balance_to_u128_option0(price).unwrap();
			let contribution0 = Self::balance_to_u128_option0(i.1).unwrap();

			let price1 = Self::balance_to_f64_option0(price).unwrap();
			let contribution1 = Self::balance_to_f64_option0(i.1).unwrap();
			let frac = (contribution1 / price1).round();
			let mut share = FixedU128::saturating_from_rational(contribution0, price0)
				.saturating_mul_int(1000u128);
			let fl = share as f64;
			if (fl + 0.5) < frac {
				share += 1;
			}

			debug_assert!(share < 1000);
			debug_assert!(share > 0);

			vec.push((i.0.clone(), share));
			//Update Virtual_account storage
			Virtual::<T>::mutate(collection_id, item_id, |val| {
				let mut val0 = val.clone().unwrap();
				val0.owners.push(i.0.clone());
				*val = Some(val0);
			});
			//Update owners in Tokens storage
			Tokens::<T>::mutate(&virtual_acc, |val| {
				let amount: <T as Assets::Config>::Balance =
					share.saturated_into::<<T as Assets::Config>::Balance>();
				let mut val0 = val.clone().unwrap();
				val0.owners.push((i.0.clone(), amount));
				*val = Some(val0);
			});
		}
		vec
	}

	///Create 1000 Ownership tokens owned by a virtual account
	pub fn create_tokens(
		origin: OriginFor<T>,
		collection_id: T::NftCollectionId,
		item_id: T::NftItemId,
		account: T::AccountId,
	) -> DispatchResult {
		//Get token class Id:
		ensure!(Virtual::<T>::get(collection_id, item_id).is_some(), Error::<T>::InvalidValue);
		let token_id = Virtual::<T>::get(collection_id, item_id).unwrap().token_id;
		let to = T::Lookup::unlookup(account.clone());
		TokenId::<T>::mutate(|val| {
			let val0 = *val;
			*val = val0 + 1;
		});

		//Create token class
		let res = Assets::Pallet::<T>::force_create(
			origin.clone(),
			token_id.into(),
			to.clone(),
			true,
			One::one(),
		);
		debug_assert!(res.is_ok());

		//Set class metadata
		let token_name = format!("FairOwner_nbr{:?}", token_id.clone())
			.as_bytes()
			.to_vec()
			.try_into()
			.unwrap();
		let token_symbol =
			format!("FO{:?}", token_id.clone()).as_bytes().to_vec().try_into().unwrap();
		let decimals = 1;
		Assets::Pallet::<T>::force_set_metadata(
			origin,
			token_id.into(),
			token_name,
			token_symbol,
			decimals,
			false,
		)
		.ok();

		//mint 1000 tokens
		let res0 = Assets::Pallet::<T>::mint(
			RawOrigin::Signed(account.clone()).into(),
			token_id.into(),
			to,
			Self::u32_to_balance_option(1000).unwrap(),
		);
		debug_assert!(res0.is_ok());

		//Update supply in Tokens storage
		Tokens::<T>::mutate(account, |val| {
			let mut val0 = val.clone().unwrap();
			val0.supply = Assets::Pallet::<T>::total_supply(token_id.into());
			*val = Some(val0);
		});

		Ok(())
	}

	///Distribute the ownership tokens to the group of new owners
	pub fn distribute_tokens(
		account: T::AccountId,
		collection_id: T::NftCollectionId,
		item_id: T::NftItemId,
	) -> DispatchResult {
		ensure!(Virtual::<T>::get(collection_id, item_id).is_some(), Error::<T>::InvalidValue);
		let token_id = Virtual::<T>::get(collection_id, item_id).unwrap().token_id;
		let total_tokens = Assets::Pallet::<T>::total_supply(token_id.into());
		debug_assert!(total_tokens == Self::u32_to_balance_option(1000).unwrap());
		let shares = Self::owner_and_shares(collection_id, item_id, total_tokens);

		let from = T::Lookup::unlookup(account.clone());
		let origin: OriginFor<T> = RawOrigin::Signed(account).into();

		for share in shares.iter() {
			let amount0 = share.clone().1;
			debug_assert!(amount0 > 0);
			let amount: <T as Assets::Config>::Balance =
				share.clone().1.saturated_into::<<T as Assets::Config>::Balance>();
			debug_assert!(!amount.clone().is_zero());
			let to = T::Lookup::unlookup(share.clone().0);
			Assets::Pallet::<T>::force_transfer(
				origin.clone(),
				token_id.into(),
				from.clone(),
				to,
				amount,
			)
			.ok();
		}

		Ok(())
	}

	// Conversion of u32 to Balance
	pub fn u32_to_balance_option(input: u32) -> Option<T::Balance> {
		input.try_into().ok()
	}

	// Conversion of BalanceOf<T> to u128
	pub fn balance_to_u128_option0(input: HousingFund::BalanceOf<T>) -> Option<u128> {
		input.try_into().ok()
	}
	// Conversion of BalanceOf<T> to u128
	pub fn balance_to_u128_option(input: <T as Assets::Config>::Balance) -> Option<u128> {
		input.try_into().ok()
	}

	// Conversion of BalanceOf<T> to f64
	pub fn balance_to_f64_option0(input: HousingFund::BalanceOf<T>) -> Option<f64> {
		let integer: u64 = input.try_into().ok().unwrap();
		let float = integer as f64;
		Some(float)
	}
}
