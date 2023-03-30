//! # Bidding pallet
//!
//! The Bidding pallet provides functionality to assembble investors and associate them to an
//! onboarded asset
//!
//! ## Overview
//!
//! The pallet checks each epoch time if new assets are avalaible to make a bid with an assembled
//! list of investors according multiple characteristics
//!
//! #### Dispatchable Functions
//!
//! * 'force_process_onboarded_asset' - extrinsic to manually launch the process of onboarded assets
//! * 'force_process_onboarded_asset' - extrinsic to manually launch the process of finalised assets
//!
//! #### Functions
//! * 'process_finalised_finalised_assets' - execute the token distribution between investors for
//!   the finalised assets
//! * 'process_onboarded_assetss' - execute the token distribution between investors for the
//!   finalised assets
//! * 'process_onboarded_assets' - execute the workflow to associate an onboarded onboarded asset to
//!   a list of investors and make and make

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
//pub mod weights;
//pub use weights::WeightInfo;

mod structs;
pub use crate::structs::*;

pub use pallet_housing_fund as Housing_Fund;
pub use pallet_nft as Nft;
pub use pallet_onboarding as Onboarding;
pub use pallet_share_distributor as ShareDistributor;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	use frame_system::{pallet_prelude::*, WeightInfo};

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

	pub type HousingFundAccount<T> = Housing_Fund::AccountIdOf<T>;
	pub type HousingFundBalance<T> = Housing_Fund::BalanceOf<T>;
	pub type EligibleContribution<T> = (HousingFundAccount<T>, HousingFundBalance<T>, HousingFundBalance<T>);
	pub type UserBalance<T> = (HousingFundAccount<T>, HousingFundBalance<T>);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Not enough fund for the house
		HousingFundNotEnough(
			T::NftCollectionId,
			T::NftItemId,
			HousingFundBalance<T>,
			BlockNumberOf<T>,
		),
		/// Bidding on the house is successful
		HouseBiddingSucceeded(
			T::NftCollectionId,
			T::NftItemId,
			HousingFundBalance<T>,
			BlockNumberOf<T>,
		),
		/// Bidding on the house failed
		HouseBiddingFailed(
			T::NftCollectionId,
			T::NftItemId,
			HousingFundBalance<T>,
			BlockNumberOf<T>,
			Vec<UserBalance<T>>,
		),
		/// Failed to assemble a list of investors for an onboarded asset
		FailedToAssembleInvestors(
			T::NftCollectionId,
			T::NftItemId,
			HousingFundBalance<T>,
			BlockNumberOf<T>,
		),
		/// No new onboarded houses found
		NoHousesOnboardedFound(BlockNumberOf<T>),
		/// Selected investors don't have enough fund to bid for the asset
		NotEnoughAmongEligibleInvestors(
			T::NftCollectionId,
			T::NftItemId,
			HousingFundBalance<T>,
			BlockNumberOf<T>,
		),
		/// No new finalised houses found
		NoHousesFinalisedFound(BlockNumberOf<T>),
		/// A finalised house has been distributed among investors
		SellAssetToInvestorsSuccessful(T::NftCollectionId, T::NftItemId, BlockNumberOf<T>),

		/// A finalised house failed to be distributed among investors
		SellAssetToInvestorsFailed(T::NftCollectionId, T::NftItemId, BlockNumberOf<T>),

		/// Processing an asset
		ProcessingAsset(T::NftCollectionId, T::NftItemId, HousingFundBalance<T>),

		/// Potential owners list successfully created
		InvestorListCreationSuccessful(
			T::NftCollectionId,
			T::NftItemId,
			HousingFundBalance<T>,
			Vec<UserBalance<T>>,
		),
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
		#[pallet::weight(10_000)]
		pub fn force_process_onboarded_asset(_origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			Self::process_onboarded_assets()
		}

		#[pallet::weight(10_000)]
		pub fn force_process_finalised_asset(_origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			Self::process_finalised_assets()
		}
	}
}

use enum_iterator::all;
use frame_support::pallet_prelude::*;

impl<T: Config> Pallet<T> {
	fn begin_block(now: T::BlockNumber) -> Weight {
		let max_block_weight = Weight::from_ref_time(1000_u64);

		if (now % T::NewAssetScanPeriod::get()).is_zero() {
			Self::process_onboarded_assets().ok();
			Self::process_finalised_assets().ok();
		}

		max_block_weight
	}

	/// Process finalised assets to distribute tokens among investors for assets
	pub fn process_finalised_assets() -> DispatchResultWithPostInfo {
		// We retrieve houses with finalised status
		let houses = Onboarding::Pallet::<T>::get_finalised_houses();

		if houses.is_empty() {
			// If no houses are found, an event is raised
			let block = <frame_system::Pallet<T>>::block_number();
			Self::deposit_event(Event::NoHousesFinalisedFound(block));
			return Ok(().into())
		}

		let houses_iter = houses.iter();

		// For each finalised houses, the ownership transfer is executed
		for item in houses_iter {
			let result = ShareDistributor::Pallet::<T>::create_virtual(
				frame_system::RawOrigin::Root.into(),
				item.0,
				item.1,
			);

			let block_number = <frame_system::Pallet<T>>::block_number();
			match result {
				Ok(_) => {
					Self::deposit_event(Event::SellAssetToInvestorsSuccessful(
						item.0,
						item.1,
						block_number,
					));
				},
				Err(_e) => {
					Self::deposit_event(Event::SellAssetToInvestorsFailed(
						item.0,
						item.1,
						block_number,
					));
				},
			}
		}

		Ok(().into())
	}

	/// Process onboarded assets to make make a bid on them and define a investors list
	pub fn process_onboarded_assets() -> DispatchResultWithPostInfo {
		let houses = Onboarding::Pallet::<T>::get_onboarded_houses();
		let block_number = <frame_system::Pallet<T>>::block_number();

		if houses.is_empty() {
			Self::deposit_event(Event::NoHousesOnboardedFound(block_number));
			return Ok(().into())
		}

		for (collection_id, item_id, house) in houses.into_iter() {
			// Checks on price format
			if house.price.is_none() {
				continue
			}

			let amount_wrap = Self::convert_balance(house.price.unwrap());
			if amount_wrap.is_none() {
				continue
			}

			let amount = amount_wrap.unwrap();
			Self::deposit_event(Event::ProcessingAsset(collection_id, item_id, amount));

			// Check if Housing Fund has enough fund for the asset
			if !Housing_Fund::Pallet::<T>::check_available_fund(amount) {
				Self::deposit_event(Event::HousingFundNotEnough(
					collection_id,
					item_id,
					amount,
					block_number,
				));
				continue
			}

			// Retrieves the investors list and their contributions
			let investor_shares = Self::create_investor_list(amount);

			// Check that the investor list creation was successful
			if investor_shares.is_empty() {
				Self::deposit_event(Event::FailedToAssembleInvestors(
					collection_id,
					item_id,
					amount,
					block_number,
				));
				continue
			}

			Self::deposit_event(Event::InvestorListCreationSuccessful(
				collection_id,
				item_id,
				amount,
				investor_shares.clone(),
			));

			let result = Housing_Fund::Pallet::<T>::house_bidding(
				collection_id,
				item_id,
				amount,
				investor_shares.clone(),
			);

			match result {
				Ok(_) => {
					Self::deposit_event(Event::HouseBiddingSucceeded(
						collection_id,
						item_id,
						amount,
						block_number,
					));

					let collections = all::<Nft::PossibleCollections>().collect::<Vec<_>>();
					let mut possible_collection = Nft::PossibleCollections::HOUSES;
					for item in collections.iter() {
						let value: T::NftCollectionId = item.value().into();
						if value == collection_id {
							possible_collection = *item;
							break
						}
					}

					let owner: T::AccountId =
						Nft::Pallet::<T>::owner(collection_id, item_id).unwrap();

					Onboarding::Pallet::<T>::change_status(
						frame_system::RawOrigin::Signed(owner).into(),
						possible_collection,
						item_id,
						Onboarding::AssetStatus::FINALISING,
					)
					.ok();
				},
				Err(_e) => {
					Self::deposit_event(Event::HouseBiddingFailed(
						collection_id,
						item_id,
						amount,
						block_number,
						investor_shares,
					));
					continue
				},
			}

			Self::simulate_notary_intervention();
		}

		Ok(().into())
	}

	/// Create the list of investor and their contribution for a given asset's price
	/// It follows the following rules:
	/// - the oldest contribution comes first
	/// - no more than T::MaximumSharePerInvestor share per investor
	/// - no less than T::MinimumSharePerInvestor share per investor
	/// The total contribution from the investor list should be equal to the asset's price
	fn create_investor_list(
		amount: HousingFundBalance<T>,
	) -> Vec<UserBalance<T>> {
		let mut result: Vec<UserBalance<T>> =
			Vec::new();
		let percent = Self::u64_to_balance_option(100).unwrap();
		// We get contributions following the min-max rules
		let contributions = Self::get_eligible_investors_contribution(amount);

		let contributions_length =
			Self::u64_to_balance_option(contributions.1.len() as u64).unwrap();

		// We check that the total amount of the contributions allow to buy the asset
		// And that the minimum number of investors is ok
		if contributions.0 < amount ||
			contributions_length <
				(percent /
					Self::u64_to_balance_option(T::MaximumSharePerInvestor::get()).unwrap())
		{
			return result
		}

		// We have at least more than the maximum possible investors
		if contributions_length >=
			(percent / Self::u64_to_balance_option(T::MinimumSharePerInvestor::get()).unwrap())
		{
			result = Self::get_common_investor_distribution(
				amount,
				Self::u64_to_balance_option(T::MinimumSharePerInvestor::get()).unwrap(),
				contributions.1,
			);
		}
		// We have the minimum of investors
		else if contributions_length ==
			(percent / Self::u64_to_balance_option(T::MaximumSharePerInvestor::get()).unwrap())
		{
			result = Self::get_common_investor_distribution(
				amount,
				Self::u64_to_balance_option(T::MaximumSharePerInvestor::get()).unwrap(),
				contributions.1,
			);
		}
		// We have less than the maximum investors and more than the minimum investors
		else {
			result = Self::get_investor_distribution(amount, contributions.1)
		}

		result
	}

	/// Get a list of tuple of account id and their contribution set at the same amount
	fn get_common_investor_distribution(
		amount: HousingFundBalance<T>,
		common_share: HousingFundBalance<T>,
		eligible_contributions: Vec<EligibleContribution<T>>,
	) -> Vec<UserBalance<T>> {
		let percent = Self::u64_to_balance_option(100).unwrap();
		let mut result: Vec<UserBalance<T>> =
			Vec::new();

		for item in eligible_contributions.iter() {
			result.push((item.0.clone(), common_share * amount / percent));
		}

		result
	}

	/// Get a list of tuple of account id and their contribution with different values
	/// The contribubtions follow the min-max rule of the amount
	fn get_investor_distribution(
		amount: HousingFundBalance<T>,
		eligible_contributions: Vec<(
			HousingFundAccount<T>,
			HousingFundBalance<T>,
			HousingFundBalance<T>,
		)>,
	) -> Vec<UserBalance<T>> {
		let percent = Self::u64_to_balance_option(100).unwrap();
		let zero_percent = Self::u64_to_balance_option(0).unwrap();
		let mut actual_percentage: HousingFundBalance<T> = percent;
		let mut result: Vec<UserBalance<T>> =
			Vec::new();
		let mut count: u64 = 1;
		let contributions_length: u64 = eligible_contributions.len() as u64;

		// We iterate through shares matching the rule min-max contribution
		// The eligible contributions are enough to buy the asset
		// The definitive shares will be determined by this loop
		// Each round, 100% is decremented by the share of the contribution processed
		for item in eligible_contributions.iter() {
			let item_share;

			// We are checking the last item so it takes the remaining percentage
			if count == contributions_length {
				item_share = actual_percentage;
			} else if item.1 >= actual_percentage {
				// The current account is given a median share as its maximum available share will
				// break the distribution rule
				item_share = actual_percentage /
					Self::u64_to_balance_option(contributions_length - count + 1).unwrap();
			} else {
				// We calculate what is the share if a median rule is applied on the actual
				// contribution and the remaining ones
				let share_median_diff = (actual_percentage - item.1) /
					Self::u64_to_balance_option(contributions_length - count).unwrap();

				// We check that the distribution between accounts will respect rules if the maximum
				// available share is given to the current account
				if share_median_diff <
					Self::u64_to_balance_option(T::MinimumSharePerInvestor::get()).unwrap()
				{
					// The current account is given a median share as its maximum available share
					// will break the distribution rule
					item_share = actual_percentage /
						Self::u64_to_balance_option(contributions_length - count + 1).unwrap();
				} else {
					// The account is given its maximum available share as the remaining
					// contributions will follow the min-max rule
					item_share = item.1;
				}
			}

			// We add the account and the amount of its share
			result.push((item.0.clone(), item_share * amount / percent));

			actual_percentage -= item_share;
			count += 1;

			if actual_percentage == zero_percent {
				break
			}
		}

		result
	}

	/// Get
	/// - a list of tuples (AccountId, Share, Amount) following the min-max share rule
	/// - the total amount of the list
	fn get_eligible_investors_contribution(
		amount: HousingFundBalance<T>,
	) -> (
		HousingFundBalance<T>,
		Vec<(HousingFundAccount<T>, HousingFundBalance<T>, HousingFundBalance<T>)>,
	) {
		let mut result: Vec<(
			HousingFundAccount<T>,
			HousingFundBalance<T>,
			HousingFundBalance<T>,
		)> = Vec::new();
		let contributions = Housing_Fund::Pallet::<T>::get_contributions();
		let mut ordered_account_id_list: Vec<HousingFundAccount<T>> = Vec::new();
		let mut ordered_contributions: Vec<(
			HousingFundAccount<T>,
			Housing_Fund::Contribution<T>,
		)> = Vec::new();
		let zero_percent = Self::u64_to_balance_option(0).unwrap();
		let mut total_share: HousingFundBalance<T> = Self::u64_to_balance_option(0).unwrap();

		// the contributions are ordered by block number ascending order
		for _ in 0..contributions.len() {
			let oldest_contribution = Self::get_oldest_contribution(
				ordered_account_id_list.clone(),
				contributions.clone(),
			);
			ordered_account_id_list.push(oldest_contribution.0.clone());
			ordered_contributions.push(oldest_contribution.clone());
		}

		// Add only contribution matching the minimum share contribution condition
		for (account_id, contribution) in ordered_contributions.into_iter() {
			let (share, value) = Self::get_investor_share(amount, contribution.clone());
			if share > zero_percent {
				result.push((account_id, share, value));
				total_share += value;
			}
		}

		(total_share, result)
	}

	fn simulate_notary_intervention() {}

	/// Get the oldest contribution which accountId is not present in the ordered_list
	fn get_oldest_contribution(
		ordered_list: Vec<HousingFundAccount<T>>,
		contributions: Vec<(HousingFundAccount<T>, Housing_Fund::Contribution<T>)>,
	) -> (HousingFundAccount<T>, Housing_Fund::Contribution<T>) {
		let mut contributions_cut: Vec<(
			HousingFundAccount<T>,
			Housing_Fund::Contribution<T>,
		)> = Vec::new();

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
		amount: HousingFundBalance<T>,
		contribution: Housing_Fund::Contribution<T>,
	) -> (HousingFundBalance<T>, HousingFundBalance<T>) {
		let mut share: HousingFundBalance<T> = Self::u64_to_balance_option(0).unwrap();
		let mut value: HousingFundBalance<T> = Self::u64_to_balance_option(0).unwrap();
		// If the available amount is greater than the maximum amount, then the maximum amount is
		// returned
		if contribution.available_balance >=
			Self::get_amount_percentage(amount, T::MaximumSharePerInvestor::get())
		{
			share = Self::u64_to_balance_option(T::MaximumSharePerInvestor::get()).unwrap();
			value = Self::get_amount_percentage(amount, T::MaximumSharePerInvestor::get());
		}
		// If the avalable amount is greater than the minimum but less than the maximum amount then
		// the share is calculated as a percentage
		else if contribution.available_balance >=
			Self::get_amount_percentage(amount, T::MinimumSharePerInvestor::get())
		{
			share =
				contribution.available_balance * Self::u64_to_balance_option(100).unwrap() / amount;
			value = contribution.available_balance;
		}

		(share, value)
	}

	fn get_amount_percentage(
		amount: HousingFundBalance<T>,
		percentage: u64,
	) -> HousingFundBalance<T> {
		amount * Self::u64_to_balance_option(percentage).unwrap() /
			Self::u64_to_balance_option(100).unwrap()
	}

	fn convert_balance(amount: Onboarding::BalanceOf<T>) -> Option<HousingFundBalance<T>> {
		let value: Option<u128> = amount.try_into().ok();
		let result: Option<HousingFundBalance<T>> = value.unwrap().try_into().ok();
		result
	}

	pub fn u64_to_balance_option(input: u64) -> Option<HousingFundBalance<T>> {
		input.try_into().ok()
	}
}
