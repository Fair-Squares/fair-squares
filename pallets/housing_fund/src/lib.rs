//! # Housing fund pallet
//!
//! The housing fund pallet provides methods to manage the fund used to buy houses
//!
//! ## Overview
//!
//! The housing fund pallet enable third parties to transfer or withdraw funds to a common pot for
//! house purchase
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! * 'contribute_to_fund' - an account with the investor role can transfer funds to the pot
//! * 'withdraw_fund' - an account with the investor role can withdraw funds from the pot if the
//!   amount is available

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
//pub mod weights;

mod functions;
mod structs;

pub use crate::structs::*;
pub use pallet_nft as NFT;
pub use pallet_roles as ROLES;
//pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		traits::{Currency, ExistenceRequirement, Get},
		transactional, PalletId,
	};
	use frame_system::WeightInfo;
	use sp_std::vec;

	pub const PERCENT_FACTOR: u64 = 100000;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + NFT::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type LocalCurrency: frame_support::traits::Currency<Self::AccountId>
			+ frame_support::traits::ReservableCurrency<Self::AccountId>;
		type MinContribution: Get<BalanceOf<Self>>;
		type FundThreshold: Get<BalanceOf<Self>>;
		type MaxFundContribution: Get<BalanceOf<Self>>;
		type MaxInvestorPerHouse: Get<u32>;
		type PalletId: Get<PalletId>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::type_value]
	pub fn DefaultFundBalance<T: Config>() -> FundInfo<T> {
		FundInfo {
			total: Pallet::<T>::u64_to_balance_option(0).unwrap(),
			transferable: Pallet::<T>::u64_to_balance_option(0).unwrap(),
			reserved: Pallet::<T>::u64_to_balance_option(0).unwrap(),
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn fund_balance)]
	pub type FundBalance<T> = StorageValue<_, FundInfo<T>, ValueQuery, DefaultFundBalance<T>>;

	#[pallet::storage]
	#[pallet::getter(fn contributions)]
	// Distribution of investor's contributions
	pub type Contributions<T> =
		StorageMap<_, Blake2_128Concat, AccountIdOf<T>, Contribution<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn reservations)]
	// Housing fund reservations
	pub type Reservations<T> = StorageMap<
		_,
		Blake2_128Concat,
		(NftCollectionId<T>, NftItemId<T>),
		FundOperation<T>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn purchases)]
	// Housing fund used for purchases
	pub(super) type Purchases<T> = StorageMap<
		_,
		Blake2_128Concat,
		(NftCollectionId<T>, NftItemId<T>),
		FundOperation<T>,
		OptionQuery,
	>;

	// Pallets use events to inform users when important changes are made.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Account's contribution successfully added to the fund
		ContributeSucceeded(AccountIdOf<T>, BalanceOf<T>, BlockNumberOf<T>),
		/// Withdraw by account succeeded
		WithdrawalSucceeded(
			AccountIdOf<T>,
			BalanceOf<T>,
			structs::WithdrawalReason,
			BlockNumberOf<T>,
		),
		/// Fund reservation succeded
		FundReservationSucceeded(T::NftCollectionId, T::NftItemId, BalanceOf<T>, BlockNumberOf<T>),
		FundReservationCancelled(T::NftCollectionId, T::NftItemId, BalanceOf<T>, BlockNumberOf<T>),
		PurchaseFundValidated(T::NftCollectionId, T::NftItemId, BalanceOf<T>, BlockNumberOf<T>),
		FundUnreservedForPurchase(T::NftCollectionId, T::NftItemId, BalanceOf<T>, BlockNumberOf<T>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		/// Must have enough to contribute
		NotEnoughToContribute,
		/// Must have enough to withdraw
		NotEnoughFundToWithdraw,
		/// Fund Must have enough in transferable for withdraw action
		NotEnoughInTransferableForWithdraw,
		/// Must contribute at least the minimum amount of funds
		ContributionTooSmall,
		/// Must be a contributor to the fund
		NotAContributor,
		/// Contributor must have enough available balance
		NotEnoughAvailableBalance,
		/// Not enough i the fund to bid a house
		NotEnoughFundForHouse,
		/// Must have the investor role,
		NotAnInvestor,
		/// Must not have more investor than the max acceppted
		NotMoreThanMaxInvestorPerHouse,
		/// The reservation doesn't exist in the storage
		NoFundReservationFound,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Allow an account to contribute to the common fund
		/// The origin must be signed
		/// - 'amount': the amount deposited in the fund
		/// Emits ContributeSucceeded event when successful
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn contribute_to_fund(
			origin: OriginFor<T>,
			amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			// Check that the extrinsic was signed and get the signer.
			let who = ensure_signed(origin)?;

			// Check that the account has the investor role
			ensure!(
				ROLES::Pallet::<T>::investors(who.clone()).is_some(),
				Error::<T>::NotAnInvestor
			);

			// Check if it is the minimal contribution
			ensure!(amount >= T::MinContribution::get(), Error::<T>::ContributionTooSmall);

			// Check if account has enough to contribute
			ensure!(
				T::LocalCurrency::free_balance(&who) >= amount,
				Error::<T>::NotEnoughToContribute
			);

			// Get the block number for timestamp
			let block_number = <frame_system::Pallet<T>>::block_number();

			let contribution_log = ContributionLog { amount, block_number };

			// Get the fund balance
			let mut fund = FundBalance::<T>::get();

			if !Contributions::<T>::contains_key(&who) {
				let contribution = Contribution {
					account_id: who.clone(),
					available_balance: amount,
					reserved_balance: Self::u64_to_balance_option(0).unwrap(),
					contributed_balance: Self::u64_to_balance_option(0).unwrap(),
					has_withdrawn: false,
					block_number,
					contributions: vec![contribution_log],
					withdraws: Vec::new(),
				};

				Contributions::<T>::insert(&who, contribution);
			} else {
				Contributions::<T>::mutate(&who, |val| {
					let old_contrib = val.clone().unwrap();
					let mut contribution_logs = old_contrib.contributions.clone();
					// update the contributions history
					contribution_logs.push(contribution_log.clone());

					let new_contrib = Contribution {
						account_id: who.clone(),
						available_balance: old_contrib.available_balance + amount,
						block_number,
						contributions: contribution_logs,
						..old_contrib
					};
					*val = Some(new_contrib);
				});
			}

			// Update fund with new transferable amount
			fund.contribute_transferable(amount);
			FundBalance::<T>::mutate(|val| {
				*val = fund.clone();
			});

			// The amount is transferred to the treasurery
			T::LocalCurrency::transfer(
				&who,
				&Pallet::<T>::fund_account_id(),
				amount,
				ExistenceRequirement::AllowDeath,
			)?;

			// Emit an event.
			Self::deposit_event(Event::ContributeSucceeded(who, amount, block_number));

			Ok(().into())
		}

		/// Withdraw the account contribution from the fund
		/// The origin must be signed
		/// - amount : the amount to be withdrawn from the fund
		/// Emits WithdrawalSucceeded event when successful
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn withdraw_fund(
			origin: OriginFor<T>,
			amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			// Check that the extrinsic was signed and get the signer.
			let who = ensure_signed(origin)?;

			// Check that the account has the investor role
			ensure!(
				ROLES::Pallet::<T>::investors(who.clone()).is_some(),
				Error::<T>::NotAnInvestor
			);

			// Check if the account has contributed to the fund
			ensure!(Contributions::<T>::contains_key(&who), Error::<T>::NotAContributor);

			// Get the contribution's account
			let contribution = Contributions::<T>::get(who.clone()).unwrap();

			// Retrieve the balance of the account
			let contribution_amount = contribution.get_total_balance();

			// Check that the amount is not superior to the total balance of the contributor
			ensure!(amount <= contribution_amount, Error::<T>::NotEnoughFundToWithdraw);

			// Get the fund balance
			let mut fund = FundBalance::<T>::get();

			// Check that the fund has enough transferable for the withdraw
			ensure!(fund.can_take_off(amount), Error::<T>::NotEnoughInTransferableForWithdraw);

			// Get the block number for timestamp
			let block_number = <frame_system::Pallet<T>>::block_number();

			let withdraw_log = ContributionLog { amount, block_number };

			Contributions::<T>::mutate(&who, |val| {
				let old_contrib = val.clone().unwrap();
				let mut withdraw_logs = old_contrib.withdraws.clone();
				// update the withdraws history
				withdraw_logs.push(withdraw_log.clone());

				let new_contrib = Contribution {
					available_balance: old_contrib.available_balance - amount,
					has_withdrawn: true,
					block_number,
					withdraws: withdraw_logs.clone(),
					..old_contrib
				};
				*val = Some(new_contrib);
			});

			// Update fund with new transferable amount
			fund.withdraw_transferable(amount);
			FundBalance::<T>::mutate(|val| {
				*val = fund.clone();
			});

			// The amount is transferred from the treasury to the account
			T::LocalCurrency::transfer(
				&Pallet::<T>::fund_account_id(),
				&who,
				amount,
				ExistenceRequirement::AllowDeath,
			)?;

			// Emit an event.
			Self::deposit_event(Event::WithdrawalSucceeded(
				who,
				amount,
				structs::WithdrawalReason::NotDefined,
				block_number,
			));

			Ok(().into())
		}
	}
}
