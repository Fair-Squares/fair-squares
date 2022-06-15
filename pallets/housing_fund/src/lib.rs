#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;

mod functions;
mod structs;

pub use crate::structs::*;
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		dispatch::DispatchResult,
		sp_runtime::traits::AccountIdConversion,
		traits::{Currency, ExistenceRequirement, Get, ReservableCurrency},
		transactional, PalletId,
	};
	use sp_std::vec;

	pub const TREASURE_PALLET_ID: PalletId = PalletId(*b"py/trsry");
	pub const PERCENT_FACTOR: u64 = 100000;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: ReservableCurrency<Self::AccountId>;
		type MinContribution: Get<BalanceOf<Self>>;
		type FundThreshold: Get<BalanceOf<Self>>;
		type MaxFundContribution: Get<BalanceOf<Self>>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn fund_balance)]
	pub type FundBalance<T> = StorageValue<_, FundInfo<T>>;

	#[pallet::storage]
	#[pallet::getter(fn contributions)]
	// Distribution of investor's contributions
	pub(super) type Contributions<T> =
		StorageMap<_, Blake2_128Concat, AccountIdOf<T>, Contribution<T>, OptionQuery>;

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
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Allow an account to contribute to the common fund
		/// The origin must be signed
		/// - 'amount': the amount deposited in the fund
		/// Emits ContributeSucceeded event when successful
		#[pallet::weight(T::WeightInfo::contribute_to_fund())]
		#[transactional]
		pub fn contribute_to_fund(
			origin: OriginFor<T>,
			amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			// Check that the extrinsic was signed and get the signer.
			let who = ensure_signed(origin)?;

			// Check if it is the minimal contribution
			ensure!(amount.clone() >= T::MinContribution::get(), Error::<T>::ContributionTooSmall);

			// Check if account has enough to contribute
			ensure!(
				T::Currency::free_balance(&who) >= amount.clone(),
				Error::<T>::NotEnoughToContribute
			);

			// Get the block number for timestamp
			let block_number = <frame_system::Pallet<T>>::block_number();

			let contribution_log =
				ContributionLog { amount: amount.clone(), block_number: block_number.clone() };

			// Get the fund balance
			let wrap_fund = FundBalance::<T>::get();

			ensure!(wrap_fund.is_none() == false, Error::<T>::NoneValue);

			let mut fund = wrap_fund.unwrap();

			// Get the total fund to calculate the shares
			let mut total_fund = amount.clone();
			total_fund += fund.total.clone();
			// total_fund += T::Currency::total_balance(&TREASURE_PALLET_ID.into_account_truncating());

			if !Contributions::<T>::contains_key(&who) {
				let contribution = Contribution {
					account_id: who.clone(),
					total_balance: amount.clone(),
					share: 0,
					has_withdrawn: false,
					block_number: block_number.clone(),
					contributions: vec![contribution_log.clone()],
					withdraws: Vec::new(),
				};

				Contributions::<T>::insert(&who, contribution);
			} else {
				Contributions::<T>::mutate(&who, |val| {
					let unwrap_val = val.clone().unwrap();
					let mut contribution_logs = unwrap_val.contributions.clone();
					// update the contributions history
					contribution_logs.push(contribution_log.clone());

					let contrib = Contribution {
						account_id: who.clone(),
						total_balance: unwrap_val.total_balance + amount.clone(),
						share: unwrap_val.share,
						has_withdrawn: unwrap_val.has_withdrawn,
						block_number: block_number.clone(),
						contributions: contribution_logs,
						withdraws: Vec::new(),
					};
					*val = Some(contrib);
				});
			}

			// Update fund with new transferable amount			
			fund.contribute_transferable(amount.clone());
			FundBalance::<T>::mutate(|val| {
				*val = Some(fund);
			});

			// The amount is transferred to the treasurery
			T::Currency::transfer(
				&who,
				&TREASURE_PALLET_ID.into_account_truncating(),
				amount.clone(),
				ExistenceRequirement::AllowDeath,
			)?;

			// Update the shares of each contributor according to the new total balance
			Self::update_contribution_share(total_fund.clone());

			// Emit an event.
			Self::deposit_event(Event::ContributeSucceeded(who, amount, block_number));

			Ok(().into())
		}

		/// Withdraw the account contribution from the fund
		/// The origin must be signed
		/// Emits WithdrawalSucceeded event when successful
		#[pallet::weight(T::WeightInfo::withdraw_fund())]
		#[transactional]
		pub fn withdraw_fund(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResultWithPostInfo {
			// Check that the extrinsic was signed and get the signer.
			let who = ensure_signed(origin)?;

			// Check if the account has contributed to the fund
			ensure!(Contributions::<T>::contains_key(&who), Error::<T>::NotAContributor);

			// Get the contribution's account
			let contribution = Contributions::<T>::get(who.clone()).unwrap();

			// Retrieve the balance of the account
			let contribution_amount = contribution.total_balance.clone();

			// Check that the amount is not superior to the total balance of the contributor
			ensure!(amount.clone() <= contribution_amount, Error::<T>::NotEnoughFundToWithdraw);

			// Get the fund balance
			let wrap_fund = FundBalance::<T>::get();
			ensure!(wrap_fund.is_none() == false, Error::<T>::NoneValue);

			let mut fund = wrap_fund.unwrap();

			// Check taht the fund has enough transferable for the withdraw
			ensure!(fund.can_withdraw(amount.clone()), Error::<T>::NotEnoughInTransferableForWithdraw);

			// Get the block number for timestamp
			let block_number = <frame_system::Pallet<T>>::block_number();

			let withdraw_log = ContributionLog { amount: amount.clone(), block_number: block_number.clone() };

			Contributions::<T>::mutate(&who, |val| {
				let unwrap_val = val.clone().unwrap();
				let contribution_logs = unwrap_val.contributions.clone();
				let mut withdraw_logs = unwrap_val.withdraws.clone();
				// update the withdraws history
				withdraw_logs.push(withdraw_log.clone());

				let contrib = Contribution {
					account_id: who.clone(),
					total_balance: unwrap_val.total_balance - amount.clone(),
					share: unwrap_val.share,
					has_withdrawn: true,
					block_number: block_number.clone(),
					contributions: contribution_logs.clone(),
					withdraws: withdraw_logs.clone()
				};
				*val = Some(contrib);
			});

			// Update fund with new transferable amount
			fund.withdraw_transferable(amount.clone());
			FundBalance::<T>::mutate(|val| {
				*val = Some(fund.clone());
			});

			// The amount is transferred from the treasurery to the account
			T::Currency::transfer(
				&TREASURE_PALLET_ID.into_account_truncating(),
				&who,
				amount.clone(),
				ExistenceRequirement::AllowDeath,
			)?;

			// Get the total balance to claculate the updated shares
			let total_balance = fund.clone().total;

			// Update the shares of each contributor according to the new total balance
			Self::update_contribution_share(total_balance.clone());

			// Emit an event.
			Self::deposit_event(Event::WithdrawalSucceeded(
				who,
				contribution_amount,
				structs::WithdrawalReason::NotDefined,
				block_number,
			));

			Ok(().into())
		}
	}
}
