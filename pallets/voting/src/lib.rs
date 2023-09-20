//! # Voting pallet
//!
//! The voting pallet provides methods to manage the voting processing through house council vote
//! and investors voting
//!
//! ## Overview
//!
//! This pallet manage the voting of a proposal by the House Council and an investor assemblee
//!
//! #### Dispatchable Functions
//! * 'submit_proposal' - an account with the seller role submit a proposal for a house purchase
//! * 'call_democracy_proposal' - configure a proposal to go through the democracy vote processing
//! * 'call_dispatch' - execute the house purchase proposal
//! * 'council_vote' - a member of the House Council vote for the first step going through the
//!   Collective pallet
//! * 'council_close_vote' - a member of the House Council close the collective vote session
//! * 'investor_vote' - an investor vote for the proposal during the democracy voting step

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;



pub use pallet_collective as COLL;
pub use pallet_democracy as DEM;
pub use pallet_roles as ROLES;
pub use pallet_utility as UTIL;
use COLL::Instance1;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
//pub mod weights;
//pub use weights::WeightInfo;

mod functions;
mod types;
pub use types::*;
pub use functions::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);
	
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config + COLL::Config<Instance1> + DEM::Config + ROLES::Config+UTIL::Config
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type WeightInfo: WeightInfo;
		type Delay: Get<BlockNumberOf<Self>>;
		type CheckDelay: Get<BlockNumberOf<Self>>;
		type InvestorVoteAmount: Get<BalanceOf<Self>>;
		type LocalCurrency: ReservableCurrency<Self::AccountId>;
		type HouseCouncilOrigin: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;
		type PreimageProvider: QueryPreimage + StorePreimage;
		#[pallet::constant]
		type CheckPeriod2: Get<BlockNumberFor<Self>>;

	}

	

	#[pallet::storage]
	#[pallet::getter(fn voting_proposals)]
	pub type VotingProposals<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		u32,
		VotingProposal<T, Box<<T as COLL::Config<Instance1>>::Proposal>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn collective_proposals)]
	pub type CollectiveProposals<T: Config> =
		StorageMap<_, Blake2_128Concat, u32, BlockNumberOf<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn democracy_proposals)]
	pub type DemocracyProposals<T: Config> =
		StorageMap<_, Blake2_128Concat, DEM::ReferendumIndex, BlockNumberOf<T>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A proposal has been added by a House coucil member
		HouseCouncilAddedProposal(T::AccountId, T::Hash, BlockNumberOf<T>),
		/// A proposal has been closed by a House coucil member
		HouseCouncilClosedProposal(T::AccountId, u32, BlockNumberOf<T>),
		/// A member of the House Council has voted
		HouseCouncilVoted(T::AccountId, u32, BlockNumberOf<T>),
		/// A investor has voted
		InvestorVoted(T::AccountId, u32, BlockNumberOf<T>),
		/// The investor vote session has started
		InvestorVoteSessionStarted(u32, BlockNumberOf<T>),
		/// TODO: to remove, Event for test purpose
		CollectiveMotionChecked(BlockNumberOf<T>),
		/// TODO: to remove, Event for test purpose
		CollectiveMotionPassed(BlockNumberOf<T>),
		/// TODO: to remove, Event for test purpose
		CollectiveMotionFailed(BlockNumberOf<T>),
		/// TODO: to remove, Event for test purpose
		DemocracyMotionFailed(BlockNumberOf<T>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		ProposalNotExists,
		/// Must have the investor role,
		NotAnInvestor,
		/// Must have the seller role
		NotASeller,
		/// Must be a member of the House Council
		NotAHouseCouncilMember,
		/// The proposal must exists
		ProposalDoesNotExist,
		/// The collective proposal have failed
		FailedToCreateCollectiveProposal,
	}

	/*#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// Weight: see `begin_block`
		fn on_initialize(n: BlockNumberOf<T>) -> Weight {
			Self::begin_block(n)
		}
	}*/

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		

		/// Submit a proposal through the voting process
		/// The origin must be signed and have the Seller role
		/// - proposal : the proposal to be executed at the end of the vote process
		/// - collective_passed_call : action to be executed when the proposal pass the collective
		///   vote
		/// - collective_failed_call : action to be executed when the proposal fail the collective
		///   vote
		/// - democracy_failed_call : action to be executed when the proposal fail the democracy
		///   vote
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn submit_proposal(
			origin: OriginFor<T>,
			proposal:  UtilCall<T>,
			collective_passed_call: Box<<T as frame_system::Config>::RuntimeCall>,
			collective_failed_call: Box<<T as frame_system::Config>::RuntimeCall>,
			democracy_failed_call: Box<<T as frame_system::Config>::RuntimeCall>,
		) -> DispatchResultWithPostInfo {
			// Check that the extrinsic was signed and get the signer
			let who = ensure_signed(origin.clone())?;

			// Check that the account has the seller role
			ensure!(ROLES::Pallet::<T>::sellers(who.clone()).is_some(), Error::<T>::NotASeller);
			let prop =  T::PreimageProvider::bound(proposal.clone()).unwrap();
			let proposal_id = DEM::Pallet::<T>::public_prop_count();

			let council_member = COLL::Pallet::<T, Instance1>::members()[0].clone();
			// create the final dispatch call of the proposal in democracy
			let call_ini = vec!(Self::call_dispatch 
				( council_member.clone(),
				proposal_id,
				proposal)
			);
			let call = Box::new(UTIL::Call::<T>::batch{calls:call_ini});

			

			
			
			Ok(().into())
		}

		/// Pass the proposal to the democracy pallet
		/// The origin must come from the collective palllet
		/// - account_id : the account of the issuer of the proposal
		/// - proposal_id : hash of the initial proposal call
		/// - proposal : call encapsulating the inital proposal
		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn call_democracy_proposal(
			origin: OriginFor<T>,
			account_id: AccountIdOf<T>,
			proposal_id: u32,
			call: <T as frame_system::Config>::RuntimeCall,
		) -> DispatchResultWithPostInfo {
			T::HouseCouncilOrigin::ensure_origin(origin)?;

			ensure!(
				VotingProposals::<T>::contains_key(proposal_id),
				Error::<T>::ProposalDoesNotExist
			);
            let delay = <T as Config>::Delay::get();
			// Start Democracy referendum
			let referendum_index = Self::start_dem_referendum(call,delay.clone());
			
            // Update the voting
			let mut proposal = VotingProposals::<T>::get(proposal_id).unwrap();
			proposal.democracy_referendum_index = referendum_index;
			proposal.collective_step = true;

			VotingProposals::<T>::mutate(proposal_id, |val| {
				*val = Some(proposal.clone());
			});

			let block_number = <frame_system::Pallet<T>>::block_number();
			let democratie_motion_duration = block_number
				.saturating_add(<T as DEM::Config>::VotingPeriod::get())
				.saturating_add(delay);

			// Set the the storage to be watched for the democracy process
			DemocracyProposals::<T>::insert(proposal_id, democratie_motion_duration);
			let origin_account= frame_system::RawOrigin::Signed(account_id);

			// Execute the dispatch for collective vote passed
			let dispatch_proposal = vec!(Self::get_dem_formatted_call(proposal.collective_passed_call));
			UTIL::Pallet::<T>::batch(origin_account.into(),dispatch_proposal).ok();
			Self::deposit_event(Event::InvestorVoteSessionStarted(proposal_id, block_number));

			Ok(().into())
		}
/*

		/// House council member vote for a proposal
		/// The origin must be signed and member of the House Council
		/// - proposal_hash : hash of the dispatch to be executed
		/// - approve : value of the vote (true or false)
		#[pallet::call_index(3)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn council_vote(
			origin: OriginFor<T>,
			proposal_id: u32,
			approve: bool,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())?;

			// Check that the caller is a house council member
			ensure!(
				COLL::Pallet::<T, Instance1>::members().contains(&who),
				Error::<T>::NotAHouseCouncilMember
			);

			// Check that the proposal exists
			ensure!(
				VotingProposals::<T>::contains_key(proposal_id),
				Error::<T>::ProposalDoesNotExist
			);

			let proposal = VotingProposals::<T>::get(proposal_id).unwrap();

			// Execute the collective vote
			let result = COLL::Pallet::<T, Instance1>::vote(
				origin,
				proposal.collective_hash,
				proposal.collective_index,
				approve,
			);

			match result {
				Ok(_) => {
					let block_number = <frame_system::Pallet<T>>::block_number();
					Self::deposit_event(Event::HouseCouncilVoted(who, proposal_id, block_number));
				},
				Err(e) => return Err(e),
			}

			Ok(().into())
		}

		/// Close a vote on a proposal
		/// The origin must be signed and member of the House Council
		/// proposal hash : hash of the proposalto be executed
		#[pallet::call_index(4)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn council_close_vote(
			origin: OriginFor<T>,
			proposal_id: u32,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())?;

			// Check that the caller is a member of the House Council
			ensure!(
				COLL::Pallet::<T, Instance1>::members().contains(&who),
				Error::<T>::NotAHouseCouncilMember
			);

			// Check that the proposal exists in the storage
			ensure!(
				VotingProposals::<T>::contains_key(proposal_id),
				Error::<T>::ProposalDoesNotExist
			);

			let proposal = VotingProposals::<T>::get(proposal_id).unwrap();
			let proposal_len = proposal.collective_call.encoded_size();
			let proposal_weight = proposal.collective_call.get_dispatch_info().weight;

			let result = COLL::Pallet::<T, Instance1>::close(
				origin,
				proposal.collective_hash,
				proposal.collective_index,
				proposal_weight,
				proposal_len as u32,
			);

			match result {
				Ok(_) => {
					let block_number = <frame_system::Pallet<T>>::block_number();
					Self::deposit_event(Event::HouseCouncilClosedProposal(
						who,
						proposal_id,
						block_number,
					));
				},
				Err(e) => return Err(e),
			}

			// We set the flag making the democracy pass vote
			let mut vote_proposal = VotingProposals::<T>::get(proposal_id).unwrap();
			vote_proposal.collective_closed = true;

			VotingProposals::<T>::mutate(proposal_id, |val| {
				*val = Some(vote_proposal);
			});

			Ok(().into())
		}

		/// Investor vote for a proposal
		/// The origin must be signed and and have the investor role
		/// - proposal_hash : hash of the dispatch to be executed
		/// - approve : value of the vote (true or false)
		#[pallet::call_index(5)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn investor_vote(
			origin: OriginFor<T>,
			proposal_id: u32,
			approve: bool,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())?;

			// Check that the account has the investor role
			ensure!(
				ROLES::Pallet::<T>::investors(who.clone()).is_some(),
				Error::<T>::NotAnInvestor
			);

			// Check that the proposal exists in the storage
			ensure!(
				VotingProposals::<T>::contains_key(proposal_id),
				Error::<T>::ProposalDoesNotExist
			);

			let proposal = VotingProposals::<T>::get(proposal_id).unwrap();


			let amount = T::InvestorVoteAmount::get();

			let democracy_vote = DEM::AccountVote::Standard {
				vote: DEM::Vote { aye: approve, conviction: DEM::Conviction::None },
				balance: amount,
			};

			let result = DEM::Pallet::<T>::vote(
				origin,
				proposal.democracy_referendum_index,
				democracy_vote,
			);

			match result {
				Ok(_) => {
					let block_number = <frame_system::Pallet<T>>::block_number();
					Self::deposit_event(Event::InvestorVoted(who, proposal_id, block_number));
				},
				Err(e) => return Err(e.into()),
			}

			Ok(().into())
		}*/
	}
}


impl<T: Config> Pallet<T> {
	


	

	
}