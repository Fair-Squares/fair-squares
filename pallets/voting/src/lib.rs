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

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub use pallet_collective as COLL;
pub use pallet_democracy as DEMO;
pub use pallet_roles as ROLES;
use COLL::Instance1;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
//pub mod weights;
//pub use weights::WeightInfo;

mod structs;

pub use crate::structs::*;

use frame_support::{inherent::Vec, pallet_prelude::Weight, traits::Get};
use pallet_roles::{Saturating, Zero};

type DemoBalanceOf<T> =
	<<T as DEMO::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		inherent::Vec,
		pallet_prelude::*,
		sp_runtime::traits::Hash,
		traits::{ReservableCurrency, UnfilteredDispatchable},
		weights::GetDispatchInfo,
	};
	use frame_system::WeightInfo;
	use frame_system::{pallet_prelude::*, RawOrigin};

	use scale_info::prelude::boxed::Box;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config + COLL::Config<Instance1> + DEMO::Config + ROLES::Config
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Call: Parameter
			+ UnfilteredDispatchable<Origin = <Self as frame_system::Config>::Origin>
			+ From<Call<Self>>
			+ GetDispatchInfo;
		type WeightInfo: WeightInfo;
		type Delay: Get<Self::BlockNumber>;
		type CheckDelay: Get<Self::BlockNumber>;
		type InvestorVoteAmount: Get<u128>;
		type LocalCurrency: ReservableCurrency<Self::AccountId>;
		type HouseCouncilOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

		#[pallet::constant]
		type CheckPeriod: Get<Self::BlockNumber>;

		#[pallet::constant]
		type MinimumDepositVote: Get<BalanceOf<Self>>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn voting_proposals)]
	pub type VotingProposals<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::Hash,
		VotingProposal<T, Box<<T as COLL::Config<Instance1>>::Proposal>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn collective_proposals)]
	pub type CollectiveProposals<T: Config> =
		StorageMap<_, Blake2_128Concat, T::Hash, BlockNumberOf<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn democracy_proposals)]
	pub type DemocracyProposals<T: Config> =
		StorageMap<_, Blake2_128Concat, T::Hash, BlockNumberOf<T>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A proposal has been added by a House coucil member
		HouseCouncilAddedProposal(T::AccountId, T::Hash, BlockNumberOf<T>),
		/// A proposal has been closed by a House coucil member
		HouseCouncilClosedProposal(T::AccountId, T::Hash, BlockNumberOf<T>),
		/// A member of the House Council has voted
		HouseCouncilVoted(T::AccountId, T::Hash, BlockNumberOf<T>),
		/// A investor has voted
		InvestorVoted(T::AccountId, T::Hash, BlockNumberOf<T>),
		/// The investor vote session has started
		InvestorVoteSessionStarted(T::Hash, BlockNumberOf<T>),
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

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// Weight: see `begin_block`
		fn on_initialize(n: T::BlockNumber) -> Weight {
			Self::begin_block(n)
		}
	}

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
		#[pallet::weight(10_000)]
		pub fn submit_proposal(
			origin: OriginFor<T>,
			proposal: Box<<T as Config>::Call>,
			collective_passed_call: Box<<T as Config>::Call>,
			collective_failed_call: Box<<T as Config>::Call>,
			democracy_failed_call: Box<<T as Config>::Call>,
		) -> DispatchResultWithPostInfo {
			// Check that the extrinsic was signed and get the signer
			let who = ensure_signed(origin)?;

			// Check that the account has the seller role
			ensure!(ROLES::Pallet::<T>::sellers(who.clone()).is_some(), Error::<T>::NotASeller);

			let proposal_hash = T::Hashing::hash_of(&proposal);

			let council_member = COLL::Pallet::<T, Instance1>::members()[0].clone();
			// create the final dispatch call of the proposal in democracy
			let call = Call::<T>::call_dispatch {
				account_id: council_member,
				proposal_hash,
				proposal: proposal.clone(),
			};
			let call_formatted = Self::get_formatted_call(call.into());
			let call_dispatch = Box::new(call_formatted);

			// create the democracy call to be proposed in collective
			let democracy_call = Call::<T>::call_democracy_proposal {
				account_id: who.clone(),
				proposal_id: proposal_hash,
				proposal: call_dispatch.clone(),
			};

			// call the collective propose
			let democracy_call_formatted_wrap =
				Self::get_formatted_collective_proposal(democracy_call.into());

			// Check that the call to the democracy pallet is correctly created
			ensure!(
				democracy_call_formatted_wrap.is_some(),
				Error::<T>::FailedToCreateCollectiveProposal
			);

			let democracy_call_formatted = Box::new(democracy_call_formatted_wrap.unwrap());

			// Retrieve the index of the proposal in Collective pallet
			let collective_index = COLL::Pallet::<T, Instance1>::proposal_count();

			let collective_origin =
				Self::get_origin(COLL::Pallet::<T, Instance1>::members()[0].clone());

			let result = COLL::Pallet::<T, Instance1>::propose(
				collective_origin,
				2,
				democracy_call_formatted.clone(),
				democracy_call_formatted.encoded_size() as u32,
			);

			match result {
				Ok(_) => {},
				Err(e) => return Err(e),
			}

			// create the VotingProposal
			let voting_proposal: VotingProposal<T, Box<<T as COLL::Config<Instance1>>::Proposal>> =
				VotingProposal::new(
					who.clone(),
					proposal,
					collective_passed_call,
					collective_failed_call,
					democracy_failed_call,
					proposal_hash,
					collective_index,
					democracy_call_formatted.clone(),
					T::Hashing::hash_of(&democracy_call_formatted),
					T::Hashing::hash_of(&call_dispatch),
				);

			VotingProposals::<T>::insert(proposal_hash, voting_proposal);

			let block_number = <frame_system::Pallet<T>>::block_number();

			let collective_motion_duration = block_number
				.saturating_add(<T as COLL::Config<Instance1>>::MotionDuration::get())
				.saturating_add(T::Delay::get());

			// Add the proposal to the collective watchlist
			CollectiveProposals::<T>::insert(proposal_hash, collective_motion_duration);

			// deposit event
			Self::deposit_event(Event::HouseCouncilAddedProposal(who, proposal_hash, block_number));

			Ok(().into())
		}

		/// Pass the proposal to the democracy pallet
		/// The origin must come from the collective palllet
		/// - account_id : the account of the issuer of the proposal
		/// - proposal_id : hash of the initial proposal call
		/// - proposal : call encapsulating the inital proposal
		#[pallet::weight(10_000)]
		pub fn call_democracy_proposal(
			origin: OriginFor<T>,
			account_id: AccountIdOf<T>,
			proposal_id: T::Hash,
			proposal: Box<<T as Config>::Call>,
		) -> DispatchResultWithPostInfo {
			T::HouseCouncilOrigin::ensure_origin(origin)?;

			ensure!(
				VotingProposals::<T>::contains_key(proposal_id),
				Error::<T>::ProposalDoesNotExist
			);

			let proposal_hash = T::Hashing::hash_of(&proposal);
			let proposal_encoded: Vec<u8> = proposal.encode();

			// Call Democracy note_pre_image
			DEMO::Pallet::<T>::note_preimage(
				RawOrigin::Signed(account_id.clone()).into(),
				proposal_encoded,
			)?;

			let deposit = T::MinimumDepositVote::get();

			// A part of the initial deposit is freed to be reserved in the Democracy::propose()
			// function
			T::LocalCurrency::unreserve(&account_id, deposit);

			let threshold = DEMO::VoteThreshold::SimpleMajority;
			let delay = <T as Config>::Delay::get();

			// Start Democracy referendum

			let referendum_index =
				DEMO::Pallet::<T>::internal_start_referendum(proposal_hash, threshold, delay);

			// Update the voting
			let mut proposal = VotingProposals::<T>::get(proposal_id).unwrap();
			proposal.democracy_referendum_index = referendum_index;
			proposal.collective_step = true;

			VotingProposals::<T>::mutate(proposal_id, |val| {
				*val = Some(proposal.clone());
			});

			let block_number = <frame_system::Pallet<T>>::block_number();
			let democration_motion_duration = block_number
				.saturating_add(<T as DEMO::Config>::VotingPeriod::get())
				.saturating_add(delay);

			// Set the the storage to be watched for the democracy process
			DemocracyProposals::<T>::insert(proposal_id, democration_motion_duration);

			// Execute the dispatch for collective vote passed
			proposal
				.collective_passed_call
				.dispatch_bypass_filter(frame_system::RawOrigin::Signed(account_id).into())
				.ok();

			Self::deposit_event(Event::InvestorVoteSessionStarted(proposal_hash, block_number));

			Ok(().into())
		}

		/// Build the call to be executed when the proposal pass the democracy vote
		/// The origin must come from the collective palllet
		/// - account_id : the account of a member of the House Council
		/// - proposal_hash : hash of the initial proposal call
		/// - proposal : call encapsulating the inital proposal
		#[pallet::weight(10_000)]
		pub fn call_dispatch(
			origin: OriginFor<T>,
			account_id: AccountIdOf<T>,
			proposal_hash: T::Hash,
			proposal: Box<<T as Config>::Call>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			// We set the flag making the democracy pass vote
			let mut vote_proposal = VotingProposals::<T>::get(proposal_hash).unwrap();
			vote_proposal.proposal_executed = true;

			VotingProposals::<T>::mutate(proposal_hash, |val| {
				*val = Some(vote_proposal);
			});

			// The proposal is executed
			proposal
				.dispatch_bypass_filter(frame_system::RawOrigin::Signed(account_id).into())
				.ok();

			Ok(().into())
		}

		/// House council member vote for a proposal
		/// The origin must be signed and member of the House Council
		/// - proposal_hash : hash of the dispatch to be executed
		/// - approve : value of the vote (true or false)
		#[pallet::weight(10_000)]
		pub fn council_vote(
			origin: OriginFor<T>,
			proposal_hash: T::Hash,
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
				VotingProposals::<T>::contains_key(proposal_hash),
				Error::<T>::ProposalDoesNotExist
			);

			let proposal = VotingProposals::<T>::get(proposal_hash).unwrap();

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
					Self::deposit_event(Event::HouseCouncilVoted(who, proposal_hash, block_number));
				},
				Err(e) => return Err(e),
			}

			Ok(().into())
		}

		/// Close a vote on a proposal
		/// The origin must be signed and member of the House Council
		/// proposal hash : hash of the proposalto be executed
		#[pallet::weight(10_000)]
		pub fn council_close_vote(
			origin: OriginFor<T>,
			proposal_hash: T::Hash,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())?;

			// Check that the caller is a member of the House Council
			ensure!(
				COLL::Pallet::<T, Instance1>::members().contains(&who),
				Error::<T>::NotAHouseCouncilMember
			);

			// Check that the proposal exists in the storage
			ensure!(
				VotingProposals::<T>::contains_key(proposal_hash),
				Error::<T>::ProposalDoesNotExist
			);

			let proposal = VotingProposals::<T>::get(proposal_hash).unwrap();
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
						proposal_hash,
						block_number,
					));
				},
				Err(e) => return Err(e),
			}

			// We set the flag making the democracy pass vote
			let mut vote_proposal = VotingProposals::<T>::get(proposal_hash).unwrap();
			vote_proposal.collective_closed = true;

			VotingProposals::<T>::mutate(proposal_hash, |val| {
				*val = Some(vote_proposal);
			});

			Ok(().into())
		}

		/// Investor vote for a proposal
		/// The origin must be signed and and have the investor role
		/// - proposal_hash : hash of the dispatch to be executed
		/// - approve : value of the vote (true or false)
		#[pallet::weight(10_000)]
		pub fn investor_vote(
			origin: OriginFor<T>,
			proposal_hash: T::Hash,
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
				VotingProposals::<T>::contains_key(proposal_hash),
				Error::<T>::ProposalDoesNotExist
			);

			let proposal = VotingProposals::<T>::get(proposal_hash).unwrap();
			let amount_wrap = Self::u128_to_balance_option(T::InvestorVoteAmount::get());

			ensure!(amount_wrap.is_some(), Error::<T>::NoneValue);

			let amount = amount_wrap.unwrap();

			let democracy_vote = DEMO::AccountVote::Standard {
				vote: DEMO::Vote { aye: approve, conviction: DEMO::Conviction::None },
				balance: amount,
			};

			let result = DEMO::Pallet::<T>::vote(
				origin,
				proposal.democracy_referendum_index,
				democracy_vote,
			);

			match result {
				Ok(_) => {
					let block_number = <frame_system::Pallet<T>>::block_number();
					Self::deposit_event(Event::InvestorVoted(who, proposal_hash, block_number));
				},
				Err(e) => return Err(e.into()),
			}

			Ok(().into())
		}
	}
}

use frame_support::dispatch::UnfilteredDispatchable;

impl<T: Config> Pallet<T> {
	// Conversion of u64 to BalanxceOf<T>
	pub fn u128_to_balance_option(input: u128) -> Option<DemoBalanceOf<T>> {
		input.try_into().ok()
	}

	// Conversion of BalanceOf<T> to u32
	pub fn balance_to_u32_option(input: BalanceOf<T>) -> Option<u32> {
		input.try_into().ok()
	}

	pub fn get_formatted_collective_proposal(
		call: <T as Config>::Call,
	) -> Option<<T as COLL::Config<Instance1>>::Proposal> {
		let call_encoded: Vec<u8> = call.encode();
		let ref_call_encoded = &call_encoded;

		if let Ok(call_formatted) = <T as pallet_collective::Config<Instance1>>::Proposal::decode(
			&mut &ref_call_encoded[..],
		) {
			Some(call_formatted)
		} else {
			None
		}
	}

	pub fn get_formatted_call(call: <T as Config>::Call) -> <T as Config>::Call {
		call
	}

	pub fn get_origin(account_id: AccountIdOf<T>) -> <T as frame_system::Config>::Origin {
		frame_system::RawOrigin::Signed(account_id).into()
	}

	/// Current era is ending; check if the proposal has passed some steps
	/// Check the proposals being processed in the collective pallet
	/// Check the proposals being processed in the democracy pallet
	fn begin_block(now: T::BlockNumber) -> Weight {
		let max_block_weight = Weight::from_ref_time(1000 as u64);

		if (now % T::CheckPeriod::get()).is_zero() {
			let collectives_iter = CollectiveProposals::<T>::iter();
			let mut collectives_hash = Vec::new();

			for elt in collectives_iter {
				if elt.1 <= now {
					let voting = VotingProposals::<T>::get(elt.0).unwrap();

					if voting.collective_closed {
						// the collective step not passed means it has been rejected by the House
						// Council
						if !voting.collective_step {
							voting
								.collective_failed_call
								.dispatch_bypass_filter(
									frame_system::RawOrigin::Signed(voting.account_id.clone())
										.into(),
								)
								.ok();
						}

						// the vote doesn't need to be watched in the collective proposal storage
						// for this step anymore
						collectives_hash.push(elt.0);
					}
				}
			}

			let voting_hash_iter = collectives_hash.iter();
			for hash in voting_hash_iter {
				CollectiveProposals::<T>::remove(hash);
			}

			let democracies_iter = DemocracyProposals::<T>::iter();
			let mut democracies_hash = Vec::new();

			for elt in democracies_iter {
				if elt.1 <= now {
					let voting = VotingProposals::<T>::get(elt.0).unwrap();

					if !voting.proposal_executed {
						voting
							.democracy_failed_call
							.dispatch_bypass_filter(
								frame_system::RawOrigin::Signed(voting.account_id.clone()).into(),
							)
							.ok();
					}

					// the democracy doesn't need to be watched in the democracy proposal storage
					// for this step anymore
					democracies_hash.push(elt.0);
				}
			}

			let demo_hash_iter = democracies_hash.iter();
			for elt in demo_hash_iter {
				DemocracyProposals::<T>::remove(elt);
			}
		}

		max_block_weight
	}
}
