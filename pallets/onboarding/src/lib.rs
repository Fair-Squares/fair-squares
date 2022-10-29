//! # Onboarding Pallet
//!
//! The Onboarding Pallet allows the Seller role to create an asset proposal
//! and submit it for a Review by a Council in the FairSquares framework.
//!
//! ## Overview
//!
//! The Onboarding Pallet the following actions available to the Seller role:
//! - Create and Submit a proposal
//! - Create a proposal without submission phase
//! - Edit unsubmitted proposal price
//! - Submit an awaiting proposal
//!
//!And the following actions sent to pallet voting through calls
//! - Execute NFT & funds transfer
//! - Reject a proposal for price editing purpose (asset is marked as REJECTED and re-submission is
//!   possible)
//! - Reject a proposal for destruction (NFT is burned, asset is marked as SLASH)
//!
//! ### Dispatchable Functions
//! #### Role setting
//!
//! * `do_something` - Used in a Call to initialize the fields of the VotingCalls struct.
//!  
//! * `set_price` - Modify the price of an Existing proposal with the status EDIT or REJECTED
//! Proposal price is the only part that can be edited
//!
//! * `do_buy` - Execute the buy/sell transaction.
//! Funds reserved during proposal creation are unreserved.
//! Sent to the voting pallet as a Call.
//!
//! * `reject_edit` - Reject a submitted proposal for price editing,
//! and a portion of the amount reserved during proposal creation is slashed.
//! Sent to the voting pallet as a Call.
//!
//! * `reject_destroy` - Reject a submitted proposal for destruction,
//! and all of the amount reserved during proposal creation is slashed.
//! Sent to the voting pallet as a Call.
//!
//! * `create_and_submit_proposal` - Creation and submission of a proposal.
//! A struct containing Calls for the voting pallet is also created and stored.
//! the proposal submission is optionnal, and can be disabled through the value
//! of the boolean `submit`. A defined amount that will be slashed in case of
//! proposal rejection is also reserved.
//!
//! * `submit_awaiting` - Submit/edit an awaiting proposal for review.
//! This is also used for re-submission of rejected proposals.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]

mod functions;
mod types;

pub use functions::*;
pub use types::*;

pub use pallet_housing_fund as HousingFund;
pub use pallet_nft as Nft;
pub use pallet_roles as Roles;
pub use pallet_sudo as Sudo;
pub use pallet_voting as Votes;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
//pub mod weights;
//pub use weights::WeightInfo;

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;
pub type CallOf<T> = <T as Votes::Config>::Call;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, PalletId};
	use frame_system::WeightInfo;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ Roles::Config
		+ Nft::Config
		+ Sudo::Config
		+ Votes::Config
		+ HousingFund::Config
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: ReservableCurrency<Self::AccountId>;
		type Prop: Parameter
			+ Dispatchable<Origin = <Self as frame_system::Config>::Origin>
			+ From<Call<Self>>;
		#[pallet::constant]
		type ProposalFee: Get<BalanceOf<Self>>;
		type WeightInfo: WeightInfo;

		#[pallet::constant]
		type FeesAccount: Get<PalletId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::storage]
	#[pallet::getter(fn prices)]
	/// Stores token info
	pub(super) type Prices<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::NftCollectionId,
		Blake2_128Concat,
		T::NftItemId,
		BalanceOf<T>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn houses)]
	/// Stores Asset info
	pub type Houses<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::NftCollectionId,
		Blake2_128Concat,
		T::NftItemId,
		Asset<T>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn voting_calls)]
	/// Stores Calls
	pub(super) type Vcalls<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::NftCollectionId,
		Blake2_128Concat,
		T::NftItemId,
		VotingCalls<T>,
		OptionQuery,
	>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),

		/// The price for a token was updated
		TokenPriceUpdated {
			who: T::AccountId,
			collection: T::NftCollectionId,
			item: T::NftItemId,
			price: Option<BalanceOf<T>>,
		},

		/// Token was sold to a new owner
		TokenSold {
			owner: T::AccountId,
			buyer: T::AccountId,
			collection: T::NftCollectionId,
			item: T::NftItemId,
			price: BalanceOf<T>,
		},

		/// Proposal Created
		ProposalCreated {
			who: T::AccountId,
			collection: T::NftCollectionId,
			item: T::NftItemId,
			price: Option<BalanceOf<T>>,
		},
		/// Proposal submited for review
		ProposalSubmitted {
			who: T::AccountId,
			collection: T::NftCollectionId,
			item: T::NftItemId,
			price: Option<BalanceOf<T>>,
		},
		/// Proposal rejected for editing
		RejectedForEditing {
			by_who: T::AccountId,
			collection: T::NftCollectionId,
			item: T::NftItemId,
		},
		/// Proposal rejected for destruction
		RejectedForDestruction {
			by_who: T::AccountId,
			collection: T::NftCollectionId,
			item: T::NftItemId,
		},
		///Funds reserved
		FundsReserved { from_who: T::AccountId, amount: Option<BalanceOf<T>> },
		///Funds slashed
		SlashedFunds { from_who: T::AccountId, amount: Option<BalanceOf<T>> },
		///StatusChanged
		AssetStatusChanged {
			changed_to: AssetStatus,
			collection: T::NftCollectionId,
			item: T::NftItemId,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		/// The acting account does not correspond to the token owner
		NotTheTokenOwner,
		/// Class or instance does not exist
		CollectionOrItemUnknown,
		/// Cannot buy from yourself
		BuyFromSelf,
		/// Item is not for sale
		NotForSale,
		/// NFT Item cannot be edited
		CannotEditItem,
		/// NFT Item has not been approved for sell
		ItemNotApproved,
		/// NFT Item Cannot be submitted for review
		CannotSubmitItem,
		/// NFT ITEM must be reviewed first
		ReviewNedeed,
		/// Investors vote is needed first
		VoteNedeed,
		/// Insufficient balance for proposal creation
		InsufficientBalance,
		/// Action reserved to Seller role
		ReservedToSeller,
		/// Failed to unreserved fund in Housing fund
		HousingFundUnreserveFundFailed,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		//#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		#[pallet::weight(10_000)]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored(something, who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(10_000)]
		#[transactional]
		pub fn change_status(
			origin: OriginFor<T>,
			collection: NftCollectionOf,
			item_id: T::NftItemId,
			status: AssetStatus,
		) -> DispatchResult {
			let _caller = ensure_signed(origin.clone()).unwrap();
			let coll_id: T::NftCollectionId = collection.clone().value().into();
			Self::status(collection, item_id, status);
			Self::deposit_event(Event::AssetStatusChanged {
				changed_to: status,
				collection: coll_id,
				item: item_id,
			});

			Ok(())
		}

		/// Modify the price of an Existing proposal
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn set_price(
			origin: OriginFor<T>,
			collection: NftCollectionOf,
			item_id: T::NftItemId,
			new_price: Option<BalanceOf<T>>,
		) -> DispatchResult {
			let caller = ensure_signed(origin.clone()).unwrap();
			let collection_id: T::NftCollectionId = collection.clone().value().into();
			ensure!(
				Houses::<T>::contains_key(collection_id, item_id),
				Error::<T>::CollectionOrItemUnknown
			);

			Houses::<T>::mutate_exists(collection_id, item_id, |val| {
				let mut v0 = val.clone().unwrap();
				v0.price = new_price;
				*val = Some(v0)
			});

			let asset = Self::houses(collection_id, item_id).unwrap();
			let status = asset.status;
			ensure!(
				status == AssetStatus::EDITING || status == AssetStatus::REJECTED,
				Error::<T>::CannotEditItem
			);

			Self::price(origin, collection, item_id, new_price).ok();

			Self::deposit_event(Event::TokenPriceUpdated {
				who: caller,
				collection: collection_id,
				item: item_id,
				price: new_price,
			});

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn reject_edit(
			origin: OriginFor<T>,
			collection: NftCollectionOf,
			item_id: T::NftItemId,
			_infos: Asset<T>,
		) -> DispatchResult {
			let caller = ensure_signed(origin.clone()).unwrap();
			let collection_id: T::NftCollectionId = collection.clone().value().into();
			ensure!(
				Houses::<T>::contains_key(collection_id, item_id),
				Error::<T>::CollectionOrItemUnknown
			);
			let house = Self::houses(collection_id, item_id).unwrap();
			ensure!(
				house.status == AssetStatus::REVIEWING || house.status == AssetStatus::VOTING,
				Error::<T>::CannotSubmitItem
			);
			Self::change_status(origin, collection, item_id, AssetStatus::REJECTED).ok();

			let owner = Nft::Pallet::<T>::owner(collection_id, item_id).unwrap();
			let balance = <T as Config>::Currency::reserved_balance(&owner);

			let wrap_balance = Self::balance_to_u64_option(balance).unwrap();
			let slash = wrap_balance * 10 / 100;
			let wrap_remain = wrap_balance - slash;
			let fees = Self::u64_to_balance_option(slash).unwrap();
			let remain = Self::u64_to_balance_option(wrap_remain).unwrap();

			<T as pallet::Config>::Currency::unreserve(&owner, fees);
			let res = <T as pallet::Config>::Currency::transfer(
				&owner,
				&Self::account_id(),
				fees,
				ExistenceRequirement::AllowDeath,
			);
			debug_assert!(res.is_ok());

			let res1 = <T as pallet::Config>::Currency::reserve(&owner, remain);
			debug_assert!(res1.is_ok());

			Self::deposit_event(Event::RejectedForEditing {
				by_who: caller.clone(),
				collection: collection_id,
				item: item_id,
			});

			Self::deposit_event(Event::SlashedFunds { from_who: caller, amount: Some(fees) });

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn reject_destroy(
			origin: OriginFor<T>,
			collection: NftCollectionOf,
			item_id: T::NftItemId,
			_infos: Asset<T>,
		) -> DispatchResult {
			let caller = ensure_signed(origin.clone()).unwrap();
			let collection_id: T::NftCollectionId = collection.clone().value().into();
			ensure!(
				Houses::<T>::contains_key(collection_id, item_id),
				Error::<T>::CollectionOrItemUnknown
			);
			let house = Self::houses(collection_id, item_id).unwrap();
			ensure!(
				house.status == AssetStatus::REVIEWING || house.status == AssetStatus::VOTING,
				Error::<T>::CannotSubmitItem
			);
			Self::change_status(origin.clone(), collection, item_id, AssetStatus::SLASH).ok();
			let owner = Nft::Pallet::<T>::owner(collection_id, item_id).unwrap();
			Nft::Pallet::<T>::burn(origin, collection, item_id).ok();
			let balance = <T as Config>::Currency::reserved_balance(&owner);
			ensure!(balance > Zero::zero(), Error::<T>::NoneValue);
			<T as pallet::Config>::Currency::unreserve(&owner, balance);
			let res = <T as pallet::Config>::Currency::transfer(
				&owner,
				&Self::account_id(),
				balance,
				ExistenceRequirement::AllowDeath,
			);
			debug_assert!(res.is_ok());

			Self::deposit_event(Event::RejectedForDestruction {
				by_who: caller.clone(),
				collection: collection_id,
				item: item_id,
			});

			Self::deposit_event(Event::SlashedFunds { from_who: caller, amount: Some(balance) });

			Ok(())
		}

		/// `create_and_submit_proposal` - Creation and submission of a proposal.
		/// the proposal submission is optionnal, and can be disabled through the value
		/// of the boolean `submit`.
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn create_and_submit_proposal(
			origin: OriginFor<T>,
			collection: NftCollectionOf,
			price: Option<BalanceOf<T>>,
			metadata: Nft::BoundedVecOfUnq<T>,
			submit: bool,
		) -> DispatchResult {
			let caller = ensure_signed(origin.clone()).unwrap();
			ensure!(Roles::Pallet::<T>::sellers(&caller).is_some(), Error::<T>::ReservedToSeller);
			let idx = collection.clone().value() as usize;

			// Get itemId and infos from minted nft
			let item_id: T::NftItemId = Nft::ItemsCount::<T>::get()[idx].into();

			//Create asset

			let balance1 = <T as Config>::Currency::free_balance(&caller);

			let res0 = Self::balance_to_u64_option(price.unwrap()).unwrap();
			let perc = Self::balance_to_u64_option(T::ProposalFee::get()).unwrap();
			let res1 = perc * res0 / 100;
			let balance0 = Self::u64_to_balance_option(res1).unwrap();
			ensure!(balance1 > balance0, Error::<T>::InsufficientBalance);

			<T as Config>::Currency::reserve(&caller, balance0).ok();
			Self::create_asset(origin.clone(), collection, metadata, price, item_id).ok();

			let collection_id: T::NftCollectionId = collection.clone().value().into();

			let house = Self::houses(collection_id, item_id).unwrap();

			let _new_call = VotingCalls::<T>::new(collection_id, item_id).ok();

			//Create Call for collective-to-democracy status change
			let call1: T::Prop =
				Call::<T>::change_status { collection, item_id, status: AssetStatus::VOTING }
					.into();
			let call1_wrap = Box::new(call1);
			Vcalls::<T>::mutate(collection_id, item_id, |val| {
				let mut v0 = val.clone().unwrap();
				v0.democracy_status = call1_wrap;
				*val = Some(v0);
			});

			//Create Call for proposal reject_edit
			let call2: T::Prop =
				Call::<T>::reject_edit { collection, item_id, infos: house.clone() }.into();
			let call2_wrap = Box::new(call2);
			Vcalls::<T>::mutate(collection_id, item_id, |val| {
				let mut v0 = val.clone().unwrap();
				v0.reject_edit = call2_wrap;
				*val = Some(v0);
			});

			//Create Call for proposal reject_destroy
			let call3: T::Prop =
				Call::<T>::reject_destroy { collection, item_id, infos: house }.into();
			let call3_wrap = Box::new(call3);
			Vcalls::<T>::mutate(collection_id, item_id, |val| {
				let mut v0 = val.clone().unwrap();
				v0.reject_destroy = call3_wrap;
				*val = Some(v0);
			});

			//Create Call for asset status change after Investor's vote
			let call4: T::Prop =
				Call::<T>::change_status { collection, item_id, status: AssetStatus::ONBOARDED }
					.into();
			let call4_wrap = Box::new(call4);
			Vcalls::<T>::mutate(collection_id, item_id, |val| {
				let mut v0 = val.clone().unwrap();
				v0.after_vote_status = call4_wrap;
				*val = Some(v0);
			});

			Self::deposit_event(Event::ProposalCreated {
				who: caller.clone(),
				collection: collection_id,
				item: item_id,
				price,
			});

			Self::deposit_event(Event::FundsReserved {
				from_who: caller.clone(),
				amount: Some(balance0),
			});

			if submit {
				//Change asset status to REVIEWING
				Self::change_status(origin.clone(), collection, item_id, AssetStatus::REVIEWING)
					.ok();
				//Send Proposal struct to voting pallet
				//get the needed call and convert them to pallet_voting format
				let out_call = Vcalls::<T>::get(collection_id, item_id).unwrap();

				let w_status0 = Box::new(
					Self::get_formatted_collective_proposal(*out_call.democracy_status).unwrap(),
				);
				let w_status1 = Box::new(
					Self::get_formatted_collective_proposal(*out_call.after_vote_status).unwrap(),
				);

				let w_r_destroy = Box::new(
					Self::get_formatted_collective_proposal(*out_call.reject_destroy).unwrap(),
				);
				let w_r_edit = Box::new(
					Self::get_formatted_collective_proposal(*out_call.reject_edit).unwrap(),
				);
				//Send Calls struct to voting pallet
				Votes::Pallet::<T>::submit_proposal(
					origin,
					w_status1,
					w_status0,
					w_r_destroy,
					w_r_edit,
				)
				.ok();

				Self::deposit_event(Event::ProposalSubmitted {
					who: caller,
					collection: collection_id,
					item: item_id,
					price,
				});
			}

			Ok(())
		}

		///Submit an awaiting proposal for review
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn submit_awaiting(
			origin: OriginFor<T>,
			collection: NftCollectionOf,
			item_id: T::NftItemId,
			price: Option<BalanceOf<T>>,
			data: Option<Nft::BoundedVecOfUnq<T>>,
		) -> DispatchResult {
			let caller = ensure_signed(origin.clone()).unwrap();
			ensure!(Roles::Pallet::<T>::sellers(&caller).is_some(), Error::<T>::ReservedToSeller);

			let collection_id: T::NftCollectionId = collection.clone().value().into();
			ensure!(
				Houses::<T>::contains_key(collection_id, item_id),
				Error::<T>::CollectionOrItemUnknown
			);
			let house = Self::houses(collection_id, item_id).unwrap();
			ensure!(
				house.status == AssetStatus::EDITING || house.status == AssetStatus::REJECTED,
				Error::<T>::CannotSubmitItem
			);

			//Edit asset price
			let price0 = Prices::<T>::get(collection_id, item_id).unwrap();

			let data0 = Nft::Pallet::<T>::items(collection_id, item_id).unwrap().metadata;
			let data1 = data.unwrap_or(data0.clone());
			let collection_owner = Nft::Pallet::<T>::collection_owner(collection_id).unwrap();
			if data1 != data0 {
				let res =
					Nft::Pallet::<T>::set_metadata(collection_owner, collection_id, item_id, data1);
				debug_assert!(res.is_ok());
			}

			let mut b = price.unwrap_or(price0);
			if b == Zero::zero() {
				b = price0;
			} else {
				Self::set_price(origin.clone(), collection, item_id, Some(b)).ok();
			}

			//Change asset status to REVIEWING
			Self::change_status(origin.clone(), collection, item_id, AssetStatus::REVIEWING).ok();

			//get the needed call and convert them to pallet_voting format
			let out_call = Vcalls::<T>::get(collection_id, item_id).unwrap();
			let w_status1 = Box::new(
				Self::get_formatted_collective_proposal(*out_call.after_vote_status).unwrap(),
			);
			let w_status0 = Box::new(
				Self::get_formatted_collective_proposal(*out_call.democracy_status).unwrap(),
			);
			let w_r_destroy = Box::new(
				Self::get_formatted_collective_proposal(*out_call.reject_destroy).unwrap(),
			);
			let w_r_edit =
				Box::new(Self::get_formatted_collective_proposal(*out_call.reject_edit).unwrap());
			//Send Calls struct to voting pallet
			Votes::Pallet::<T>::submit_proposal(
				origin,
				w_status1,
				w_status0,
				w_r_destroy,
				w_r_edit,
			)
			.ok();

			Self::deposit_event(Event::ProposalSubmitted {
				who: caller,
				collection: collection_id,
				item: item_id,
				price: Some(b),
			});

			Ok(())
		}
	}
}
