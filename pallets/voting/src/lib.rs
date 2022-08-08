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
pub use pallet_democracy as DEMO;
pub use pallet_roles as ROLES;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::WeightInfo;

mod structs;

pub use crate::structs::*;

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
	};
	use frame_system::pallet_prelude::*;
	use scale_info::{
		Type,
		prelude::boxed::Box};
	use frame_system::Call as SystemCall;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + COLL::Config::<Instance1>+DEMO::Config + ROLES::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Call: Parameter + Dispatchable<Origin = <Self as frame_system::Config>::Origin> + From<Call<Self>>;
		type WeightInfo: WeightInfo;
		type Delay: Get<Self::BlockNumber>;
		type InvestorVoteAmount: Get<u128>;
		type Currency: ReservableCurrency<Self::AccountId>;
		type CollectiveProposal: Parameter 
			+ Dispatchable<Origin = <Self as pallet_collective::Config::<Instance1>>::Origin, PostInfo = PostDispatchInfo>
			+ From<frame_system::Call<Self>>
			+ GetDispatchInfo;
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

	
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
		SomethingStored2(u32),
		ProposalAdded(T::AccountId),
		ProposalHouseCouncilClosed(T::AccountId),
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
	}


	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(<T as pallet::Config>::WeightInfo::do_something(100))]
		pub fn do_something(origin: OriginFor<T>, account_id: AccountIdOf<T>, something: u32) -> DispatchResultWithPostInfo {

			// ensure_root(origin.clone())?;
			// let who = ensure_signed(origin.clone())?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			// Self::deposit_event(Event::SomethingStored(something, who));
			Self::deposit_event(Event::SomethingStored(something, account_id));
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
			amount: BalanceOf<T>,
			proposal: Box<<T as Config>::Call>
		) -> DispatchResultWithPostInfo {
			
			// Check that the extrinsic was signed and get the signer
			let who = ensure_signed(origin.clone())?;

			// Check that the account has the investor role
			ensure!(
				ROLES::Pallet::<T>::sellers(who.clone()).is_some(),
				Error::<T>::NotASeller
			);

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn submit_proposal_bis(
			origin: OriginFor<T>,
			proposal: Box<<T as COLL::Config::<Instance1>>::Proposal>,
		) -> DispatchResultWithPostInfo {
			
			let who = ensure_signed(origin.clone())?;

			let proposal_hash = T::Hashing::hash_of(&proposal);
			let proposal_index = COLL::Pallet::<T, Instance1>::proposal_count();

			let result = COLL::Pallet::<T, Instance1>::propose(
				origin.clone(), 
				2, 
				proposal.clone(), 
				proposal.encoded_size() as u32
			);

			match result {
				Ok(n) => {
					Self::deposit_event(Event::ProposalAdded(who.clone()));
				},
				Err(e) => { return Err(e); },
			}

			
			let vote_proposal = VoteProposal::new(who.clone(), proposal.clone(), proposal_index);

			VoteProposals::<T>::insert(proposal_hash.clone(), vote_proposal.clone());

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn council_vote(origin: OriginFor<T>,proposal_hash: T::Hash, approve: bool) -> DispatchResultWithPostInfo {

			let who = ensure_signed(origin.clone())?;

			let proposal = VoteProposals::<T>::get(proposal_hash.clone()).unwrap();

			COLL::Pallet::<T, Instance1>::vote(origin.clone(), proposal_hash.clone(), proposal.proposal_index, approve.clone());

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn council_close_vote(origin: OriginFor<T>, proposal_hash: T::Hash) -> DispatchResultWithPostInfo {

			let who = ensure_signed(origin.clone())?;

			let mut proposal = VoteProposals::<T>::get(proposal_hash.clone()).unwrap();
			let proposal_len = proposal.proposal_call.encoded_size();
			let proposal_weight = proposal.proposal_call.get_dispatch_info().weight;

			let result = COLL::Pallet::<T, Instance1>::close(
				origin.clone(), 
				proposal_hash.clone(), 
				proposal.proposal_index, 
				proposal_weight.clone(), 
				proposal_len.clone() as u32
			);

			match result {
				Ok(n) => {
					Self::deposit_event(Event::ProposalHouseCouncilClosed(who.clone()));
				},
				Err(e) => { return Err(e); },
			}

			// let call = Call::<T>::call_as_provider(who.clone(), proposal.clone());
			let proposal_encoded: Vec<u8> = proposal.proposal_call.encode();
			DEMO::Pallet::<T>::note_preimage(origin.clone(),proposal_encoded)?;
			let deposit = <T as DEMO::Config>::MinimumDeposit::get();
			DEMO::Pallet::<T>::propose(origin.clone(),proposal_hash.clone(),deposit.clone())?;
			
			let threshold = DEMO::VoteThreshold::SimpleMajority;
            let delay = <T as Config>::Delay::get();
            let referendum_index = DEMO::Pallet::<T>::internal_start_referendum(proposal_hash.clone(), threshold,delay);
			proposal.referendum_index = referendum_index;

			VoteProposals::<T>::mutate(&proposal_hash, |val| {
				*val = Some(proposal);
			});


			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn call_democracy(origin: OriginFor<T>, proposal: Box<<T as Config>::Call>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())?;

			// let mut proposal = VoteProposals::<T>::get(proposal_hash.clone()).unwrap();
			let proposal_hash = T::Hashing::hash_of(&proposal);

			// let proposal_encoded: Vec<u8> = proposal.encode();
			// DEMO::Pallet::<T>::note_preimage(origin.clone(),proposal_encoded)?;
			// let deposit = <T as DEMO::Config>::MinimumDeposit::get();
			// DEMO::Pallet::<T>::propose(origin.clone(),proposal_hash.clone(),deposit.clone())?;

			// let threshold = DEMO::VoteThreshold::SimpleMajority;
            // let delay = <T as Config>::Delay::get();
            // let referendum_index = DEMO::Pallet::<T>::internal_start_referendum(proposal_hash.clone(), threshold,delay);
			// proposal.referendum_index = referendum_index;

			// ensure!(VoteProposals::<T>::contains_key(&proposal_hash), Error::<T>::ProposalNotExists);

			// VoteProposals::<T>::mutate(&proposal_hash, |val| {
			// 	*val = Some(proposal);
			// });

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn send_proposal(origin: OriginFor<T>, proposal: Box<<T as Config>::Call>, proposition: Box<<T as Config>::CollectiveProposal>) -> DispatchResultWithPostInfo {

			let who = ensure_signed(origin.clone())?;

			let call_dispatch = Box::new(Call::<T>::call_dispatch{account_id: who.clone(), proposal: proposal.clone()});
			// let call_democracy_proposal = Self::convert_call(Box::new(Call::<T>::call_democracy_proposal{ proposal: call_dispatch.clone()}));
			// let call_democracy_proposal = Box::new(<T as Config>::CollectiveProposal::<T>::call_democracy_proposal{ proposal: call_dispatch.clone()});
			
			// let result = COLL::Pallet::<T, Instance1>::propose(
			// 	origin.clone(), 
			// 	2, 
			// 	proposition.clone(),
			// 	proposition.encoded_size() as u32
			// );

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn call_democracy_proposal(origin: OriginFor<T>, proposal: Box<Call<T>>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())?;
			let proposal_hash = T::Hashing::hash_of(&proposal);

			let proposal_encoded: Vec<u8> = proposal.encode();
			DEMO::Pallet::<T>::note_preimage(origin.clone(),proposal_encoded)?;
			let deposit = <T as DEMO::Config>::MinimumDeposit::get();
			DEMO::Pallet::<T>::propose(origin.clone(),proposal_hash.clone(),deposit.clone())?;

			let threshold = DEMO::VoteThreshold::SimpleMajority;
            let delay = <T as Config>::Delay::get();
            DEMO::Pallet::<T>::internal_start_referendum(proposal_hash.clone(), threshold,delay);

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn call_dispatch(origin: OriginFor<T>, account_id: AccountIdOf<T>, proposal: Box<<T as Config>::Call>) -> DispatchResultWithPostInfo {
			ensure_root(origin.clone())?;

			let res = proposal.dispatch(frame_system::RawOrigin::Signed(account_id.clone()).into());

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn investor_vote(origin: OriginFor<T>, proposal_hash: T::Hash, approve: bool) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())?;
			Self::deposit_event(Event::Step1(who.clone()));

			let proposal = VoteProposals::<T>::get(proposal_hash.clone()).unwrap();
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

			DEMO::Pallet::<T>::vote(origin.clone(), proposal.referendum_index, democracy_vote.clone());

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
}