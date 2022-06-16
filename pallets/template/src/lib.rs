//! # Template Pallet
//!
//! This pallet manages the complete workflow of the Fairsquares app 

#![cfg_attr(not(feature = "std"), no_std)]


pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;
mod roles;
mod helpers;
pub use crate::roles::*;
pub use pallet_nft::pallet as NftL;
pub use pallet_uniques as UNQ;
pub use pallet_democracy as DMC;
pub use pallet_sudo as SUDO;
pub use pallet_nft::{BoundedVecOfUnq, ClassInfoOf, InstanceInfoOf };
pub use scale_info::prelude::string::String;




#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
   pub use super::*;     
   pub const PALLET_ID: PalletId = PalletId(*b"ex/cfund");
   pub const TREASURE_PALLET_ID: PalletId = PalletId(*b"py/trsry");
   
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
   pub trait Config: frame_system::Config+NftL::Config+UNQ::Config+DMC::Config+SUDO::Config{
      /// Because this pallet emits events, it depends on the runtime's definition of an event.
      type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
      type Currency: ReservableCurrency<Self::AccountId>;
      type MinContribution: Get<BalanceOf<Self>>;
      type SubmissionDeposit: Get<BalanceOf<Self>>;
      type Delay: Get<Self::BlockNumber>;

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
   pub(super) fn MyDefault1<T: Config>() -> Idle<T> { (Vec::new(),Vec::new()) }
	#[pallet::storage]
   ///Registry of Sellers organized by AccountId
	pub(super) type WaitingList<T: Config> = StorageValue<_, Idle<T>, ValueQuery,MyDefault1<T>>;

   #[pallet::storage]
   ///Registry of General Fund contribution's (Creation time,amount,contribution infos), organized by accountId
	pub(super) type ContributionsLog<T: Config> = StorageMap<_, Twox64Concat,AccountIdOf<T>,(BlockNumberOf<T>,BalanceOf<T>,Contribution::<T>), OptionQuery>;
   
   #[pallet::storage]
   ///Registry of minted house's organized by houses indexes
   pub(super) type MintedHouseLog<T:Config> = StorageMap<_, Twox64Concat,HouseIndex,House<T>, ValueQuery>;

   ///Registry of created proposal's (Creation time,value,house,voting status), organized by proposal index
   #[pallet::storage]
	pub(super) type ProposalLog<T: Config> = StorageMap<_, Twox64Concat,ProposalIndex,(BlockNumberOf<T>,BalanceOf<T>,House<T>,Bool), ValueQuery>;


   #[pallet::storage]
	/// The total number of contributions that have so far been submitted.
	pub(super) type ContribIndex<T: Config> = StorageValue<_, ContributionIndex, ValueQuery>;

   #[pallet::storage]
	/// The total number of proposals that have so far been submitted.
	pub(super) type HouseInd<T: Config> = StorageValue<_, HouseIndex, ValueQuery>;

   #[pallet::type_value]
   pub(super) fn MyDefault<T: Config>() -> BalanceOf<T> { Zero::zero() }
   
   #[pallet::storage]
	/// Funds reserved for spending.
	pub(super) type ReserveFunds<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery,MyDefault<T>>;

   #[pallet::storage]
	/// The total number of proposals that have so far been submitted.
	pub(super) type ProposalInd<T: Config> = StorageValue<_, ProposalIndex, ValueQuery>;

   #[pallet::storage]
   pub(super) type MintedNftLog<T:Config> = StorageDoubleMap<_, Twox64Concat, T::AccountId,Twox64Concat,HouseIndex,(ClassOf<T>,InstanceOf<T>,NfT<T>), OptionQuery>;

   






   

   // Pallets use events to inform users when important changes are made.
   // https://docs.substrate.io/v3/runtime/events-and-errors
   #[pallet::event]
   #[pallet::generate_deposit(pub(super) fn deposit_event)]
   pub enum Event<T: Config> {
      /// Event documentation should end with an array that provides descriptive names for event
      /// parameters. [something, who]
      InvestorCreated(T::BlockNumber,T::AccountId),
		TenantCreated(T::BlockNumber,T::AccountId),
		SellerCreated(T::BlockNumber,T::AccountId),
		ServicerCreated(T::BlockNumber,T::AccountId),
		AccountCreationApproved(T::BlockNumber,T::AccountId),
		SellerAccountCreationRejected(T::BlockNumber,T::AccountId),
		ServicerAccountCreationRejected(T::BlockNumber,T::AccountId),
		CreationRequestCreated(T::BlockNumber,T::AccountId),
      Created( T::BlockNumber),
      ProposalCreated(T::BlockNumber),
      HouseMinted(HouseIndex, T::BlockNumber), 
      Contributed(T::AccountId,BalanceOf<T>,T::BlockNumber,),
      Withdrew(T::AccountId,BalanceOf<T>,T::BlockNumber,),
      Retiring(T::BlockNumber),
      Dissolved(T::BlockNumber,T::AccountId,),
      Dispensed(T::BlockNumber,T::AccountId,),
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
      /// Inexistent account
      NoAccount,
      /// Not a Seller account
      NotSellerAccount,
      /// Not an Investor account
      NotInvestorAccount,
      /// Amount to high for the fund
      OverFundCapacity,
      /// Asset is not yet minted
      NoAsset,
      ///Not enough funds available for this purchase
      NotEnoughFunds,
      ///Only One role allowed
      OneRoleAllowed,
      ///Invalid Operation
		InvalidOperation

   }
   

   // Dispatchable functions allows users to interact with the pallet and invoke state changes.
   // These functions materialize as "extrinsics", which are often compared to transactions.
   // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
   #[pallet::call]
   impl<T: Config> Pallet<T> {


      #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
      ///Several kind of accounts can be created
      ///Seller and servicer accounts require approval from an admin
      pub fn create_account(origin:OriginFor<T>, account_type:Accounts) -> DispatchResult{
         let caller = ensure_signed(origin.clone())?; 
         match account_type{
            Accounts::INVESTOR => {
               Self::check_storage(caller.clone())?;
               let _acc = Investor::<T>::new(origin);
               let now = <frame_system::Pallet<T>>::block_number();
			      Self::deposit_event(Event::InvestorCreated(now,caller));
               Ok(().into())
            },
            Accounts::SELLER => {
               Self::check_storage(caller.clone())?;
               //Bring the decision for this account creation to a vote
               let _acc = HouseSeller::<T>::new(origin);
               let now = <frame_system::Pallet<T>>::block_number();
			      Self::deposit_event(Event::CreationRequestCreated(now,caller));
               Ok(().into())
            },
            Accounts::TENANT => {
				   Self::check_storage(caller.clone())?;
               let _acc = Tenant::<T>::new(origin);
               let now = <frame_system::Pallet<T>>::block_number();
			      Self::deposit_event(Event::TenantCreated(now,caller));
               Ok(().into())
            },
		      Accounts::SERVICER => {
				   Self::check_storage(caller.clone())?;
               let _acc = Servicer::<T>::new(origin);
               let now = <frame_system::Pallet<T>>::block_number();
			      Self::deposit_event(Event::CreationRequestCreated(now,caller));
               Ok(().into())
            },
         }
        
         
      }

      #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
      ///Approval function for Sellers and Servicers. Only for admin level.
      pub fn account_approval(origin:OriginFor<T>,account: T::AccountId)-> DispatchResult{
         ensure_root(origin.clone())?;
		   let caller = ensure_signed(origin)?;
		   ensure!(caller.clone()!=account.clone(),Error::<T>::InvalidOperation);
         Self::approve_account(account)?;
		   let now = <frame_system::Pallet<T>>::block_number();
		   Self::deposit_event(Event::AccountCreationApproved(now,caller));
         Ok(().into())

      }

      #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
      ///Creation Refusal function for Sellers and Servicers. Only for admin level.
	  pub fn account_rejection(origin:OriginFor<T>,account: T::AccountId) -> DispatchResult{
		   ensure_root(origin.clone())?;
		   let caller = ensure_signed(origin)?;
		   ensure!(caller.clone()!=account.clone(),Error::<T>::InvalidOperation);
		   Self::reject_account(account)?;
		   Ok(().into())
	  }


      #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
      ///This function is used to contribute to the house fund.
      pub fn contribute(origin:OriginFor<T>,value:BalanceOf<T>) -> DispatchResult{
         let caller = ensure_signed(origin.clone())?;
         ensure!(InvestorLog::<T>::contains_key(&caller),Error::<T>::NotInvestorAccount);
         let investor = InvestorLog::<T>::get(caller).unwrap();
         investor.contribute(origin,value).ok();
         Ok(().into())
      }

      #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
	  ///The caller will transfer his manager authority to a different account
	  pub fn set_manager(origin:OriginFor<T>,new: <T::Lookup as StaticLookup>::Source)->DispatchResult{
		//ensure_signed(origin.clone())?;
		ensure_root(origin.clone())?;
		SUDO::Pallet::<T>::set_key(origin,new).ok();
		Ok(().into())
	}
       
      
      #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
      ///This function create a proposal from an asset previously minted
      ///On creation approval, a proposal is sent to the Democracy system, and a SimpleMajority Referandum is started
      pub fn create_proposal(origin:OriginFor<T>,value: BalanceOf<T>,metadata:Vec<u8>) -> DispatchResult{
         let creator= ensure_signed(origin.clone())?;
         //Make sure that enough funds are available
         let total_fund = Pallet::<T>::pot();
         ensure!(total_fund > Zero::zero(),Error::<T>::OverFundCapacity);
         ensure!(total_fund > value,Error::<T>::OverFundCapacity);        
         let reserve = ReserveFunds::<T>::get();
         let av = reserve+value.clone();
         ensure!(total_fund > av,Error::<T>::OverFundCapacity);
         

         // Ensure that the caller account is a Seller account
         ensure!(HouseSellerLog::<T>::contains_key(&creator),Error::<T>::NotSellerAccount);
         let seller = HouseSellerLog::<T>::get(creator.clone()).unwrap(); 
         //creating a house slot
         seller.mint_house(origin.clone());
         let house_index:HouseIndex = HouseInd::<T>::get();
         let now0 = <frame_system::Pallet<T>>::block_number();
         let pindex = ProposalInd::<T>::get()+1;

         Self::deposit_event(Event::HouseMinted(house_index.clone(),now0));

         // Ensure that the House index is registered
         ensure!(MintedHouseLog::<T>::contains_key(house_index.clone()),Error::<T>::NoAsset);
         
         // Ensure that the seller owns the rights on the indexed house 
         let house = MintedHouseLog::<T>::get(house_index.clone());
         let howner = house.clone().owners;
         ensure!(howner.contains(&creator), Error::<T>::NoAccount);


         // Create Proposal         
         seller.new_proposal(origin.clone(),value.clone(),house_index.clone(),metadata.clone())?;
         //Submit preimage for running proposal
         DMC::Pallet::<T>::note_preimage(origin,metadata.clone())?;
         //Mint the proposal nft 
         Self::mint_house_nft(creator,house_index,metadata);

         //Update the storages 
         let now = <frame_system::Pallet<T>>::block_number();
         let store = (now.clone(),value,house,false);
         ProposalInd::<T>::put(pindex.clone());
         ProposalLog::<T>::insert(pindex,store);
         Self::deposit_event(Event::ProposalCreated(now));
         
         Ok(().into())

      }

      #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
      ///This function allows the user to cast his votes for the active referandums.
      ///The vote configuration selected is 'Standard'. 
      pub fn proposal_vote(origin:OriginFor<T>,ref_index:DMC::ReferendumIndex, vote:DMC::Vote)-> DispatchResult{
         let caller = ensure_signed(origin.clone())?;
         ensure!(InvestorLog::<T>::contains_key(&caller),Error::<T>::NotInvestorAccount);
         let bal= Self::u32_to_balance_option2(500).unwrap();
         let vtype = DMC::AccountVote::Standard{ vote: vote, balance: bal };
         DMC::Pallet::<T>::vote(origin,ref_index,vtype).ok();

         Ok(().into())
      }

      //ToDo
      //Periodically check the vote results, and update necessary storages accordingly
     
    
   
   }
   
   
}
