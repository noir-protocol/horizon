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

use alloc::{string::String, vec, vec::Vec};
use cosmos_sdk_proto::{cosmwasm::wasm, prost::Message, Any};
use pallet_cosmos_types::{coin::Coin, tx_msgs::Msg};
use pallet_cosmos_x_auth_migrations::legacytx::stdsign::LegacyMsg;
use serde::{Deserialize, Serialize};

pub mod msg_store_code {
	use super::*;

	#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
	pub struct AccessConfig {
		pub addresses: Vec<String>,
		pub permission: i32,
	}

	impl From<wasm::v1::AccessConfig> for AccessConfig {
		fn from(config: wasm::v1::AccessConfig) -> Self {
			Self { addresses: config.addresses, permission: config.permission }
		}
	}

	#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
	pub struct MsgStoreCode {
		pub instantiate_permission: Option<AccessConfig>,
		pub sender: String,
		pub wasm_byte_code: Vec<u8>,
	}

	impl TryFrom<&Any> for MsgStoreCode {
		type Error = ();

		fn try_from(any: &Any) -> Result<Self, Self::Error> {
			let msg = wasm::v1::MsgStoreCode::decode(&mut &*any.value).map_err(|_| ())?;
			Ok(Self {
				instantiate_permission: msg.instantiate_permission.map(Into::into),
				sender: msg.sender,
				wasm_byte_code: msg.wasm_byte_code,
			})
		}
	}

	impl LegacyMsg for MsgStoreCode {
		const AMINO_NAME: &'static str = "wasm/MsgStoreCode";
	}

	impl Msg for MsgStoreCode {
		fn get_signers(self) -> Vec<String> {
			vec![self.sender.clone()]
		}
	}
}

pub mod msg_instantiate_contract2 {
	use super::*;

	#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
	pub struct MsgInstantiateContract2 {
		pub admin: String,
		pub code_id: u64,
		pub fix_msg: bool,
		pub funds: Vec<Coin>,
		pub label: String,
		pub msg: Vec<u8>,
		pub salt: Vec<u8>,
		pub sender: String,
	}

	impl TryFrom<&Any> for MsgInstantiateContract2 {
		type Error = ();

		fn try_from(any: &Any) -> Result<Self, Self::Error> {
			let msg =
				wasm::v1::MsgInstantiateContract2::decode(&mut &*any.value).map_err(|_| ())?;
			Ok(Self {
				admin: msg.admin,
				code_id: msg.code_id,
				fix_msg: msg.fix_msg,
				funds: msg.funds.iter().map(Into::into).collect(),
				label: msg.label,
				msg: msg.msg,
				salt: msg.salt,
				sender: msg.sender,
			})
		}
	}

	impl LegacyMsg for MsgInstantiateContract2 {
		const AMINO_NAME: &'static str = "wasm/MsgInstantiateContract2";
	}

	impl Msg for MsgInstantiateContract2 {
		fn get_signers(self) -> Vec<String> {
			vec![self.sender.clone()]
		}
	}
}

pub mod msg_execute_contract {
	use super::*;

	#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
	pub struct MsgExecuteContract {
		pub contract: String,
		pub funds: Vec<Coin>,
		pub msg: Vec<u8>,
		pub sender: String,
	}

	impl TryFrom<&Any> for MsgExecuteContract {
		type Error = ();

		fn try_from(any: &Any) -> Result<Self, Self::Error> {
			let msg = wasm::v1::MsgExecuteContract::decode(&mut &*any.value).map_err(|_| ())?;
			Ok(Self {
				contract: msg.contract,
				funds: msg.funds.iter().map(Into::into).collect(),
				msg: msg.msg,
				sender: msg.sender,
			})
		}
	}

	impl LegacyMsg for MsgExecuteContract {
		const AMINO_NAME: &'static str = "wasm/MsgExecuteContract";
	}

	impl Msg for MsgExecuteContract {
		fn get_signers(self) -> Vec<String> {
			vec![self.sender.clone()]
		}
	}
}

pub mod msg_migrate_contract {
	use super::*;

	#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
	pub struct MsgMigrateContract {
		pub code_id: u64,
		pub contract: String,
		pub msg: Vec<u8>,
		pub sender: String,
	}

	impl TryFrom<&Any> for MsgMigrateContract {
		type Error = ();

		fn try_from(any: &Any) -> Result<Self, Self::Error> {
			let msg = wasm::v1::MsgMigrateContract::decode(&mut &*any.value).map_err(|_| ())?;
			Ok(Self {
				code_id: msg.code_id,
				contract: msg.contract,
				msg: msg.msg,
				sender: msg.sender,
			})
		}
	}

	impl LegacyMsg for MsgMigrateContract {
		const AMINO_NAME: &'static str = "wasm/MsgMigrateContract";
	}

	impl Msg for MsgMigrateContract {
		fn get_signers(self) -> Vec<String> {
			vec![self.sender.clone()]
		}
	}
}

pub mod msg_update_admin {
	use super::*;

	#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
	pub struct MsgUpdateAdmin {
		pub contract: String,
		pub new_admin: String,
		pub sender: String,
	}

	impl TryFrom<&Any> for MsgUpdateAdmin {
		type Error = ();

		fn try_from(any: &Any) -> Result<Self, Self::Error> {
			let msg = wasm::v1::MsgUpdateAdmin::decode(&mut &*any.value).map_err(|_| ())?;
			Ok(Self { contract: msg.contract, new_admin: msg.new_admin, sender: msg.sender })
		}
	}

	impl LegacyMsg for MsgUpdateAdmin {
		const AMINO_NAME: &'static str = "wasm/MsgUpdateAdmin";
	}

	impl Msg for MsgUpdateAdmin {
		fn get_signers(self) -> Vec<String> {
			vec![self.sender.clone()]
		}
	}
}
