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
pub use pallet_share_distributor as ShareDistributor;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	pub const PERCENT_FACTOR: u64 = 100;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + ShareDistributor::Config {
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

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// No enough fund for the house
		HousingFundNotEnough(T::NftCollectionId, T::NftItemId, Housing_Fund::BalanceOf<T>, BlockNumberOf<T>),
		// The bidding on the house is successful
		HouseBiddingSucceeded(T::NftCollectionId, T::NftItemId, Housing_Fund::BalanceOf<T>, BlockNumberOf<T>),
		// The bidding on the house failed
		HouseBiddingFailed(T::NftCollectionId, T::NftItemId, Housing_Fund::BalanceOf<T>, BlockNumberOf<T>),
		/// A list of investor cannot be assembled for an onboarded asset
		FailedToAssembleInvestors(T::NftCollectionId, T::NftItemId, Housing_Fund::BalanceOf<T>, BlockNumberOf<T>),
		/// No new onboarded houses found
		NoNewHousesFound(BlockNumberOf<T>),
		/// Selected investors don't have enough to bid for the asset
		NotEnoughAmongElligibleInvestors(T::NftCollectionId, T::NftItemId, Housing_Fund::BalanceOf<T>, BlockNumberOf<T>),
		/// No new finalised houses found
		NoHousesFinalisedFound(BlockNumberOf<T>),
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
			Self::process_onboarded_assets();
			Self::process_finalised_assets();
		}

		max_block_weight
	}

	pub fn process_finalised_assets() -> DispatchResultWithPostInfo {

		// We retrieve houses with finalised status
		let houses = Onboarding::Pallet::<T>::get_finalised_houses().clone();

		if houses.len() == 0 {
			// If no houses are found, an event is raised
			let block = <frame_system::Pallet<T>>::block_number();
			Self::deposit_event(Event::NoHousesFinalisedFound(block));
			return Ok(().into());
		}

		let houses_iter = houses.iter();

		// For each finalised houses, the ownership transfer is executed
		for item in houses_iter {
			ShareDistributor::Pallet::<T>::create_virtual(frame_system::RawOrigin::Root.into(), item.0.clone(), item.1.clone());
		}

		Ok(().into())
	}

	pub fn process_onboarded_assets() -> DispatchResultWithPostInfo {

		let houses = Onboarding::Pallet::<T>::get_onboarded_houses().clone();

		if houses.len() == 0 {
			let block = <frame_system::Pallet<T>>::block_number();
			Self::deposit_event(Event::NoNewHousesFound(block));
			return Ok(().into());
		}

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
				Self::deposit_event(Event::FailedToAssembleInvestors(
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
		let percent = Self::u64_to_balance_option(100).unwrap();
		// We get contributions following the min-max rules
		let contributions = Self::get_eligible_investors_contribution(amount.clone());

		// We check that the total amount of the contributions allow to buy the asset
		if contributions.0 < amount {
			return result;
		}

		let contributions_length = Self::u64_to_balance_option(contributions.1.len() as u64).unwrap();

		// We have at least more than the maximum possible investors
		if contributions_length >= (percent / Self::u64_to_balance_option(T::MinimumSharePerInvestor::get()).unwrap()) {
			result = Self::get_common_investor_distribution(amount.clone(), Self::u64_to_balance_option(T::MinimumSharePerInvestor::get()).unwrap(), contributions.1.clone());
		}
		// We have the minimum of investors
		else if contributions_length == (percent / Self::u64_to_balance_option(T::MaximumSharePerInvestor::get()).unwrap()) {
			result = Self::get_common_investor_distribution(amount.clone(), Self::u64_to_balance_option(T::MaximumSharePerInvestor::get()).unwrap(), contributions.1.clone());
		}
		// We have less than the maximum investors and more than the minimum investors
		else {
			result = Self::get_investor_distribution(amount.clone(), contributions.1.clone())
		}

		result
	}

	/// Get a list of tuple of account id and their contribution set at the same amount
	fn get_common_investor_distribution(
		amount: Housing_Fund::BalanceOf<T>, 
		common_share: Housing_Fund::BalanceOf<T>, 
		eligible_contributions: Vec<(Housing_Fund::AccountIdOf<T>, Housing_Fund::BalanceOf<T>, Housing_Fund::BalanceOf<T>)>,
	) -> Vec<(Housing_Fund::AccountIdOf<T>, Housing_Fund::BalanceOf<T>)> {
		let percent = Self::u64_to_balance_option(100).unwrap();
		let mut result: Vec<(Housing_Fund::AccountIdOf<T>, Housing_Fund::BalanceOf<T>)> = Vec::new();

		for item in eligible_contributions.iter() {
			result.push((item.0.clone(), common_share * amount / percent));
		}

		result
	}

	/// Get a list of tuple of account id and their contribution with different values
	/// The contribubtions follow the min-max rule of the amount
	fn get_investor_distribution(
		amount: Housing_Fund::BalanceOf<T>, 
		eligible_contributions: Vec<(Housing_Fund::AccountIdOf<T>, Housing_Fund::BalanceOf<T>, Housing_Fund::BalanceOf<T>)>,
	) -> Vec<(Housing_Fund::AccountIdOf<T>, Housing_Fund::BalanceOf<T>)> {
		let percent = Self::u64_to_balance_option(100).unwrap();
		let zero_percent = Self::u64_to_balance_option(0).unwrap();
		let mut actual_percentage: Housing_Fund::BalanceOf<T> = percent.clone();
		let mut result: Vec<(Housing_Fund::AccountIdOf<T>, Housing_Fund::BalanceOf<T>)> = Vec::new();
		let mut count: u64 = 1;
		let contributions_length: u64 = eligible_contributions.len() as u64;

		// We iterate through shares matching the rule min-max contribution
		// The eligible contributions are enough to buy the asset
		// The definitive shares will be determined by this loop
		// Each round, 100% is decremented by the share of the contribution processed
		for item in eligible_contributions.iter() {
			let mut item_share = Self::u64_to_balance_option(0).unwrap();

			// We are checking the last item so it takes the remaining percentage
			if count == contributions_length {
				item_share = actual_percentage;
			}
			else {
				// We calculate what is the share if a median rule is applied on the actual contribution and the remaining ones
				let share_median_diff = (actual_percentage - item.1.clone())/Self::u64_to_balance_option(contributions_length - count).unwrap();
				
				// We check that the distribution between accounts will respect rules if the maximum available share is given to the current account
				if share_median_diff < Self::u64_to_balance_option(T::MinimumSharePerInvestor::get()).unwrap() {
					// The current account is given a median share as its maximum available share will break the distribution rule
					item_share = actual_percentage / Self::u64_to_balance_option(contributions_length - count + 1).unwrap();
				}
				else {
					// The account is given its maximum available share as the remaining contributions will follow the min-max rule
					item_share = item.1.clone();
				}
			}

			// We add the account and the amount of its share
			result.push((item.0.clone(), item_share.clone()  * amount / percent));
			
			actual_percentage -= item_share;
			count += 1;

			if actual_percentage == zero_percent {
				break;
			}
		}

		result
	}

	/// Get
	/// - a list of tuples (AccountId, Share, Amount) following the min-max share rule
	/// - the total amount of the list
	fn get_eligible_investors_contribution(amount: Housing_Fund::BalanceOf<T>) 
	-> (Housing_Fund::BalanceOf<T>, Vec<(Housing_Fund::AccountIdOf<T>, Housing_Fund::BalanceOf<T>, Housing_Fund::BalanceOf<T>)>) {
		let mut result: Vec<(Housing_Fund::AccountIdOf<T>, Housing_Fund::BalanceOf<T>, Housing_Fund::BalanceOf<T>)> = Vec::new();
		let contributions = Housing_Fund::Pallet::<T>::get_contributions();
		let mut ordered_accountid_list: Vec<Housing_Fund::AccountIdOf<T>> = Vec::new();
		let mut ordered_contributions: Vec<(Housing_Fund::AccountIdOf<T>, Housing_Fund::Contribution<T>)> = Vec::new();
		let zero_percent = Self::u64_to_balance_option(0).unwrap();
		let mut total_share: Housing_Fund::BalanceOf<T> = Self::u64_to_balance_option(0).unwrap();

		// the contributions are ordered by block number ascending order
		for i in 0..contributions.len() {
			let oldest_contribution = Self::get_oldest_contribution(ordered_accountid_list.clone(), contributions.clone());
			ordered_accountid_list.push(oldest_contribution.0.clone());
			ordered_contributions.push(oldest_contribution.clone());
		}

		let contributions_iter = ordered_contributions.iter();

		// Add only contribution matching the minimum share contribution condition
		for item in contributions_iter {
			let share = Self::get_investor_share(amount.clone(), item.1.clone());
			if share.0 > zero_percent {
				result.push((item.0.clone(), share.0, share.1.clone()));
				total_share += share.1.clone();
			}
		}

		(total_share, result)
	}

	fn simulate_notary_intervention() {
		
	}

	/// Get the oldest contribution which accountId is not present in the ordered_list
	fn get_oldest_contribution(
		ordered_list: Vec<Housing_Fund::AccountIdOf<T>>, 
		contributions: Vec<(Housing_Fund::AccountIdOf<T>, Housing_Fund::Contribution<T>)>
	) -> (Housing_Fund::AccountIdOf<T>, Housing_Fund::Contribution<T>) {
		let mut contributions_cut:Vec<(Housing_Fund::AccountIdOf<T>, Housing_Fund::Contribution<T>)> = Vec::new();

		// We build the list where the min will be searched
		for item in contributions.iter() {
			if !ordered_list.contains(&item.0) {
				contributions_cut.push(item.clone());
			}
		}

		let mut min = contributions_cut[0].clone();

		for item in contributions_cut.iter() {
			if item.1.block_number < min.1.block_number {
				min = item.clone();
			}
		}

		min
	}

	// Get the share of the house price from a given contribution
	fn get_investor_share(
		amount: Housing_Fund::BalanceOf<T>, 
		contribution: Housing_Fund::Contribution<T>
	) -> (Housing_Fund::BalanceOf<T>, Housing_Fund::BalanceOf<T>) {
		
		let mut share: Housing_Fund::BalanceOf<T> = Self::u64_to_balance_option(0).unwrap();
		let mut value: Housing_Fund::BalanceOf<T> = Self::u64_to_balance_option(0).unwrap();
		// If the available amount is greater than the maximum amount, then the maximum amount is returned
		if contribution.available_balance >= Self::get_amount_percentage(amount.clone(), T::MaximumSharePerInvestor::get()) {
			share = Self::u64_to_balance_option(T::MaximumSharePerInvestor::get()).unwrap();
			value = Self::get_amount_percentage(amount.clone(), T::MaximumSharePerInvestor::get());
		}
		// If the avalable amount is greater than the minimum but less than the maximum amount then the share is calculated as a percentage
		else if contribution.available_balance >= Self::get_amount_percentage(amount.clone(), T::MinimumSharePerInvestor::get()) {
			share = contribution.available_balance * Self::u64_to_balance_option(100).unwrap() / amount;
			value = contribution.available_balance;
		}
		
		(share, value)
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
