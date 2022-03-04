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
	
   pub type PropIndex = u32;//Kazu:for proposals	
   type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
   pub type ContIndex<T> = Vec<AccountIdOf<T>>;//Kazu:nbr of contributors
   type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
   //type ProposalInfoOf<T> = Proposal<ContIndex<T>, BalanceOf<T>,ClassId<T>, TokenId<T>,Bool>; //Kazu:for proposals
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
   

   // Pallets use events to inform users when important changes are made.
   // https://docs.substrate.io/v3/runtime/events-and-errors
   #[pallet::event]
   #[pallet::generate_deposit(pub(super) fn deposit_event)]
   pub enum Event<T: Config> {
      /// Event documentation should end with an array that provides descriptive names for event
      /// parameters. [something, who]
      SomethingStored(u32, T::AccountId),
      Created( <T as frame_system::Config>::BlockNumber),
      Created2(PropIndex, <T as frame_system::Config>::BlockNumber), //Kazu:for creation of a proposal
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
      pub fn do_something(origin: OriginFor<T>, something: u32, acc:AccountIdOf<T>,rent:BalanceOf<T>) -> DispatchResult {
         // Check that the extrinsic was signed and get the signer.
         // This function will return an error if the extrinsic is not signed.
         // https://docs.substrate.io/v3/runtime/origins
         let who = ensure_signed(origin)?;
         let dev=Investor::new(&acc,something);
         let _tenant=Tenant::new(&acc,rent);

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
      
      /// Kazu:Create a new Proposal
      #[pallet::weight(10_000)]
      pub fn create_prop(
         origin: OriginFor<T>,
         powner0: AccountIdOf<T>,
         value: BalanceOf<T>,			
//         _cdatas:ClassData<T>, //Kazu: Added ClassData parameter from orml_nft pallet
//         _tdatas:TokenData<T> //Kazu: Added TokenData parameter from orml_nft pallet
      ) -> DispatchResultWithPostInfo {
      
         // Check the inputs
         let creator = ensure_signed(origin)?;
         
         // Execute treatment
         /// TODO : extract execution from following commented code
         
         let now = <frame_system::Pallet<T>>::block_number();
         
//         let creator = ensure_signed(origin)?;
		
//         let deposit = T::SubmissionDeposit::get();
//         let imb = T::Currency::withdraw(
//            &creator,
//            deposit,
//            WithdrawReasons::TRANSFER,
//            ExistenceRequirement::AllowDeath,
//         )?;


//         //Kazu: I need to understand what goes in metadata and data parameters. For now I use a dummy vector for metada, and I create a NFT
//         let vv = vec![3,5];
//         let vv2 = vv.clone();
//         //Kazu:Creating the nftClassId and the tokenId(Minting)
//         let mut powner= <ContIndex<T>>::new();
//         powner.push(powner0);
		
//         let class_id = orml_nft::Pallet::<T>::create_class(&powner[0],vv,Default::default())?;
//         let token_id = orml_nft::Pallet::<T>::mint(&powner[0],class_id,vv2,Default::default())?;
//         let balance:BalanceOf<T> = Zero::zero();
//         let funded:Bool = false;
//         let index = <PropCount<T>>::get();
//         // not protected against overflow, see safemath section
//         <PropCount<T>>::put(index + 1);
//         // No fees are paid here if we need to create this account; that's why we don't just
//         // use the stock `transfer`.
//         //T::Currency::resolve_creating(&TREASURE_PALLET_ID.into_account(), imb);
//         T::Currency::resolve_creating(&Self::fund_account_id(index), imb);

//         //Kazu:Storing the created proposal informations inside the Props storage
//         <Props<T>>::insert(
//            index,
//            Proposal { powner,value,class_id,token_id,balance,funded},
//         );
         
         // Raise event
         Self::deposit_event(Event::Created( now));
         
         // Exit
         Ok(().into())
      }
      
      /// Contribute funds to an existing fund
      #[pallet::weight(10_000)]
      pub fn contribute(
         origin: OriginFor<T>,
         account: T::AccountId,//Kazu:Added this for compatibility with storageMap
         value: BalanceOf<T>,
      ) -> DispatchResultWithPostInfo {
      
         // Check the inputs
         let who = ensure_signed(origin)?;
         ensure!(value >= T::MinContribution::get(), Error::<T>::ContributionTooSmall);
         
         // Execute treatment
         /// TODO : extract execution from following commented code
         let now = <frame_system::Pallet<T>>::block_number();
         
//         //Kazu:creating a new contribution object below with the current infos
//         let c1= self::ContrIb{
//            contribution: value,
//            account: &account,
//         };
         
         //Kazu:if id is already in storage, update storage value by adding the new contribution, or else
         //insert the new Id/contribution
//         if ContStore::<T>::contains_key(c1.account){
//            ContStore::<T>::mutate(c1.account,|val|{
//               *val += c1.contribution;
//            })
//         } else {
//            ContStore::<T>::insert(&account,value);
//            //let mut ve=<ContAcc<T>>::get();
//            ContAcc::<T>::mutate(|val|{
//               val.push(account);
//            })				
//         }

         // Add contribution to the fund
//         T::Currency::transfer(
//            &who,
//            &TREASURE_PALLET_ID.into_account(),
//            value,
//            ExistenceRequirement::AllowDeath,
//         )?;			

//         let balance = Self::contribution_get(&who);
//         let balance = balance.saturating_add(value);
//         Self::contribution_put(&who, &balance);
         
         // Raise event
         let balance = value; // TODO just for the prototype, to remove
         Self::deposit_event(Event::Contributed(who, balance, now));
         
         // Exit
         Ok(().into())
      }
      
      ///Proposal transactions
      #[pallet::weight(10_000)]
      pub fn fund_prop(
      //Kazu: Origin is the account paying for transaction fees.
         _origin: OriginFor<T>,
         index1: PropIndex				
      )-> DispatchResultWithPostInfo{
      
         // Check the inputs TODO
         //let prop = Props::<T>::get(index1);
         //Ensure that proposal has not been funded yet, or return an error 
         //ensure!(prop.funded==false, Error::<T>::AlreadyFunded);
         
         // Execute treatment
         /// TODO : extract execution from following commented code
         
         //Kazu: Pay the proposal owner From Treasurery
	
//         let value = prop.value;
//         let powner = &prop.powner[0];
//         let ben = &TREASURE_PALLET_ID.into_account();

//         T::Currency::transfer(
//            &ben,
//            &powner,
//            value,
//            ExistenceRequirement::AllowDeath,
//         )?;

//         let ve1=<ContAcc<T>>::get();
//         let mut total0 = 0;
//         for i in ve1.iter(){
//            let contrib = Self::contr_ib(i);
//            let contrib1 = TryInto::<u64>::try_into(contrib).ok();
//            let b0= match contrib1{
//               Some(x) => x,
//               None => 0,
//            };
//            total0+=b0
//         }
//         //Determine which contributor is included into the proposal
//         //Creating a vector containing contributor's IDs
//         let ve=<ContAcc<T>>::get();
//         //For each contributor ID
//         for i in ve.iter(){

            //pick-up 1st contribution and convert it from type Balance to u64
//            let contrib = Self::contr_ib(i);
//            let contrib1 = TryInto::<u64>::try_into(contrib).ok();

//           //pick-up proposal value and convert it from type Balance to u64
//            let pr = TryInto::<u64>::try_into(value).ok();
//		let pric= match pr{
//			Some(x) => x,
//			None => 0,
//		};
//		//contrib1 is an enum collection, so we use match to extract the contribution
//		let b0= match contrib1{
//			Some(x) => x,
//			None => 0,
//		};
		

//		//In order to use divisions, we need both values to be floats
//		let price2 = pric as f64;
//		let b00= b0 as f64;
//		let b11= total0 as f64;
//		//contribution percentage calculation 
//		let  per = 100.0*(b00/b11);
//		let newcon=(&per*price2) as u32;
//		let perc = per as u8;

//		let newb=TryInto::<BalanceOf<T>>::try_into(newcon).ok();
//		let b= match newb {
//			Some(x) => x,
//			None => Zero::zero(),
//		};
//		//We need to update the contribution storage
//		ContStore::<T>::mutate(i,|val|{
//			
//			*val-= b;
//		});
//
//			
//		let _class_id = orml_nft::Pallet::<T>::transfer(&powner,&i,(prop.class_id,prop.token_id),perc);
//         }
//
//
//
//	//change owner to new owners and proposal status
//	<Props<T>>::mutate(index1,|val| {
//		let  ve0= <ContAcc<T>>::get();
//		val.powner.pop();
//		for v in ve0.iter(){
//			let l=v.clone();
//			val.powner.push(l);
//		}
//		val.funded=true;
//	});
         
         // Raise event
         // TODO
         
         // Exit
         //distribute NFTs to contributors
	 Ok(().into()) 		
      }
      
      /// Withdraw full balance of a contributor to treasury
      #[pallet::weight(10_000)]
      pub fn withdraw(
         origin: OriginFor<T>,
         #[pallet::compact]index: PropIndex,
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
