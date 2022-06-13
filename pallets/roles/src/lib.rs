#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;
mod structs;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;


pub use crate::structs::*;
#[frame_support::pallet]
pub mod pallet {
	pub use super::*;
	
	
	///This enum contains the roles selectable at account creation
	#[derive(Clone, Encode, Decode,PartialEq, Eq, TypeInfo)]
	#[cfg_attr(feature = "std", derive(Debug))]
	pub enum Accounts{
		INVESTOR,
		SELLER,
		TENANT,
		SERVICER,
	}   

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: ReservableCurrency<Self::AccountId>;
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
   ///Registry of Investors organized by AccountId
	pub(super) type InvestorLog<T: Config> = StorageMap<_, Twox64Concat, AccountIdOf<T>, Investor::<T>, OptionQuery>;

   #[pallet::storage]
   ///Registry of Sellers organized by AccountId
	pub(super) type HouseSellerLog<T: Config> = StorageMap<_, Twox64Concat, AccountIdOf<T>, HouseSeller::<T>, OptionQuery>;

	#[pallet::storage]
   ///Registry of Tenants organized by AccountId
	pub(super) type TenantLog<T: Config> = StorageMap<_, Twox64Concat, AccountIdOf<T>, Tenant::<T>, OptionQuery>;

	#[pallet::storage]
   ///Registry of Servicers organized by AccountId
	pub(super) type ServicerLog<T: Config> = StorageMap<_, Twox64Concat, AccountIdOf<T>, Servicer::<T>, OptionQuery>;

	#[pallet::type_value]
   pub(super) fn MyDefault<T: Config>() -> Idle<T> { (Vec::new(),Vec::new()) }
	#[pallet::storage]
   ///Registry of Sellers organized by AccountId
	pub(super) type WaitingList<T: Config> = StorageValue<_, Idle<T>, ValueQuery,MyDefault<T>>;




	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
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

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {


		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
      pub fn create_account(origin:OriginFor<T>, account_type:Accounts) -> DispatchResult{
         let caller = ensure_signed(origin.clone())?; 
         match account_type{
            Accounts::INVESTOR => {
               ensure!(InvestorLog::<T>::contains_key(&caller)==false,Error::<T>::NoneValue);
               ensure!(HouseSellerLog::<T>::contains_key(&caller)==false,Error::<T>::NoneValue);
			   ensure!(ServicerLog::<T>::contains_key(&caller)==false,Error::<T>::NoneValue);
               ensure!(TenantLog::<T>::contains_key(&caller)==false,Error::<T>::NoneValue);
               let _acc = Investor::<T>::new(origin);
               Ok(().into())
            },
            Accounts::SELLER => {
               ensure!(HouseSellerLog::<T>::contains_key(&caller)==false,Error::<T>::NoneValue);
               ensure!(InvestorLog::<T>::contains_key(&caller)==false,Error::<T>::NoneValue);
			   ensure!(ServicerLog::<T>::contains_key(&caller)==false,Error::<T>::NoneValue);
               ensure!(TenantLog::<T>::contains_key(&caller)==false,Error::<T>::NoneValue);
               //Bring the decision for this account creation to a vote
               let _acc = HouseSeller::<T>::new(origin);
               Ok(().into())
            },
            Accounts::TENANT => {
				ensure!(HouseSellerLog::<T>::contains_key(&caller)==false,Error::<T>::NoneValue);
				ensure!(InvestorLog::<T>::contains_key(&caller)==false,Error::<T>::NoneValue);
				ensure!(ServicerLog::<T>::contains_key(&caller)==false,Error::<T>::NoneValue);
               ensure!(TenantLog::<T>::contains_key(&caller)==false,Error::<T>::NoneValue);
               let _acc = Tenant::<T>::new(origin);
               Ok(().into())
            },
			Accounts::SERVICER => {
				ensure!(HouseSellerLog::<T>::contains_key(&caller)==false,Error::<T>::NoneValue);
				ensure!(InvestorLog::<T>::contains_key(&caller)==false,Error::<T>::NoneValue);
				ensure!(ServicerLog::<T>::contains_key(&caller)==false,Error::<T>::NoneValue);
               ensure!(TenantLog::<T>::contains_key(&caller)==false,Error::<T>::NoneValue);
               let _acc = Servicer::<T>::new(origin);
               Ok(().into())
            },
         }
        
         
      }

		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
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