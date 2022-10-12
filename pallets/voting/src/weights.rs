// This file is part of Substrate.

// Copyright (C) 2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Autogenerated weights for pallet_template
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-07-11, STEPS: `100`, REPEAT: 40, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `ubuntu`, CPU: `AMD Ryzen 7 4800H with Radeon Graphics`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/ld-node
// benchmark
// pallet
// --chain
// dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet
// pallet_template
// --extrinsic
// *
// --steps
// 100
// --repeat
// 40
// --output
// pallets/template/src/weights.rs
// --template
// assets/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_template.
pub trait WeightInfo {
	fn do_something(s: u32, ) -> Weight;
}

/// Weights for pallet_template using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	// Storage: TemplateModule Something (r:0 w:1)
	/// The range of component `s` is `[0, 100]`.
	fn do_something(_s: u32, ) -> Weight {
		Weight::from_ref_time(
			(15_009_000 as u64)
			.saturating_add(T::DbWeight::get().writes(1).ref_time())
		)
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: TemplateModule Something (r:0 w:1)
	/// The range of component `s` is `[0, 100]`.
	fn do_something(_s: u32, ) -> Weight {
		Weight::from_ref_time(
			(15_009_000 as u64)
			.saturating_add(RocksDbWeight::get().writes(1).ref_time())
		)
	}
}