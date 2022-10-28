
//! Autogenerated weights for `pallet_roles`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-10-28, STEPS: `20`, REPEAT: 10, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `ubuntu`, CPU: `AMD Ryzen 9 4900HS with Radeon Graphics`
//! EXECUTION: None, WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/fs-node
// benchmark
// pallet
// --chain
// dev
// --pallet
// pallet_roles
// --extrinsic
// *
// --steps
// 20
// --repeat
// 10
// --output
// pallets/roles/src/weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight}};
use sp_std::marker::PhantomData;

pub trait WeightInfo {
	fn investor(_:u32) -> Weight;
	fn approval(_:u32) -> Weight;
	fn rejection(_:u32) -> Weight;
	fn set_admin(_:u32) -> Weight;
}

/// Weight functions for `pallet_roles`.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	// Storage: RoleModule HouseSellerLog (r:1 w:0)
	// Storage: RoleModule InvestorLog (r:1 w:1)
	// Storage: RoleModule ServicerLog (r:1 w:0)
	// Storage: RoleModule TenantLog (r:1 w:0)
	// Storage: RoleModule TotalMembers (r:1 w:1)
	// Storage: RoleModule AccountsRolesLog (r:0 w:1)
	/// The range of component `b` is `[0, 200]`.
	fn investor(_b: u32, ) -> Weight {
		Weight::from_ref_time(49_629_000 as u64)
			.saturating_add(T::DbWeight::get().reads(5 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	// Storage: Sudo Key (r:1 w:0)
	// Storage: RoleModule TotalMembers (r:1 w:1)
	// Storage: RoleModule RoleApprovalList (r:1 w:1)
	// Storage: RoleModule HouseSellerLog (r:0 w:1)
	// Storage: RoleModule AccountsRolesLog (r:0 w:1)
	/// The range of component `b` is `[0, 200]`.
	fn approval(_b: u32, ) -> Weight {
		Weight::from_ref_time(45_444_000 as u64)
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	// Storage: Sudo Key (r:1 w:0)
	// Storage: RoleModule RoleApprovalList (r:1 w:1)
	/// The range of component `b` is `[0, 200]`.
	fn rejection(_b: u32, ) -> Weight {
		Weight::from_ref_time(40_553_000 as u64)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Sudo Key (r:1 w:1)
	// Storage: RoleModule ServicerLog (r:1 w:2)
	// Storage: RoleModule AccountsRolesLog (r:1 w:1)
	// Storage: RoleModule RoleApprovalList (r:1 w:1)
	/// The range of component `b` is `[0, 200]`.
	fn set_admin(_b: u32, ) -> Weight {
		Weight::from_ref_time(61_986_000 as u64)
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(5 as u64))
	}
}
