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

pub mod msg_send {
	use alloc::{
		string::{String, ToString},
		vec::Vec,
	};
	use cosmos_sdk_proto::cosmos::bank::v1beta1::MsgSend;
	use serde_json::{Map, Value};

	const AMINO_NAME: &str = "cosmos-sdk/MsgSend";

	pub fn get_sign_bytes(msg: &MsgSend) -> Value {
		let mut value = Map::new();
		value.insert("from_address".to_string(), Value::String(msg.from_address.clone()));
		value.insert("to_address".to_string(), Value::String(msg.to_address.clone()));

		let mut coins = Vec::<Value>::new();
		for amt in msg.amount.iter() {
			let mut coin = Map::new();
			coin.insert("amount".to_string(), Value::String(amt.amount.clone()));
			coin.insert("denom".to_string(), Value::String(amt.denom.clone()));

			coins.push(Value::Object(coin));
		}

		value.insert("amount".to_string(), Value::Array(coins));

		let mut legacy_msg = Map::new();
		legacy_msg.insert("type".to_string(), Value::String(AMINO_NAME.to_string()));
		legacy_msg.insert("value".to_string(), Value::Object(value));

		Value::Object(legacy_msg)
	}

	pub fn get_signers(msg: &MsgSend) -> Vec<String> {
		alloc::vec![msg.from_address.clone()]
	}
}
