
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
mod functions;
mod types;

pub use functions::*;
pub use types::*;

//#[cfg(feature = "runtime-benchmarks")]
//mod benchmarking;
//pub mod weights;
//pub use weights::*;
pub use pallet_roles as Roles;
pub use pallet_collective as Coll;
pub use pallet_nft as Nft;
use Coll::Instance1;

// All pallet logic is defined in its own module and must be annotated by the `pallet` attribute.
#[frame_support::pallet]
pub mod pallet {
	// Import various useful types required by all FRAME pallets.
	use super::*;

	// The `Pallet` struct serves as a placeholder to implement traits, methods and dispatchables
	// (`Call`s) in this pallet.
	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// The pallet's configuration trait.
	///
	/// All our types and constants a pallet depends on must be declared here.
	/// These types are defined generically and made concrete when the pallet is declared in the
	/// `runtime/src/lib.rs` file of your chain.
	#[pallet::config]
	pub trait Config: frame_system::Config+Roles::Config+Coll::Config<Instance1>+Nft::Config {
		/// The overarching runtime event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Currency: ReservableCurrency<Self::AccountId>;
		type RuntimeCall: Parameter
			+ UnfilteredDispatchable<RuntimeOrigin = <Self as frame_system::Config>::RuntimeOrigin>
			+ From<Call<Self>>
			+ GetDispatchInfo;		
		#[pallet::constant]
		type CheckPeriod: Get<BlockNumberFor<Self>>;
		
		type HousingCouncilOrigin: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;
	
	}

	/// A storage item for this pallet.
	///
	/// In this template, we are declaring a storage item called `Something` that stores a single
	/// `u32` value. Learn more about runtime storage here: <https://docs.substrate.io/build/runtime-storage/>
	/// The [`getter`] macro generates a function to conveniently retrieve the value from storage.
	#[pallet::storage]
	#[pallet::getter(fn something)]
	pub type Something<T> = StorageValue<_, u32>;

	//This storage should not be necessary, as we already have approval waiting list
	#[pallet::storage]
	#[pallet::getter(fn get_submitted_proposal)]
	pub type SellerProposal<T: Config> =
		StorageMap<_, Twox64Concat, AccountIdOf<T>, ProposalOf<T>, OptionQuery>;


	
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A user has successfully set a new value.
		SomethingStored {
			/// The new value set.
			something: u32,
			/// The account who set the new value.
			who: T::AccountId,
		},
		/// Request for new role accepted
		ProposalApproved(BlockNumberFor<T>, T::AccountId),
		/// Request for new role Rejected
		ProposalRejected(BlockNumberFor<T>, T::AccountId),
		/// A proposal has been added by a Background Council member
		HousingCouncilAddedProposal{for_who: T::AccountId, proposal_index: u32, when: BlockNumberFor<T>},
		/// A proposal has been closed by a Background Council member
		HousingCouncilSessionClosed{who: T::AccountId, proposal_index: u32, when: BlockNumberFor<T>},
		/// A member of the Background Council has voted
		HousingCouncilVoted{who: T::AccountId, proposal_index: u32, when: BlockNumberFor<T>},
	}

	
	#[pallet::error]
	pub enum Error<T> {
		/// The value retrieved was `None` as no value was previously set.
		NoneValue,
		/// There was an attempt to increment the value in storage over `u32::MAX`.
		StorageOverflow,
		/// No Pending Request from this Seller
		NoPendingRequest,
		/// This is not a Council Member
		NotACouncilMember,
		/// This proposal does not exist
		ProposalDoesNotExist,
		/// Only one proposal submission is allowed per governance round
		OnlyOneSubmissionPerRound
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(n: BlockNumberFor<T>) -> Weight {
			Self::begin_block(n)
		}
	}

	
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a single u32 value as a parameter, writes the value
		/// to storage and emits an event.
		///
		/// It checks that the _origin_ for this call is _Signed_ and returns a dispatch
		/// error if it isn't. Learn more about origins here: <https://docs.substrate.io/build/origins/>
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let who = ensure_signed(origin)?;

			// Update storage.
			Something::<T>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored { something, who });

			// Return a successful `DispatchResult`
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		///
		/// It checks that the caller is a signed origin and reads the current value from the
		/// `Something` storage item. If a current value exists, it is incremented by 1 and then
		/// written back to storage.
		///
		/// ## Errors
		///
		/// The function will return an error under the following conditions:
		///
		/// - If no value has been set ([`Error::NoneValue`])
		/// - If incrementing the value in storage causes an arithmetic overflow
		///   ([`Error::StorageOverflow`])
		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match Pallet::<T>::something() {
				// Return an error if the value has not been set.
				None => Err(Error::<T>::NoneValue.into()),
				Some(old) => {
					// Increment the value read from storage. This will cause an error in the event
					// of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					Something::<T>::put(new);
					Ok(())
				},
			}		
		
		}

		#[pallet::call_index(2)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn collective_approval(
			origin: OriginFor<T>,
			collection: Nft::PossibleCollections,
			item_id: T::NftItemId,
		) -> DispatchResult{
			
			let _caller = T::HousingCouncilOrigin::ensure_origin(origin.clone())?;
			let collection_id:T::NftCollectionId = collection.value().into();
			//get owner
			let owner = Nft::Pallet::<T>::owner(collection_id,item_id).unwrap();
			//Change status
			Self::status(owner.clone()).ok();
			let now = <frame_system::Pallet<T>>::block_number();
			let mut proposal0=Self::get_submitted_proposal(owner.clone()).unwrap();
			proposal0.approved=true;
			SellerProposal::<T>::mutate(owner.clone(),|val|{
				*val=Some(proposal0);
			});
			Self::deposit_event(Event::ProposalApproved(now, owner));

			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn seller_proposal_evaluation(origin: OriginFor<T>, collection: Nft::PossibleCollections,item_id: T::NftItemId) -> DispatchResultWithPostInfo {
			let caller = ensure_signed(origin)?;

			ensure!(
				Coll::Pallet::<T, Instance1>::members().contains(&caller),
				Error::<T>::NotACouncilMember
			);
			//get asset owner
			let collection_id:T::NftCollectionId = collection.value().into();
			let owner = Nft::Pallet::<T>::owner(collection_id,item_id).unwrap();
			ensure!(
				!SellerProposal::<T>::contains_key(&owner),Error::<T>::OnlyOneSubmissionPerRound
			);
			Self::start_house_council_session(owner,collection,item_id).ok();
			
			Ok(().into())
		}

		/// Housing council member vote for a proposal
		/// The origin must be signed and member of the Background Council
		/// - candidate : account requesting the role
		/// - approve : value of the vote (true or false)
		#[pallet::call_index(4)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn housing_council_vote(origin:OriginFor<T>,seller:T::AccountId,approve:bool) -> DispatchResultWithPostInfo {
			let caller = ensure_signed(origin)?;
			ensure!(
				Coll::Pallet::<T, Instance1>::members().contains(&caller),
				Error::<T>::NotACouncilMember
			);
			let proposal_all = Self::get_submitted_proposal(&seller).unwrap();
			let index = proposal_all.proposal_index;
			let result = Self::vote_action(caller.clone(),seller,approve);
			

			match result{
				Ok(_) => {
					let now = <frame_system::Pallet<T>>::block_number();
					// deposit event
					Self::deposit_event(Event::HousingCouncilVoted{
						who: caller,
						proposal_index: index,
						when: now,
						});
					},
				Err(e) => return Err(e),
				}
			

			Ok(().into())
		}

		/// Housing council member close the vote session for a proposal
		/// The origin must be signed and member of the Background Council
		/// - seller : account submitting the proposal
		#[pallet::call_index(5)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn housing_council_close(origin:OriginFor<T>,seller:T::AccountId) -> DispatchResultWithPostInfo{
			let caller = ensure_signed(origin)?;
			let mut proposal_all = Self::get_submitted_proposal(&seller).unwrap();
			let index = proposal_all.proposal_index;
			let result = Self::closing_vote(caller.clone(),seller.clone());
			

			match result{
				Ok(_) => {
					let now = <frame_system::Pallet<T>>::block_number();

			Self::deposit_event(Event::HousingCouncilSessionClosed{
				who: caller,
				proposal_index: index,
				when: now,
			});
				},
				Err(e) => return Err(e),
			}
			proposal_all = Self::get_submitted_proposal(&seller).unwrap();
			if proposal_all.approved==true{
				SellerProposal::<T>::remove(&seller);
			}
			
			Ok(().into())
		}

		///Creation Refusal function for Sellers and Servicers. Only for admin level.
		#[pallet::call_index(6)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn proposal_rejection(origin: OriginFor<T>, account: T::AccountId) -> DispatchResultWithPostInfo {
			let _sender = ensure_signed(origin.clone())?;

			let proposal = Self::get_submitted_proposal(&account);
			ensure!(proposal.is_some(), Error::<T>::ProposalDoesNotExist);

			SellerProposal::<T>::remove(&account);

			let now = <frame_system::Pallet<T>>::block_number();
					Self::deposit_event(Event::ProposalRejected(now, account));
			
			
			Ok(().into())
		}


	}
}