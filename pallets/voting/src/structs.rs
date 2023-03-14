pub use super::*;

pub use frame_support::{
	codec::{Decode, Encode},
	traits::Currency,
	RuntimeDebug,
};

use scale_info::{prelude::boxed::Box, TypeInfo};

pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type BalanceOf<T> = <<T as Config>::LocalCurrency as Currency<AccountIdOf<T>>>::Balance;
pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct ProposalParams<T: Config> {
	pub call: Box<<T as Config>::Call>,
	pub hash: T::Hash,
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct CollectiveParams<T: Config, U> {
	pub call_pass: Box<<T as Config>::Call>,
	pub call_fail: Box<<T as Config>::Call>,
	pub index: u32,
	pub call: U,
	pub hash: T::Hash,
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct DemocracyParams<T: Config> {
	pub call_fail: Box<<T as Config>::Call>,
	pub hash: T::Hash,
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct VotingProposal<T: Config, U> {
	pub account_id: AccountIdOf<T>,
	pub proposal_call: Box<<T as Config>::Call>,
	pub proposal_hash: T::Hash,
	pub collective_call: U,
	pub collective_passed_call: Box<<T as Config>::Call>,
	pub collective_failed_call: Box<<T as Config>::Call>,
	pub collective_index: u32,
	pub collective_hash: T::Hash,
	pub collective_step: bool,
	pub collective_closed: bool,
	pub democracy_failed_call: Box<<T as Config>::Call>,
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
