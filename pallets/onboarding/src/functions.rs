use super::*;

pub use codec::HasCompact;
pub use frame_support::{
	codec::{Decode, Encode},
	dispatch::{DispatchResult, Dispatchable, EncodeLike},
	ensure,
	inherent::Vec,
	traits::{
		tokens::nonfungibles::*, BalanceStatus, Currency, ExistenceRequirement, Get,
		ReservableCurrency,
	},
	transactional, BoundedVec,
};
pub use frame_system::{ensure_signed, pallet_prelude::*, RawOrigin};

pub use sp_runtime::{
	traits::{AccountIdConversion, AtLeast32BitUnsigned, Saturating, StaticLookup, Zero},
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
		Nft::Pallet::<T>::mint(origin.clone(), collection, metadata).ok();

		let infos = Nft::Items::<T>::get(coll_id, item_id).unwrap();
		// Set asset price
		Self::price(origin, collection, item_id, new_price).ok();
		// Create Asset
		Asset::<T>::new(coll_id, item_id, infos, new_price).ok();

		Ok(())
	}

	pub fn status(collection: NftCollectionOf, item_id: T::NftItemId, status: AssetStatus) {
		let collection_id: T::NftCollectionId = collection.clone().value().into();
		Houses::<T>::mutate(collection_id, item_id, |val| {
			let mut v0 = val.clone().unwrap();
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
		let collection_id: T::NftCollectionId = collection.clone().value().into();

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

	///Execute the buy/sell transaction

	pub fn do_buy(
		collection: NftCollectionOf,
		item_id: T::NftItemId,
		buyer: T::AccountId,
		_infos: Asset<T>,
	) -> DispatchResult {
		let collection_id: T::NftCollectionId = collection.clone().value().into();
		let origin: OriginFor<T> = frame_system::RawOrigin::Root.into();
		let origin2: OriginFor<T> = frame_system::RawOrigin::Signed(buyer.clone()).into();

		//Check that the house item exists and has the correct status
		ensure!(
			Houses::<T>::contains_key(collection_id, item_id),
			Error::<T>::CollectionOrItemUnknown
		);
		let asset = Self::houses(collection_id, item_id).unwrap();
		let status = asset.status;
		ensure!(status == AssetStatus::FINALISED, Error::<T>::VoteNedeed);

		//Check that the owner is not the buyer
		let owner = Nft::Pallet::<T>::owner(collection_id, item_id)
			.ok_or(Error::<T>::CollectionOrItemUnknown)?;
		ensure!(buyer != owner, Error::<T>::BuyFromSelf);
		let balance = <T as Config>::Currency::reserved_balance(&owner);
		let _returned = <T as Config>::Currency::unreserve(&owner, balance);

		// The reserved funds in Housing Fund from the house bidding are unreserved for the transfer
		// transaction
		HousingFund::Pallet::<T>::unreserve_house_bidding_amount(collection_id, item_id).ok();

		//Transfer funds from HousingFund to owner
		let price = Prices::<T>::get(collection_id, item_id).unwrap();
		let fund_id = T::PalletId::get().into_account_truncating();
		<T as Config>::Currency::transfer(
			&fund_id,
			&owner,
			price,
			ExistenceRequirement::KeepAlive,
		)?;
		let to = T::Lookup::unlookup(buyer.clone());
		Nft::Pallet::<T>::transfer(origin, collection, item_id, to)?;
		Self::deposit_event(Event::TokenSold {
			owner,
			buyer,
			collection: collection_id,
			item: item_id,
			price,
		});

		//change status
		Self::change_status(origin2, collection, item_id, AssetStatus::PURCHASED).ok();

		Ok(())
	}

	pub fn get_formatted_collective_proposal(
		call: <T as Config>::Prop,
	) -> Option<<T as Votes::Config>::Call> {
		let call_encoded: Vec<u8> = call.encode();
		let ref_call_encoded = &call_encoded;

		if let Ok(call_formatted) = <T as Votes::Config>::Call::decode(&mut &ref_call_encoded[..]) {
			Some(call_formatted)
		} else {
			None
		}
	}

	// Conversion of u64 to BalanxceOf<T>
	pub fn u64_to_balance_option(input: u64) -> Option<BalanceOf<T>> {
		input.try_into().ok()
	}

	// Conversion of BalanceOf<T> to u32
	pub fn balance_to_u64_option(input: BalanceOf<T>) -> Option<u64> {
		input.try_into().ok()
	}

	pub fn account_id() -> T::AccountId {
		T::FeesAccount::get().into_account_truncating()
	}

	pub fn get_onboarded_houses() -> Vec<(
		<T as pallet_nft::Config>::NftCollectionId,
		<T as pallet_nft::Config>::NftItemId,
		types::Asset<T>,
	)> {
		Houses::<T>::iter()
			.filter(|val| val.2.status == types::AssetStatus::ONBOARDED)
			.map(|elt| (elt.0, elt.1, elt.2))
			.collect()
	}

	pub fn get_finalised_houses() -> Vec<(
		<T as pallet_nft::Config>::NftCollectionId,
		<T as pallet_nft::Config>::NftItemId,
		types::Asset<T>,
	)> {
		Houses::<T>::iter()
			.filter(|val| val.2.status == types::AssetStatus::FINALISED)
			.map(|elt| (elt.0, elt.1, elt.2))
			.collect()
	}
}
