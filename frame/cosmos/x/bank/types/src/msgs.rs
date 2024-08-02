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
	use core::str::FromStr;
	use cosmos_sdk_proto::{
		cosmos::bank::v1beta1::MsgSend,
		prost::alloc::string::{String, ToString},
	};
	use serde_json::{Map, Value};
	use sp_std::vec::Vec;

	const AMINO_NAME: &str = "cosmos-sdk/MsgSend";

	pub fn sign_bytes(msg: MsgSend) -> Vec<u8> {
		let mut value = Map::new();
		value.insert(String::from_str("from_address").unwrap(), Value::String(msg.from_address));
		value.insert(String::from_str("to_address").unwrap(), Value::String(msg.to_address));

		let mut coins = Vec::<Value>::new();
		for amt in msg.amount.iter() {
			let mut coin = Map::new();
			coin.insert(String::from_str("amount").unwrap(), Value::String(amt.amount.clone()));
			coin.insert(String::from_str("denom").unwrap(), Value::String(amt.denom.clone()));
			coins.push(Value::Object(coin));
		}

		value.insert(String::from_str("amount").unwrap(), Value::Array(coins));

		let mut legacy_msg = Map::new();
		legacy_msg.insert(
			String::from_str("type").unwrap(),
			Value::String(String::from_str(AMINO_NAME).unwrap()),
		);
		legacy_msg.insert(String::from_str("value").unwrap(), Value::Object(value));

		Value::Object(legacy_msg).to_string().into_bytes()
	}
}
