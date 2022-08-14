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
//! - Reject a proposal for price editing purpose (asset is marked as REJECTEDIT and re-submission is possible)
//! - Reject a proposal for destruction (NFT is burned, asset is marked as REJECTBURN)
//!
//! ### Dispatchable Functions
//! #### Role setting
//!
//! * `do_something` - Used in a Call to initialize the fields of the VotingCalls struct.
//!  
//! * `set_price` - Modify the price of an Existing proposal with the status EDIT or REJECTEDIT
//! Proposal price is the only part that can be edited
//! 
//! * `do_buy` - Execute the buy/sell transaction.
//! Sent to the voting pallet as a Call.
//!
//! * `reject_edit` - Reject a submitted proposal for price editing.
//! Sent to the voting pallet as a Call.
//!
//! * `reject_destroy` - Reject a submitted proposal for destruction.
//! Sent to the voting pallet as a Call.
//!
//! * `create_and_submit_proposal` - Creation and submission of a proposal.
//! A struct containing Calls for the voting pallet is also created and stored.
//! the proposal submission is optionnal, and can be disabled through the value 
//! of the boolean `submit`.
//!
//! * `submit_awaiting` - Submit/edit an awaiting proposal for review.
//! This is also used for re-submission of rejected proposals. 


#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]


mod types;
mod functions;

pub use types::*;
pub use functions::*;

pub use pallet_roles as Roles;
pub use pallet_nft as Nft;
pub use pallet_sudo as Sudo;
pub use pallet_voting as Votes; 

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::WeightInfo;


pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;


#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;	

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + Roles::Config + Nft::Config + Sudo::Config + Votes::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: ReservableCurrency<Self::AccountId>;
		type Prop: Parameter + Dispatchable<Origin = <Self as frame_system::Config>::Origin> + From<Call<Self>>;
		type ProposalFee: Get<BalanceOf<Self>>;
		type WeightInfo: WeightInfo;
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
	pub(super) type Houses<T: Config> = StorageDoubleMap<
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
		InsufficientBalance
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		//#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::do_something(100))]
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

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		#[transactional]
		pub fn change_status(
			origin: OriginFor<T>,
			collection: NftCollectionOf,
			item_id: T::NftItemId,
			status:AssetStatus,
		)-> DispatchResult {
			let _caller = ensure_signed(origin.clone()).unwrap();
			Self::status(collection,item_id,status);
			Ok(())
		}

		/// Modify the price of an Existing proposal
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		#[transactional]
		pub fn set_price(
			origin: OriginFor<T>,
			collection: NftCollectionOf,
			item_id: T::NftItemId,
			new_price: Option<BalanceOf<T>>,
		) -> DispatchResult {
			let caller = ensure_signed(origin.clone()).unwrap();
			let collection_id: T::NftCollectionId = collection.clone().value().into();
			ensure!(Houses::<T>::contains_key(collection_id.clone(),item_id.clone()),Error::<T>::CollectionOrItemUnknown);

			Houses::<T>::mutate_exists(collection_id.clone(),item_id.clone(),|val|{
				let mut v0 = val.clone().unwrap();
				v0.price = new_price;
				*val = Some(v0)
			});
			
			let asset = Self::houses(collection_id.clone(),item_id.clone()).unwrap();
			let status = asset.status;
			ensure!(status == AssetStatus::EDITING || status == AssetStatus::REJECTEDIT,Error::<T>::CannotEditItem);

			Self::price(origin,collection,item_id.clone(),new_price.clone()).ok();
			
			Self::deposit_event(Event::TokenPriceUpdated {
                who: caller,
                collection: collection_id.clone(),
                item: item_id.clone(),
                price: new_price,
            });

			Ok(())
		}

		///Execute the buy/sell transaction
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn do_buy(
            origin: OriginFor<T>,
            collection: NftCollectionOf,
            item_id: T::NftItemId,
			_infos: Asset<T>,
        ) -> DispatchResult {
			let buyer = ensure_signed(origin.clone()).unwrap();
            let collection_id: T::NftCollectionId = collection.clone().value().into();

			//Check that the house item exists and has the correct status
			ensure!(Houses::<T>::contains_key(collection_id.clone(),item_id.clone()),Error::<T>::CollectionOrItemUnknown);			
			let asset = Self::houses(collection_id.clone(),item_id.clone()).unwrap();
			let status = asset.status;
			ensure!(status == AssetStatus::VOTING,Error::<T>::VoteNedeed);
			
			//Check that the owner is not the buyer 
            let owner = Nft::Pallet::<T>::owner(collection_id.clone(), item_id.clone()).ok_or(Error::<T>::CollectionOrItemUnknown)?;
            ensure!(buyer != owner, Error::<T>::BuyFromSelf);
			
			//Execute transaction
            let owner_origin = <T as frame_system::Config>::Origin::from(RawOrigin::Signed(owner.clone()));
            let price = Prices::<T>::get(collection_id.clone(), item_id.clone()).unwrap();
            <T as Config>::Currency::transfer(&buyer, &owner, price, ExistenceRequirement::KeepAlive)?;
            let to = T::Lookup::unlookup(buyer.clone());
            Nft::Pallet::<T>::transfer(owner_origin, collection, item_id.clone(), to)?;
            Self::deposit_event(Event::TokenSold {
                owner,
                buyer,
                collection: collection_id.clone(),
                item: item_id.clone(),
                price,
            });

			//change status
			Self::change_status(origin.clone(),collection.clone(),item_id.clone(),AssetStatus::APPROVED).ok();


            Ok(())
        }

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn reject_edit(
			origin: OriginFor<T>,
            collection: NftCollectionOf,
            item_id: T::NftItemId,
			_infos: Asset<T>,
		) -> DispatchResult{
				let caller = ensure_signed(origin.clone()).unwrap();
				let collection_id: T::NftCollectionId = collection.clone().value().into();
				ensure!(Houses::<T>::contains_key(collection_id.clone(),item_id.clone()),Error::<T>::CollectionOrItemUnknown);
				let house = Self::houses(collection_id.clone(),item_id.clone()).unwrap();
				ensure!(house.status == AssetStatus::REVIEWING || house.status == AssetStatus::VOTING,Error::<T>::CannotSubmitItem);
				Self::change_status(origin.clone(),collection.clone(),item_id.clone(),AssetStatus::REJECTEDIT).ok();

				let owner = Nft::Pallet::<T>::owner(collection_id.clone(), item_id.clone()).unwrap();
				let balance = <T as Config>::Currency::reserved_balance(&owner);

				let wrap_balance = Self::balance_to_u64_option(balance).unwrap();
				let slash = wrap_balance*10/100;
				let fees = Self::u64_to_balance_option(slash).unwrap();
				<T as Config>::Currency::slash_reserved(&owner,fees);
				//<T as Config>::Currency::rapatriate_reserved(&owner,beneficiary,fees,BalanceStatus::free);

				Self::deposit_event(Event::RejectedForEditing {
					by_who: caller,
					collection: collection_id.clone(),
					item: item_id.clone(),
				});

				Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn reject_destroy(
			origin: OriginFor<T>,
            collection: NftCollectionOf,
            item_id: T::NftItemId,
			_infos: Asset<T>,
		) -> DispatchResult{
			let caller = ensure_signed(origin.clone()).unwrap();
				let collection_id: T::NftCollectionId = collection.clone().value().into();
				ensure!(Houses::<T>::contains_key(collection_id.clone(),item_id.clone()),Error::<T>::CollectionOrItemUnknown);
				let house = Self::houses(collection_id.clone(),item_id.clone()).unwrap();
				ensure!(house.status == AssetStatus::REVIEWING || house.status == AssetStatus::VOTING,Error::<T>::CannotSubmitItem);
				Self::change_status(origin.clone(),collection.clone(),item_id.clone(),AssetStatus::REJECTBURN).ok();
				let owner = Nft::Pallet::<T>::owner(collection_id, item_id).unwrap();
				Nft::Pallet::<T>::burn(origin,collection,item_id.clone()).ok();				
				let balance = <T as Config>::Currency::reserved_balance(&owner);
				ensure!(balance>Zero::zero(),Error::<T>::NoneValue);
				<T as Config>::Currency::slash_reserved(&owner,balance);
				//<T as Config>::Currency::rapatriate_reserved(&owner,beneficiary,balance,BalanceStatus::free);

				Self::deposit_event(Event::RejectedForDestruction {
					by_who: caller,
					collection: collection_id.clone(),
					item: item_id.clone(),
				});

				Ok(())
		}
       
		
		/// `create_and_submit_proposal` - Creation and submission of a proposal.
		/// the proposal submission is optionnal, and can be disabled through the value 
		/// of the boolean `submit`.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		#[transactional]
		pub fn create_and_submit_proposal(
            origin: OriginFor<T>,
            collection: NftCollectionOf,
            price: Option<BalanceOf<T>>,
			metadata: Nft::BoundedVecOfUnq<T>,
			submit: bool,
            )-> DispatchResult {

				let caller = ensure_signed(origin.clone()).unwrap();
				let idx = collection.clone().value() as usize;
				
				// Get itemId and infos from minted nft
				let item_id: T::NftItemId = Nft::ItemsCount::<T>::get()[idx].into();

				//Create asset
				let balance0 = T::ProposalFee::get();
				let balance1 = <T as Config>::Currency::free_balance(&caller);
				ensure!(balance1>balance0,Error::<T>::InsufficientBalance);

				<T as Config>::Currency::reserve(&caller,T::ProposalFee::get()).ok();
				Self::create_asset(origin.clone(),collection.clone(),metadata,price.clone(),item_id.clone()).ok();
			
				
				let collection_id: T::NftCollectionId = collection.clone().value().into();
				
				let house = Self::houses(collection_id.clone(),item_id.clone()).unwrap();
			
				//Create Call for the sell/buy transaction
				let _new_call = VotingCalls::<T>::new(collection_id.clone(),item_id.clone()).ok();
				let call0:T::Prop = Call::<T>::do_buy{collection: collection.clone(),item_id: item_id.clone(),infos:house.clone()}.into();
				Vcalls::<T>::mutate(collection_id.clone(),item_id.clone(),|val|{
					let mut v0 = val.clone().unwrap();
					v0.buy = call0;
					*val = Some(v0);
				});
				
				//Create Call for collective-to-democracy status change
				let call1:T::Prop = Call::<T>::change_status{collection: collection.clone(),item_id: item_id.clone(),status: AssetStatus::VOTING}.into();
				Vcalls::<T>::mutate(collection_id,item_id.clone(),|val|{
					let mut v0 = val.clone().unwrap();
					v0.democracy_status = call1;
					*val = Some(v0);
				});

				//Create Call for proposal reject_edit
				let call2:T::Prop = Call::<T>::reject_edit{collection: collection.clone(),item_id: item_id.clone(),infos: house.clone()}.into();
				Vcalls::<T>::mutate(collection_id,item_id.clone(),|val|{
					let mut v0 = val.clone().unwrap();
					v0.reject_edit = call2;
					*val = Some(v0);
				});


				//Create Call for proposal reject_destroy
				let call3:T::Prop = Call::<T>::reject_destroy{collection: collection.clone(),item_id: item_id.clone(),infos: house.clone()}.into();
				Vcalls::<T>::mutate(collection_id.clone(),item_id.clone(),|val|{
					let mut v0 = val.clone().unwrap();
					v0.reject_destroy = call3;
					*val = Some(v0);
				});

				Self::deposit_event(Event::ProposalCreated {
					who: caller.clone(),
					collection: collection_id.clone(),
					item: item_id.clone(),
					price: price.clone(),
				});

				if submit == true{
					//Change asset status to REVIEWING
					Self::change_status(origin.clone(),collection.clone(),item_id.clone(),AssetStatus::REVIEWING).ok();
					//Send Proposal struct to voting pallet

					Self::deposit_event(Event::ProposalSubmitted {
						who: caller,
						collection: collection_id.clone(),
						item: item_id.clone(),
						price: price,
					});
				}	

				Ok(())

				}
				

		///Submit an awaiting proposal for review
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		#[transactional]
		pub fn submit_awaiting(
            origin: OriginFor<T>,
            collection: NftCollectionOf,
            item_id: T::NftItemId,
			price: Option<BalanceOf<T>>,
            )-> DispatchResult {
				let caller = ensure_signed(origin.clone()).unwrap();
				let collection_id: T::NftCollectionId = collection.clone().value().into();
				ensure!(Houses::<T>::contains_key(collection_id.clone(),item_id.clone()),Error::<T>::CollectionOrItemUnknown);
				let house = Self::houses(collection_id.clone(),item_id.clone()).unwrap();
				ensure!(house.status == AssetStatus::EDITING || house.status == AssetStatus::REJECTEDIT,Error::<T>::CannotSubmitItem);

				//Edit asset price
				let price0 = Prices::<T>::get(collection_id.clone(),item_id.clone()).unwrap();
				let b = price.unwrap_or(price0);
				if price0 != b{
				Self::set_price(origin.clone(),collection.clone(),item_id.clone(),Some(b)).ok();
				}
		
				//Change asset status to REVIEWING
				Self::change_status(origin.clone(),collection.clone(),item_id.clone(),AssetStatus::REVIEWING).ok();
				let house = Self::houses(collection_id.clone(),item_id.clone()).unwrap();
			
				//Update Call for the sell/buy transaction
				let call:T::Prop = Call::<T>::do_buy{collection: collection,item_id: item_id.clone(),infos:house}.into();
				Vcalls::<T>::mutate(collection_id,item_id.clone(),|val|{
					let mut v0 = val.clone().unwrap();
					v0.buy = call;
					*val = Some(v0);
				});
				
				//Send Calls struct to voting pallet

				Self::deposit_event(Event::ProposalSubmitted {
					who: caller,
					collection: collection_id.clone(),
					item: item_id.clone(),
					price: price,
				});

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
