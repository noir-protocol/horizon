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

use crate::{error::DecodeError, registry, tx::SequenceNumber};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Coin {
	pub amount: String,
	pub denom: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StdFee {
	pub amount: Vec<Coin>,
	pub gas: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegacyMsg {
	pub r#type: String,
	pub value: Value,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StdSignDoc {
	pub account_number: String,
	pub chain_id: String,
	pub fee: StdFee,
	pub memo: String,
	pub msgs: Vec<LegacyMsg>,
	pub sequence: String,
}

impl StdSignDoc {
	pub fn new(
		tx: &cosmrs::Tx,
		chain_id: String,
		account_number: u64,
		sequence: SequenceNumber,
	) -> Result<Self, DecodeError> {
		let fee = StdFee {
			amount: tx
				.auth_info
				.fee
				.amount
				.iter()
				.map(|amt| Coin { amount: amt.amount.to_string(), denom: amt.denom.to_string() })
				.collect(),
			gas: tx.auth_info.fee.gas_limit.to_string(),
		};
		let mut msgs = Vec::<LegacyMsg>::new();
		for msg in tx.body.messages.clone().into_iter() {
			let legacy_msg = match registry::REGISTRY.get() {
				Some(reg) => reg.legacy_msg(&msg.into())?,
				None => return Err(DecodeError::InvalidTypeUrl),
			};
			msgs.push(legacy_msg);
		}

		Ok(Self {
			chain_id,
			sequence: sequence.to_string(),
			account_number: account_number.to_string(),
			fee,
			memo: tx.body.memo.clone(),
			msgs,
		})
	}

	pub fn bytes(&self) -> Result<Vec<u8>, DecodeError> {
		Ok(serde_json::to_value(self)
			.map_err(|_| DecodeError::InvalidSignDoc)?
			.to_string()
			.as_bytes()
			.to_vec())
	}
}
