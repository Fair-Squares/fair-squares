//! # Bidding pallet
//!
//! The Bidding pallet provide functionality to assembble investors and associate them to an onboarded asset
//!
//! ## Overview
//!
//! The pallet check each epoch time if new assets are avalaible to make a bid with an assembled list of investors
//! according multiple characteristics
//!
//! #### Dispatchable Functions
//! 
//! No dispachable functions
//! 
//! #### Functions
//! * 'process_asset' - execute the workflow to associate an asset to a list of investors
//! 



#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::WeightInfo;

mod structs;
pub use crate::structs::*;

pub use pallet_housing_fund;
pub use pallet_onboarding;
pub use pallet_nft;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type WeightInfo: WeightInfo;
		type Currency: ReservableCurrency<Self::AccountId>;
		type SimultaneousAssetBidder: Get<u64>;
		type MaxTriesBid: Get<u64>;
		type MaxTriesAseemblingInvestor: Get<u64>;
		type MaximumSharePerInvestor: Get<u64>;
		type MinimumSharePerInvestor: Get<u64>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn something)]
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		//#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::do_something(100))]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored(something, who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => return Err(Error::<T>::NoneValue.into()),
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				},
			}
		}
	}
}

use frame_support::{
	pallet_prelude::*
};

impl<T: Config> Pallet<T> {

	pub fn process_asset() -> DispatchResultWithPostInfo {
		
		Ok(().into())
	}

	fn check_new_asset() -> bool {
		true
	}

	fn check_housing_fund(amount: BalanceOf<T>) -> bool {
		true
	}

	fn create_investor_list() -> Vec<(AccountIdOf<T>, BalanceOf<T>)> {
		
		let result: Vec<(AccountIdOf<T>, BalanceOf<T>)> = Vec::new();

		result
	}

	fn simulate_notary_intervention() {
		
	}
}
