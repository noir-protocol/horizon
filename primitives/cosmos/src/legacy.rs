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

use crate::error::DecodeTxError;
#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sp_core::hashing::sha2_256;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct Coin {
	pub amount: String,
	pub denom: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct AminoSignFee {
	pub amount: Vec<Coin>,
	pub gas: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct MsgSend {
	pub amount: Vec<Coin>,
	pub from_address: String,
	pub to_address: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct Any {
	pub r#type: String,
	pub value: Value,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct AminoSignDoc {
	pub account_number: String,
	pub chain_id: String,
	pub fee: AminoSignFee,
	pub memo: String,
	pub msgs: Vec<Any>,
	pub sequence: String,
}

#[cfg(all(feature = "std", feature = "with-serde"))]
impl AminoSignDoc {
	pub fn new(
		tx: &cosmrs::Tx,
		signer: &cosmrs::tx::SignerInfo,
		chain_id: String,
	) -> Result<Self, DecodeTxError> {
		let fee = AminoSignFee {
			amount: tx
				.auth_info
				.fee
				.amount
				.iter()
				.map(|amt| Coin { amount: amt.amount.to_string(), denom: amt.denom.to_string() })
				.collect(),
			gas: tx.auth_info.fee.gas_limit.to_string(),
		};
		let mut msgs = Vec::<Any>::new();
		for msg in tx.body.messages.clone().into_iter() {
			match msg.type_url.as_str() {
				"/cosmos.bank.v1beta1.MsgSend" => {
					let cosmrs::proto::cosmos::bank::v1beta1::MsgSend {
						from_address,
						to_address,
						amount,
					} = msg.to_msg().map_err(|_| DecodeTxError::InvalidMsgData)?;
					let amount = amount
						.iter()
						.map(|amt| Coin { amount: amt.amount.clone(), denom: amt.denom.clone() })
						.collect::<Vec<Coin>>();
					let value = serde_json::to_value(MsgSend { from_address, to_address, amount })
						.map_err(|_| DecodeTxError::InvalidMsgData)?;

					msgs.push(Any { r#type: "cosmos-sdk/MsgSend".to_string(), value });
				},
				_ => {
					return Err(DecodeTxError::InvalidMsgData);
				},
			}
		}

		Ok(Self {
			chain_id,
			sequence: signer.sequence.to_string(),
			account_number: "0".to_string(),
			fee,
			memo: tx.body.memo.clone(),
			msgs,
		})
	}

	pub fn bytes(&self) -> Result<Vec<u8>, DecodeTxError> {
		Ok(serde_json::to_value(self)
			.map_err(|_| DecodeTxError::InvalidSignDoc)?
			.to_string()
			.as_bytes()
			.to_vec())
	}

	pub fn hash(&self) -> Result<[u8; 32], DecodeTxError> {
		Ok(sha2_256(&self.bytes()?))
	}
}

#[cfg(test)]
#[cfg(feature = "with-serde")]
mod tests {
	use super::AminoSignDoc;
	use base64ct::{Base64, Encoding};

	#[test]
	fn test_sign_amino_doc_hash() {
		let tx_bytes =  "CpoBCpcBChwvY29zbW9zLmJhbmsudjFiZXRhMS5Nc2dTZW5kEncKLWNvc21vczFxZDY5bnV3ajk1Z3RhNGFramd5eHRqOXVqbXo0dzhlZG1xeXNxdxItY29zbW9zMW41amd4NjR6dzM4c3M3Nm16dXU0dWM3amV5cXcydmZqazYwZmR6GhcKBGFjZHQSDzEwMDAwMDAwMDAwMDAwMBJsCk4KRgofL2Nvc21vcy5jcnlwdG8uc2VjcDI1NmsxLlB1YktleRIjCiECChCRNB/lZkv6F4LV4Ed5aJBoyRawTLNl7DFTdVaE2aESBAoCCH8SGgoSCgRhY2R0EgoxMDQwMDAwMDAwEIDa8esEGkBgXIiPoBpecG7QpKDJPaztFogqvmxjDHF5ORfWBrOoSzf0+AAmch1CXrG4OmiKL0y8v9ITx0QzWYUc7ueXcdIm";
		let tx_bytes = Base64::decode_vec(tx_bytes).unwrap();
		let tx = cosmrs::Tx::from_bytes(&tx_bytes).unwrap();
		let signer_info = tx.auth_info.signer_infos.first().unwrap();
		let sign_doc = AminoSignDoc::new(&tx, signer_info, "dev".to_string()).unwrap();
		let hash = &sign_doc.hash().unwrap();
		assert_eq!(
			array_bytes::bytes2hex("", hash),
			"714d4bdfdbd0bd630ebdf93b1f6eba7d3c752e92bbab6c9d3d9c93e1777348bb"
		);
	}
}
