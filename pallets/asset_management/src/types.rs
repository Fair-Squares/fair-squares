pub use super::*;
pub use frame_support::{
	assert_ok,
	dispatch::{DispatchResult, EncodeLike},
	inherent::Vec,
	pallet_prelude::*,
    weights::GetDispatchInfo,
	sp_runtime::{
		traits::{AccountIdConversion, Hash, One, Saturating, StaticLookup, Zero},
		PerThing, Percent,
	},
	storage::child,
	traits::{
		UnfilteredDispatchable,Currency, ExistenceRequirement, Get, LockableCurrency, ReservableCurrency, WithdrawReasons,
	},
	PalletId,
};
pub use frame_system::{ensure_signed, pallet_prelude::*, RawOrigin};
pub use scale_info::{prelude::vec, TypeInfo};
pub use sp_std::boxed::Box;
pub use sp_runtime::{
	traits::{BadOrigin, BlakeTwo256, IdentityLookup},
	Perbill,
};

pub type BalanceOf<T> =
	<<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;

#[derive(Clone, Encode, Decode, PartialEq, Eq, TypeInfo, Copy)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum VoteResult {
	AWAITING,
	ACCEPTED,
	REJECTED,
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct RepVote<T: Config> {
	///Asset owner who made the proposal
	pub caller_account: T::AccountId,
	///Virtual account corresponding to the asset
	pub virtual_account: T::AccountId,
	///Candidate for the representative role
	pub candidate_account: T::AccountId,
	///Vote result
	pub vote_result: VoteResult,
	///Session creation block 
	pub when: BlockNumberOf<T>,
	
}

impl<T: Config> RepVote<T> {
	pub fn new(
		caller_account: T::AccountId,
		virtual_account: T::AccountId,
		candidate_account: T::AccountId,
		referendum_index: Dem::ReferendumIndex,	
	) -> DispatchResult {
		let vote_result = VoteResult::AWAITING;
		let when = <frame_system::Pallet<T>>::block_number();
		let session = RepVote::<T>{caller_account,virtual_account,candidate_account,vote_result,when};
		ProposalsLog::<T>::insert(referendum_index,session);
		Ok(())
		
	}
}