#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;
mod roles;
pub use crate::roles::*;

pub use pallet_nft::pallet as NftL;


#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
   use super::*;
   use frame_support::{
      dispatch::DispatchResult,
      pallet_prelude::*,
      sp_runtime::traits::{Hash, Zero},
      storage::child,
      traits::{Currency, Get, ReservableCurrency},
      PalletId		
   };
   use frame_system::{ensure_signed};
   use frame_support::inherent::Vec;
   use pallet_nft::{BlockNumberOf, ClassData, ClassIdOf, TokenIdOf,Properties,CID,ClassType};
   

   //const PALLET_ID: PalletId = PalletId(*b"ex/cfund");
   //const TREASURE_PALLET_ID: PalletId = PalletId(*b"py/trsry");

   /// Configure the pallet by specifying the parameters and types on which it depends.
   #[pallet::config]
   pub trait Config: frame_system::Config {
      /// Because this pallet emits events, it depends on the runtime's definition of an event.
      type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
      type Currency: ReservableCurrency<Self::AccountId>;
      type MinContribution: Get<BalanceOf<Self>>;
   }
	
   pub type HouseIndex = u32;
   pub type Owners<T> = Vec<AccountIdOf<T>>;
   type AccountIdOf<T> = <T as frame_system::Config>::AccountId;   
   type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
   type Bool = bool;


   #[pallet::pallet]
   #[pallet::generate_store(pub(super) trait Store)]
   pub struct Pallet<T>(_);



   // The pallet's runtime storage items.
   // https://docs.substrate.io/v3/runtime/storage
   #[pallet::storage]
   #[pallet::getter(fn something)]
   // Learn more about declaring storage items:
   // https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
   pub type Something<T> = StorageValue<_, u32>;
   

   #[pallet::storage]
	#[pallet::getter(fn contrib_log)]
	pub(super) type ContributionsLog<T> = StorageMap<_, Blake2_128Concat, AccountIdOf<T>, BalanceOf<T>, ValueQuery>;

   

   // Pallets use events to inform users when important changes are made.
   // https://docs.substrate.io/v3/runtime/events-and-errors
   #[pallet::event]
   #[pallet::generate_deposit(pub(super) fn deposit_event)]
   pub enum Event<T: Config> {
      /// Event documentation should end with an array that provides descriptive names for event
      /// parameters. [something, who]
      SomethingStored(u32, T::AccountId),
      Created( <T as frame_system::Config>::BlockNumber),
      Created2(HouseIndex, <T as frame_system::Config>::BlockNumber), //Kazu:for creation of a proposal
      Contributed(
         <T as frame_system::Config>::AccountId,
         BalanceOf<T>,
         <T as frame_system::Config>::BlockNumber,
      ),
      Withdrew(
         <T as frame_system::Config>::AccountId,
         BalanceOf<T>,
         <T as frame_system::Config>::BlockNumber,
      ),
      Retiring(<T as frame_system::Config>::BlockNumber),
      Dissolved(		
         <T as frame_system::Config>::BlockNumber,
         <T as frame_system::Config>::AccountId,
      ),
      Dispensed(		
         <T as frame_system::Config>::BlockNumber,
         <T as frame_system::Config>::AccountId,
      ),
   }
   

   // Errors inform users that something went wrong.
   #[pallet::error]
   pub enum Error<T> {
      /// Error names should be descriptive.
      NoneValue,
      /// Errors should have helpful documentation associated with them.
      StorageOverflow,
      /// Crowdfund must end after it starts
      EndTooEarly,
      /// Must contribute at least the minimum amount of funds
      ContributionTooSmall,
      /// The fund index specified does not exist
      InvalidIndex,
      /// The crowdfund's contribution period has ended; no more contributions will be accepted
      ContributionPeriodOver,
      /// You may not withdraw or dispense funds while the fund is still active
      FundStillActive,
      /// You cannot withdraw funds because you have not contributed any
      NoContribution,
      /// You cannot dissolve a fund that has not yet completed its retirement period
      FundNotRetired,
      /// Cannot dispense funds from an unsuccessful fund
      UnsuccessfulFund,
      /// Proposal already Funded
      AlreadyFunded
   }
   

   // Dispatchable functions allows users to interact with the pallet and invoke state changes.
   // These functions materialize as "extrinsics", which are often compared to transactions.
   // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
   #[pallet::call]
   impl<T: Config> Pallet<T> {
      /// An example dispatchable that takes a singles value as a parameter, writes the value to
      /// storage and emits an event. This function must be dispatched by a signed extrinsic.
      #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
      pub fn do_something(origin: OriginFor<T>, something: u32, acc:AccountIdOf<T>,rent:BalanceOf<T>,cd:CID,prop:Properties,start:Option<BlockNumberOf<T>>,end:Option<BlockNumberOf<T>>,cl:ClassIdOf<T>) -> DispatchResult { // cl:ClassIdOf<T>
         // Check that the extrinsic was signed and get the signer.
         // This function will return an error if the extrinsic is not signed.
         // https://docs.substrate.io/v3/runtime/origins
         let who = ensure_signed(origin)?;
         let dev=Investor::new(&acc,something);
         let _tenant=Tenant::new(&acc,rent);
         
         //let class0= NftL::Pallet::mint(&who,acc,cl,cd,1);

         // Update storage.
         <Something<T>>::put(dev.nft+something);

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
            None => Err(Error::<T>::NoneValue)?,
            Some(old) => {
               // Increment the value read from storage; will error in the event of overflow.
               let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
               // Update the value in storage with the incremented result.
               <Something<T>>::put(new);
               Ok(())
            },
         }
      }
      
 
      
      /// Withdraw full balance of a contributor to treasury
      #[pallet::weight(10_000)]
      pub fn withdraw(
         origin: OriginFor<T>,
         #[pallet::compact]index: HouseIndex,
      ) -> DispatchResultWithPostInfo {
	
	// Check the inputs
	let who = ensure_signed(origin)?;
	let balance = Self::contribution_get(&who);
	ensure!(balance > Zero::zero(), Error::<T>::NoContribution);
	
	// Execute treatment
	/// TODO : extract execution from following commented code
	
	let now = <frame_system::Pallet<T>>::block_number();
	
//	let _fund = Self::props(index);
//	// ensure!(fund.end < now, Error::<T>::FundStillActive);


	// Return funds to caller without charging a transfer fee
//	let _ = T::Currency::resolve_into_existing(
//		&who,
//		T::Currency::withdraw(
//			&TREASURE_PALLET_ID.into_account(),
//			balance,
//			WithdrawReasons::TRANSFER,
//			ExistenceRequirement::AllowDeath,
//		)?,
//	);

	// Update storage
	Self::contribution_kill( &who);
	
	// Raise event
	Self::deposit_event(Event::Withdrew(who, balance, now));
	
	// Exit
	Ok(().into())
      }
   }
   
   impl<T: Config> Pallet<T> {
   
      /// Each fund stores information about its contributors and their contributions in a child trie
      // This helper function calculates the id of the associated child trie.
      pub fn id_from_index() -> child::ChildInfo {
         let mut buf = Vec::new();
         buf.extend_from_slice(b"treasury");
         //buf.extend_from_slice(&index.to_le_bytes()[..]);

         child::ChildInfo::new_default(T::Hashing::hash(&buf[..]).as_ref())
      }
   
      /// Lookup a contribution in the associated child trie.
      pub fn contribution_get(who: &T::AccountId) -> BalanceOf<T> {
         let id = Self::id_from_index();
         who.using_encoded(|b| child::get_or_default::<BalanceOf<T>>(&id, b))
      }
      
      /// Remove a contribution from an associated child trie.
      pub fn contribution_kill(who: &T::AccountId) {
         let id = Self::id_from_index();
         who.using_encoded(|b| child::kill(&id, b));
      }
   }
}
