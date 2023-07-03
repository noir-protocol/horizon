// SPDX-License-Identifier: Apache-2.0
// This file is part of Frontier.
//
// Copyright (c) 2020-2022 Parity Technologies (UK) Ltd.
//
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

use frame_support::weights::Weight;
use sp_core::Get;
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_cosmos.
pub trait WeightInfo {
	fn transact(tx: &hp_cosmos::Tx) -> Weight;
}

/// Weights for pallet_cosmos using the Horizon node and recommended hardware.
pub struct HorizonWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for HorizonWeight<T> {
	fn transact(tx: &hp_cosmos::Tx) -> Weight {
		let total_weight = Weight::default();
		for msg in tx.body.messages.iter() {
			match msg {
				hp_cosmos::Msg::MsgSend { .. } => {
					use pallet_balances::WeightInfo;
					total_weight.saturating_add(
						pallet_balances::weights::SubstrateWeight::<T>::transfer()
							.saturating_add(T::DbWeight::get().reads(2)),
					);
				},
			}
		}
		total_weight
	}
}
