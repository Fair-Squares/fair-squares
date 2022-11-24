//!# Asset_Management Pallet
//!
//!The Asset_Management pallet is used for everything that is related to
//!the asset management by its owners.
//!
//!## Overview
//!
//!The Asset_Management gives the possibility to the asset Owners, through governance, 
//!to implement the following actions:
//!- Elect a Representative that will micro-manage the asset
//!- Demote a previously elected Representative
//!- Allow the representative to submit a list of Tenants to the Owners
//!- Allow the owners to vote on list of tenants submitted by the Representative
//!
//!### Dispatchable Functions
//!
//!* `launch_representative_session` - An Owner creates a referendum for the available proposals: 
//!   Elect or demote a Representative.
//!
//!* `owners_vote` - Each asset owner can vote on ongoing referendum.
//!
//!* `representative_approval` - Private call used as a proposal for Representative election.
//!
//!* `demote_representative` - Private call used as a proposal for Representative demotion.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
pub use pallet_democracy as Dem;
pub use pallet_nft as Nft;
pub use pallet_onboarding as Onboarding;
pub use pallet_housing_fund as HFund;
pub use pallet_assets as Assetss;
pub use pallet_roles as Roles;
pub use pallet_share_distributor as Share;

mod functions;
mod types;
pub use crate::types::*;
pub use functions::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

//#[cfg(feature = "runtime-benchmarks")]
//mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_system::WeightInfo;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + HFund::Config + Onboarding::Config +Roles::Config + Dem::Config + Share::Config + Nft::Config + Assetss::Config{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Call: Parameter
			+ UnfilteredDispatchable<Origin = <Self as frame_system::Config>::Origin>
			+ From<Call<Self>>
			+ GetDispatchInfo;
		type Delay: Get<Self::BlockNumber>;
		type CheckDelay: Get<Self::BlockNumber>;
		type InvestorVoteAmount: Get<u128>;
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		type WeightInfo: WeightInfo;

		#[pallet::constant]
		type MinimumDepositVote: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type CheckPeriod: Get<Self::BlockNumber>;
	}

	//Store the referendum_index and the struct containing the virtual_account/caller/potential_rep/vote_result
	#[pallet::storage]
	#[pallet::getter(fn proposals)]
	pub type ProposalsLog<T: Config> =
		StorageMap<_, Blake2_128Concat, Dem::ReferendumIndex, RepVote<T>, OptionQuery>;	

	#[pallet::storage]
	#[pallet::getter(fn indexes)]
	pub type ProposalsIndexes<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Dem::ReferendumIndex, OptionQuery>;	
	

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		///A voting session to elect a representative has started
		RepresentativeVoteSessionStarted{
			caller: T::AccountId,
			candidate: T::AccountId,
			asset_account: T::AccountId,
		},
		///An investor voted
		InvestorVoted{
			caller: T::AccountId,
			session_number: Dem::ReferendumIndex,
			when: BlockNumberOf<T>,
		},
		///A representative role was granted
		RepresentativeCandidateApproved{
			candidate: T::AccountId,
			asset_account: T::AccountId,
			when: BlockNumberOf<T>,
		},
		///An account was stripped of its Representative role
		RepresentativeDemoted{
			candidate: T::AccountId,
			asset_account: T::AccountId,
			when: BlockNumberOf<T>,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// The account is not an Asset account
		NotAnAssetAccount,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		///The proposal could not be created
		FailedToCreateProposal,
		///This Preimage already exists
		DuplicatePreimage,
		///Not an owner in the corresponding virtual account
		NotAnOwner,
		///The Asset Does not Exists
		NotAnAsset,
		///This referendum does not exists
		NotAValidReferendum,
		///This referendum is over
		ReferendumCompleted,
		///Not enough funds in the account
		NotEnoughFunds,
	}


	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
	
		fn on_initialize(n: T::BlockNumber) -> Weight {
			Self::begin_block(n)
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		
		/// Build the call to be executed when the proposal pass the democracy vote
		/// The origin must but root
		/// - account_id : the virtual account of the asset of the proposal
		/// - proposal : call encapsulating the inital proposal
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn execute_call_dispatch(
			origin: OriginFor<T>, 
			account_id: AccountIdOf<T>,
			proposal: Box<<T as Config>::Call>
		) -> DispatchResultWithPostInfo {

			ensure_root(origin)?;

			proposal
				.dispatch_bypass_filter(frame_system::RawOrigin::Signed(account_id.clone()).into())
				.ok();

			Ok(().into())
		}

		/// An owner trigger a vote session with a proposal for an asset
		/// The origin must be an owner of the asset
		/// - asset_type: type of the asset
		/// - asset_id: id of the asset
		/// - representative: an account with the representative role to be designed
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn launch_representative_session(
			origin:OriginFor<T>,
			asset_type: Nft::PossibleCollections, 
			asset_id: T::NftItemId,
			representative: T::AccountId,
			proposal: VoteProposals

		) -> DispatchResultWithPostInfo{

			let caller = ensure_signed(origin.clone())?;

			//Get the asset virtual account if it exists
			let collection_id: T::NftCollectionId = asset_type.value().into();
			let ownership = Share::Pallet::<T>::virtual_acc(collection_id,asset_id);
			ensure!(ownership.is_some(),Error::<T>::NotAnAsset);

			//Ensure that the caller is an owner related to the virtual account
			ensure!(Self::caller_can_vote(&caller,ownership.clone().unwrap()),Error::<T>::NotAnOwner);

			

			let virtual_account = ownership.clone().unwrap().virtual_account;
			let deposit = T::MinimumDeposit::get();

			//Ensure that the virtual account has enough funds
			for f in ownership.unwrap().owners{
				<T as Dem::Config>::Currency::transfer(
					&f,
					&virtual_account,
					deposit,
					ExistenceRequirement::AllowDeath,
				).ok();
			}
			//Create the call 

			let proposal_call= match proposal {
				VoteProposals::ElectRepresentative =>{ 
					//Check that the account is in the representative waiting list
			ensure!(Roles::Pallet::<T>::get_pending_representatives(&representative).is_some(),"No pending representative with this account");
					 Call::<T>::representative_approval {
						rep_account: representative.clone(),
						collection: collection_id,
						item: asset_id,
					}},
				VoteProposals::DemoteRepresentative => 
					Call::<T>::demote_representative {
						rep_account: representative.clone(),
						collection: collection_id,
						item: asset_id,
					},
				
			};
			

			//Format the call and create the proposal Hash
			let proposal_hash = Self::create_proposal_hash_and_note(virtual_account.clone(),proposal_call);

			let threshold = Dem::VoteThreshold::SimpleMajority;
			let delay = <T as Config>::Delay::get();

			let referendum_index = Dem::Pallet::<T>::internal_start_referendum(proposal_hash, threshold, delay);

			//Create data for proposals Log
			RepVote::<T>::new(
				caller.clone(), 
				virtual_account.clone(), 
				representative.clone(), 
				referendum_index, 
				collection_id, 
				asset_id,
			).ok();

			//Emit Event
			Self::deposit_event(Event::RepresentativeVoteSessionStarted{
				caller,
				candidate: representative,
				asset_account: virtual_account,
			});
			
			Ok(().into())
		}

		

		///The function below allows the owner to vote.
		///The balance locked and used for vote conviction corresponds 
		///to the number of ownership tokens possessed by the voter.  
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn owners_vote(origin: OriginFor<T>, referendum_index: Dem::ReferendumIndex, vote:bool) -> DispatchResult {
			let voter = ensure_signed(origin.clone())?;
			//Check that the referendum exists and is active
			ensure!(ProposalsLog::<T>::contains_key(referendum_index),Error::<T>::NotAValidReferendum);
			//Check the referendum status
			let infos = Self::proposals(referendum_index).unwrap();
			let status = infos.vote_result;
			ensure!(status==VoteResult::AWAITING,Error::<T>::ReferendumCompleted);
			//check that caller can vote
			let ownership = Share::Pallet::<T>::virtual_acc(infos.collection_id,infos.item_id).unwrap();
			ensure!(Self::caller_can_vote(&voter,ownership.clone()),Error::<T>::NotAnOwner);
			//Get number of FS tokens own by caller
			let tokens = Assetss::Pallet::<T>::balance(ownership.token_id.into(),&voter);
			let token0 = Self::balance_to_u128_option(tokens).unwrap();
			let token1 = Self::u128_to_balance_option(token0).unwrap();

			let v = Dem::Vote { aye: vote, conviction: Dem::Conviction::Locked1x };
			
			Dem::Pallet::<T>::vote(origin.clone(),referendum_index,Dem::AccountVote::Standard { vote: v, balance: token1 }).ok();
			
			//Emit event
			Self::deposit_event(Event::InvestorVoted{
				caller: voter,
				session_number: referendum_index,
				when: <frame_system::Pallet<T>>::block_number(),
			});



			Ok(())
		}

		///Approval of a Representative role request
		///The caller is the virtual account linked 
		///to the asset
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn representative_approval(origin: OriginFor<T>, rep_account: T::AccountId,collection: T::NftCollectionId,item: T::NftItemId) -> DispatchResult {
			let caller = ensure_signed(origin)?;

			//Check that the caller is a stored virtual account
			ensure!(
				caller == Share::Pallet::<T>::virtual_acc(collection, item).unwrap().virtual_account,
				Error::<T>::NotAnAssetAccount
			);
			
			//Approve role request
			Self::approve_representative(caller.clone(),rep_account.clone()).ok();

			Self::deposit_event(Event::RepresentativeCandidateApproved{
				candidate: rep_account,
				asset_account: caller,
				when: <frame_system::Pallet<T>>::block_number(),
			});

			Ok(())
		}

		///Demotion of a previously elected Representative
		///The caller is the virtual account linked 
		///to the asset
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn demote_representative(origin: OriginFor<T>, rep_account: T::AccountId,collection: T::NftCollectionId,item: T::NftItemId) -> DispatchResult {
			let caller = ensure_signed(origin)?;

			//Check that the caller is a stored virtual account
			ensure!(
				caller == Share::Pallet::<T>::virtual_acc(collection, item).unwrap().virtual_account,
				Error::<T>::NotAnAssetAccount
			);
			
			//revoke Representative Role
			Self::revoke_representative(rep_account.clone()).ok();

			Self::deposit_event(Event::RepresentativeDemoted{
				candidate: rep_account,
				asset_account: caller,
				when: <frame_system::Pallet<T>>::block_number(),
			});

			Ok(())
		}
	}
}
