#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

pub trait WeightInfo {
    fn contribute_to_fund() -> Weight;
    fn withdraw_fund() -> Weight;
    fn house_bidding() -> Weight;
}

/// TODO implement realistic weights for the different functions below
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn contribute_to_fund() -> Weight {
      (10_000 as Weight)
    }
    fn withdraw_fund() -> Weight {
      (10_000 as Weight)
    }
    fn house_bidding() -> Weight {
      (10_000 as Weight)
    }
}