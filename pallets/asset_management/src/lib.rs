//!# Asset_Management Pallet
//!
//!The Asset_Management pallet is used for everything that is related to
//!the asset management by its owners.
//!
//!## Overview
//!
//!The Asset_Management gives the possibility to the asset Owners, through governance,
//!to implement the following actions:
//! - Elect a Representative that will micro-manage the asset
//! - Demote a previously elected Representative
//! - Allow the representative to submit a list of Tenants to the Owners
//! - Allow the owners to vote on list of tenants submitted by the Representative
//!
//!### Dispatchable Functions
//!
//! * `launch_representative_session` - An Owner creates a referendum for the following available
//!   proposals:
//!   - Elect a Representative.
//!   - Demote a Representative.
//!
//! * `owners_vote` - Each asset owner can vote in an ongoing referendum.
//!
//! * `representative_approval` - Call used as a proposal for Representative election.
//!
//! * `demote_representative` - Call used as a proposal for Representative demotion.
//!
//! * `launch_tenant_session` - A Representative creates a referendum for the following available
//!   proposals:
//!   - Admit a Tenant for a given asset.
//!   - Evict  a Tenant from a given asset.
//!
//! * `link_tenant_to_asset` - Call used as a proposal to link an accepted tenant with an existing
//!   asset.
//!
//! * `unlink_tenant_to_asset` - Call used as a proposal to remove the link between a tenant and an
//!   asset.
//!
//! * `request_guaranty_payment` - Call used to send a guaranty deposit payment request to a tenant.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
pub use pallet_assets as Assetss;
pub use pallet_democracy as Dem;
pub use pallet_housing_fund as HFund;
pub use pallet_identity as Ident;
pub use pallet_nft as Nft;
pub use pallet_onboarding as Onboarding;
pub use pallet_payment as Payment;
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
	pub trait Config:
		frame_system::Config
		+ HFund::Config
		+ Onboarding::Config
		+ Roles::Config
		+ Dem::Config
		+ Share::Config
		+ Nft::Config
		+ Assetss::Config
		+ Ident::Config
		+ Payment::Config
	{
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
		type Guaranty: Get<u32>;

		#[pallet::constant]
		type RoR: Get<Percent>;

		#[pallet::constant]
		type MinimumDepositVote: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type RepFees: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type ContractLength: Get<Self::BlockNumber>;

		#[pallet::constant]
		type CheckPeriod: Get<Self::BlockNumber>;

		#[pallet::constant]
		type RentCheck: Get<Self::BlockNumber>;
	}

	//Store the referendum_index and the struct containing the
	// virtual_account/caller/potential_rep/vote_result
	#[pallet::storage]
	#[pallet::getter(fn proposals)]
	pub type ProposalsLog<T: Config> =
		StorageMap<_, Blake2_128Concat, Dem::ReferendumIndex, ProposalRecord<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn guaranty)]
	pub type GuarantyPayment<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId, // payment issuer
		Blake2_128Concat,
		T::AccountId, // payment recipient
		Payment::PaymentDetail<T>,
	>;

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
		RepresentativeVoteSessionStarted {
			caller: T::AccountId,
			candidate: T::AccountId,
			asset_account: T::AccountId,
		},
		/// A voting session to link a tenant to an asset has started
		TenantVoteSessionStarted {
			representative: T::AccountId,
			tenant: T::AccountId,
			asset_account: T::AccountId,
		},
		///An investor voted
		InvestorVoted {
			caller: T::AccountId,
			session_number: Dem::ReferendumIndex,
			when: BlockNumberOf<T>,
		},
		///A representative role was granted
		RepresentativeCandidateApproved {
			candidate: T::AccountId,
			asset_account: T::AccountId,
			when: BlockNumberOf<T>,
		},
		///An account was stripped of its Representative role
		RepresentativeDemoted {
			candidate: T::AccountId,
			asset_account: T::AccountId,
			when: BlockNumberOf<T>,
		},
		/// A tenant is linked with an asset
		TenantLinkedToAsset {
			tenant: T::AccountId,
			collection: T::NftCollectionId,
			item: T::NftItemId,
			asset_account: T::AccountId,
		},
		/// A tenant is demoted and unlinked with an asset
		TenantDemoted {
			tenant: T::AccountId,
			collection: T::NftCollectionId,
			item: T::NftItemId,
			asset_account: T::AccountId,
		},
		///The amount of the tenant debt
		TenantDebt { tenant: T::AccountId, debt: BalanceOf<T>, when: BlockNumberOf<T> },

		/// Guaranty payment request sent
		GuarantyPaymentRequested {
			tenant: T::AccountId,
			asset_account: T::AccountId,
			amount: Payment::BalanceOf<T>,
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
		/// The account is not a representative
		NotARepresentative,
		/// Not an active Representative
		NotAnActiveRepresentative,
		/// The asset is not linked to the representative
		AssetOutOfControl,
		/// The candidate is not a tenant
		NotATenant,
		/// An asset is already linked to the provided tenant
		AlreadyLinkedWithAsset,
		/// The tenant is not linked to the asset
		TenantAssetNotLinked,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		/// The proposal could not be created
		FailedToCreateProposal,
		/// This Preimage already exists
		DuplicatePreimage,
		/// Not an owner in the corresponding virtual account
		NotAnOwner,
		/// The Asset Does not Exists
		NotAnAsset,
		/// This referendum does not exists
		NotAValidReferendum,
		/// This referendum is over
		ReferendumCompleted,
		/// Not enough funds in the account
		NotEnoughFunds,
		/// Payment request already sent
		ExistingPaymentRequest,
		/// Not enough funds in the tenant account
		NotEnoughTenantFunds,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(n: T::BlockNumber) -> Weight {
			Self::begin_block(n)
		}

		fn on_idle(n: T::BlockNumber, _max_weight: Weight) -> Weight {
			Self::finish_block(n)
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
			proposal: Box<<T as Config>::Call>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			proposal
				.dispatch_bypass_filter(frame_system::RawOrigin::Signed(account_id.clone()).into())
				.ok();

			Ok(().into())
		}

		/// Using the function below, an owner triggers a vote session with a proposal for an asset
		/// The origin must be an owner of the asset
		/// - asset_type: type of the asset
		/// - asset_id: id of the asset
		/// - representative: an account with the representative role to be designed
		/// - proposal contains the extrinsics to be executed depending on the vote result
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn launch_representative_session(
			origin: OriginFor<T>,
			asset_type: Nft::PossibleCollections,
			asset_id: T::NftItemId,
			representative: T::AccountId,
			proposal: VoteProposals,
		) -> DispatchResultWithPostInfo {
			let caller = ensure_signed(origin.clone())?;

			//Get the asset virtual account if it exists
			let collection_id: T::NftCollectionId = asset_type.value().into();
			let ownership = Share::Pallet::<T>::virtual_acc(collection_id, asset_id);
			ensure!(ownership.is_some(), Error::<T>::NotAnAsset);

			//Ensure that the caller is an owner related to the virtual account
			ensure!(
				Self::caller_can_vote(&caller, ownership.clone().unwrap()),
				Error::<T>::NotAnOwner
			);

			let virtual_account = ownership.clone().unwrap().virtual_account;
			let deposit = T::MinimumDeposit::get();

			//Ensure that the virtual account has enough funds
			for f in ownership.unwrap().owners {
				<T as Dem::Config>::Currency::transfer(
					&f,
					&virtual_account,
					deposit,
					ExistenceRequirement::AllowDeath,
				)
				.ok();
			}

			//Create the call
			let proposal_call = match proposal {
				VoteProposals::Election => {
					//Check that the account is in the representative waiting list
					ensure!(
						Roles::Pallet::<T>::get_pending_representatives(&representative).is_some(),
						"No pending representative with this account"
					);
					Call::<T>::representative_approval {
						rep_account: representative.clone(),
						collection: collection_id,
						item: asset_id,
					}
				},
				VoteProposals::Demotion => Call::<T>::demote_representative {
					rep_account: representative.clone(),
					collection: collection_id,
					item: asset_id,
				},
			};

			//Format the call and create the proposal Hash
			let proposal_hash =
				Self::create_proposal_hash_and_note(virtual_account.clone(), proposal_call);

			let threshold = Dem::VoteThreshold::SimpleMajority;
			let delay = <T as Config>::Delay::get();

			let referendum_index =
				Dem::Pallet::<T>::internal_start_referendum(proposal_hash, threshold, delay);

			//Create data for proposals Log
			ProposalRecord::<T>::new(
				caller.clone(),
				virtual_account.clone(),
				representative.clone(),
				referendum_index,
				collection_id,
				asset_id,
			)
			.ok();

			//Emit Event
			Self::deposit_event(Event::RepresentativeVoteSessionStarted {
				caller,
				candidate: representative,
				asset_account: virtual_account,
			});

			Ok(().into())
		}

		/// The function below allows the owner to vote.
		/// The balance locked and used for vote conviction corresponds
		/// to the number of ownership tokens possessed by the voter.
		/// The origin must be an owner of the asset
		/// - referendum_index: index of the referendum the voter is taking part in
		/// - vote: aye or nay
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn owners_vote(
			origin: OriginFor<T>,
			referendum_index: Dem::ReferendumIndex,
			vote: bool,
		) -> DispatchResult {
			let voter = ensure_signed(origin.clone())?;
			//Check that the referendum exists and is active
			ensure!(
				ProposalsLog::<T>::contains_key(referendum_index),
				Error::<T>::NotAValidReferendum
			);
			//Check the referendum status
			let infos = Self::proposals(referendum_index).unwrap();
			let status = infos.vote_result;
			ensure!(status == VoteResult::AWAITING, Error::<T>::ReferendumCompleted);
			//check that caller can vote
			let ownership =
				Share::Pallet::<T>::virtual_acc(infos.collection_id, infos.item_id).unwrap();
			ensure!(Self::caller_can_vote(&voter, ownership.clone()), Error::<T>::NotAnOwner);
			//Get number of FS tokens own by caller
			let tokens = Assetss::Pallet::<T>::balance(ownership.token_id.into(), &voter);
			let token0 = Self::balance_to_u128_option(tokens).unwrap();
			let token1 = Self::u128_to_balance_option(token0).unwrap();

			let v = Dem::Vote { aye: vote, conviction: Dem::Conviction::Locked1x };

			Dem::Pallet::<T>::vote(
				origin.clone(),
				referendum_index,
				Dem::AccountVote::Standard { vote: v, balance: token1 },
			)
			.ok();

			//Emit event
			Self::deposit_event(Event::InvestorVoted {
				caller: voter,
				session_number: referendum_index,
				when: <frame_system::Pallet<T>>::block_number(),
			});

			Ok(())
		}

		/// The function below allows the approval of a Representative role request
		/// The origin must be the virtual account connected to the asset
		/// - rep_account: account Of the candidate to the representative account
		/// - collection: collection number of the asset.
		/// - item: item number of the asset.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn representative_approval(
			origin: OriginFor<T>,
			rep_account: T::AccountId,
			collection: T::NftCollectionId,
			item: T::NftItemId,
		) -> DispatchResult {
			let caller = ensure_signed(origin.clone())?;

			//Check that the caller is a stored virtual account
			ensure!(
				caller
					== Share::Pallet::<T>::virtual_acc(collection, item).unwrap().virtual_account,
				Error::<T>::NotAnAssetAccount
			);

			//Approve role request
			Self::approve_representative(origin, rep_account.clone()).ok();

			Self::deposit_event(Event::RepresentativeCandidateApproved {
				candidate: rep_account,
				asset_account: caller,
				when: <frame_system::Pallet<T>>::block_number(),
			});

			Ok(())
		}

		/// The function below allows the demotion of a previously elected Representative
		/// The origin must be the virtual account connected to the asset
		/// - rep_account: account Of the candidate to the representative account
		/// - collection: collection_id of the asset.
		/// - item: item_id of the asset.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn demote_representative(
			origin: OriginFor<T>,
			rep_account: T::AccountId,
			collection: T::NftCollectionId,
			item: T::NftItemId,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;

			//Check that the caller is a stored virtual account
			ensure!(
				caller
					== Share::Pallet::<T>::virtual_acc(collection, item).unwrap().virtual_account,
				Error::<T>::NotAnAssetAccount
			);

			//revoke Representative Role
			Self::revoke_representative(rep_account.clone()).ok();

			Self::deposit_event(Event::RepresentativeDemoted {
				candidate: rep_account,
				asset_account: caller,
				when: <frame_system::Pallet<T>>::block_number(),
			});

			Ok(())
		}

		/// Using the function below, a representative triggers a vote session with a proposal for a tenant to be linked with
		/// an asset The origin must be a representative
		/// - asset_type: type of the asset
		/// - asset_id: id of the asset
		/// - tenant: an account with the tenant role
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn launch_tenant_session(
			origin: OriginFor<T>,
			asset_type: Nft::PossibleCollections,
			asset_id: T::NftItemId,
			tenant: T::AccountId,
			proposal: VoteProposals,
			judgement: Ident::Judgement<IdentBalanceOf<T>>,
		) -> DispatchResult {
			let caller = ensure_signed(origin.clone())?;

			// Ensure that the caller is a representative
			let rep = Roles::Pallet::<T>::reps(caller.clone());
			ensure!(rep.is_some(), Error::<T>::NotARepresentative);
			let rep = rep.unwrap();
			ensure!(rep.activated, Error::<T>::NotAnActiveRepresentative);

			// Get the asset virtual account if exists
			let collection_id: T::NftCollectionId = asset_type.value().into();
			let ownership = Share::Pallet::<T>::virtual_acc(collection_id, asset_id);
			ensure!(ownership.is_some(), Error::<T>::NotAnAsset);

			//Compare guaranty payment amount+fees with tenant free_balance
			let guaranty = Self::calculate_guaranty(collection_id, asset_id);
			let fee0 = Self::balance_to_u128_option1(T::RepFees::get()).unwrap();
			let fee1 = T::IncentivePercentage::get()
				* Self::u128_to_balance_option2(guaranty.clone()).unwrap();
			let total_amount = guaranty + fee0 + Self::balance_to_u128_option1(fee1).unwrap();
			let tenant_bal0: BalanceOf<T> = <T as Config>::Currency::free_balance(&tenant);
			let tenant_bal = Self::balance_to_u128_option1(tenant_bal0).unwrap();

			let asset_account = ownership.unwrap().virtual_account;
			ensure!(rep.assets_accounts.contains(&asset_account), Error::<T>::AssetOutOfControl);

			// Ensure that provided account is a valid tenant
			let tenant0 = Roles::Pallet::<T>::tenants(tenant.clone());
			ensure!(tenant0.is_some(), Error::<T>::NotATenant);

			let tenant0 = tenant0.unwrap();
			match proposal {
				VoteProposals::Election => {
					// Ensure that the tenant is not linked to an asset
					ensure!(tenant0.asset_account.is_none(), Error::<T>::AlreadyLinkedWithAsset);
					//Ensure there is no existing payment request for this asset
					ensure!(
						Self::guaranty(&tenant0.account_id, &asset_account).is_none(),
						Error::<T>::ExistingPaymentRequest
					);
					//ensure that tenant can pay Guaranty deposit
					ensure!(tenant_bal > total_amount, Error::<T>::NotEnoughTenantFunds);
					//provide judgement
					let index = rep.index;
					let target = T::Lookup::unlookup(tenant.clone());
					Ident::Pallet::<T>::provide_judgement(
						origin.clone(),
						index.into(),
						target,
						judgement.clone(),
					)
					.ok();
				},
				VoteProposals::Demotion => {
					// Ensure that the tenant is linked to the asset
					ensure!(
						tenant0.asset_account == Some(asset_account.clone()),
						Error::<T>::TenantAssetNotLinked
					);
					let house = Onboarding::Pallet::<T>::houses(collection_id, asset_id).unwrap();
					ensure!(house.tenants.contains(&tenant), Error::<T>::TenantAssetNotLinked)
				},
			};

			let deposit = T::MinimumDeposit::get();

			// Ensure that the representative has enough funds
			<T as Dem::Config>::Currency::transfer(
				&caller,
				&asset_account,
				deposit,
				ExistenceRequirement::AllowDeath,
			)
			.ok();

			let call = match proposal {
				VoteProposals::Election => Call::<T>::request_guaranty_payment {
					from: tenant.clone(),
					collection: collection_id,
					item: asset_id,
					judgement,
				},
				VoteProposals::Demotion => Call::<T>::unlink_tenant_to_asset {
					tenant: tenant.clone(),
					collection: collection_id,
					item: asset_id,
				},
			};

			let proposal_hash = Self::create_proposal_hash_and_note(asset_account.clone(), call);

			let threshold = Dem::VoteThreshold::SimpleMajority;

			let delay = <T as Config>::Delay::get();

			let referendum_index =
				Dem::Pallet::<T>::internal_start_referendum(proposal_hash, threshold, delay);

			// Create data for proposals Log
			ProposalRecord::<T>::new(
				caller.clone(),
				asset_account.clone(),
				tenant.clone(),
				referendum_index,
				collection_id,
				asset_id,
			)
			.ok();

			//Emit Event
			Self::deposit_event(Event::TenantVoteSessionStarted {
				representative: caller,
				tenant,
				asset_account,
			});

			Ok(())
		}

		/// The function below links an accepted tenant with an existing asset
		/// The origin must be the virtual account connected to the asset
		/// - tenant: an account with the tenant role
		/// - collection: collection_id of the asset
		/// - item: item_id of the asset
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn link_tenant_to_asset(
			origin: OriginFor<T>,
			tenant: T::AccountId,
			collection: T::NftCollectionId,
			item: T::NftItemId,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;

			// Ensure the caller is the virtual account of the asset
			let asset_account =
				Share::Pallet::<T>::virtual_acc(collection, item).unwrap().virtual_account;
			ensure!(caller == asset_account, Error::<T>::NotAnAssetAccount);

			Self::tenant_link_asset(tenant.clone(), collection, item, asset_account.clone()).ok();

			Self::deposit_event(Event::TenantLinkedToAsset {
				tenant,
				collection,
				item,
				asset_account,
			});

			Ok(())
		}

		/// The function below sends a guaranty deposiy payment request to a tenant. This extrinsic is executed
		/// After a positive tenant_session.
		/// The origin must be the virtual account connected to the asset
		/// - tenant: an account with the tenant role linked to the asset
		/// - collection: collection_id of the asset
		/// - item: item_id of the asset
		/// - _judgement is provided by the representative while creating a tenant session  
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn request_guaranty_payment(
			origin: OriginFor<T>,
			from: T::AccountId,
			collection: T::NftCollectionId,
			item: T::NftItemId,
			_judgement: Ident::Judgement<IdentBalanceOf<T>>,
		) -> DispatchResult {
			let creator = ensure_signed(origin.clone())?;

			// Ensure the caller is the virtual account of the asset
			let asset_account =
				Share::Pallet::<T>::virtual_acc(collection, item).unwrap().virtual_account;
			ensure!(creator == asset_account, Error::<T>::NotAnAssetAccount);

			//Launch payment request
			Self::guaranty_payment(origin, from.clone(), collection, item).ok();
			let payment = Self::guaranty(from.clone(), asset_account).unwrap();
			let now = <frame_system::Pallet<T>>::block_number();

			Self::deposit_event(Event::GuarantyPaymentRequested {
				tenant: from,
				asset_account: creator,
				amount: payment.amount,
				when: now,
			});

			Ok(())
		}

		/// The function below unlinks a tenant with an asset
		/// The origin must be the virtual account connected to the asset
		/// - tenant: an account with the tenant role linked to the asset
		/// - collection: collection_id of the asset
		/// - item: item_id of the asset
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn unlink_tenant_to_asset(
			origin: OriginFor<T>,
			tenant: T::AccountId,
			collection: T::NftCollectionId,
			item: T::NftItemId,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;

			// Ensure the caller is the virtual account of the asset
			let asset_account =
				Share::Pallet::<T>::virtual_acc(collection, item).unwrap().virtual_account;
			ensure!(caller == asset_account, Error::<T>::NotAnAssetAccount);

			Self::tenant_unlink_asset(tenant.clone(), collection, item).ok();

			Self::deposit_event(Event::TenantDemoted { tenant, collection, item, asset_account });

			Ok(())
		}
	}
}
