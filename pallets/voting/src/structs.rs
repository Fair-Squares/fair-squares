pub use super::*;

pub use frame_support::{
    codec::{Decode, Encode},
    RuntimeDebug,
    traits::{Currency},
};

use scale_info::TypeInfo;

pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct VoteProposal<T: Config, U> {
    pub account_id: AccountIdOf<T>,
    pub proposal_call: U,
    pub proposal_index: u32,
    pub referendum_index: u32,
}
impl<T: Config, U> VoteProposal<T, U> {
    pub fn new(account_id: AccountIdOf<T>, proposal_call: U, proposal_index: u32) -> VoteProposal<T, U> {
        Self { 
            account_id, 
            proposal_call, 
            proposal_index,
            referendum_index: 100,
        }
    }
}