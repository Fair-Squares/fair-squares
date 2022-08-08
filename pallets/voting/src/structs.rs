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

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct VotingProposal<T: Config, U, V> {
    pub account_id: AccountIdOf<T>,
    pub proposal_call: V,
    pub proposal_hash: T::Hash,
    pub collective_index: u32,
    pub collective_call: U,
    pub collective_hash: T::Hash,
    pub democracy_referendum_index: u32,
    pub democracy_hash: T::Hash,
}
impl<T: Config, U, V> VotingProposal<T, U, V> {
    pub fn new(
        account_id: AccountIdOf<T>, 
        proposal_call: V,
        proposal_hash: T::Hash,
        collective_index: u32, 
        collective_call: U,
        collective_hash: T::Hash,
        democracy_hash: T::Hash
    ) -> VotingProposal<T, U, V> {
        Self { 
            account_id, 
            proposal_call, 
            proposal_hash,
            collective_index,
            collective_call,
            collective_hash,
            democracy_referendum_index: 0,
            democracy_hash
        }
    }
}