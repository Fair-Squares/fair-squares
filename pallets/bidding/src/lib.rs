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

pub use pallet_housing_fund as Housing_Fund;
pub use pallet_onboarding as Onboarding;
pub use pallet_nft as Nft;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	pub const PERCENT_FACTOR: u64 = 100;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + Onboarding::Config + Housing_Fund::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type WeightInfo: WeightInfo;
		type Currency: ReservableCurrency<Self::AccountId>;
		type SimultaneousAssetBidder: Get<u64>;
		type MaxTriesBid: Get<u64>;
		type MaxTriesAseemblingInvestor: Get<u64>;
		type MaximumSharePerInvestor: Get<u64>;
		type MinimumSharePerInvestor: Get<u64>;
		#[pallet::constant]
		type NewAssetScanPeriod: Get<Self::BlockNumber>;
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
		// The house is already being processed
		HouseAlreadyInBiddingProcess(T::NftCollectionId, T::NftItemId, Housing_Fund::BalanceOf<T>, BlockNumberOf<T>),
		// No enough fund for the house
		HousingFundNotEnough(T::NftCollectionId, T::NftItemId, Housing_Fund::BalanceOf<T>, BlockNumberOf<T>),
		// The bidding on the house is successful
		HouseBiddingSucceeded(T::NftCollectionId, T::NftItemId, Housing_Fund::BalanceOf<T>, BlockNumberOf<T>),
		// The bidding on the house failed
		HouseBiddingFailed(T::NftCollectionId, T::NftItemId, Housing_Fund::BalanceOf<T>, BlockNumberOf<T>),
		/// A list of investor cannot be assembled for an onboarded asset
		FailedToAssembleInvestor(T::NftCollectionId, T::NftItemId, Housing_Fund::BalanceOf<T>, BlockNumberOf<T>),
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// Weight: see `begin_block`
		fn on_initialize(n: T::BlockNumber) -> Weight {
			Self::begin_block(n)
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		
		
	}
}

use frame_support::{
	pallet_prelude::*
};

impl<T: Config> Pallet<T> {

	fn begin_block(now: T::BlockNumber) -> Weight {
		let max_block_weight: u64 = 1000;

		if (now % T::NewAssetScanPeriod::get()).is_zero() {
			Self::process_asset();
		}

		max_block_weight
	}

	pub fn process_asset() -> DispatchResultWithPostInfo {

		let houses = Onboarding::Pallet::<T>::get_onboarded_houses().clone();
		let houses_iter = houses.iter();

		for item in houses_iter {
			// Checks on price format
			if item.2.price.is_some() == false {
				continue;
			}
			
			let amount_wrap = Self::convert_balance(item.2.price.unwrap());
			if amount_wrap.is_some() == false {
				continue;
			}

			let amount = amount_wrap.unwrap();

			// Check if Housing Fund has enough fund for the asset
			if Housing_Fund::Pallet::<T>::check_available_fund(amount.clone()) == false {
				let block = <frame_system::Pallet<T>>::block_number();
				Self::deposit_event(Event::HousingFundNotEnough(item.0.clone(), item.1.clone(), amount.clone(), block));
				continue;
			}

			// Retrives the ivestors list and their contributions
			let investors_shares = Self::create_investor_list(amount.clone());

			// Checki that the investor list creation was successful
			if investors_shares.len() == 0 {
				let block = <frame_system::Pallet<T>>::block_number();
				Self::deposit_event(Event::FailedToAssembleInvestor(
					item.0.clone(), item.1.clone(), amount.clone(), block,
				));
				continue;
			}

			let result = Housing_Fund::Pallet::<T>::house_bidding(item.0.clone(), item.1.clone(), amount.clone(), investors_shares.clone());

			let block_number = <frame_system::Pallet<T>>::block_number();
			match result {
				Ok(_) => {
					
					Self::deposit_event(Event::HouseBiddingSucceeded(
						item.0.clone(), item.1.clone(), amount.clone(), block_number,
					));
				},
				Err(e) => {
					Self::deposit_event(Event::HouseBiddingFailed(
						item.0.clone(), item.1.clone(), amount.clone(), block_number,
					));
					continue;
				},
			}

			Self::simulate_notary_intervention();
		}

		Ok(().into())
	}

	/// Create the list of investor and their contribution for a given asset's price
	/// It follows the rules:
	/// - the oldest contribution comes first
	/// - no more than T::MaximumSharePerInvestor share per investor
	/// - no less than T::MinimumSharePerInvestor share per investor
	/// The total contribution from the investor list should be equal to the asset's price
	fn create_investor_list(amount: Housing_Fund::BalanceOf<T>) -> Vec<(Housing_Fund::AccountIdOf<T>, Housing_Fund::BalanceOf<T>)> {
		
		let mut result: Vec<(Housing_Fund::AccountIdOf<T>, Housing_Fund::BalanceOf<T>)> = Vec::new();
		let mut ordered_list: Vec<Housing_Fund::AccountIdOf<T>> = Vec::new();
		let contributions = Housing_Fund::Pallet::<T>::get_contributions();
		let percent = Self::u64_to_balance_option(100).unwrap();
		let zero_percent = Self::u64_to_balance_option(0).unwrap();
		let mut actual_percentage: Housing_Fund::BalanceOf<T> = zero_percent.clone();

		for i in 0..contributions.len() {
			// We have completed the share distribution so we can end the processus
			if actual_percentage == percent {
				break;
			}

			// Check if the remaining available percentage is less that the minimum share per investor
			if actual_percentage > percent || actual_percentage > Self::u64_to_balance_option(100 - T::MinimumSharePerInvestor::get()).unwrap() {
				result = Vec::new();
				break;
			} 

			let oldest_contribution = Self::get_oldest_contribution(ordered_list.clone(), contributions.clone());
			// We update the list of oldest contributions
			ordered_list.push(oldest_contribution.0.clone());
			let mut share = Self::get_investor_share(amount.clone(), oldest_contribution.1.clone());

			// Check if the contribution share was successful
			if share == zero_percent {
				continue;
			}

			let available_percentage = percent - actual_percentage;
			if share > available_percentage {
				if share - available_percentage >= Self::u64_to_balance_option(T::MinimumSharePerInvestor::get()).unwrap() {
					share = share - available_percentage;
				}
				else {
					share = zero_percent;
				}
			}

			// Check if the comparison of the share against the available share was successful
			if share == zero_percent {
				continue;
			}

			// We add the investor and its share in the list
			result.push(
				(oldest_contribution.0.clone(), share * amount / percent)
			);
			// We update the actual amount's percentage utilized in the processus
			actual_percentage += share;
		}

		// Check if all percentage has been completed
		if actual_percentage != percent {
			result = Vec::new();
		}		

		result
	}

	fn simulate_notary_intervention() {
		
	}

	/// Get the oldest contribution which accountId is not present in the ordered_list
	fn get_oldest_contribution(
		ordered_list: Vec<Housing_Fund::AccountIdOf<T>>, 
		contributions: Vec<(Housing_Fund::AccountIdOf<T>, Housing_Fund::Contribution<T>)>
	) -> (Housing_Fund::AccountIdOf<T>, Housing_Fund::Contribution<T>) {
		let mut min = contributions[0].clone();

		for item in contributions.iter() {
			if item.1.block_number < min.1.block_number && !ordered_list.contains(&item.0) {
				min = item.clone();
			}
		}

		min
	}

	// Get the share of the house price from a given contribution
	fn get_investor_share(
		amount: Housing_Fund::BalanceOf<T>, 
		contribution: Housing_Fund::Contribution<T>
	) -> Housing_Fund::BalanceOf<T> {
		
		let mut share: Housing_Fund::BalanceOf<T> = Self::u64_to_balance_option(0).unwrap();
		if contribution.available_balance >= Self::get_amount_percentage(amount.clone(), T::MaximumSharePerInvestor::get()) {
			share = Self::u64_to_balance_option(T::MaximumSharePerInvestor::get()).unwrap();
		}
		else if contribution.available_balance >= Self::get_amount_percentage(amount.clone(), T::MinimumSharePerInvestor::get()) {
			share = contribution.available_balance * Self::u64_to_balance_option(100).unwrap() / amount
		}
		
		share
	}

	fn get_amount_percentage(amount: Housing_Fund::BalanceOf<T>, percentage: u64) -> Housing_Fund::BalanceOf<T> {
		amount * Self::u64_to_balance_option(percentage.clone()).unwrap() / Self::u64_to_balance_option(100).unwrap()
	}

	fn convert_balance(amount: Onboarding::BalanceOf<T>) -> Option<Housing_Fund::BalanceOf<T>> {
		let value: Option<u128> = amount.try_into().ok();
		let result: Option<Housing_Fund::BalanceOf<T>> = value.unwrap().try_into().ok();
		result
	}

	pub fn u64_to_balance_option(input: u64) -> Option<Housing_Fund::BalanceOf<T>> {
		input.try_into().ok()
	}
}
