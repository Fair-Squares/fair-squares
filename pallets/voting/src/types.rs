pub use super::*;
pub use frame_support::{
	ensure,
    pallet_prelude::*,
	error::BadOrigin,
	dispatch::{GetDispatchInfo,PostDispatchInfo},
	traits::{
		defensive_prelude::*,UnfilteredDispatchable,IsSubType,
		schedule::{v3::Named as ScheduleNamed, DispatchTime},
		Bounded, Currency, EnsureOrigin, Get, Hash as PreimageHash, LockIdentifier,
		LockableCurrency, OnUnbalanced, QueryPreimage, ReservableCurrency, StorePreimage,
		WithdrawReasons,
	},
	weights::Weight,
};
pub use frame_system::{ensure_signed, ensure_root, pallet_prelude::*, RawOrigin,WeightInfo};
pub use frame_system::pallet_prelude::{BlockNumberFor, OriginFor};
pub use frame_system::pallet_prelude::*;
pub use sp_std::prelude::*;
pub use sp_runtime::{
	traits::{Bounded as ArithBounded, One, Saturating, StaticLookup, Zero,Dispatchable},
	ArithmeticError, DispatchError, DispatchResult,
};
pub use scale_info::{prelude::{boxed::Box,vec}, TypeInfo};
pub use serde::{Deserialize, Serialize};

pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type BlockNumberOf<T> = BlockNumberFor<T>;
pub type Coll1Proposal<T> = <T as pallet_collective::Config<Instance1>>::Proposal;

pub type UtilCall<T> = <T as UTIL::Config>::RuntimeCall;
pub type BalanceOf<T> =
	<<T as DEM::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct ProposalParams<T: Config> {
	pub call: Box<<T as Config>::RuntimeCall>,
	pub hash: T::Hash,
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct CollectiveParams<T: Config, U> {
	pub call_pass: <T as Config>::RuntimeCall,
	pub call_fail: Box<<T as Config>::RuntimeCall>,
	pub index: u32,
	pub call: U,
	pub hash: T::Hash,
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct DemocracyParams<T: Config> {
	pub call_fail: Box<<T as Config>::RuntimeCall>,
	pub hash: T::Hash,
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct VotingProposal<T: Config, U> {
	pub account_id: AccountIdOf<T>,
	pub proposal_call: Box<<T as Config>::RuntimeCall>,
	pub proposal_hash: T::Hash,
	pub collective_call: U,
	pub collective_passed_call: <T as Config>::RuntimeCall,
	pub collective_failed_call: Box<<T as Config>::RuntimeCall>,
	pub collective_index: u32,
	pub collective_hash: T::Hash,
	pub collective_step: bool,
	pub collective_closed: bool,
	pub democracy_failed_call: Box<<T as Config>::RuntimeCall>,
	pub democracy_referendum_index: u32,
	pub democracy_hash: T::Hash,
	pub proposal_executed: bool,
}
impl<T: Config, U> VotingProposal<T, U> {
	pub fn new(
		account_id: AccountIdOf<T>,
		proposal: ProposalParams<T>,
		collective: CollectiveParams<T, U>,
		democracy: DemocracyParams<T>,
	) -> VotingProposal<T, U> {
		Self {
			account_id,
			proposal_call: proposal.call,
			proposal_hash: proposal.hash,
			collective_passed_call: collective.call_pass,
			collective_failed_call: collective.call_fail,
			collective_index: collective.index,
			collective_call: collective.call,
			collective_hash: collective.hash,
			democracy_failed_call: democracy.call_fail,
			democracy_hash: democracy.hash,
			democracy_referendum_index: 0,
			proposal_executed: false,
			collective_step: false,
			collective_closed: false,
		}
	}
}