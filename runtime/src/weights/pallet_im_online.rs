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

//! Autogenerated weights for pallet_im_online
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-07-31, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("prod-test"), DB CACHE: 1024

// Executed Command:
// ./../target/release/joystream-node
// benchmark
// pallet
// --pallet=pallet_im_online
// --extrinsic=*
// --chain=prod-test
// --steps=50
// --repeat=20
// --execution=wasm
// --template=./../devops/frame-weight-template.hbs
// --output=./../runtime/src/weights/pallet_im_online.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

pub use pallet_im_online::weights::WeightInfo;

/// Weights for pallet_im_online using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	// Storage: Session Validators (r:1 w:0)
	// Proof Skipped: Session Validators (max_values: Some(1), max_size: None, mode: Measured)
	// Storage: Session CurrentIndex (r:1 w:0)
	// Proof Skipped: Session CurrentIndex (max_values: Some(1), max_size: None, mode: Measured)
	// Storage: ImOnline Keys (r:1 w:0)
	// Proof: ImOnline Keys (max_values: Some(1), max_size: Some(320002), added: 320497, mode: MaxEncodedLen)
	// Storage: ImOnline ReceivedHeartbeats (r:1 w:1)
	// Proof: ImOnline ReceivedHeartbeats (max_values: None, max_size: Some(10021032), added: 10023507, mode: MaxEncodedLen)
	// Storage: ImOnline AuthoredBlocks (r:1 w:0)
	// Proof: ImOnline AuthoredBlocks (max_values: None, max_size: Some(56), added: 2531, mode: MaxEncodedLen)
	/// The range of component `k` is `[1, 1000]`.
	/// The range of component `e` is `[1, 100]`.
	fn validate_unsigned_and_then_heartbeat(k: u32, e: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `359 + k * (32 ±0)`
		//  Estimated: `10345712 + k * (64 ±0) + e * (25 ±0)`
		// Minimum execution time: 78_000 nanoseconds.
		Weight::from_parts(45_808_559, 0u64)
			.saturating_add(Weight::from_parts(0, 10345712))
			// Standard Error: 1_668
			.saturating_add(Weight::from_parts(74_644, 0u64).saturating_mul(k.into()))
			// Standard Error: 16_808
			.saturating_add(Weight::from_parts(593_796, 0u64).saturating_mul(e.into()))
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
			.saturating_add(Weight::from_parts(0, 64).saturating_mul(k.into()))
			.saturating_add(Weight::from_parts(0, 25).saturating_mul(e.into()))
	}
}
