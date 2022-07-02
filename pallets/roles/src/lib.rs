//! # Roles Pallet
//!
//! The Roles Pallet is used to set a role for a given AccountId in the FairSquares framework
//!
//! ## Overview
//!
//! The Roles Pallet provides roles management capabilities through the following actions:
//! - Role setting
//! - Role attribution to an AccountId
//! - Role attribution approval or rejection
//! During role setting, the user selects a role from the Accounts enum. Each
//! role has access to specific set of actions used in Fairsquares. there are currently 5 kinds of
//! roles available for selection:
//! - INVESTOR
//! - TENANT
//! - SERVICER
//! - SELLER
//! The 5th role which is the accounts administrator role is not available during role setting.
//! Sellers and Servicers roles, must be verified/approved by an administrator in order to become
//! active
//!
//! ### Dispatchable Functions
//! #### Role setting
//! * `set_role` - Create one of the 4 selectable type of role.
//! In the case of Sellers and Servicers, requests are transfered to a Role approval list
//!
//! #### Roles management by Administrator
//! * `account_approval` - This function allows the administrator to verify/approve Seller and
//! Servicer role connection to the requesting AccountId.
//! Verified AccountId are activated, i.e., the requesting AccountId is stored into the
//! corresponding role storage.
//!
//! * `account_rejection` - This function allows the administrator to reject Seller and Servicer
//! role connection to the requesting AccountId
//! that are in the approval list, but do not fullfill the FaiSquares guideline.
//!
//! * `set_manager` - This function allows the current manager to transfer his Administrative
//! authority to a different user/account.
//! Only the current manager can use this function, and he will lose all administrative power by
//! using this function.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod helpers;
mod structs;
pub mod weights;
pub use crate::structs::*;
pub use pallet_sudo as SUDO;
use sp_std::{fmt::Debug, prelude::*};
pub use weights::WeightInfo;
#[frame_support::pallet]
pub mod pallet {
	pub use super::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + SUDO::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: ReservableCurrency<Self::AccountId>;
		type WeightInfo: WeightInfo;
		type MaxMembers: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn investors)]
	///Registry of Investors organized by AccountId
	pub(super) type InvestorLog<T: Config> =
		StorageMap<_, Twox64Concat, AccountIdOf<T>, Investor<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn sellers)]
	///Registry of Sellers organized by AccountId
	pub(super) type HouseSellerLog<T: Config> =
		StorageMap<_, Twox64Concat, AccountIdOf<T>, HouseSeller<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn tenants)]
	///Registry of Tenants organized by AccountId
	pub(super) type TenantLog<T: Config> =
		StorageMap<_, Twox64Concat, AccountIdOf<T>, Tenant<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn servicers)]
	///Registry of Servicers organized by AccountId
	pub(super) type ServicerLog<T: Config> =
		StorageMap<_, Twox64Concat, AccountIdOf<T>, Servicer<T>, OptionQuery>;

	#[pallet::type_value]
	///Initializing function for the approval waiting list
	pub(super) fn MyDefault<T: Config>() -> Idle<T> {
		(Vec::new(), Vec::new())
	}
	#[pallet::storage]
	#[pallet::getter(fn get_pending_approvals)]
	///Approval waiting list for Sellers and Servicers
	pub(super) type RoleApprovalList<T: Config> =
		StorageValue<_, Idle<T>, ValueQuery, MyDefault<T>>;

	#[pallet::storage]
	#[pallet::getter(fn get_roles)]
	///Registry of Roles by AccountId
	pub(super) type AccountsRolesLog<T: Config> =
		StorageMap<_, Twox64Concat, AccountIdOf<T>, Accounts, OptionQuery>;

	#[pallet::type_value]
	///Initializing function for the total number of members
	pub(super) fn MyDefault1<T: Config>() -> u32 {
		let t0 = 0;
		t0
	}

	#[pallet::storage]
	#[pallet::getter(fn total_members)]
	pub(super) type TotalMembers<T> = StorageValue<_, u32, ValueQuery, MyDefault1<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		InvestorCreated(T::BlockNumber, T::AccountId),
		TenantCreated(T::BlockNumber, T::AccountId),
		SellerCreated(T::BlockNumber, T::AccountId),
		ServicerCreated(T::BlockNumber, T::AccountId),
		AccountCreationApproved(T::BlockNumber, T::AccountId),
		AccountCreationRejected(T::BlockNumber, T::AccountId),
		SellerAccountCreationRejected(T::BlockNumber, T::AccountId),
		ServicerAccountCreationRejected(T::BlockNumber, T::AccountId),
		CreationRequestCreated(T::BlockNumber, T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		NoneValue,
		/// Error on initialization.
		InitializationError,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		///One role is allowed
		OneRoleAllowed,
		///Invalid Operation
		InvalidOperation,
		///Require Sudo
		RequireSudo,
		///Account is not in waiting list
		NotInWaitingList,
		/// Account already in the waiting list
		AlreadyWaiting,
		///Maximum limit for number of members exceeded
		TotalMembersExceeded,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(<T as pallet::Config>::WeightInfo::investor(T::MaxMembers::get()))]
		///Account creation function. Only one role per account is permitted.
		pub fn set_role(origin: OriginFor<T>, account_type: Accounts) -> DispatchResult {
			let caller = ensure_signed(origin.clone())?;
			Self::check_storage(caller.clone())?;
			let now = <frame_system::Pallet<T>>::block_number();
			let count0 = Self::total_members();
			match account_type {
				Accounts::INVESTOR => {
					let _acc =
						Investor::<T>::new(origin).map_err(|_| <Error<T>>::InitializationError)?;
					AccountsRolesLog::<T>::insert(&caller, Accounts::INVESTOR);
					TotalMembers::<T>::put(count0 + 1);
					Self::deposit_event(Event::InvestorCreated(now, caller));
				},
				Accounts::SELLER => {
					Self::check_role_approval_list(caller.clone())?;
					let _acc = HouseSeller::<T>::new(origin)
						.map_err(|_| <Error<T>>::InitializationError)?;
					Self::deposit_event(Event::CreationRequestCreated(now, caller));
				},
				Accounts::TENANT => {
					let _acc =
						Tenant::<T>::new(origin).map_err(|_| <Error<T>>::InitializationError)?;
					AccountsRolesLog::<T>::insert(&caller, Accounts::TENANT);
					TotalMembers::<T>::put(count0 + 1);
					Self::deposit_event(Event::TenantCreated(now, caller));
				},
				Accounts::SERVICER => {
					Self::check_role_approval_list(caller.clone())?;
					let _acc =
						Servicer::<T>::new(origin).map_err(|_| <Error<T>>::InitializationError)?;
					Self::deposit_event(Event::CreationRequestCreated(now, caller));
				},
			}

			Ok(().into())
		}

		#[pallet::weight(<T as pallet::Config>::WeightInfo::approval(T::MaxMembers::get()))]
		///Approval function for Sellers and Servicers. Only for admin level.
		pub fn account_approval(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			ensure!(
				sender.clone() == SUDO::Pallet::<T>::key().unwrap(),
				"only the current sudo key can sudo"
			);
			let count0 = Self::total_members();
			TotalMembers::<T>::put(count0 + 1);
			Self::approve_account(sender.clone(), account.clone())?;
			let now = <frame_system::Pallet<T>>::block_number();
			Self::deposit_event(Event::AccountCreationApproved(now, account));
			Ok(().into())
		}

		#[pallet::weight(<T as pallet::Config>::WeightInfo::rejection(T::MaxMembers::get()))]
		///Creation Refusal function for Sellers and Servicers. Only for admin level.
		pub fn account_rejection(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			ensure!(
				sender == SUDO::Pallet::<T>::key().unwrap(),
				"only the current sudo key can sudo"
			);
			Self::reject_account(account.clone())?;
			let now = <frame_system::Pallet<T>>::block_number();
			Self::deposit_event(Event::AccountCreationRejected(now, account));
			Ok(().into())
		}

		#[pallet::weight(<T as pallet::Config>::WeightInfo::set_admin(T::MaxMembers::get()))]
		///The caller will transfer his admin authority to a different account
		pub fn set_manager(
			origin: OriginFor<T>,
			new: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			ensure!(
				sender == SUDO::Pallet::<T>::key().unwrap(),
				"only the current sudo key can sudo"
			);
			SUDO::Pallet::<T>::set_key(origin, new).ok();
			Ok(().into())
		}
	}
}
