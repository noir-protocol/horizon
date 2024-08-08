// This file is part of Horizon.

// Copyright (C) 2023 Haderech Pte. Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]

use pallet_cosmos_types::events::AbciEvent;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_runtime::traits::Block as BlockT;
use sp_std::vec::Vec;

#[derive(Clone, Decode, Encode, Debug, TypeInfo, Serialize, Deserialize)]
pub struct GasInfo {
	pub gas_wanted: u64,
	pub gas_used: u64,
}

#[derive(Clone, Decode, Encode, Debug, TypeInfo, Serialize, Deserialize)]
pub struct SimulateResponse {
	pub gas_info: GasInfo,
	pub events: Vec<AbciEvent>,
}

sp_api::decl_runtime_apis! {
	pub trait CosmosTxRuntimeApi {
		fn convert_tx(tx_bytes: Vec<u8>) -> <Block as BlockT>::Extrinsic;

		fn simulate(tx_bytes: Vec<u8>) -> SimulateResponse;
	}
}
