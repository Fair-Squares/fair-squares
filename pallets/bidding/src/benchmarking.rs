//! Benchmarking setup for pallet-template

use super::*;

#[allow(unused)]
use crate::Pallet as Bidding;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
