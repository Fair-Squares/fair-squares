#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub use pallet_collective as COLL;
use COLL::Instance1;
use COLL::ensure_members;
pub use pallet_democracy as DEMO;
pub use pallet_roles as ROLES;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::WeightInfo;

mod structs;

pub use crate::structs::*;

use frame_support::inherent::Vec;

type DemoBalanceOf<T> =
	<<T as DEMO::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;



#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		dispatch::Dispatchable,
		weights::{PostDispatchInfo,GetDispatchInfo},
		inherent::Vec,
		traits::{ReservableCurrency, UnfilteredDispatchable},
		pallet_prelude::*,
		sp_runtime::{traits::{Hash}, SaturatedConversion},
		error::BadOrigin,
	};
	use frame_system::{
		RawOrigin,
		pallet_prelude::*,
	};

	use scale_info::{
		Type,
		prelude::boxed::Box};
	use frame_system::Call as SystemCall;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + COLL::Config::<Instance1> + DEMO::Config + ROLES::Config  {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Call: Parameter + Dispatchable<Origin = <Self as frame_system::Config>::Origin> + From<Call<Self>>;
		type WeightInfo: WeightInfo;
		type Delay: Get<Self::BlockNumber>;
		type InvestorVoteAmount: Get<u128>;
		type Currency: ReservableCurrency<Self::AccountId>;
		// type Redirection: Parameter + Dispatchable<Origin = <Self as frame_system::Config>::Origin> + From<Call<Self>>;
		// type CollectiveProposal: Parameter 
		// 	+ Dispatchable<Origin = <Self as pallet_collective::Config::<Instance1>>::Origin, PostInfo = PostDispatchInfo>
		// 	+ From<frame_system::Call<Self>>
		// 	+ GetDispatchInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn something)]
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::storage]
	#[pallet::getter(fn vote_proposals)]
	pub type VoteProposals<T: Config> = 
		StorageMap<_, Blake2_128Concat, T::Hash, VoteProposal<T, Box<<T as COLL::Config::<Instance1>>::Proposal>>, OptionQuery>;


	#[pallet::storage]
	#[pallet::getter(fn voting_proposals)]
	pub type VotingProposals<T: Config> = 
		StorageMap<_, Blake2_128Concat, T::Hash, VotingProposal<T, Box<<T as COLL::Config::<Instance1>>::Proposal>>, OptionQuery>;

	
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
		SomethingStored2(u32),
		HouseCouncilAddedProposal(T::AccountId, T::Hash, BlockNumberOf<T>),
		HouseCouncilClosedProposal(T::AccountId, T::Hash, BlockNumberOf<T>),
		HouseCouncilVoted(T::AccountId, T::Hash, BlockNumberOf<T>),
		InvestorVoted(T::AccountId, T::Hash, BlockNumberOf<T>),
		Step1(T::AccountId),
		Step2(T::AccountId),
		Step3(T::AccountId),
		Step4(T::AccountId),
		Step5(T::AccountId),
		Step6(T::AccountId),
		Step7(T::AccountId),
		Step8(T::AccountId),
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
		FailedToCreateCollectiveProposal
	}


	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(<T as pallet::Config>::WeightInfo::do_something(100))]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResultWithPostInfo {

			let who = ensure_signed(origin.clone())?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored(something, who));
			// Return a successful DispatchResultWithPostInfo
			Ok(().into())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
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
					Ok(())
				},
			}
		}

		#[pallet::weight(10_000)]
		pub fn submit_proposal(
			origin: OriginFor<T>,
			proposal: Box<<T as Config>::Call>
		) -> DispatchResultWithPostInfo {
			
			// Check that the extrinsic was signed and get the signer
			let who = ensure_signed(origin.clone())?;

			// Check that the account has the investor role
			ensure!(
				ROLES::Pallet::<T>::sellers(who.clone()).is_some(),
				Error::<T>::NotASeller
			);

			// create the final dispatch call of the proposal in democracy
			let call = Call::<T>::call_dispatch{account_id: who.clone(), proposal: proposal.clone()};
			let call_formatted = Self::get_formatted_call(call.clone().into());
			let call_dispatch = Box::new(call_formatted.clone());
			let proposal_hash= T::Hashing::hash_of(&proposal);

			// create the democracy call to be proposed in collective
			let democracy_call = Call::<T>::call_democracy_proposal{account_id: who.clone(), proposal_id: proposal_hash.clone(), proposal: call_dispatch.clone()};			

			// call the collective propose
			let democracy_call_formatted_wrap = Self::get_formatted_collective_proposal(democracy_call.clone().into());

			ensure!(
				democracy_call_formatted_wrap.is_none() == false,
				Error::<T>::FailedToCreateCollectiveProposal
			);

			let democracy_call_formatted = Box::new(democracy_call_formatted_wrap.unwrap());
			
			let collective_index = COLL::Pallet::<T, Instance1>::proposal_count();			

			let result = COLL::Pallet::<T, Instance1>::propose(
				origin.clone(), 
				2, 
				democracy_call_formatted.clone().into(), 
				democracy_call_formatted.clone().encoded_size() as u32
			);

			// create the VotingProposal
			let voting_proposal: VotingProposal<T, Box<<T as COLL::Config::<Instance1>>::Proposal>> = VotingProposal::new(
				who.clone(), 
				proposal.clone(), 
				proposal_hash.clone(), 
				collective_index.clone(),
				democracy_call_formatted.clone(),
				T::Hashing::hash_of(&democracy_call_formatted),
				T::Hashing::hash_of(&call_dispatch)
			);

			VotingProposals::<T>::insert(proposal_hash.clone(), voting_proposal.clone());

			// deposit event
			let block_number = <frame_system::Pallet<T>>::block_number();
			Self::deposit_event(Event::HouseCouncilAddedProposal(who.clone(), proposal_hash.clone(), block_number.clone()));

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn submit_proposal_to_democracy(
			origin: OriginFor<T>,
			proposal: Box<<T as Config>::Call>
		) -> DispatchResultWithPostInfo {
			
			// Check that the extrinsic was signed and get the signer
			let who = ensure_signed(origin.clone())?;

			let call_dispatch = Self::get_formatted_call(Call::<T>::call_dispatch{account_id: who.clone(), proposal: proposal.clone()}.into());
			let box_call_dispatch = Box::new(call_dispatch.clone());

			let proposal_hash = T::Hashing::hash_of(&box_call_dispatch);
			let proposal_encoded: Vec<u8> = box_call_dispatch.encode();

			// Call Democracy note_pre_image
			DEMO::Pallet::<T>::note_preimage(origin.clone(),proposal_encoded.into())?;
			let deposit = <T as DEMO::Config>::MinimumDeposit::get();

			// Call Democracy propose
			DEMO::Pallet::<T>::propose(origin.clone(),proposal_hash.clone(),deposit.clone())?;

			let threshold = DEMO::VoteThreshold::SimpleMajority;
            let delay = <T as Config>::Delay::get();

			// Start Democracy referendum
            let referendum_index = DEMO::Pallet::<T>::internal_start_referendum(proposal_hash.clone(), threshold,delay);
			
			let democracy_call_formatted_wrap = Self::get_formatted_collective_proposal(call_dispatch.clone().into());
			
			ensure!(
				democracy_call_formatted_wrap.is_none() == false,
				Error::<T>::FailedToCreateCollectiveProposal
			);

			let democracy_call_formatted = Box::new(democracy_call_formatted_wrap.unwrap());

			// create the VotingProposal
			let mut voting_proposal: VotingProposal<T, Box<<T as COLL::Config::<Instance1>>::Proposal>> = VotingProposal::new(
				who.clone(), 
				proposal.clone(), 
				proposal_hash.clone(), 
				0,
				democracy_call_formatted.clone(),
				T::Hashing::hash_of(&democracy_call_formatted),
				T::Hashing::hash_of(&call_dispatch)
			);

			voting_proposal.democracy_referendum_index = referendum_index;

			VotingProposals::<T>::insert(proposal_hash.clone(), voting_proposal.clone());

			// deposit event
			let block_number = <frame_system::Pallet<T>>::block_number();
			Self::deposit_event(Event::HouseCouncilAddedProposal(who.clone(), proposal_hash.clone(), block_number.clone()));

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn call_democracy_proposal(origin: OriginFor<T>, account_id: AccountIdOf<T>, proposal_id: T::Hash, proposal: Box<<T as Config>::Call>) -> DispatchResultWithPostInfo {
			// let who = ensure_signed(origin.clone())?;

			// ensure!(
			// 	COLL::Pallet::<T, Instance1>::members().contains(&who),
			// 	Error::<T>::NotAHouseCouncilMember
			// );

			ensure!(
				VotingProposals::<T>::contains_key(&proposal_id),
				Error::<T>::ProposalDoesNotExist
			);

			let proposal_hash = T::Hashing::hash_of(&proposal);
			let proposal_encoded: Vec<u8> = proposal.encode();

			// Call Democracy note_pre_image
			DEMO::Pallet::<T>::note_preimage(RawOrigin::Signed(account_id.clone()).into(),proposal_encoded.into())?;
			let deposit = <T as DEMO::Config>::MinimumDeposit::get();

			// Call Democracy propose
			DEMO::Pallet::<T>::propose(RawOrigin::Signed(account_id.clone()).into(),proposal_hash.clone(),deposit.clone())?;

			let threshold = DEMO::VoteThreshold::SimpleMajority;
            let delay = <T as Config>::Delay::get();

			// Start Democracy referendum
            let referendum_index = DEMO::Pallet::<T>::internal_start_referendum(proposal_hash.clone(), threshold,delay);

			let mut proposal = VotingProposals::<T>::get(proposal_id.clone()).unwrap();
			proposal.democracy_referendum_index = referendum_index;

			VotingProposals::<T>::mutate(&proposal_id, |val| {
				*val = Some(proposal);
			});

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn call_dispatch(origin: OriginFor<T>, account_id: AccountIdOf<T>, proposal: Box<<T as Config>::Call>) -> DispatchResultWithPostInfo {
			
			ensure_root(origin.clone())?;
			// let who = ensure_signed(origin.clone())?;

			let res = proposal.dispatch(frame_system::RawOrigin::Signed(account_id.clone()).into());

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn council_vote(origin: OriginFor<T>,proposal_hash: T::Hash, approve: bool) -> DispatchResultWithPostInfo {

			let who = ensure_signed(origin.clone())?;

			ensure!(
				COLL::Pallet::<T, Instance1>::members().contains(&who),
				Error::<T>::NotAHouseCouncilMember
			);

			ensure!(
				VotingProposals::<T>::contains_key(&proposal_hash),
				Error::<T>::ProposalDoesNotExist
			);

			let proposal = VotingProposals::<T>::get(proposal_hash.clone()).unwrap();

			let result = COLL::Pallet::<T, Instance1>::vote(origin.clone(), proposal.collective_hash, proposal.collective_index, approve.clone());

			match result {
				Ok(n) => {
					let block_number = <frame_system::Pallet<T>>::block_number();
					Self::deposit_event(Event::HouseCouncilVoted(who.clone(), proposal_hash.clone(), block_number.clone()));
				},
				Err(e) => { return Err(e); },
			}
			
			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn council_close_vote(origin: OriginFor<T>, proposal_hash: T::Hash) -> DispatchResultWithPostInfo {

			let who = ensure_signed(origin.clone())?;

			ensure!(
				COLL::Pallet::<T, Instance1>::members().contains(&who),
				Error::<T>::NotAHouseCouncilMember
			);

			ensure!(
				VotingProposals::<T>::contains_key(&proposal_hash),
				Error::<T>::ProposalDoesNotExist
			);

			let proposal = VotingProposals::<T>::get(proposal_hash.clone()).unwrap();
			let proposal_len = proposal.collective_call.encoded_size();
			let proposal_weight = proposal.collective_call.get_dispatch_info().weight;

			let result = COLL::Pallet::<T, Instance1>::close(
				origin.clone(), 
				proposal.collective_hash, 
				proposal.collective_index, 
				proposal_weight.clone(), 
				proposal_len.clone() as u32
			);

			match result {
				Ok(n) => {
					let block_number = <frame_system::Pallet<T>>::block_number();
					Self::deposit_event(Event::HouseCouncilClosedProposal(who.clone(), proposal_hash.clone(), block_number.clone()));
				},
				Err(e) => { return Err(e); },
			}

			// If vote is disaproved, then call the failed_proposal


			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn investor_vote(origin: OriginFor<T>, proposal_hash: T::Hash, approve: bool) -> DispatchResultWithPostInfo {
			
			let who = ensure_signed(origin.clone())?;
			
			// // Check that the account has the investor role
			// ensure!(
			// 	ROLES::Pallet::<T>::investors(who.clone()).is_some(),
			// 	Error::<T>::NotAnInvestor
			// );

			ensure!(
				VotingProposals::<T>::contains_key(&proposal_hash),
				Error::<T>::ProposalDoesNotExist
			);

			let proposal = VotingProposals::<T>::get(proposal_hash.clone()).unwrap();
			let amount_wrap = Self::u128_to_balance_option(T::InvestorVoteAmount::get());

			ensure!(amount_wrap.is_some(), Error::<T>::NoneValue);

			let amount = amount_wrap.unwrap();

			let democracy_vote = DEMO::AccountVote::Standard{
				vote: DEMO::Vote{ 
					aye: approve, 
					conviction: DEMO::Conviction::None 
				}, 
				balance: amount
			};

			DEMO::Pallet::<T>::vote(origin.clone(), proposal.democracy_referendum_index, democracy_vote.clone());

			// match result {
			// 	Ok(n) => {
			// 		let block_number = <frame_system::Pallet<T>>::block_number();
			// 		Self::deposit_event(Event::InvestorVoted(who.clone(), proposal_hash.clone(), block_number.clone()));
			// 	},
			// 	Err(e) => { return Err(e); },
			// }

			Ok(().into())
		}		
	}
}

use frame_support::dispatch::Dispatchable;

impl<T: Config> Pallet<T> 
{
	// Conversion of u64 to BalanxceOf<T>
	pub fn u128_to_balance_option(input: u128) -> Option<DemoBalanceOf<T>> {
		input.try_into().ok()
	}

	// Conversion of BalanceOf<T> to u32
	pub fn balance_to_u32_option(input: BalanceOf<T>) -> Option<u32> {
		input.try_into().ok()
	}

	pub fn get_formatted_collective_proposal(call: <T as Config>::Call) -> Option<<T as COLL::Config::<Instance1>>::Proposal> {
		let call_encoded: Vec<u8> = call.encode();
		let mut ref_call_encoded = &call_encoded;

		if let Ok(call_formatted) = <T as pallet_collective::Config::<Instance1>>::Proposal::decode(&mut &ref_call_encoded[..]) {
			Some(call_formatted)
		}else {
			None
		}
	}

	pub fn get_formatted_call(call: <T as Config>::Call) -> <T as Config>::Call {
		
		call.into()
	}
}