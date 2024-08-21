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

use cosmos_sdk_proto::{
	cosmwasm::wasm::v1::{MsgInstantiateContract2, MsgStoreCode},
	prost::alloc::string::{String, ToString},
};
use serde_json::{Map, Value};
use sp_std::{vec, vec::Vec};

pub mod msg_store_code {
	use super::*;

	pub fn get_sign_bytes(msg: &MsgStoreCode) -> Value {
		let mut value = Map::new();
		value.insert("sender".to_string(), Value::String(msg.sender.clone()));
		value.insert("wasm_byte_code".to_string(), Value::from(msg.wasm_byte_code.clone()));

		if let Some(config) = msg.instantiate_permission.clone() {
			let mut permission = Map::new();
			permission.insert("permission".to_string(), Value::from(config.permission));
			permission.insert(
				"addresses".to_string(),
				Value::Array(
					config.addresses.into_iter().map(Value::String).collect::<Vec<Value>>(),
				),
			);

			value.insert("instantiate_permission".to_string(), Value::Object(permission));
		} else {
			value.insert("instantiate_permission".to_string(), Value::Null);
		}

		Value::Object(value)
	}

	pub fn get_signers(msg: &MsgStoreCode) -> Vec<String> {
		vec![msg.sender.clone()]
	}
}

pub mod msg_instantiate_contract2 {
	use super::*;

	pub fn get_sign_bytes(msg: &MsgInstantiateContract2) -> Value {
		let mut value = Map::new();

		value.insert("sender".to_string(), Value::String(msg.sender.clone()));
		value.insert("admin".to_string(), Value::String(msg.admin.clone()));
		value.insert("code_id".to_string(), Value::from(msg.code_id));
		value.insert("label".to_string(), Value::String(msg.label.clone()));
		value.insert(
			"msg".to_string(),
			Value::Array(msg.msg.clone().into_iter().map(Value::from).collect::<Vec<Value>>()),
		);
		let funds = msg
			.funds
			.clone()
			.into_iter()
			.map(|coin| {
				let mut fund = Map::new();
				fund.insert("denom".to_string(), Value::String(coin.denom.clone()));
				fund.insert("amount".to_string(), Value::String(coin.amount.clone()));
				fund
			})
			.map(Value::Object)
			.collect::<Vec<Value>>();
		value.insert("funds".to_string(), Value::Array(funds));
		value.insert(
			"salt".to_string(),
			Value::Array(msg.salt.clone().into_iter().map(Value::from).collect::<Vec<Value>>()),
		);
		value.insert("fix_msg".to_string(), Value::Bool(msg.fix_msg));

		Value::Object(value)
	}

	pub fn get_signers(msg: &MsgInstantiateContract2) -> Vec<String> {
		vec![msg.sender.clone()]
	}
}
