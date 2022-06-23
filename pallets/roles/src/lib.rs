#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;


#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod structs;


pub use crate::structs::*;
pub use pallet_sudo as SUDO;
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
	pub trait Config: frame_system::Config+SUDO::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: ReservableCurrency<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);


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
   ///Waiting list for Sellers and Servicers
	pub(super) type WaitingList<T: Config> = StorageValue<_, Idle<T>, ValueQuery,MyDefault<T>>;




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
		AccountCreationRejected(T::BlockNumber,T::AccountId),
		SellerAccountCreationRejected(T::BlockNumber,T::AccountId),
		ServicerAccountCreationRejected(T::BlockNumber,T::AccountId),
		CreationRequestCreated(T::BlockNumber,T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		///One role is allowed
		OneRoleAllowed,
		///Invalid Operation
		InvalidOperation,
		///Require Sudo
		RequireSudo

	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {


		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		///Account creation function. Only one role per account is permitted.
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
         Self::approve_account(account.clone())?;
		 let now = <frame_system::Pallet<T>>::block_number();
		 Self::deposit_event(Event::AccountCreationApproved(now,account));
         Ok(().into())

      }


	  #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
      ///Creation Refusal function for Sellers and Servicers. Only for admin level.
	  pub fn account_rejection(origin:OriginFor<T>,account: T::AccountId) -> DispatchResult{
		ensure_root(origin.clone())?;
		Self::reject_account(account.clone())?;
		let now = <frame_system::Pallet<T>>::block_number();
		Self::deposit_event(Event::AccountCreationRejected(now,account));
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




	}



	impl<T: Config> Pallet<T> {
		//Helper function for account creation approval by admin only
		pub fn approve_account(who: T::AccountId) -> DispatchResult{
			let waitlist = WaitingList::<T>::get();
			let sellers =  waitlist.0;
			let servicers = waitlist.1;
			for sell in sellers.iter(){
			   if sell.account_id == who.clone(){
				  HouseSellerLog::<T>::insert(&who,sell.clone());
				  let index = sellers.iter().position(|x| *x == *sell).unwrap();
				  WaitingList::<T>::mutate(|val|{
					 val.0.remove(index);
				  });
				  let now = <frame_system::Pallet<T>>::block_number();
				  Self::deposit_event(Event::SellerCreated(now,who.clone()));
			   }
			}
			for serv in servicers.iter(){
			   if serv.account_id == who.clone(){
				  ServicerLog::<T>::insert(&who,serv);
				  let index = servicers.iter().position(|x| *x == *serv).unwrap();
				  WaitingList::<T>::mutate(|val|{
					 val.1.remove(index);
				  });
				  let now = <frame_system::Pallet<T>>::block_number();
				  Self::deposit_event(Event::ServicerCreated(now,who.clone()));
			   }
			}
			Ok(().into())

		  }

		pub fn check_storage(caller:T::AccountId) -> DispatchResult{
			ensure!(HouseSellerLog::<T>::contains_key(&caller)==false,Error::<T>::OneRoleAllowed);
            ensure!(InvestorLog::<T>::contains_key(&caller)==false,Error::<T>::OneRoleAllowed);
			ensure!(ServicerLog::<T>::contains_key(&caller)==false,Error::<T>::OneRoleAllowed);
            ensure!(TenantLog::<T>::contains_key(&caller)==false,Error::<T>::OneRoleAllowed);
			Ok(().into())
		}

		//Helper function for account creation rejection by admin only
		pub fn reject_account(who: T::AccountId)-> DispatchResult{
			let waitlist = WaitingList::<T>::get();
			let sellers =  waitlist.0;
			let servicers = waitlist.1;
			for sell in sellers.iter(){
				if sell.account_id == who.clone(){
				   let index = sellers.iter().position(|x| *x == *sell).unwrap();
				   WaitingList::<T>::mutate(|val|{
					  val.0.remove(index);
				   });
				   let now = <frame_system::Pallet<T>>::block_number();
				   Self::deposit_event(Event::SellerAccountCreationRejected(now,who.clone()));
				}
			 }

			 for serv in servicers.iter(){
				if serv.account_id == who.clone(){
				   let index = servicers.iter().position(|x| *x == *serv).unwrap();
				   WaitingList::<T>::mutate(|val|{
					  val.1.remove(index);
				   });
				   let now = <frame_system::Pallet<T>>::block_number();
				   Self::deposit_event(Event::ServicerAccountCreationRejected(now,who.clone()));
				}
			 }
			 Ok(().into())
		}


	}

}
