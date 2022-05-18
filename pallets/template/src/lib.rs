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

   #[pallet::storage]
   #[pallet::getter(fn fund_amount)]
   // The total amount deposited by investors
   pub(super) type FundAmount<T> = StorageValue<
      _, 
      BalanceOf<T>>;

   #[pallet::storage]
   #[pallet::getter(fn contributions)]
   // Distribution of investor's contributions
   pub(super) type Contributions<T> = StorageMap<
      _, 
      Blake2_128Concat, 
      AccountIdOf<T>, 
      (BalanceOf<T>, u32, AccountIdOf<T>), 
      OptionQuery
      >;
   
   #[pallet::storage]
   #[pallet::getter(fn contribution_log)]
   // Investor's contribution history storage
   pub type ContributionLog<T> = StorageMap<
      _, 
      Blake2_128Concat, 
      AccountIdOf<T>, 
      Vec<Contribution<T>>, 
      ValueQuery
      >;
   
   #[pallet::storage]
   #[pallet::getter(fn investors)]
   pub(super) type Investors<T> = StorageMap<
      _, 
      Blake2_128Concat, 
      AccountIdOf<T>, 
      Investor<T>, 
      OptionQuery
      >;

   #[pallet::storage]
   #[pallet::getter(fn house_owners)]
   pub(super) type HouseOwners<T> = StorageMap<
      _, 
      Blake2_128Concat, 
      AccountIdOf<T>, 
      HouseOwner<T>, 
      OptionQuery
      >;

   #[pallet::storage]
   #[pallet::getter(fn tenants)]
   pub(super) type Tenants<T> = StorageMap<
      _, 
      Blake2_128Concat, 
      AccountIdOf<T>, 
      Tenant<T>, 
      OptionQuery
      >;

   #[pallet::storage]
   #[pallet::getter(fn ownership_index)]
   // Count of ownerships created until then
   pub(super) type OwnershipIndex<T> = StorageValue<_, StorageIndex, ValueQuery>;
   
   #[pallet::storage]
   #[pallet::getter(fn ownerships)]
   // Links between a house and owners
   pub(super) type Ownerships<T> = StorageMap<
      _, 
      Blake2_128Concat, 
      StorageIndex, 
      Ownership<T>, 
      OptionQuery
      >;
   
   #[pallet::storage]
   #[pallet::getter(fn house_index)]
   // Count of total house minted
   pub(super) type HouseIndex<T> = StorageValue<_, StorageIndex, ValueQuery>;

   #[pallet::storage]
   #[pallet::getter(fn mintedhouses)]
   // Houses minted and not owned by investors
   pub(super) type MintedHouses<T> = StorageMap<
      _, 
      Blake2_128Concat, 
      StorageIndex, 
      HouseMinted::<T,NftIndex>, 
      OptionQuery
      >;

   #[pallet::storage]
   #[pallet::getter(fn fs_houses)]
   // Houses owned by investors
   pub(super) type FSHouses<T> = StorageMap<
      _, 
      Blake2_128Concat, 
      StorageIndex, 
      HouseMinted::<T,NftIndex>, 
      OptionQuery
      >;

   #[pallet::storage]
   #[pallet::getter(fn proposal_index)]
   // Count of proposals
   pub type ProposalIndex<T> = StorageValue<_, StorageIndex, ValueQuery>;

   #[pallet::storage]
   #[pallet::getter(fn proposals)]
   // Proposal created for houses
   pub(super) type Proposals<T> = StorageMap<
      _, 
      Blake2_128Concat, 
      StorageIndex, 
      Proposal<T>, 
      OptionQuery
      >;
   
   #[pallet::storage]
   #[pallet::getter(fn vote_index)]
   // Count of votes
   pub type VoteIndex<T> = StorageValue<_, StorageIndex, ValueQuery>;

   #[pallet::storage]
   #[pallet::getter(fn votes)]
   // Votes of investors for proposals
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
      /// A house was minted
      MintedHouse(<T as frame_system::Config>::AccountId, <T as frame_system::Config>::BlockNumber),
      /// A proposal was created
      CreatedProposal(<T as frame_system::Config>::AccountId, <T as frame_system::Config>::BlockNumber),
      /// A vote was created
      CreatedVote(<T as frame_system::Config>::AccountId, <T as frame_system::Config>::BlockNumber),
      /// A house ownership distribution between investors was realized
      RealizedDistribution(<T as frame_system::Config>::AccountId, <T as frame_system::Config>::BlockNumber),
      Created2(u32, <T as frame_system::Config>::BlockNumber), //Kazu:for creation of a proposal
      /// A contribution was deposited
      Contributed(
         <T as frame_system::Config>::AccountId,
         BalanceOf<T>,
         <T as frame_system::Config>::BlockNumber,
      ),
      /// A contribution was withdrawn
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
      AccountCreated(<T as frame_system::Config>::AccountId, Role)
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
      ProposalOutDated,
      // The proposal valuation exceed fund limit
      ProposalExceedFundLimit
   }
   

   // Dispatchable functions allows users to interact with the pallet and invoke state changes.
   // These functions materialize as "extrinsics", which are often compared to transactions.
   // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
   #[pallet::call]
   impl<T: Config> Pallet<T> {
      
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

      #[pallet::weight(T::WeightInfo::create_account())]
      pub fn create_investor_account(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
         let who = ensure_signed(origin)?;
         if Self::create_account(who.clone(), Role::INVESTOR) {
            Self::deposit_event(Event::AccountCreated(who.clone(), Role::INVESTOR));
         }
         Ok(().into())
      }

      #[pallet::weight(T::WeightInfo::create_account())]
      pub fn create_houseowner_account(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
         let who = ensure_signed(origin)?;
         if Self::create_account(who.clone(), Role::HOUSE_OWNER) {
            Self::deposit_event(Event::AccountCreated(who.clone(), Role::HOUSE_OWNER));
         }
         Ok(().into())
      }

      #[pallet::weight(T::WeightInfo::create_account())]
      pub fn create_tenant_account(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
         let who = ensure_signed(origin)?;
         if Self::create_account(who.clone(), Role::TENANT) {
            Self::deposit_event(Event::AccountCreated(who.clone(), Role::TENANT));            
         }
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
      pub fn add_contribution_fund(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResultWithPostInfo {
         // Check the inputs
         let who = ensure_signed(origin)?;

         ensure!(amount >= T::MinContribution::get(), Error::<T>::ContributionTooSmall);

         Self::set_roles(who.clone(), Role::INVESTOR);

         let wrap_investor = Investors::<T>::get(who.clone());
         ensure!(wrap_investor.is_none() == false, Error::<T>::IncorrectRole);

         let investor = wrap_investor.unwrap();
         
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
               Self::deposit_event(Event::Contributed(who.clone(), amount, block_number));
               Ok(().into()) 
            },
            Err(e) => Err(e),
        }
      }
   
      // A house owner can mint a new house
      #[pallet::weight(T::WeightInfo::mint_house())]
      #[transactional]
      pub fn mint_house(origin: OriginFor<T>, name: Vec<u8>) -> DispatchResultWithPostInfo {
         // Check the inputs
         let who = ensure_signed(origin)?;

         Self::set_roles(who.clone(), Role::HOUSE_OWNER);

         let wrap_house_owner = HouseOwners::<T>::get(who.clone());
         ensure!(wrap_house_owner.is_none() == false, Error::<T>::IncorrectRole);

         let house_owner = wrap_house_owner.unwrap();

         let result = house_owner.mint_house(name.clone());

         match result {
            Ok(n)  => { 

               let block_number = <frame_system::Pallet<T>>::block_number();
               Self::deposit_event(Event::MintedHouse(who.clone(), block_number));
               Ok(().into()) 
            },
            Err(e) => Err(e),
        }
      }

      // Create a proposal from a house
      #[pallet::weight(T::WeightInfo::create_proposal())]
      #[transactional]
      pub fn create_proposal(origin: OriginFor<T>, house_id: StorageIndex, valuation: BalanceOf<T>) -> DispatchResultWithPostInfo {
         // Check the inputs
         let who = ensure_signed(origin)?;

         Self::set_roles(who.clone(), Role::HOUSE_OWNER);

         let wrap_house_owner = HouseOwners::<T>::get(who.clone());
         ensure!(wrap_house_owner.is_none() == false, Error::<T>::IncorrectRole);

         let house_owner = wrap_house_owner.unwrap();

         let result = house_owner.create_proposal(house_id.clone(), valuation.clone());

         match result {
            Ok(n)  => { 

               let block_number = <frame_system::Pallet<T>>::block_number();
               Self::deposit_event(Event::CreatedProposal(who.clone(), block_number));
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
         proposal_id: StorageIndex, 
         status: bool) -> DispatchResultWithPostInfo {

         // Check the inputs
         let who = ensure_signed(origin)?;

         Self::set_roles(who.clone(), Role::INVESTOR);

         let wrap_investor = Investors::<T>::get(who.clone());
         ensure!(wrap_investor.is_none() == false, Error::<T>::IncorrectRole);

         let investor = wrap_investor.unwrap();

         let result = investor.vote_proposal(proposal_id.clone(), status.clone());

         match result {
            Ok(n)  => { 

               let block_number = <frame_system::Pallet<T>>::block_number();
               Self::deposit_event(Event::CreatedVote(who.clone(), block_number));
               Ok(().into()) 
            },
            Err(e) => Err(e),
        }
      }
   }
}
