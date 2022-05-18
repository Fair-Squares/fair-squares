#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

pub trait WeightInfo {
    fn cause_error() -> Weight;
	fn withdraw() -> Weight;
	fn create_account() -> Weight;
	fn manage_proposal() -> Weight;
	fn withdraw_house_contribution() -> Weight;
	fn add_contribution_fund() -> Weight;
	fn mint_house() -> Weight;
	fn create_proposal() -> Weight;
	fn vote_proposal() -> Weight;
}

/// TODO implement realistic weights for the different functions below
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn cause_error() -> Weight {
		(10_000 as Weight)
	}
    fn withdraw() -> Weight {
		(10_000 as Weight)
	}
    fn create_account() -> Weight {
		(10_000 as Weight)
	}
    fn manage_proposal() -> Weight {
		(10_000 as Weight)
	}
    fn withdraw_house_contribution() -> Weight {
		(10_000 as Weight)
	}
    fn add_contribution_fund() -> Weight {
		(10_000 as Weight)
	}
    fn mint_house() -> Weight {
		(10_000 as Weight)
	}
    fn create_proposal() -> Weight {
		(10_000 as Weight)
	}
    fn vote_proposal() -> Weight {
		(10_000 as Weight)
	}
}
