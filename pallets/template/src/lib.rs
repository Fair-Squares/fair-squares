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
mod items;
mod functions;
pub mod weights;

pub use crate::roles::*;
pub use items::*;

use pallet_nft::pallet as NftL;

pub use weights::WeightInfo;


#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub const INVESTOR_ROLE: u16 = 1;
pub const HOUSE_OWNER_ROLE: u16 = 2;
pub const TENANT_ROLE: u16 = 3;

#[frame_support::pallet]
pub mod pallet {
   use super::*;
   use frame_support::{
      dispatch::DispatchResult,
      transactional,
      sp_runtime::traits::{AccountIdConversion, Zero},
      traits::{Currency, ExistenceRequirement, Get, ReservableCurrency},
      PalletId		
   };
   use frame_system::{ensure_signed};
   use frame_support::inherent::Vec;
   //use std::mem;
   

   pub const PALLET_ID: PalletId = PalletId(*b"ex/cfund");
   pub const TREASURE_PALLET_ID: PalletId = PalletId(*b"py/trsry");

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

   // The pallet's runtime storage items.
   // https://docs.substrate.io/v3/runtime/storage
   #[pallet::storage]
   #[pallet::getter(fn something)]
   // Learn more about declaring storage items:
   // https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
   pub type Something<T> = StorageValue<_, u32>;

   #[pallet::storage]
   #[pallet::getter(fn fund_amount)]
   pub(super) type FundAmount<T> = StorageValue<
      _, 
      BalanceOf<T>, 
      ValueQuery
      >;

   #[pallet::storage]
   #[pallet::getter(fn contributions)]
   pub type Contributions<T> = StorageMap<
      _, 
      Blake2_128Concat, 
      AccountIdOf<T>, 
      (BalanceOf<T>, u32), 
      ValueQuery
      >;
   
   #[pallet::storage]
   #[pallet::getter(fn contribution_log)]
   pub type ContributionLog<T> = StorageMap<
      _, 
      Blake2_128Concat, 
      AccountIdOf<T>, 
      Vec<Contribution<T>>, 
      ValueQuery
      >;

   #[pallet::storage]
   #[pallet::getter(fn roles)]
   pub(super) type Roles<T> = StorageMap<
      _, 
      Blake2_128Concat, 
      AccountIdOf<T>, 
      Vec<Role>, 
      ValueQuery
      >;

   #[pallet::storage]
   #[pallet::getter(fn ownership_index)]
   pub(super) type OwnershipIndex<T> = StorageValue<_, StorageIndex, ValueQuery>;

   #[pallet::storage]
   #[pallet::getter(fn ownerships)]
   pub(super) type Ownerships<T> = StorageDoubleMap<
      _, 
      Blake2_128Concat, 
      (StorageIndex, AccountIdOf<T>), 
      Blake2_128Concat, 
      StorageIndex, 
      Ownership<T>, 
      OptionQuery
      >;
   
   #[pallet::storage]
   #[pallet::getter(fn house_index)]
   pub(super) type HouseIndexBis<T> = StorageValue<_, StorageIndex, ValueQuery>;

   #[pallet::storage]
   #[pallet::getter(fn minted_houses)]
   pub(super) type MintedHouses<T> = StorageMap<
      _, 
      Blake2_128Concat, 
      StorageIndex, 
      HouseMinted::<T,NftIndex>, 
      OptionQuery
      >;

   #[pallet::storage]
   #[pallet::getter(fn fs_houses)]
   pub(super) type FSHouses<T> = StorageMap<
      _, 
      Blake2_128Concat, 
      StorageIndex, 
      HouseMinted::<T,NftIndex>, 
      OptionQuery
      >;

   #[pallet::storage]
   #[pallet::getter(fn proposal_index)]
   pub type ProposalIndex<T> = StorageValue<_, StorageIndex, ValueQuery>;

   #[pallet::storage]
   #[pallet::getter(fn proposals)]
   pub(super) type Proposals<T> = StorageDoubleMap<
      _, 
      Blake2_128Concat, 
      (StorageIndex, AccountIdOf<T>), 
      Blake2_128Concat, 
      StorageIndex, 
      Proposal<T>, 
      OptionQuery
      >;
   
   #[pallet::storage]
   #[pallet::getter(fn vote_index)]
   pub type VoteIndex<T> = StorageValue<_, StorageIndex, ValueQuery>;

   #[pallet::storage]
   #[pallet::getter(fn votes)] 
   pub type Votes<T> = StorageDoubleMap<
      _, 
      Blake2_128Concat, 
      StorageIndex, 
      Blake2_128Concat, 
      AccountIdOf<T>, 
      Vote<T>, 
      OptionQuery
      >;  
   

   // Pallets use events to inform users when important changes are made.
   // https://docs.substrate.io/v3/runtime/events-and-errors
   #[pallet::event]
   #[pallet::generate_deposit(pub(super) fn deposit_event)]
   pub enum Event<T: Config> {
      /// Event documentation should end with an array that provides descriptive names for event
      /// parameters. [something, who]
      SomethingStored(u32, T::AccountId),
      Created( <T as frame_system::Config>::BlockNumber),
      MintedHouse(<T as frame_system::Config>::AccountId, <T as frame_system::Config>::BlockNumber),
      CreatedProposal(<T as frame_system::Config>::AccountId, <T as frame_system::Config>::BlockNumber),
      CreatedVote(<T as frame_system::Config>::AccountId, <T as frame_system::Config>::BlockNumber),
      RealizedDistribution(<T as frame_system::Config>::AccountId, <T as frame_system::Config>::BlockNumber),
      Created2(u32, <T as frame_system::Config>::BlockNumber), //Kazu:for creation of a proposal
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
      /// No contribution exist for the account
      ContributionNotExists,
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
      AlreadyFunded,
      /// Account not registered
      UnregisteredAccount,
      /// Incorrect role for action
      IncorrectRole,
      // Not owned house
      NotOwnedHouse,
      // A proposal is active for a house
      AlreadyActiveProposal,
      // The investor cannot vote twice
      AlreadyVotedProposal,
      // The proposal is no longer active
      ProposalOutDated
   }
   

   // Dispatchable functions allows users to interact with the pallet and invoke state changes.
   // These functions materialize as "extrinsics", which are often compared to transactions.
   // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
   #[pallet::call]
   impl<T: Config> Pallet<T> {
      
      /// An example dispatchable that may throw a custom error.
      #[pallet::weight(T::WeightInfo::cause_error())]
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
      #[pallet::weight(T::WeightInfo::withdraw())]
      pub fn withdraw(
         origin: OriginFor<T>,
         #[pallet::compact]index: StorageIndex,
      ) -> DispatchResultWithPostInfo {
	
	// Check the inputs
	let who = ensure_signed(origin)?;
	let balance = Self::contribution_get(&who);
	ensure!(balance > Zero::zero(), Error::<T>::NoContribution);
	
	// Execute treatment
	/// TODO : extract execution from following commented code
	
	let now = <frame_system::Pallet<T>>::block_number();

	// Update storage
	Self::contribution_kill( &who);
	
	// Raise event
	Self::deposit_event(Event::Withdrew(who, balance, now));
	
	// Exit
	Ok(().into())
      }
      
      // Process a proposal to distribute house share between investissor according to the votes
      #[pallet::weight(T::WeightInfo::manage_proposal())]
      #[transactional]
      pub fn manage_proposal(origin: OriginFor<T>, 
         house_id: StorageIndex,
         house_owner_account: AccountIdOf<T>, 
         proposal_id: StorageIndex
      ) -> DispatchResultWithPostInfo {
      
         // Check the inputs
         let who = ensure_signed(origin)?;

         let engine_processor = EngineProcessor::<T>::new(who.clone());
         let result = engine_processor.manage_proposal(house_id, house_owner_account.clone(), proposal_id);
         
         match result {
            Ok(n)  => { 

               let block_number = <frame_system::Pallet<T>>::block_number();
               // Raise event
               Self::deposit_event(Event::RealizedDistribution(who.clone(), block_number));
               Ok(().into()) 
            },
            Err(e) => Err(e),
        }
      }
      
      /// Withdraw full balance of a contributor to treasury
      #[pallet::weight(T::WeightInfo::withdraw_house_contribution())]
      pub fn withdraw_house_contribution(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
      
         // Check the inputs
         let who = ensure_signed(origin)?;
         // Execute treatment
         // Raise event
         // Exit
	      Ok(().into())
      }

      // Add a contribution to the common fund for thhe investissor
      #[pallet::weight(T::WeightInfo::add_contribution_fund())]
      #[transactional]
      pub fn add_contribution_fund(origin: OriginFor<T>, account: AccountIdOf<T>, amount: BalanceOf<T>) -> DispatchResultWithPostInfo {
         // Check the inputs
         let who = ensure_signed(origin)?;

         // Check that the account has a attributed investor role
         let _account = account.clone();
         let exist_account = Roles::<T>::contains_key(&_account);

         ensure!(exist_account == true, Error::<T>::UnregisteredAccount);

         // Checks if has the investor role
         let role_entry = Roles::<T>::get(_account);
         let role = role_entry.get(0).unwrap();
         let mut role_iter = role.roles.iter();
         let exist_investor_role = role_iter.position(|&x| x == INVESTOR_ROLE);

         ensure!(exist_investor_role.is_none() == false, Error::<T>::IncorrectRole);

         ensure!(amount >= T::MinContribution::get(), Error::<T>::ContributionTooSmall);

         let investor = Investor::<T>::new(account.clone());

         let result = investor.add_contribution_fund(amount);

         match result {
            Ok(n)  => { 

               T::Currency::transfer(
                  &who,
                  &TREASURE_PALLET_ID.into_account(),
                  amount,
                  ExistenceRequirement::AllowDeath,
               )?;

               let block_number = <frame_system::Pallet<T>>::block_number();
               Self::deposit_event(Event::Contributed(account.clone(), amount, block_number));
               Ok(().into()) 
            },
            Err(e) => Err(e),
        }

        // Ok(().into())
      }
   
      // A house owner can mint a new house
      #[pallet::weight(T::WeightInfo::mint_house())]
      #[transactional]
      pub fn mint_house(origin: OriginFor<T>, account: AccountIdOf<T>) -> DispatchResultWithPostInfo {
         // Check the inputs
         let who = ensure_signed(origin)?;

         let _account = account.clone();
         let exist_account = Roles::<T>::contains_key(&_account);

         ensure!(exist_account == true, Error::<T>::UnregisteredAccount);

         // Checks if has the house owner role
         let role_entry = Roles::<T>::get(_account);
         let role = role_entry.get(0).unwrap();
         let mut role_iter = role.roles.iter();
         let exist_house_owner_role = role_iter.position(|&x| x == HOUSE_OWNER_ROLE);

         ensure!(exist_house_owner_role.is_none() == false, Error::<T>::IncorrectRole);

         /// TODO Call nft pallet to get ids
         let house_owner = HouseOwnerBis::<T>::new(account.clone());
         let result = house_owner.mint_house();

         match result {
            Ok(n)  => { 

               let block_number = <frame_system::Pallet<T>>::block_number();
               Self::deposit_event(Event::MintedHouse(account.clone(), block_number));
               Ok(().into()) 
            },
            Err(e) => Err(e),
        }
      }

      // Create a proposal from a house
      #[pallet::weight(T::WeightInfo::create_proposal())]
      #[transactional]
      pub fn create_proposal(origin: OriginFor<T>, account_id: AccountIdOf<T>, house_id: StorageIndex, valuation: u32) -> DispatchResultWithPostInfo {
         // Check the inputs
         let who = ensure_signed(origin)?;

         // Check that the account has a attributed investor role
         let _account_id = account_id.clone();
         let exist_account = Roles::<T>::contains_key(&_account_id);

         ensure!(exist_account == true, Error::<T>::UnregisteredAccount);

         // Checks if has the house owner role
         let role_entry = Roles::<T>::get(_account_id);
         let role = role_entry.get(0).unwrap();
         let mut role_iter = role.roles.iter();
         let exist_house_owner_role = role_iter.position(|&x| x == HOUSE_OWNER_ROLE);

         ensure!(exist_house_owner_role.is_none() == false, Error::<T>::IncorrectRole);

         let _house_owner = HouseOwnerBis::<T>::new(account_id.clone());
         let result = _house_owner.create_proposal(house_id, valuation);

         match result {
            Ok(n)  => { 

               let block_number = <frame_system::Pallet<T>>::block_number();
               Self::deposit_event(Event::CreatedProposal(account_id.clone(), block_number));
               Ok(().into()) 
            },
            Err(e) => Err(e),
        }
      }

      // An investissor can add a vote to a proposal
      #[pallet::weight(T::WeightInfo::vote_proposal())]
      #[transactional]
      pub fn vote_proposal(
         origin: OriginFor<T>, 
         account_id: AccountIdOf<T>, 
         house_id: StorageIndex, 
         house_owner_account: AccountIdOf<T>, 
         proposal_id: StorageIndex, 
         status: bool) -> DispatchResultWithPostInfo {

         // Check the inputs
         let who = ensure_signed(origin)?;

         // Check that the account has a attributed investor role
         let _account_id = account_id.clone();
         let exist_account = Roles::<T>::contains_key(&_account_id);

         ensure!(exist_account == true, Error::<T>::UnregisteredAccount);

         // Checks if has the investor role
         let role_entry = Roles::<T>::get(_account_id);
         let role = role_entry.get(0).unwrap();
         let mut role_iter = role.roles.iter();
         let exist_investor_role = role_iter.position(|&x| x == INVESTOR_ROLE);

         ensure!(exist_investor_role.is_none() == false, Error::<T>::IncorrectRole);

         let investor = Investor::<T>::new(account_id.clone());
         let result = investor.vote_proposal(house_id, house_owner_account, proposal_id, status);

         match result {
            Ok(n)  => { 

               let block_number = <frame_system::Pallet<T>>::block_number();
               Self::deposit_event(Event::CreatedVote(account_id.clone(), block_number));
               Ok(().into()) 
            },
            Err(e) => Err(e),
        }
      }
   }
}
