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

use core::str::FromStr;
use cosmos_sdk_proto::{
	cosmos::{
		bank::v1beta1::MsgSend,
		tx::v1beta1::{
			mode_info::{Single, Sum},
			ModeInfo, SignDoc, Tx, TxRaw,
		},
	},
	prost::alloc::string::{String, ToString},
	traits::Message,
};
use pallet_cosmos_x_auth_migrations::legacytx::stdsign::StdSignDoc;
use pallet_cosmos_x_auth_signing::sign_mode_handler::SignerData;
use pallet_cosmos_x_bank_types::msgs::msg_send;
use serde_json::{Map, Value};
use sp_std::vec::Vec;

pub struct SignModeHandler;

impl pallet_cosmos_x_auth_signing::sign_mode_handler::SignModeHandler for SignModeHandler {
	fn get_sign_bytes(mode: &ModeInfo, data: &SignerData, tx: &Tx) -> Result<Vec<u8>, ()> {
		let sum = mode.sum.clone().ok_or(())?;
		let sign_bytes = match sum {
			Sum::Single(Single { mode }) => match mode {
				1 /* SIGN_MODE_DIRECT */ => {
					let tx_raw = TxRaw::decode(&mut &*tx.encode_to_vec()).unwrap();
					SignDoc {
						body_bytes: tx_raw.body_bytes,
						auth_info_bytes: tx_raw.auth_info_bytes,
						chain_id: data.chain_id.clone(),
						account_number: data.account_number,
					}.encode_to_vec()
				},
				127 /* SIGN_MODE_LEGACY_AMINO_JSON */ => {
					let auth_info = tx.auth_info.clone().ok_or(())?;
					let fee = auth_info.fee.ok_or(())?;
					let body = tx.body.clone().ok_or(())?;

					let mut coins = Vec::<Value>::new();
					for amt in fee.amount.iter() {
						let mut coin = Map::new();
						coin.insert(String::from_str("amount").unwrap(), Value::String(amt.amount.clone()));
						coin.insert(String::from_str("denom").unwrap(), Value::String(amt.denom.clone()));
						coins.push(Value::Object(coin));
					}

					let mut std_fee = Map::new();
					std_fee.insert(String::from_str("gas").unwrap(), Value::String(fee.gas_limit.to_string()));
					std_fee.insert(String::from_str("amount").unwrap(), Value::Array(coins));

					let mut msgs = Vec::<Value>::new();
					for message in body.messages.iter() {
						match message.type_url.as_str() {
							"/cosmos.bank.v1beta1.MsgSend" => {
								let msg = MsgSend::decode(&mut &*message.value).unwrap();
								let msg_json = msg_send::get_sign_bytes(&msg);
								msgs.push(msg_json);
							},
							_ => return Err(()),
						}
					}

					let sign_doc = StdSignDoc {
						account_number: data.account_number.to_string(),
						chain_id: data.chain_id.clone(),
						fee: Value::Object(std_fee),
						memo: body.memo,
						msgs,
						sequence: data.sequence.to_string(),
					};
					serde_json::to_value(sign_doc).map_err(|_|())?.to_string().as_bytes().to_vec()
				},
				_ => return Err(()),
			},
			_ => return Err(()),
		};

		Ok(sign_bytes)
	}
}
