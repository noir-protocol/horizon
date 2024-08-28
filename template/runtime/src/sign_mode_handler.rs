// This file is part of Hrozion.

// Copyright (C) 2023 Haderech Pte. Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use alloc::{string::ToString, vec::Vec};
use cosmos_sdk_proto::{
	cosmos::{
		bank::v1beta1::MsgSend,
		tx::v1beta1::{
			mode_info::{Single, Sum},
			ModeInfo, SignDoc, Tx, TxRaw,
		},
	},
	cosmwasm::wasm::v1::{
		MsgExecuteContract, MsgInstantiateContract2, MsgMigrateContract, MsgStoreCode,
		MsgUpdateAdmin,
	},
	traits::Message,
};
use pallet_cosmos_x_auth_migrations::legacytx::stdsign::StdSignDoc;
use pallet_cosmos_x_auth_signing::{
	any_match,
	sign_mode_handler::{SignModeHandlerError, SignerData},
};
use pallet_cosmos_x_bank_types::msgs::msg_send;
use pallet_cosmos_x_wasm_types::tx::{
	msg_execute_contract, msg_instantiate_contract2, msg_migrate_contract, msg_store_code,
	msg_update_admin,
};
use serde_json::{Map, Value};

pub struct SignModeHandler;

impl pallet_cosmos_x_auth_signing::sign_mode_handler::SignModeHandler for SignModeHandler {
	fn get_sign_bytes(
		mode: &ModeInfo,
		data: &SignerData,
		tx: &Tx,
	) -> Result<Vec<u8>, SignModeHandlerError> {
		let sum = mode.sum.as_ref().ok_or(SignModeHandlerError::EmptyModeInfo)?;
		let sign_bytes = match sum {
			Sum::Single(Single { mode }) => match mode {
				1 /* SIGN_MODE_DIRECT */ => {
					let tx_raw = TxRaw::decode(&mut &*tx.encode_to_vec()).map_err(|_| SignModeHandlerError::DecodeTxError)?;
					SignDoc {
						body_bytes: tx_raw.body_bytes,
						auth_info_bytes: tx_raw.auth_info_bytes,
						chain_id: data.chain_id.clone(),
						account_number: data.account_number,
					}.encode_to_vec()
				},
				127 /* SIGN_MODE_LEGACY_AMINO_JSON */ => {
					let fee = tx.auth_info.as_ref().and_then(|auth_info| auth_info.fee.as_ref()).ok_or(SignModeHandlerError::EmptyFee)?;
					let body = tx.body.as_ref().ok_or(SignModeHandlerError::EmptyTxBody)?;

					let mut coins = Vec::<Value>::new();
					for amt in fee.amount.iter() {
						let mut coin = Map::new();
						coin.insert("amount".to_string(), Value::String(amt.amount.clone()));
						coin.insert("denom".to_string(), Value::String(amt.denom.clone()));

						coins.push(Value::Object(coin));
					}

					let mut std_fee = Map::new();
					std_fee.insert("gas".to_string(), Value::String(fee.gas_limit.to_string()));
					std_fee.insert("amount".to_string(), Value::Array(coins));

					let mut msgs = Vec::<Value>::new();
					for msg in body.messages.iter() {
						let sign_msg = any_match!(
							msg, {
								MsgSend => MsgSend::decode(&mut &*msg.value).as_ref().map(msg_send::get_sign_bytes).map_err(|_| SignModeHandlerError::InvalidMsg),
								MsgStoreCode => MsgStoreCode::decode(&mut &*msg.value).as_ref().map(msg_store_code::get_sign_bytes).map_err(|_| SignModeHandlerError::InvalidMsg),
								MsgInstantiateContract2 => MsgInstantiateContract2::decode(&mut &*msg.value).as_ref().map(msg_instantiate_contract2::get_sign_bytes).map_err(|_| SignModeHandlerError::InvalidMsg),
								MsgExecuteContract => MsgExecuteContract::decode(&mut &*msg.value).as_ref().map(msg_execute_contract::get_sign_bytes).map_err(|_| SignModeHandlerError::InvalidMsg),
								MsgMigrateContract => MsgMigrateContract::decode(&mut &*msg.value).as_ref().map(msg_migrate_contract::get_sign_bytes).map_err(|_| SignModeHandlerError::InvalidMsg),
								MsgUpdateAdmin => MsgUpdateAdmin::decode(&mut &*msg.value).as_ref().map(msg_update_admin::get_sign_bytes).map_err(|_| SignModeHandlerError::InvalidMsg),
							},
							Err(SignModeHandlerError::InvalidMsg))?;

						msgs.push(sign_msg);
					}

					let sign_doc = StdSignDoc {
						account_number: data.account_number.to_string(),
						chain_id: data.chain_id.clone(),
						fee: Value::Object(std_fee),
						memo: body.memo.clone(),
						msgs,
						sequence: data.sequence.to_string(),
					};
					serde_json::to_value(sign_doc).map_err(|_| SignModeHandlerError::SerializeError)?.to_string().as_bytes().to_vec()
				},
				_ => return Err(SignModeHandlerError::InvalidMode),
			},
			_ => return Err(SignModeHandlerError::InvalidMode),
		};

		Ok(sign_bytes)
	}
}

#[cfg(test)]
mod tests {
	use crate::sign_mode_handler::SignModeHandler;
	use base64ct::{Base64, Encoding};
	use cosmos_sdk_proto::{
		cosmos::tx::v1beta1::{
			mode_info::{Single, Sum},
			ModeInfo, Tx,
		},
		prost::Message,
	};
	use pallet_cosmos_x_auth_signing::sign_mode_handler::{SignModeHandler as _, SignerData};
	use sp_core::sha2_256;

	#[test]
	fn get_sign_bytes_test() {
		let tx_raw = "CpMBCpABChwvY29zbW9zLmJhbmsudjFiZXRhMS5Nc2dTZW5kEnAKLWNvc21vczFxZDY5bnV3ajk1Z3RhNGFramd5eHRqOXVqbXo0dzhlZG1xeXNxdxItY29zbW9zMWdtajJleGFnMDN0dGdhZnBya2RjM3Q4ODBncm1hOW53ZWZjZDJ3GhAKBXVhdG9tEgcxMDAwMDAwEnEKTgpGCh8vY29zbW9zLmNyeXB0by5zZWNwMjU2azEuUHViS2V5EiMKIQIKEJE0H+VmS/oXgtXgR3lokGjJFrBMs2XsMVN1VoTZoRIECgIIARIfChUKBXVhdG9tEgw4ODY4ODAwMDAwMDAQgMDxxZSVFBpA9+DRmMYoIcxYF8jpNfUjMIMB4pgZ9diC8ySbnhc6YU84AA3b/0RsCr+nx9AZ27FwcrKJM/yBh8lz+/A9BFn3bg==";

		let tx_raw = Base64::decode_vec(&tx_raw).unwrap();
		let tx = Tx::decode(&mut &*tx_raw).unwrap();

		let public_key = tx
			.auth_info
			.as_ref()
			.unwrap()
			.signer_infos
			.first()
			.unwrap()
			.public_key
			.as_ref()
			.unwrap();

		let mode = ModeInfo { sum: Some(Sum::Single(Single { mode: 1 })) };
		let data = SignerData {
			address: "cosmos1qd69nuwj95gta4akjgyxtj9ujmz4w8edmqysqw".to_string(),
			chain_id: "theta-testnet-001".to_string(),
			account_number: 754989,
			sequence: 0,
			pub_key: public_key.clone(),
		};
		let expected_hash = sha2_256(&SignModeHandler::get_sign_bytes(&mode, &data, &tx).unwrap());

		let sign_doc_raw =
		"CpMBCpABChwvY29zbW9zLmJhbmsudjFiZXRhMS5Nc2dTZW5kEnAKLWNvc21vczFxZDY5bnV3ajk1Z3RhNGFramd5eHRqOXVqbXo0dzhlZG1xeXNxdxItY29zbW9zMWdtajJleGFnMDN0dGdhZnBya2RjM3Q4ODBncm1hOW53ZWZjZDJ3GhAKBXVhdG9tEgcxMDAwMDAwEnEKTgpGCh8vY29zbW9zLmNyeXB0by5zZWNwMjU2azEuUHViS2V5EiMKIQIKEJE0H+VmS/oXgtXgR3lokGjJFrBMs2XsMVN1VoTZoRIECgIIARIfChUKBXVhdG9tEgw4ODY4ODAwMDAwMDAQgMDxxZSVFBoRdGhldGEtdGVzdG5ldC0wMDEgrYou";
		let hash = sha2_256(&Base64::decode_vec(&sign_doc_raw).unwrap());

		assert_eq!(expected_hash, hash);
	}

	#[test]
	fn get_std_sign_bytes_test() {
		let tx_raw =  "CpoBCpcBChwvY29zbW9zLmJhbmsudjFiZXRhMS5Nc2dTZW5kEncKLWNvc21vczFxZDY5bnV3ajk1Z3RhNGFramd5eHRqOXVqbXo0dzhlZG1xeXNxdxItY29zbW9zMW41amd4NjR6dzM4c3M3Nm16dXU0dWM3amV5cXcydmZqazYwZmR6GhcKBGFjZHQSDzEwMDAwMDAwMDAwMDAwMBJsCk4KRgofL2Nvc21vcy5jcnlwdG8uc2VjcDI1NmsxLlB1YktleRIjCiECChCRNB/lZkv6F4LV4Ed5aJBoyRawTLNl7DFTdVaE2aESBAoCCH8SGgoSCgRhY2R0EgoxMDQwMDAwMDAwEIDa8esEGkBgXIiPoBpecG7QpKDJPaztFogqvmxjDHF5ORfWBrOoSzf0+AAmch1CXrG4OmiKL0y8v9ITx0QzWYUc7ueXcdIm";
		let tx_raw = Base64::decode_vec(tx_raw).unwrap();
		let tx = Tx::decode(&mut &*tx_raw).unwrap();

		let public_key = tx
			.auth_info
			.as_ref()
			.unwrap()
			.signer_infos
			.first()
			.unwrap()
			.public_key
			.as_ref()
			.unwrap();

		let mode = ModeInfo { sum: Some(Sum::Single(Single { mode: 127 })) };
		let data = SignerData {
			address: "cosmos1qd69nuwj95gta4akjgyxtj9ujmz4w8edmqysqw".to_string(),
			chain_id: "dev".to_string(),
			account_number: 0,
			sequence: 0,
			pub_key: public_key.clone(),
		};
		let hash = sha2_256(&SignModeHandler::get_sign_bytes(&mode, &data, &tx).unwrap());
		let hash = hex::encode(&hash);

		assert_eq!(hash, "714d4bdfdbd0bd630ebdf93b1f6eba7d3c752e92bbab6c9d3d9c93e1777348bb");
	}
}
