use super::*;
use crate::Roles::Hash;

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
	DispatchError, Percent,
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
			let mut asset = val.clone().unwrap();
			asset.status = status;
			*val = Some(asset);
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
		let origin_root: OriginFor<T> = frame_system::RawOrigin::Root.into();
		let origin_buyer: OriginFor<T> = frame_system::RawOrigin::Signed(buyer.clone()).into();

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
		Nft::Pallet::<T>::transfer(origin_root, collection, item_id, to)?;
		Self::deposit_event(Event::TokenSold {
			owner,
			buyer,
			collection: collection_id,
			item: item_id,
			price,
		});

		//change status
		Self::change_status(origin_buyer, collection, item_id, AssetStatus::PURCHASED).ok();

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

	pub fn account_id() -> T::AccountId {
		T::FeesAccount::get().into_account_truncating()
	}

	fn get_houses_by_status(
		status: types::AssetStatus,
	) -> Vec<(
		<T as pallet_nft::Config>::NftCollectionId,
		<T as pallet_nft::Config>::NftItemId,
		types::Asset<T>,
	)> {
		Houses::<T>::iter()
			.filter(|(_, _, house)| house.status == status)
			.map(|(collection_id, item_id, house)| (collection_id, item_id, house))
			.collect()
	}

	pub fn get_onboarded_houses() -> Vec<(
		<T as pallet_nft::Config>::NftCollectionId,
		<T as pallet_nft::Config>::NftItemId,
		types::Asset<T>,
	)> {
		Self::get_houses_by_status(types::AssetStatus::ONBOARDED)
	}

	pub fn get_finalised_houses() -> Vec<(
		<T as pallet_nft::Config>::NftCollectionId,
		<T as pallet_nft::Config>::NftItemId,
		types::Asset<T>,
	)> {
		Self::get_houses_by_status(types::AssetStatus::FINALISED)
	}

	pub fn get_finalising_houses() -> Vec<(
		<T as pallet_nft::Config>::NftCollectionId,
		<T as pallet_nft::Config>::NftItemId,
		types::Asset<T>,
	)> {
		Self::get_houses_by_status(types::AssetStatus::FINALISING)
	}

	pub fn do_submit_proposal(
		origin: OriginFor<T>,
		collection: NftCollectionOf,
		item_id: T::NftItemId,
	) {
		//Change asset status to REVIEWING
		Self::change_status(origin.clone(), collection, item_id, AssetStatus::REVIEWING).ok();
		//Send Proposal struct to voting pallet
		//get the needed call and convert them to pallet_voting format
		let collection_id: T::NftCollectionId = collection.clone().value().into();
		let out_call = Vcalls::<T>::get(collection_id, item_id).unwrap();

		let w_status0 =
			Box::new(Self::get_formatted_collective_proposal(*out_call.democracy_status).unwrap());
		let w_status1 =
			Box::new(Self::get_formatted_collective_proposal(*out_call.after_vote_status).unwrap());

		let w_r_destroy =
			Box::new(Self::get_formatted_collective_proposal(*out_call.reject_destroy).unwrap());
		let w_r_edit =
			Box::new(Self::get_formatted_collective_proposal(*out_call.reject_edit).unwrap());

		let proposal_hash = T::Hashing::hash_of(&w_status1);
		Houses::<T>::mutate_exists(collection_id, item_id, |val| {
			let mut v0 = val.clone().unwrap();
			v0.proposal_hash = proposal_hash;
			*val = Some(v0)
		});

		//Send Calls struct to voting pallet
		Votes::Pallet::<T>::submit_proposal(origin, w_status1, w_status0, w_r_destroy, w_r_edit)
			.ok();
	}
}
