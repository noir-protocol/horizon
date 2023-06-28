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

mod error;
#[cfg(feature = "std")]
mod legacy;

#[cfg(feature = "std")]
use core::str::FromStr;
#[cfg(feature = "std")]
use cosmrs::tendermint::chain;
#[cfg(feature = "std")]
use cosmrs::tx::SignMode;
#[cfg(feature = "std")]
use cosmrs::{self, tx::MessageExt};
use error::DecodeTxError;
#[cfg(feature = "std")]
use legacy::SignAminoDoc;
#[cfg(feature = "std")]
use sp_core::hashing::sha2_256;
use sp_core::H160;
use sp_std::vec::Vec;

pub type SequenceNumber = u64;
pub type SignatureBytes = Vec<u8>;
pub type Gas = u64;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(codec::Encode, codec::Decode, scale_info::TypeInfo))]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tx {
	pub body: Body,
	pub auth_info: AuthInfo,
	pub signatures: Vec<SignatureBytes>,
	pub hash: [u8; 32],
}

#[cfg(feature = "std")]
impl TryFrom<cosmrs::tx::Tx> for Tx {
	type Error = DecodeTxError;

	fn try_from(tx: cosmrs::tx::Tx) -> Result<Self, Self::Error> {
		let signatures = tx.signatures.iter().map(|s| s.clone()).collect::<Vec<SignatureBytes>>();
		let sign_doc = match tx.auth_info.signer_infos[0].mode_info {
			cosmrs::tx::ModeInfo::Single(single) => match single.mode {
				SignMode::Direct => {
					let chain_id = chain::Id::from_str("noir").unwrap();
					let sign_doc =
						cosmrs::tx::SignDoc::new(&tx.body, &tx.auth_info, &chain_id, 0u64)
							.map_err(|_| DecodeTxError::InvalidTxData)?;
					sign_doc.into_bytes().map_err(|_| DecodeTxError::InvalidSignDoc)?
				},
				SignMode::LegacyAminoJson => SignAminoDoc::try_from(&tx)?.to_bytes()?,
				_ => return Err(DecodeTxError::UnsupportedSigner),
			},
			_ => return Err(DecodeTxError::UnsupportedSigner),
		};
		Ok(Self {
			body: tx.body.try_into()?,
			auth_info: tx.auth_info.into(),
			signatures,
			hash: sha2_256(&sign_doc),
		})
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(codec::Encode, codec::Decode, scale_info::TypeInfo))]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Body {
	pub messages: Vec<Msg>,
}

#[cfg(feature = "std")]
impl TryFrom<cosmrs::tx::Body> for Body {
	type Error = DecodeTxError;

	fn try_from(body: cosmrs::tx::Body) -> Result<Self, Self::Error> {
		let mut messages: Vec<Msg> = Vec::new();
		for msg in body.messages {
			messages.push(msg.try_into()?);
		}
		Ok(Self { messages })
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(codec::Encode, codec::Decode, scale_info::TypeInfo))]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Msg {
	MsgSend { from_address: H160, to_address: H160, amount: u128 },
}

#[cfg(feature = "std")]
impl TryFrom<cosmrs::Any> for Msg {
	type Error = DecodeTxError;

	fn try_from(any: cosmrs::Any) -> Result<Self, Self::Error> {
		Self::try_from(&any)
	}
}

#[cfg(feature = "std")]
impl TryFrom<&cosmrs::Any> for Msg {
	type Error = DecodeTxError;

	fn try_from(any: &cosmrs::Any) -> Result<Self, Self::Error> {
		if any.type_url == "/cosmos.bank.v1beta1.MsgSend" {
			let type_msg = cosmrs::proto::cosmos::bank::v1beta1::MsgSend::from_any(any)
				.map_err(|_| DecodeTxError::InvalidMsgData)?;
			let type_msg = cosmrs::bank::MsgSend::try_from(type_msg)
				.map_err(|_| DecodeTxError::InvalidMsgData)?;
			let amount = type_msg.amount[0].amount;
			let mut from_address: [u8; 20] = [0u8; 20];
			from_address.copy_from_slice(&type_msg.from_address.to_bytes()[..]);
			let mut to_address: [u8; 20] = [0u8; 20];
			to_address.copy_from_slice(&type_msg.to_address.to_bytes()[..]);

			Ok(Msg::MsgSend {
				from_address: from_address.into(),
				to_address: to_address.into(),
				amount,
			})
		} else {
			Err(DecodeTxError::UnsupportedMsgType)
		}
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(codec::Encode, codec::Decode, scale_info::TypeInfo))]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AuthInfo {
	pub signer_infos: Vec<SignerInfo>,
	pub fee: Fee,
}

#[cfg(feature = "std")]
impl From<cosmrs::tx::AuthInfo> for AuthInfo {
	fn from(auth_info: cosmrs::tx::AuthInfo) -> Self {
		let signer_infos = auth_info
			.signer_infos
			.iter()
			.map(|s| s.clone().into())
			.collect::<Vec<SignerInfo>>();
		Self { signer_infos, fee: auth_info.fee.into() }
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(codec::Encode, codec::Decode, scale_info::TypeInfo))]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SignerInfo {
	pub public_key: Option<SignerPublicKey>,
	pub sequence: SequenceNumber,
}

#[cfg(feature = "std")]
impl From<cosmrs::tx::SignerInfo> for SignerInfo {
	fn from(signer_info: cosmrs::tx::SignerInfo) -> Self {
		let public_key = match signer_info.public_key {
			Some(pubkey) => match pubkey {
				cosmrs::tx::SignerPublicKey::Single(p) => match p.type_url() {
					cosmrs::crypto::PublicKey::ED25519_TYPE_URL => {
						let mut raw_bytes: [u8; 32] = [0u8; 32];
						raw_bytes.copy_from_slice(&p.to_bytes()[..]);
						Some(SignerPublicKey::Single(PublicKey::ED25519(raw_bytes)))
					},
					cosmrs::crypto::PublicKey::SECP256K1_TYPE_URL => {
						let mut raw_bytes: [u8; 33] = [0u8; 33];
						raw_bytes.copy_from_slice(&p.to_bytes()[..]);
						Some(SignerPublicKey::Single(PublicKey::SECP256K1(raw_bytes)))
					},
					_ => None,
				},
				_ => None,
			},
			None => None,
		};
		Self { public_key, sequence: signer_info.sequence }
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(codec::Encode, codec::Decode, scale_info::TypeInfo))]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SignerPublicKey {
	/// Single singer.
	Single(PublicKey),
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(codec::Encode, codec::Decode, scale_info::TypeInfo))]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PublicKey {
	ED25519([u8; 32]),
	SECP256K1([u8; 33]),
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(codec::Encode, codec::Decode, scale_info::TypeInfo))]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Fee {
	pub amount: u128,
	pub gas_limit: Gas,
}

#[cfg(feature = "std")]
impl From<cosmrs::tx::Fee> for Fee {
	fn from(fee: cosmrs::tx::Fee) -> Self {
		Self { amount: fee.amount[0].amount, gas_limit: fee.gas_limit }
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(codec::Encode, codec::Decode, scale_info::TypeInfo))]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Account {
	pub sequence: SequenceNumber,
	pub amount: u128,
}

#[cfg(test)]
mod tests {
	use crate::legacy::SignAminoDoc;
	use base64::{engine::general_purpose, Engine};
	use sp_core::hashing::sha2_256;

	#[test]
	fn test_sign_amino_doc_hash() {
		let tx_bytes =  "Cp0BCpgBChwvY29zbW9zLmJhbmsudjFiZXRhMS5Nc2dTZW5kEngKLWNvc21vczFwdnJhbjRkbDl1NzNxNXo0dzNtY2xnbDUzMGtsdHdxY2EwMnk4ZBItY29zbW9zMThwd3ZxajB0ZG5oZ20zM241bG4wMjBqdnk4MjBmcjI5aDJtc213GhgKBHVjZHQSEDEwMDAwMDAwMDAwMDAwMDASABJkClAKRgofL2Nvc21vcy5jcnlwdG8uc2VjcDI1NmsxLlB1YktleRIjCiED9ZPCan9HZlZbW/+hDSWLfy6cW+aPzrjSILmLmCSnUUcSBAoCCH8YABIQCgoKBHVjZHQSAjI1EKCNBhpA0YAS1zXHInFcdO2w/tZjTEWa9fNs53mTsitzpx21mxRVaJv8lJ2eErg+/IWvCWLHfsh71fMxOY2AJ7DrQIzTxg==";
		let tx_bytes = general_purpose::STANDARD.decode(tx_bytes).unwrap();
		let tx = cosmrs::Tx::from_bytes(&tx_bytes).unwrap();
		let sign_doc: SignAminoDoc = tx.try_into().unwrap();
		let hash = sha2_256(&sign_doc.to_bytes().unwrap());
		assert_eq!(
			hex::encode(hash),
			"c853e81f04e499cb842c67b8c75a1e23d60bdc02ee51ff9f5e28925f5d9706a8"
		);
	}
}
