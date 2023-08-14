#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
mod functions;
mod types;
pub use crate::types::*;
pub use functions::*;
pub use pallet_sudo as SUDO;
pub use pallet_collective as Coll;
use Coll::Instance2;
use sp_std::prelude::*;

#[frame_support::pallet]
pub mod pallet {
	pub use super::*;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: 
	frame_system::Config 
	+ SUDO::Config 
	+ Coll::Config<Instance2>{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Currency: ReservableCurrency<Self::AccountId>;
		type RuntimeCall: Parameter
			+ UnfilteredDispatchable<RuntimeOrigin = <Self as frame_system::Config>::RuntimeOrigin>
			+ From<Call<Self>>
			+ GetDispatchInfo;
		#[pallet::constant]
		type MaxMembers: Get<u32>;

		/// The maximum number of named reserves that can exist on an account.
		#[pallet::constant]
		type MaxRoles: Get<u32>;

		
		#[pallet::constant]
		type CheckPeriod: Get<BlockNumberFor<Self>>;

		
		type BackgroundCouncilOrigin: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::storage]
	#[pallet::getter(fn investors)]
	///Registry of Investors organized by AccountId
	pub(super) type InvestorLog<T: Config> =
		StorageMap<_, Twox64Concat, AccountIdOf<T>, Investor<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn sellers)]
	///Registry of Sellers organized by AccountId
	pub(super) type HouseSellerLog<T: Config> =
		StorageMap<_, Twox64Concat, AccountIdOf<T>, HouseSeller<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn notaries)]
	pub(super) type NotaryLog<T: Config> =
		StorageMap<_, Twox64Concat, AccountIdOf<T>, Notary<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn reps)]
	///Registry of Sellers organized by AccountId
	pub type RepresentativeLog<T: Config> =
		StorageMap<_, Twox64Concat, AccountIdOf<T>, Representative<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn tenants)]
	///Registry of Tenants organized by AccountId
	pub type TenantLog<T: Config> =
		StorageMap<_, Twox64Concat, AccountIdOf<T>, Tenant<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn servicers)]
	///Registry of Servicers organized by AccountId
	pub(super) type ServicerLog<T: Config> =
		StorageMap<_, Twox64Concat, AccountIdOf<T>, Servicer<T>, OptionQuery>;

	#[pallet::type_value]
	/// Initializer for the approval list of house sellers
	pub(super) fn InitPendingSellerList<T: Config>() -> Vec<HouseSeller<T>> {
		Vec::new()
	}

	#[pallet::type_value]
	/// Initializer for the approval list of servicers
	pub(super) fn InitPendingServicerList<T: Config>() -> Vec<Servicer<T>> {
		Vec::new()
	}

	#[pallet::type_value]
	/// Initializer for the approval list of notaries
	pub(super) fn InitPendingNotaryList<T: Config>() -> Vec<Notary<T>> {
		Vec::new()
	}

	#[pallet::type_value]
	/// Initializer for the approval list of representatives
	pub(super) fn InitRepApprovalList<T: Config>() -> Vec<Representative<T>> {
		Vec::new()
	}

	#[pallet::storage]
	#[pallet::getter(fn get_pending_house_sellers)]
	pub(super) type SellerApprovalList<T: Config> =
		StorageValue<_, Vec<HouseSeller<T>>, ValueQuery, InitPendingSellerList<T>>;

	#[pallet::storage]
	#[pallet::getter(fn get_pending_servicers)]
	pub(super) type ServicerApprovalList<T: Config> =
		StorageValue<_, Vec<Servicer<T>>, ValueQuery, InitPendingServicerList<T>>;

	#[pallet::storage]
	#[pallet::getter(fn get_pending_notaries)]
	pub(super) type NotaryApprovalList<T: Config> =
		StorageValue<_, Vec<Notary<T>>, ValueQuery, InitPendingNotaryList<T>>;

	#[pallet::storage]
	#[pallet::getter(fn get_pending_representatives)]
	///Approval waiting list for Representatives
	pub type RepApprovalList<T: Config> =
		StorageMap<_, Twox64Concat, AccountIdOf<T>, Representative<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_roles)]
	///Registry of Roles by AccountId
	pub type AccountsRolesLog<T: Config> =
		StorageMap<_, Twox64Concat, AccountIdOf<T>, BoundedVec<Accounts,T::MaxRoles>, ValueQuery>;

	//This storage should not be necessary, as we already have approval waiting list
	#[pallet::storage]
	#[pallet::getter(fn get_requested_role)]
	pub type RequestedRoles<T: Config> =
		StorageMap<_, Twox64Concat, AccountIdOf<T>, Proposal<T>, OptionQuery>;

	#[pallet::type_value]
	///Initializing function for the total number of members
	pub(super) fn InitTotalMembers<T: Config>() -> u32 {
		0
	}

	#[pallet::storage]
	#[pallet::getter(fn total_members)]
	pub(super) type TotalMembers<T> = StorageValue<_, u32, ValueQuery, InitTotalMembers<T>>;

	#[pallet::type_value]
	///Initializing function for the total number of Rep members
	pub fn InitRepMembers<T: Config>() -> u32 {
		0
	}

	#[pallet::storage]
	#[pallet::getter(fn rep_num)]
	///Number of active Representative
	pub type RepNumber<T: Config> = StorageValue<_, u32, ValueQuery, InitRepMembers<T>>;

	#[derive(frame_support::DefaultNoBound)]
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub new_admin: Option<T::AccountId>,
		pub representatives: Vec<T::AccountId>,
	}
	//#[cfg(feature = "std")]
	//impl<T: Config> Default for GenesisConfig<T> {
	//	fn default() -> Self {
	//		Self { new_admin: Default::default(), representatives: vec![] }
	//	}
	//}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			if self.new_admin.is_some() {
				let servicer0 = self.new_admin.clone().unwrap(); // AccountId
				let origin = <T as frame_system::Config>::RuntimeOrigin::from(RawOrigin::Signed(servicer0.clone())); //Origin
				let source = T::Lookup::unlookup(servicer0); //Source
				crate::Pallet::<T>::set_manager(origin, source).ok();
			}
		}
	}


	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored { something: u32, who: T::AccountId },
		/// Investor role successfully attributed
		InvestorCreated(BlockNumberOf<T>, T::AccountId),
		/// Tenant role successfully attributed
		TenantCreated(BlockNumberOf<T>, T::AccountId),
		/// Seller role successfully attributed
		SellerCreated(BlockNumberOf<T>, T::AccountId),
		/// Servicer role successfully attributed
		ServicerCreated(BlockNumberOf<T>, T::AccountId),
		/// Notary role successfully attributed
		NotaryCreated(BlockNumberOf<T>, T::AccountId),
		/// Request for new role accepted
		AccountCreationApproved(BlockNumberOf<T>, T::AccountId),
		/// Request for new role Rejected
		AccountCreationRejected(BlockNumberOf<T>, T::AccountId),
		/// Seller role request rejected
		SellerAccountCreationRejected(BlockNumberOf<T>, T::AccountId),
		/// Servicer role request rejected
		ServicerAccountCreationRejected(BlockNumberOf<T>, T::AccountId),
		/// Notary role request rejected
		NotaryAccountCreationRejected(BlockNumberOf<T>, T::AccountId),
		/// Role request added to the role approval waiting list
		CreationRequestCreated(BlockNumberOf<T>, T::AccountId),
		/// A proposal has been added by a Background Council member
		BackgroundCouncilAddedProposal{for_who: T::AccountId, proposal_index: u32, when: BlockNumberOf<T>},
		/// A proposal has been closed by a Background Council member
		BackgroundCouncilSessionClosed{who: T::AccountId, proposal_index: u32, when: BlockNumberOf<T>},
		/// A member of the Background Council has voted
		BackgroundCouncilVoted{who: T::AccountId, proposal_index: u32, when: BlockNumberOf<T>},

	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,

		/// Error on initialization.
		InitializationError,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		///One role is allowed
		RoleAlreadyGranted,
		///Invalid Operation
		InvalidOperation,
		///Require Sudo
		RequireSudo,
		///Account is not in waiting list
		NotInWaitingList,
		/// Account already in the waiting list
		AlreadyWaiting,
		///Maximum limit for number of members exceeded
		TotalMembersExceeded,
		/// Action reserved to servicers
		OnlyForServicers,
		/// Cannot do the approval or rejection
		UnAuthorized,
		/// This is not the accont of a council member
		NotACouncilMember,
		/// This proposal does not exists
		ProposalDoesNotExist,
		/// Maximum number of roles Exceeded
		MaximumRolesExceeded


	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(n: BlockNumberOf<T>) -> Weight {
			Self::begin_block(n)
		}
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored { something, who });
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
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
					Ok(().into())
				},
			}
		}
		//--------------------------------------------------------------------------------------------------//

		///Account creation function. Only one role per account is permitted.
		#[pallet::call_index(2)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn set_role(
			origin: OriginFor<T>,
			account: AccountIdOf<T>,
			account_type: Accounts,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			if caller != account {
				ensure!(ServicerLog::<T>::contains_key(&caller), Error::<T>::OnlyForServicers);
			}
			Self::check_account_role(account.clone())?;
			let now = <frame_system::Pallet<T>>::block_number();
			let requested = Self::get_requested_role(&account).is_some();
			match account_type {
				Accounts::INVESTOR => {				
					
					ensure!(!InvestorLog::<T>::contains_key(&caller), Error::<T>::RoleAlreadyGranted);	
					Ok(Investor::<T>::new(account.clone())).map_err(|_:Error<T>| <Error<T>>::InitializationError)?;
					let val0 = Self::get_roles(&account);
					let size = <T as Config>::MaxRoles::get() as usize;
					ensure!(val0.len() < size, Error::<T>::MaximumRolesExceeded);
					if !AccountsRolesLog::<T>::contains_key(account.clone()){
						Self::increase_total_members().ok();
					}
					AccountsRolesLog::<T>::mutate(&account,|val|{
						val.try_push(Accounts::INVESTOR).ok();
					});			
					Self::deposit_event(Event::InvestorCreated(now, account.clone()));
				},
				Accounts::SELLER => {
					ensure!(!requested, <Error<T>>::AlreadyWaiting);
					ensure!(!HouseSellerLog::<T>::contains_key(&caller), Error::<T>::RoleAlreadyGranted);
					let val0 = Self::get_roles(&account);
					let size = <T as Config>::MaxRoles::get() as usize;
					ensure!(val0.len() < size, Error::<T>::MaximumRolesExceeded);					
					Ok(HouseSeller::<T>::new(account.clone())).map_err(|_:Error<T>| <Error<T>>::InitializationError)?;
					Self::deposit_event(Event::CreationRequestCreated(now, account.clone()));
				},
				Accounts::TENANT => {
					ensure!(!TenantLog::<T>::contains_key(&caller), Error::<T>::RoleAlreadyGranted);
					Ok(Tenant::<T>::new(account.clone())).map_err(|_:Error<T>| <Error<T>>::InitializationError)?;
					let val0 = Self::get_roles(&account);
					let size = <T as Config>::MaxRoles::get() as usize;
					ensure!(val0.len() < size, Error::<T>::MaximumRolesExceeded);
					if !AccountsRolesLog::<T>::contains_key(account.clone()){
						Self::increase_total_members().ok();
					}
					AccountsRolesLog::<T>::mutate(&account,|val|{
						val.try_push(Accounts::TENANT).ok();
					});
					Self::deposit_event(Event::TenantCreated(now, account.clone()));
				},
				Accounts::SERVICER => {
					ensure!(!requested, <Error<T>>::AlreadyWaiting);
					ensure!(!ServicerLog::<T>::contains_key(&caller), Error::<T>::RoleAlreadyGranted);
					Ok(Servicer::<T>::new(account.clone())).map_err(|_:Error<T>| <Error<T>>::InitializationError)?;
					let val0 = Self::get_roles(&account);
					let size = <T as Config>::MaxRoles::get() as usize;
					ensure!(val0.len() < size, Error::<T>::MaximumRolesExceeded);
					Self::deposit_event(Event::CreationRequestCreated(now, account.clone()));
				},
				Accounts::NOTARY => {
					ensure!(!requested, <Error<T>>::AlreadyWaiting);
					ensure!(!NotaryLog::<T>::contains_key(&caller), Error::<T>::RoleAlreadyGranted);
					
					let val0 = Self::get_roles(&account);
					let size = <T as Config>::MaxRoles::get() as usize;
					ensure!(val0.len() < size, Error::<T>::MaximumRolesExceeded);
					let notary = <T as frame_system::Config>::RuntimeOrigin::from(RawOrigin::Signed(
						account.clone(),
					));
					Ok(Notary::<T>::new(notary)).map_err(|_:Error<T>| <Error<T>>::InitializationError)?;
					Self::deposit_event(Event::CreationRequestCreated(now, account.clone()));
				},
				Accounts::REPRESENTATIVE => {
					ensure!(!requested, <Error<T>>::AlreadyWaiting);
					ensure!(!RepresentativeLog::<T>::contains_key(&caller), Error::<T>::RoleAlreadyGranted);
					
					let val0 = Self::get_roles(&account);
					let size = <T as Config>::MaxRoles::get() as usize;
					ensure!(val0.len() < size, Error::<T>::MaximumRolesExceeded);
					Ok(Representative::<T>::new(account.clone()))
						.map_err(|_:Error<T>| <Error<T>>::InitializationError)?;
					Self::deposit_event(Event::CreationRequestCreated(now, account.clone()));
				},
			}

			let need_approval = !matches!(
				account_type,
				Accounts::INVESTOR | Accounts::TENANT | Accounts::REPRESENTATIVE
			);
			if need_approval {
				Self::start_council_session(account.clone(),account_type).ok();	
			
			// deposit event
			let index:u32 = Coll::Pallet::<T,Instance2>::proposal_count();
			Self::deposit_event(Event::BackgroundCouncilAddedProposal{
				for_who: account,
				proposal_index: index-1,
				when: now,
			});						
				
			} 
			Ok(())
		}

		///Approval function for Sellers, Servicers, and Notary. Only for admin level.
		#[pallet::call_index(3)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn account_approval(origin: OriginFor<T>, account: T::AccountId) -> DispatchResultWithPostInfo {
			let _sender = T::BackgroundCouncilOrigin::ensure_origin(origin.clone())?;

			let role = Self::get_requested_role(&account).unwrap().role;
			ensure!(role.is_some(), Error::<T>::NotInWaitingList);

			ensure!(role != Some(Accounts::REPRESENTATIVE), Error::<T>::UnAuthorized);

			let result = Self::approve_account(account.clone());
			match result{
				Ok(_) => {
					let now = <frame_system::Pallet<T>>::block_number();
					// deposit event
					Self::deposit_event(Event::AccountCreationApproved(now, account.clone()));
					RequestedRoles::<T>::mutate(&account,|val|{
						let mut proposal = val.clone().unwrap();
						proposal.approved = true;
						*val = Some(proposal);
						});

					},
				Err(e) => return Err(e),
			}
			
			Ok(().into())
		}

		///Creation Refusal function for Sellers and Servicers. Only for admin level.
		#[pallet::call_index(4)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn account_rejection(origin: OriginFor<T>, account: T::AccountId) -> DispatchResultWithPostInfo {
			let _sender = ensure_signed(origin.clone())?;

			let role = Self::get_requested_role(&account).unwrap().role;
			ensure!(role.is_some(), Error::<T>::NotInWaitingList);

			// We can't reject a representive role request
			ensure!(role != Some(Accounts::REPRESENTATIVE), Error::<T>::UnAuthorized);
			let result = Self::reject_account(account.clone());

			RequestedRoles::<T>::remove(&account);

			match result{
				Ok(_) => {
					let now = <frame_system::Pallet<T>>::block_number();
					Self::deposit_event(Event::AccountCreationRejected(now, account));
				},
				Err(e) => return Err(e),
			}
			
			
			Ok(().into())
		}

		
		///The caller will transfer his admin authority to a different account
		#[pallet::call_index(5)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn set_manager(
			origin: OriginFor<T>,
			new: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			let new0 = T::Lookup::lookup(new.clone())?;
			ensure!(
				sender == SUDO::Pallet::<T>::key().unwrap(),
				"only the current sudo key can sudo"
			);
			
			//Remove current Sudo from Servicers list
			if ServicerLog::<T>::contains_key(sender.clone()) {
				ServicerLog::<T>::remove(sender.clone());
			}

			//create Servicer & approve a servicer account for new Sudo
			//if the new Sudo has no role yet
			if !AccountsRolesLog::<T>::contains_key(&new0) {
				Servicer::<T>::new(new0.clone());
				Self::approve_account(new0).ok();
			}
			SUDO::Pallet::<T>::set_key(origin, new).ok();
			Ok(())
		}

		/// Background council member vote for a proposal
		/// The origin must be signed and member of the Background Council
		/// - candidate : account requesting the role
		/// - approve : value of the vote (true or false)
		#[pallet::call_index(6)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn council_vote(origin:OriginFor<T>,candidate:T::AccountId,approve:bool) -> DispatchResultWithPostInfo {
			let caller = ensure_signed(origin)?;
			ensure!(
				Coll::Pallet::<T, Instance2>::members().contains(&caller),
				Error::<T>::NotACouncilMember
			);
			let proposal_all = Self::get_requested_role(&candidate).unwrap();
			let index = proposal_all.proposal_index;
			let result = Self::vote_action(caller.clone(),candidate,approve);
			

			match result{
				Ok(_) => {
					let now = <frame_system::Pallet<T>>::block_number();
					// deposit event
					Self::deposit_event(Event::BackgroundCouncilVoted{
						who: caller,
						proposal_index: index,
						when: now,
						});
					},
				Err(e) => return Err(e),
				}
			

			Ok(().into())
		}

		/// Background council member close the vote session for a proposal
		/// The origin must be signed and member of the Background Council
		/// - candidate : account requesting the role
		#[pallet::call_index(7)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn council_close(origin:OriginFor<T>,candidate:T::AccountId) -> DispatchResultWithPostInfo{
			let caller = ensure_signed(origin)?;
			let mut proposal_all = Self::get_requested_role(&candidate).unwrap();
			let index = proposal_all.proposal_index;
			let result = Self::closing_vote(caller.clone(),candidate.clone());
			

			match result{
				Ok(_) => {
					let now = <frame_system::Pallet<T>>::block_number();

			Self::deposit_event(Event::BackgroundCouncilSessionClosed{
				who: caller,
				proposal_index: index,
				when: now,
			});
				},
				Err(e) => return Err(e),
			}
			proposal_all = Self::get_requested_role(&candidate).unwrap();
			if proposal_all.approved==true{
				RequestedRoles::<T>::remove(&candidate);
			}
			
			Ok(().into())
		}



	}
}
