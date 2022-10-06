#![cfg(feature = "runtime-benchmarks")]

use super::*;

#[allow(unused)]
use crate::Pallet as Voting;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::{Call as SystemCall, RawOrigin};

use pallet_roles::Hash;
