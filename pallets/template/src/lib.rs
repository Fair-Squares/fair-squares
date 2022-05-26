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
pub use pallet_uniques as UNQ;
pub use pallet_nft::{BoundedVecOfUnq, ClassInfoOf, InstanceInfoOf };
pub use scale_info::prelude::string::String;




#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
   pub use super::*;     
   pub const PALLET_ID: PalletId = PalletId(*b"ex/cfund");
   pub const TREASURE_PALLET_ID: PalletId = PalletId(*b"py/trsry");
   
   #[derive(Clone, Encode, Decode,PartialEq, Eq, TypeInfo)]
   #[cfg_attr(feature = "std", derive(Debug))]
   pub enum Accounts{
       INVESTOR,
       SELLER,
       TENANT,
       INVALID,
   }
   



   /// Configure the pallet by specifying the parameters and types on which it depends.
   #[pallet::config]
   pub trait Config: frame_system::Config+NftL::Config+pallet_uniques::Config {
      /// Because this pallet emits events, it depends on the runtime's definition of an event.
      type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
      type Currency: ReservableCurrency<Self::AccountId>;
      type MinContribution: Get<BalanceOf<Self>>;
      type SubmissionDeposit: Get<BalanceOf<Self>>;

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
	pub(super) type InvestorLog<T: Config> = StorageMap<_, Twox64Concat, AccountIdOf<T>, Investor::<T>, OptionQuery>;

   #[pallet::storage]
	pub(super) type HouseSellerLog<T: Config> = StorageMap<_, Twox64Concat, AccountIdOf<T>, HouseSeller::<T>, OptionQuery>;


   #[pallet::storage]
	pub(super) type ContributionsLog<T: Config> = StorageMap<_, Twox64Concat,AccountIdOf<T>,(BlockNumberOf<T>,BalanceOf<T>,Vec<Contribution::<T>>), ValueQuery>;

   #[pallet::storage]
   pub(super) type MintedHouseLog<T:Config> = StorageMap<_, Twox64Concat,HouseIndex,House<T>, ValueQuery>;

   #[pallet::storage]
	pub(super) type ProposalLog<T: Config> = StorageMap<_, Twox64Concat,ProposalIndex,(BlockNumberOf<T>,BalanceOf<T>,House<T>,Bool), ValueQuery>;


   #[pallet::storage]
	/// The total number of contributions that have so far been submitted.
	pub(super) type ContribIndex<T: Config> = StorageValue<_, ContributionIndex, ValueQuery>;

   #[pallet::storage]
	/// The total number of proposals that have so far been submitted.
	pub(super) type HouseInd<T: Config> = StorageValue<_, HouseIndex, ValueQuery>;

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
      SomethingStored(u32, T::AccountId),
      Created( <T as frame_system::Config>::BlockNumber),
      ProposalCreated(<T as frame_system::Config>::BlockNumber),
      HouseMinted(HouseIndex, <T as frame_system::Config>::BlockNumber), 
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
      /// Inexistent account
      NoAccount,
      /// Not a Seller account
      NotSellerAccount,
      /// Amount to high for the fund
      OverFundCapacity,
      /// Asset is not yet minted
      NoAsset
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
               let _acc = Investor::<T>::new(origin);
               Ok(().into())
            },
            Accounts::SELLER => {
               ensure!(HouseSellerLog::<T>::contains_key(&caller)==false,Error::<T>::NoneValue);
               let _acc = HouseSeller::<T>::new(origin);
               Ok(().into())
            },
            Accounts::TENANT => {
               
               let _acc = Tenant::<T>::new(origin);
               Ok(().into())
            },
            _=> Ok(()),
         }
         //Ok(().into())
         
      }
      ///This function is used to mint an asset slot.
      ///besides user ID input, no other information is needed. 
      #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
      pub fn create_asset(origin:OriginFor<T>) -> DispatchResult{
         let creator= ensure_signed(origin.clone())?;

         // Ensure that the caller account is a Seller account
         ensure!(HouseSellerLog::<T>::contains_key(&creator),Error::<T>::NotSellerAccount);
         let seller = HouseSellerLog::<T>::get(creator).unwrap();
         seller.mint_house(origin);
         let idx:HouseIndex = HouseInd::<T>::get();
         let now = <frame_system::Pallet<T>>::block_number();

         Self::deposit_event(Event::HouseMinted(idx,now));

         Ok(().into())

      }

      ///This function create a proposal from an asset previously minted
      #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
      pub fn create_proposal(origin:OriginFor<T>,value: BalanceOf<T>,house_index: u32, metadata:Vec<u8>) -> DispatchResult{
         let creator= ensure_signed(origin.clone())?;

         // Ensure that the caller account is a Seller account
         ensure!(HouseSellerLog::<T>::contains_key(&creator),Error::<T>::NotSellerAccount);

         // Ensure that the House index is registered
         ensure!(MintedHouseLog::<T>::contains_key(&house_index),Error::<T>::NoAsset);

         // Ensure that the seller owns the rights on the indexed house 
         let house = MintedHouseLog::<T>::get(&house_index);
         let howner = house.owners;
         ensure!(howner.contains(&creator), Error::<T>::NoAccount);

         // Ensure that the house value is not to high for the fund --> less than 1/4th
         let total_fund:BalanceOf<T> = Pallet::<T>::pot();
         let total = Self::balance_to_u32_option(total_fund).unwrap()/4;
         let f0 = Self::u32_to_balance_option(total).unwrap();
         ensure!(value<=f0,Error::<T>::OverFundCapacity);

         // Create Proposal
         let seller = HouseSellerLog::<T>::get(creator).unwrap();
         seller.new_proposal(origin,value,house_index,metadata).ok();

         let now = <frame_system::Pallet<T>>::block_number();
         Self::deposit_event(Event::ProposalCreated(now));
         
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

      pub fn fractional_transfer(from:T::AccountId, to:Vec<T::AccountId>,p_index:ProposalIndex)-> DispatchResult{
         //Check that Proposal has been accepted
         let proposal = ProposalLog::<T>::get(p_index.clone());
         ensure!(proposal.clone().3==true,Error::<T>::UnsuccessfulFund);

         let house =  proposal.2;
         let house_index = house.clone().index;
         //Check that sending account is a seller
         ensure!(HouseSellerLog::<T>::contains_key(&from),Error::<T>::NotSellerAccount);
         //Check that this seller has ownership of this house 
         let howner = house
                        .clone()
                        .owners;
         ensure!(howner.contains(&from), Error::<T>::NoAccount);
         //Get nft instance from minted nft storage
         let nft_instance = MintedNftLog::<T>::get(&from,house_index.clone()).unwrap().2.instance;
         let pot: T::AccountId=  TREASURE_PALLET_ID.into_account();
         
         //Transfer nft from Seller to pot
         let class_id:ClassOf<T> = MintedNftLog::<T>::get(&from,house_index.clone()).unwrap().0;
         let instance_id:InstanceOf<T> = MintedNftLog::<T>::get(&from,house_index.clone()).unwrap().1;
         let share:u32 = 100000;
         NftL::Pallet::<T>::do_transfer(class_id,instance_id,from.clone(),pot,share).ok();
         //Remove nft_index from house_seller struct
         let mut seller0 = (HouseSellerLog::<T>::get(&from)).unwrap();
         seller0.nft_index.remove(0);
         let seller = Some(seller0);
         HouseSellerLog::<T>::mutate(&from,|val|{
            *val = seller;
         });

         //Nft share redistribution is done in the function do_transfer of the nft_pallet

         //ToDo

         
         //Remove nft/house index from Seller's assets list 
         //for each owner ID found in the Vec 'to': 
            //Update the list of owners in the house struct
            //Update the new owners/investors nft index 
            //Update the owner and the house share in the mintednft's storage 


         Ok(().into())


      }
      
      /// Remove a contribution from an associated child trie.
      pub fn contribution_kill(who: &T::AccountId) {
         let id = Self::id_from_index();
         who.using_encoded(|b| child::kill(&id, b));
      }

      pub fn u32_to_balance_option(input: u32) -> Option<BalanceOf<T>> {
         input.try_into().ok()
       }
   
      pub fn balance_to_u32_option(input: BalanceOf<T>) -> Option<u32> {
         input.try_into().ok()
       }

       pub fn pot() -> BalanceOf<T> {
			<T as pallet::Config>::Currency::free_balance(&TREASURE_PALLET_ID.into_account())
			// Must never be less than 0 but better be safe.
			.saturating_sub(<T as pallet::Config>::Currency::minimum_balance())
	}
   }
}
