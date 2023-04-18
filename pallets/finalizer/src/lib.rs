//! # Finalizer pallet
//!
//! The finalizer pallet provides methods to the notary and the seller to manage house purchase
//! and to the seller to cancel a purchase
//!
//! ## Overview
//!
//! The finalizer pallet provides methods to the notary to validate or reject house purchase
//! and to the seller to cancel a purchase
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! * 'validate_transaction_asset' - a notary validate a purchase transaction after checked
//!   informations
//! * 'reject_transaction_asset' - a notary reject a purchase
//! * 'reject_transaction_asset' - a house owner can cancel the purchase transaction after notary
//!   validation

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
pub use pallet_housing_fund as HousingFund;
pub use pallet_nft as Nft;
pub use pallet_onboarding as Onboarding;
pub use pallet_roles as Roles;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ Roles::Config
		+ Nft::Config
		+ Onboarding::Config
		+ HousingFund::Config
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		NotaryValidatedAssetTransaction(AccountIdOf<T>, T::NftCollectionId, T::NftItemId),
		NotaryRejectedAssetTransaction(AccountIdOf<T>, T::NftCollectionId, T::NftItemId),
		SellerCancelledAssetTransaction(AccountIdOf<T>, T::NftCollectionId, T::NftItemId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Must have the notary role
		NotANotary,
		/// Must have the seller role
		NotASeller,
		/// Must be the owner of the house
		NotTheHouseOwner,
		/// Asset must exist in storage
		AssetDoesNotExist,
		/// Asset must have FINALISED status
		HouseHasNotFinalisedStatus,
		/// Asset must have FINALISING status
		HouseHasNotFinalisingStatus,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// The notary set the house status to FINALISED
		/// The origin must be signed
		/// - collection_id: the collection id of the nft asset
		/// - nft_item_id: the id of the nft asset
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn validate_transaction_asset(
			origin: OriginFor<T>,
			collection_id: T::NftCollectionId,
			nft_item_id: T::NftItemId,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			// Check that the account has the notary role
			ensure!(Roles::Pallet::<T>::notaries(who.clone()).is_some(), Error::<T>::NotANotary);

			// Check that the house exists in storage
			let house_wrap = Onboarding::Houses::<T>::get(collection_id, nft_item_id);
			ensure!(house_wrap.is_some(), Error::<T>::AssetDoesNotExist);

			// Ensure the house status is FINALISING
			ensure!(
				house_wrap.unwrap().status == Onboarding::AssetStatus::FINALISING,
				Error::<T>::HouseHasNotFinalisingStatus
			);

			let collection = Self::get_possible_collection(collection_id);

			Onboarding::Pallet::<T>::change_status(
				origin,
				collection,
				nft_item_id,
				Onboarding::AssetStatus::FINALISED,
			)
			.ok();

			Self::deposit_event(Event::NotaryValidatedAssetTransaction(
				who,
				collection_id,
				nft_item_id,
			));

			Ok(())
		}

		/// The notary set the house status to REJECTED
		/// The origin must be signed
		/// - collection_id: the collection id of the nft asset
		/// - nft_item_id: the id of the nft asset
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn reject_transaction_asset(
			origin: OriginFor<T>,
			collection_id: T::NftCollectionId,
			nft_item_id: T::NftItemId,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			// Check that the account has the notary role
			ensure!(Roles::Pallet::<T>::notaries(who.clone()).is_some(), Error::<T>::NotANotary);

			// Check that the house exists in storage
			let house_wrap = Onboarding::Houses::<T>::get(collection_id, nft_item_id);
			ensure!(house_wrap.is_some(), Error::<T>::AssetDoesNotExist);

			// Ensure the house status is FINALISING
			ensure!(
				house_wrap.unwrap().status == Onboarding::AssetStatus::FINALISING,
				Error::<T>::HouseHasNotFinalisingStatus
			);

			let collection = Self::get_possible_collection(collection_id);

			Onboarding::Pallet::<T>::change_status(
				origin,
				collection,
				nft_item_id,
				Onboarding::AssetStatus::REJECTED,
			)
			.ok();

			HousingFund::Pallet::<T>::cancel_house_bidding(collection_id, nft_item_id).ok();

			Self::deposit_event(Event::NotaryRejectedAssetTransaction(
				who,
				collection_id,
				nft_item_id,
			));

			Ok(())
		}

		/// The seller set the house status to CANCELLED
		/// The origin must be signed
		/// - collection_id: the collection id of the nft asset
		/// - nft_item_id: the id of the nft asset
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn cancel_transaction_asset(
			origin: OriginFor<T>,
			collection_id: T::NftCollectionId,
			nft_item_id: T::NftItemId,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			// Check that the account has the notary role
			ensure!(Roles::Pallet::<T>::sellers(who.clone()).is_some(), Error::<T>::NotASeller);

			// Check that the house exists in storage
			let house_wrap = Onboarding::Houses::<T>::get(collection_id, nft_item_id);
			ensure!(house_wrap.is_some(), Error::<T>::AssetDoesNotExist);

			let owner: T::AccountId = Nft::Pallet::<T>::owner(collection_id, nft_item_id).unwrap();

			// Ensure the caller is the owner of the house
			ensure!(who == owner, Error::<T>::NotTheHouseOwner);

			// Ensure the house status is FINALISED
			ensure!(
				house_wrap.unwrap().status == Onboarding::AssetStatus::FINALISED,
				Error::<T>::HouseHasNotFinalisedStatus
			);

			let collection = Self::get_possible_collection(collection_id);

			Onboarding::Pallet::<T>::change_status(
				origin,
				collection,
				nft_item_id,
				Onboarding::AssetStatus::CANCELLED,
			)
			.ok();

			HousingFund::Pallet::<T>::cancel_house_bidding(collection_id, nft_item_id).ok();

			Self::deposit_event(Event::SellerCancelledAssetTransaction(
				who,
				collection_id,
				nft_item_id,
			));

			Ok(())
		}
	}
}

use enum_iterator::all;
pub use frame_support::inherent::Vec;
impl<T: Config> Pallet<T> {
	fn get_possible_collection(collection_id: T::NftCollectionId) -> Nft::PossibleCollections {
		let collections = all::<Nft::PossibleCollections>().collect::<Vec<_>>();
		let mut possible_collection = Nft::PossibleCollections::HOUSES;
		for item in collections.iter() {
			let value: T::NftCollectionId = item.value().into();
			if value == collection_id {
				possible_collection = *item;
				break
			}
		}
		possible_collection
	}
}
