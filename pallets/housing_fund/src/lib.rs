#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;

mod structs;
mod functions;

pub use crate::structs::*;
pub use weights::WeightInfo;



#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		dispatch::DispatchResult,
		transactional,
		sp_runtime::traits::{AccountIdConversion },
		traits::{Currency, ExistenceRequirement, Get, ReservableCurrency},
		PalletId		
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

      	/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
    #[pallet::getter(fn contributions)]
    // Distribution of investor's contributions
    pub(super) type Contributions<T> = StorageMap<
      _, 
      Blake2_128Concat, 
      AccountIdOf<T>, 
      Contribution::<T>, 
      OptionQuery
      >;

	// Pallets use events to inform users when important changes are made.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Account's contribution successfully added to the fund
		ContributeSucceeded(AccountIdOf<T>, BalanceOf<T>, BlockNumberOf<T>),
		/// Withdraw by account succeeded
		WithdrawalSucceeded(AccountIdOf<T>, BalanceOf<T>, structs::WithdrawalReason, BlockNumberOf<T>),
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
		/// Must contribute at least the minimum amount of funds
		ContributionTooSmall,
		/// Must be a contributor to the fund
		NotAContributor
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		
		/// Allow an account to contribute to the common fund
		/// The origin must be signed
		/// - 'amount': the amount deposited in the fund
		/// Emits ContributeSucceeded event when successful
		#[pallet::weight(T::WeightInfo::contribute_to_fund())]
		#[transactional]
		pub fn contribute_to_fund(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResultWithPostInfo {
			// Check that the extrinsic was signed and get the signer.
			let who = ensure_signed(origin)?;

			// Check if it is the minimal contribution
			ensure!(amount.clone() >= T::MinContribution::get(), Error::<T>::ContributionTooSmall);

			// Check if account has enough to contribute
			ensure!(
				T::Currency::free_balance(&who) >= amount.clone(),
				Error::<T>::NotEnoughToContribute
			);

			// Get the block timestamp
			let block_number = <frame_system::Pallet<T>>::block_number();

			let contribution_log = ContributionLog {
				amount: amount.clone(),
				block_number: block_number.clone()
			};

			// Get the total fund to calculate the shares
			let mut total_fund = amount.clone();
			total_fund += T::Currency::total_balance(&TREASURE_PALLET_ID.into_account_truncating());

			if !Contributions::<T>::contains_key(&who) {

				let contribution = Contribution {
					account_id: who.clone(),
					total_balance: amount.clone(),
					share: 0,
					block_number: block_number.clone(),
					contributions: vec![contribution_log.clone()]
				};

				Contributions::<T>::insert(&who, contribution);
			} 
			else {
				Contributions::<T>::mutate(&who, |val| {
					let unwrap_val = val.clone().unwrap();
					let mut contribution_logs = unwrap_val.contributions.clone();
					contribution_logs.push(contribution_log.clone());

					let contrib = Contribution { 
						account_id: who.clone(), 
						total_balance: unwrap_val.total_balance + amount.clone(),
						share: unwrap_val.share,
						block_number: block_number.clone(),
						contributions: contribution_logs
					 };
					 *val = Some(contrib);
				});
			}

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
			ensure!(Contributions::<T>::contains_key(&who),Error::<T>::NotAContributor);

			// Get the contribution's account
			let contribution = Contributions::<T>::get(who.clone()).unwrap();

			// Retrieve the balance of the account
			let contribution_amount = contribution.total_balance.clone();

			Contributions::<T>::remove(&who);

			// The amount is transferred from the treasurery to the account
			T::Currency::transfer(
				&TREASURE_PALLET_ID.into_account_truncating(),
				&who,
				contribution_amount.clone(),
				ExistenceRequirement::AllowDeath,
			)?;

			// Get the total balance to claculate the updated shares
			let total_balance = T::Currency::free_balance(&TREASURE_PALLET_ID.into_account_truncating());
			
			// Update the shares of each contributor according to the new total balance
			Self::update_contribution_share(total_balance.clone());

			// Get the block timestamp
			let block_number = <frame_system::Pallet<T>>::block_number();

			// Emit an event.
			Self::deposit_event(Event::WithdrawalSucceeded(who, amount, structs::WithdrawalReason::NotDefined, block_number));

			Ok(().into())
		}
	}
}