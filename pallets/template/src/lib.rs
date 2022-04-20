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

use pallet_nft::pallet as NftL;



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
      pallet_prelude::*,
      sp_runtime::traits::{AccountIdConversion, Saturating, Hash, Zero},
      storage::child,
      traits::{Currency, ExistenceRequirement, Get, ReservableCurrency},
      PalletId		
   };
   use frame_system::{ensure_signed};
   use frame_support::inherent::Vec;
   use pallet_nft::{BlockNumberOf, ClassData, ClassIdOf, TokenIdOf,Properties,CID,ClassType};
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
   }
	
   pub type HouseIndex = u32;
   pub type OwnerIndex = u32;   
   pub type Owners<T> = Vec<HouseOwner<T>>;
   type AccountIdOf<T> = <T as frame_system::Config>::AccountId;   
   type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
   type Bool = bool;
   
   
//   #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
//   #[scale_info(skip_type_params(T))]
//   #[cfg_attr(feature = "std", derive(Debug))]
//   pub struct ContributionBis<T: Config> {
//      amount: BalanceOf<T>,
//      time: u32
//   }
 //  impl MaxEncodedLen for ContributionBis {
 //     fn max_encoded_len() -> usize {
 //        10000
 //     }
 //  }
   
   //#[derive(Clone, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
   //#[scale_info(skip_type_params(T))]
//   #[cfg_attr(feature = "std", derive(Debug))]
   //pub struct ContributionList<T: Config> {
   //   list: Vec<ContributionBis<T>>
   //}
   //impl<T: Config> MaxEncodedLen for ContributionList<T> {
   //   fn max_encoded_len() -> usize {
   //      10000
   //   }
   //}
   
   


   #[pallet::pallet]
   #[pallet::generate_store(pub(super) trait Store)]
   //#[pallet::without_storage_info]
   pub struct Pallet<T>(_);

   //#[pallet::storage]
   //#[pallet::getter(fn contribributionbis_log)]
   //pub(super) type ContributionBisLog<T> = StorageMap<_, Blake2_128Concat, AccountIdOf<T>, ContributionList::<T>, ValueQuery>;


   // The pallet's runtime storage items.
   // https://docs.substrate.io/v3/runtime/storage
   #[pallet::storage]
   #[pallet::getter(fn something)]
   // Learn more about declaring storage items:
   // https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
   pub type Something<T> = StorageValue<_, u32>;
   

   #[pallet::storage]
   #[pallet::getter(fn contribribution_log)]
   pub type ContributionsLog<T> = StorageMap<_, Blake2_128Concat, AccountIdOf<T>, BalanceOf<T>, ValueQuery>;
   
   #[pallet::storage]
   #[pallet::getter(fn house_log)]
   pub type HousesLog<T> = StorageMap<_, Blake2_128Concat, HouseIndex, House, ValueQuery>;
   
   #[pallet::storage]
   #[pallet::getter(fn house_index)]
   pub type HouseIndexLog<T> = StorageValue<_, HouseIndex, ValueQuery>;
   
   #[pallet::storage]
   #[pallet::getter(fn account_log)]
   pub type AccountsLog<T> = StorageMap<_, Blake2_128Concat, AccountIdOf<T>, u32, ValueQuery>;
   
   #[pallet::storage]
   #[pallet::getter(fn ownership_logs)]
   pub type OwnershipsLogs<T> = StorageDoubleMap<_, Blake2_128Concat, u32, Blake2_128Concat, u32, Ownership, ValueQuery>;
   
   #[pallet::storage]
   #[pallet::getter(fn ownership_log)]
   pub type OwnershipsLog<T> = StorageMap<_, Blake2_128Concat, u32, Ownership, ValueQuery>;
   
   #[pallet::storage]
   #[pallet::getter(fn ownership_index)]
   pub type OwnershipIndexLog<T> = StorageValue<_, u32, ValueQuery>;
   
   #[pallet::storage]
   #[pallet::getter(fn proposal_logs)]
   pub type ProposalsLogs<T> = StorageDoubleMap<_, Blake2_128Concat, (u32, u32), Blake2_128Concat, u32, Proposal, ValueQuery>;
   
   #[pallet::storage]
   #[pallet::getter(fn proposal_log)]
   pub type ProposalsLog<T> = StorageMap<_, Blake2_128Concat, u32, Proposal, ValueQuery>;
   
   #[pallet::storage]
   #[pallet::getter(fn proposal_index)]
   pub type ProposalIndexLog<T> = StorageValue<_, u32, ValueQuery>;
   
   #[pallet::storage]
   #[pallet::getter(fn vote_log)]
   pub type VotesLog<T> = StorageDoubleMap<_, Blake2_128Concat, u32, Blake2_128Concat, u32, Vote, ValueQuery>;
   
   #[pallet::storage]
   #[pallet::getter(fn vote_index)]
   pub type VoteIndexLog<T> = StorageValue<_, u32, ValueQuery>;
   
   #[pallet::storage]
   #[pallet::getter(fn role_log)]
   pub(super) type RolesLog<T> = StorageMap<_, Blake2_128Concat, AccountIdOf<T>, Role, ValueQuery>;
   

   // Pallets use events to inform users when important changes are made.
   // https://docs.substrate.io/v3/runtime/events-and-errors
   #[pallet::event]
   #[pallet::generate_deposit(pub(super) fn deposit_event)]
   pub enum Event<T: Config> {
      /// Event documentation should end with an array that provides descriptive names for event
      /// parameters. [something, who]
      SomethingStored(u32, T::AccountId),
      Created( <T as frame_system::Config>::BlockNumber),
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

	// Update storage
	Self::contribution_kill( &who);
	
	// Raise event
	Self::deposit_event(Event::Withdrew(who, balance, now));
	
	// Exit
	Ok(().into())
      }
      
      /// a house owner mint a house
      #[pallet::weight(10_000)]
      //#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]      
      pub fn mint_house(origin: OriginFor<T>, account: AccountIdOf<T>) -> DispatchResultWithPostInfo {
      
         // Checks the inputs
         let who = ensure_signed(origin)?;
         
         let _account = account.clone();
         let exist_account = AccountsLog::<T>::contains_key(&_account);
         
         ensure!(exist_account == true, Error::<T>::UnregisteredAccount);
         
         let _account_id = AccountsLog::<T>::get(&_account);
            
         // Checks if has the house owner role
         let role = RolesLog::<T>::get(_account);
         let mut role_iter = role.roles.iter();
         let exist_house_owner_role = role_iter.position(|&x| x == HOUSE_OWNER_ROLE);

         ensure!(exist_house_owner_role.is_none() == false, Error::<T>::IncorrectRole);
         
         /// TODO Call nft pallet to get ids
         
         // Create house
         let house = House::new(1, 1);
         let house_index = <HouseIndexLog<T>>::get();
         <HouseIndexLog<T>>::put(house_index + 1);
         
         // Create ownership relation
         let ownership = Ownership::new(_account_id, house_index, 100);
         let ownership_index = <OwnershipIndexLog<T>>::get();
         <OwnershipIndexLog<T>>::put(ownership_index + 1);
         
         // Add house to storage
         <HousesLog<T>>::insert(house_index, house);
         
         // Add ownership to storage
         //<OwnershipsLog<T>>::insert(ownership_index, ownership);
         <OwnershipsLogs<T>>::insert(house_index, _account_id, ownership);
         
         // Raise event
         let now = <frame_system::Pallet<T>>::block_number();
         Self::deposit_event(Event::Created(now));
         
         // Exit
	 Ok(().into())
      }
      
      /// a house owner create a proposal for a house
      #[pallet::weight(10_000)]
      pub fn create_proposal(origin: OriginFor<T>, account: AccountIdOf<T>, house_id: HouseIndex, valuation: u32) -> DispatchResultWithPostInfo {
      
         // Check the inputs
         let who = ensure_signed(origin)?;
         
         let _account = account.clone();
         let exist_account = AccountsLog::<T>::contains_key(&_account);
         
         ensure!(exist_account == true, Error::<T>::UnregisteredAccount);
         
         let _account_id = AccountsLog::<T>::get(&_account);
            
         // Checks if has the house owner role
         let role = RolesLog::<T>::get(_account);
         let mut role_iter = role.roles.iter();
         let exist_house_owner_role = role_iter.position(|&x| x == HOUSE_OWNER_ROLE);

         ensure!(exist_house_owner_role.is_none() == false, Error::<T>::IncorrectRole);
         
         // Check if the house id is correct
         let exist_house = HousesLog::<T>::contains_key(&house_id);
         ensure!(exist_house == true, Error::<T>::InvalidIndex);
         
         // Check if the house is owned by the account
         let ownership_exist = OwnershipsLogs::<T>::contains_key(house_id, _account_id);
         
         ensure!(ownership_exist == true, Error::<T>::NotOwnedHouse);
         
         // Check if there is already a current proposal for this house
         let mut proposal_iter = ProposalsLogs::<T>::iter_prefix_values((house_id, _account_id));
         let exist_active_proposal = proposal_iter.position(|x| x.active == true);
         ensure!(exist_active_proposal.is_none() == false, Error::<T>::AlreadyActiveProposal);
         
         // Execute treatment
         let proposal_index = <ProposalIndexLog<T>>::get();
         <ProposalIndexLog<T>>::put(proposal_index + 1);
         
         let proposal = Proposal::new(house_id, _account_id, valuation);
         <ProposalsLogs<T>>::insert((house_id, _account_id), proposal_index, proposal);         
         
         // Raise event
         let now = <frame_system::Pallet<T>>::block_number();
         Self::deposit_event(Event::Created2(proposal_index, now));
         
         // Exit
	 Ok(().into())
      }
      
      /// a investor vote for a proposal
      #[pallet::weight(10_000)]
      pub fn vote_proposal(origin: OriginFor<T>, account: AccountIdOf<T>, house_id: u32, owner_id: u32, proposal_id: u32, status: bool) -> DispatchResultWithPostInfo {
      
         // Check the inputs
         let who = ensure_signed(origin)?;
         
         let _account = account.clone();
         let exist_account = AccountsLog::<T>::contains_key(&_account);
         
         ensure!(exist_account == true, Error::<T>::UnregisteredAccount);
         
         let _account_id = AccountsLog::<T>::get(&_account);
            
         // Checks if has the investor role
         let role = RolesLog::<T>::get(_account);
         let mut role_iter = role.roles.iter();
         let exist_investor_role = role_iter.position(|&x| x == INVESTOR_ROLE);

         ensure!(exist_investor_role.is_none() == false, Error::<T>::IncorrectRole);
         
         // Check if the proposal exist
         let proposal_exist = ProposalsLogs::<T>::contains_key((house_id, owner_id), proposal_id);
         ensure!(proposal_exist == true, Error::<T>::InvalidIndex);
         
         // Check if the proposal is still active
         let proposal = ProposalsLogs::<T>::get((house_id, owner_id), proposal_id);
         ensure!(proposal.active == true, Error::<T>::ProposalOutDated);
         
         // Check if a vote already exist for this account in this proposal
         ensure!(VotesLog::<T>::contains_key(proposal_id, _account_id) == false, Error::<T>::AlreadyVotedProposal);
         
         // Execute treatment
         let vote_index = <VoteIndexLog<T>>::get();
         <VoteIndexLog<T>>::put(vote_index + 1);
         
         let vote = Vote::new(proposal_id, _account_id, status);
         <VotesLog<T>>::insert(proposal_id, _account_id, vote);
         
         // Raise event
         let now = <frame_system::Pallet<T>>::block_number();
         Self::deposit_event(Event::Created2(vote_index, now));
         
         // Exit
	 Ok(().into())
      }
      
      #[pallet::weight(10_000)]
      pub fn manage_proposal(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
      
         // Check the inputs
         let who = ensure_signed(origin)?;
         // Execute treatment
         // Raise event
         // Exit
	 Ok(().into())
      }
      
      /// Withdraw full balance of a contributor to treasury
      #[pallet::weight(10_000)]
      pub fn withdraw_house_contribution(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
      
         // Check the inputs
         let who = ensure_signed(origin)?;
         // Execute treatment
         // Raise event
         // Exit
	 Ok(().into())
      }
      
      /// a Investor contributes funds to an existing fund
      //#[pallet::weight(10_000)]
      #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
      pub fn contribute_fund(origin: OriginFor<T>, account: AccountIdOf<T>, amount: BalanceOf<T>) -> DispatchResultWithPostInfo {
      
         // Check the inputs
         let who = ensure_signed(origin)?;
         
         let _account = account.clone();
         let exist_account = AccountsLog::<T>::contains_key(&_account);
         
         ensure!(exist_account == true, Error::<T>::UnregisteredAccount);
         
         let _account_id = AccountsLog::<T>::get(&_account);
            
         // Checks if has the investor role
         let role = RolesLog::<T>::get(_account);
         let mut role_iter = role.roles.iter();
         let exist_investor_role = role_iter.position(|&x| x == INVESTOR_ROLE);

         ensure!(exist_investor_role.is_none() == false, Error::<T>::IncorrectRole);
         
         ensure!(amount >= T::MinContribution::get(), Error::<T>::ContributionTooSmall);
         
         // Execute treatment
         let contributor_id = account.clone();
         if ContributionsLog::<T>::contains_key(&contributor_id){
            ContributionsLog::<T>::mutate(&contributor_id, |val|{
                *val += amount;
            })
         } else {
            ContributionsLog::<T>::insert(&contributor_id,amount);
         }
         
         T::Currency::transfer(
            &who,
            &TREASURE_PALLET_ID.into_account(),
            amount,
            ExistenceRequirement::AllowDeath,
         )?;
         
         //let balance = Self::contribution_get(&who);
	 //let balance = balance.saturating_add(value);
	 //Self::contribution_put(&who, &balance);
         
         // Raise event
         let block_number = <frame_system::Pallet<T>>::block_number();
         Self::deposit_event(Event::Contributed(contributor_id, amount, block_number));
         
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
      
      /// Record a contribution in the associated child trie.
      pub fn contribution_put( who: &T::AccountId, balance: &BalanceOf<T>) {
         let id = Self::id_from_index();
         who.using_encoded(|b| child::put(&id, b, &balance));
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
