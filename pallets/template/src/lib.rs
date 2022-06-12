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
pub use crate::roles::*;
pub use pallet_nft::pallet as NftL;
pub use pallet_uniques as UNQ;
pub use pallet_democracy as DMC;
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
   pub trait Config: frame_system::Config+NftL::Config+UNQ::Config+DMC::Config{
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
      /// Not an Investor account
      NotInvestorAccount,
      /// Amount to high for the fund
      OverFundCapacity,
      /// Asset is not yet minted
      NoAsset,
      ///Not enough funds available for this purchase
      NotEnoughFunds,
      ///Only One role allowed
      OnlyOneRoleAllowed

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
               ensure!(InvestorLog::<T>::contains_key(&caller)==false,Error::<T>::OnlyOneRoleAllowed);
               ensure!(HouseSellerLog::<T>::contains_key(&caller)==false,Error::<T>::OnlyOneRoleAllowed);
			   ensure!(ServicerLog::<T>::contains_key(&caller)==false,Error::<T>::OnlyOneRoleAllowed);
               ensure!(TenantLog::<T>::contains_key(&caller)==false,Error::<T>::OnlyOneRoleAllowed);
               let _acc = Investor::<T>::new(origin);
               Ok(().into())
            },
            Accounts::SELLER => {
               ensure!(HouseSellerLog::<T>::contains_key(&caller)==false,Error::<T>::OnlyOneRoleAllowed);
               ensure!(InvestorLog::<T>::contains_key(&caller)==false,Error::<T>::OnlyOneRoleAllowed);
			   ensure!(ServicerLog::<T>::contains_key(&caller)==false,Error::<T>::OnlyOneRoleAllowed);
               ensure!(TenantLog::<T>::contains_key(&caller)==false,Error::<T>::OnlyOneRoleAllowed);
               //Bring the decision for this account creation to a vote
               let _acc = HouseSeller::<T>::new(origin);
               Ok(().into())
            },
            Accounts::TENANT => {
				ensure!(HouseSellerLog::<T>::contains_key(&caller)==false,Error::<T>::OnlyOneRoleAllowed);
				ensure!(InvestorLog::<T>::contains_key(&caller)==false,Error::<T>::OnlyOneRoleAllowed);
				ensure!(ServicerLog::<T>::contains_key(&caller)==false,Error::<T>::OnlyOneRoleAllowed);
               ensure!(TenantLog::<T>::contains_key(&caller)==false,Error::<T>::OnlyOneRoleAllowed);
               let _acc = Tenant::<T>::new(origin);
               Ok(().into())
            },
			Accounts::SERVICER => {
				ensure!(HouseSellerLog::<T>::contains_key(&caller)==false,Error::<T>::OnlyOneRoleAllowed);
				ensure!(InvestorLog::<T>::contains_key(&caller)==false,Error::<T>::OnlyOneRoleAllowed);
				ensure!(ServicerLog::<T>::contains_key(&caller)==false,Error::<T>::OnlyOneRoleAllowed);
               ensure!(TenantLog::<T>::contains_key(&caller)==false,Error::<T>::OnlyOneRoleAllowed);
               let _acc = Servicer::<T>::new(origin);
               Ok(().into())
            },
         }
        
         
      }

      #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
      ///Approval function for Sellers and Servicers. Only for admin level.
      pub fn account_approval(origin:OriginFor<T>,account: T::AccountId)-> DispatchResult{
         ensure_root(origin.clone())?;
         Self::approve_account(account);
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
      

      //During Investors vote, Houses linked to an approved proposal are removed from
      //the MintedHouse storage, and the boolean in the corresponding Proposal_storage
      //is turned to true.
      ///Fractional_transfer takes care of nft ownership & share distribution, as well as
      ///update of related storages.
      pub fn fractional_transfer(from:T::AccountId, to:Vec<(T::AccountId,BalanceOf<T>)>,p_index:ProposalIndex)-> DispatchResult{
         //Check that Proposal has been accepted
         let mut proposal = ProposalLog::<T>::get(p_index.clone());
         ensure!(proposal.clone().3==true,Error::<T>::UnsuccessfulFund);

         let house =  proposal.clone().2;
         let house_index = house.clone().index;
         //Check that sending account is a seller
         ensure!(HouseSellerLog::<T>::contains_key(&from),Error::<T>::NotSellerAccount);
         
         //Check that this seller has ownership of this house 
         let howner = house
                        .clone()
                        .owners;
         ensure!(howner.contains(&from), Error::<T>::NoAccount);

         //remove Seller from house owners list
         proposal.2.owners.remove(0);

         //Get nft data from minted nft storage
         let _nft_instance = MintedNftLog::<T>::get(&from,house_index.clone()).unwrap().2.instance;
         let class_id:ClassOf<T> = MintedNftLog::<T>::get(&from,house_index.clone()).unwrap().0;
         let instance_id:InstanceOf<T> = MintedNftLog::<T>::get(&from,house_index.clone()).unwrap().1;
         let mut nft_item = MintedNftLog::<T>::get(&from,house_index.clone()).unwrap();
         MintedNftLog::<T>::remove(&from,house_index.clone());
         
         //Remove nft_index from house_seller struct
         let mut seller0 = (HouseSellerLog::<T>::get(&from)).unwrap();
         seller0.nft_index.remove(0);
         let seller = Some(seller0);
         HouseSellerLog::<T>::mutate(&from,|val|{
            *val = seller;
         });

         //Nft share redistribution is done in the function do_transfer of the nft_pallet
         //Get the house value from the storage
         let value = Self::balance_to_u32_option(proposal.1).unwrap();

         for i in to{
            //Calculate nft share from amount contributed to the house
            let contribution = Self::balance_to_u32_option(i.1).unwrap();
            let share = (contribution*100000)/&value;
            
            //Update minted nft log with new owners
            
            if !(MintedNftLog::<T>::contains_key(i.0.clone(),house_index.clone())){
               nft_item.2.percent_owned = share.clone();
               MintedNftLog::<T>::insert(&i.0,&house_index,nft_item.clone());
            }
            //
            //Redistribute nft share
            NftL::Pallet::<T>::do_transfer(class_id.clone(),instance_id.clone(),from.clone(),i.clone().0,share).ok();
            
           
            //Update the list of owners in the house structs found in ProposalLog_storage & remove house item from minted house
            proposal.2.owners.push(i.0);       
         
         }
         ProposalLog::<T>::mutate(&p_index,|val|{
            *val = proposal;
         });


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

       pub fn u32_to_balance_option2(input: u32) -> Option<BalanceOf2<T>> {
         input.try_into().ok()
       }
   
      pub fn balance_to_u32_option(input: BalanceOf<T>) -> Option<u32> {
         input.try_into().ok()
       }
      
       pub fn balance_to_u32_option2(input: BalanceOf2<T>) -> Option<u32> {
         input.try_into().ok()
       }
      
       pub fn approve_account(who: T::AccountId) {
         let waitlist = WaitingList::<T>::get();
         let sellers =  waitlist.0;
         let servicers = waitlist.1;
         for sell in sellers.iter(){
            if sell.account_id == who.clone(){
               HouseSellerLog::<T>::insert(&who,sell.clone());
               let index = sellers.iter().position(|x| *x == *sell).unwrap();
               WaitingList::<T>::mutate(|val|{
                  val.0.remove(index);
               })
            }
         }
         for serv in servicers.iter(){
            if serv.account_id == who.clone(){
               ServicerLog::<T>::insert(&who,serv);
               let index = servicers.iter().position(|x| *x == *serv).unwrap();
               WaitingList::<T>::mutate(|val|{
                  val.0.remove(index);
               })
            }
         }
       }

       pub fn destroy_proposal(){}

       pub fn mint_house_nft(creator: T::AccountId, hindex:u32,metadata:Vec<u8>){
          //mint a nft with the same index as HouseInd here                       
            //mint            
            let data:BoundedVecOfUnq<T> = metadata.try_into().unwrap();
            let cl_id:ClassOf<T> = HOUSE_CLASS.into();
            let inst_id:InstanceOf<T> = hindex.into();

            let cls = NftL::Pallet::<T>::do_create_class(
                creator.clone(),
                cl_id.clone(),
                Default::default(),
                data.clone()
            ).unwrap();            
            let _nft = NftL::Pallet::<T>::do_mint(
                creator.clone(),
                cls.0,
                inst_id.clone(),
                data
            );
            let hi:InstanceOf<T> = hindex.clone().into();

            let own = NftL::TokenByOwner::<T>::get(creator.clone(),(cls.0,hi)).unwrap();
            if !(MintedNftLog::<T>::contains_key(&creator,&hindex)){
                MintedNftLog::<T>::insert(creator,hindex,(cl_id,inst_id,own));
            } 
       }

       pub fn pot() -> BalanceOf<T> {
			<T as pallet::Config>::Currency::free_balance(&TREASURE_PALLET_ID.into_account())
			// Must never be less than 0 but better be safe.
			.saturating_sub(<T as pallet::Config>::Currency::minimum_balance())
	}
   }
}
