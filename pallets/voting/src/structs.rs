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
pub struct VotingProposal<T: Config, U> {
	pub account_id: AccountIdOf<T>,
	pub proposal_call: Box<<T as Config>::Call>,
	pub collective_passed_call: Box<<T as Config>::Call>,
	pub collective_failed_call: Box<<T as Config>::Call>,
	pub democracy_failed_call: Box<<T as Config>::Call>,
	pub proposal_hash: T::Hash,
	pub collective_index: u32,
	pub collective_call: U,
	pub collective_hash: T::Hash,
	pub collective_step: bool,
	pub collective_closed: bool,
	pub democracy_referendum_index: u32,
	pub democracy_hash: T::Hash,
	pub proposal_executed: bool,
}
impl<T: Config, U> VotingProposal<T, U> {
	pub fn new(
		account_id: AccountIdOf<T>,
		proposal_call: Box<<T as Config>::Call>,
		collective_passed_call: Box<<T as Config>::Call>,
		collective_failed_call: Box<<T as Config>::Call>,
		democracy_failed_call: Box<<T as Config>::Call>,
		proposal_hash: T::Hash,
		collective_index: u32,
		collective_call: U,
		collective_hash: T::Hash,
		democracy_hash: T::Hash,
	) -> VotingProposal<T, U> {
		Self {
			account_id,
			proposal_call,
			collective_passed_call,
			collective_failed_call,
			democracy_failed_call,
			proposal_hash,
			collective_index,
			collective_call,
			collective_hash,
			democracy_referendum_index: 0,
			democracy_hash,
			proposal_executed: false,
			collective_step: false,
			collective_closed: false,
		}
	}
}
