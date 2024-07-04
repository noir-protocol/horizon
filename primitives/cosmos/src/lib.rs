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

pub mod error;
#[cfg(feature = "std")]
mod legacy;
pub mod msgs;

#[cfg(feature = "std")]
use cosmrs::{tendermint::chain, tx::SignMode};
#[cfg(feature = "std")]
use error::DecodeTxError;
#[cfg(feature = "std")]
use legacy::SignAminoDoc;
#[cfg(feature = "with-codec")]
use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "with-codec")]
use scale_info::TypeInfo;
#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "std")]
use sp_core::hashing::sha2_256;
use sp_core::{H160, H256};
#[cfg(not(feature = "std"))]
use sp_std::vec::Vec;
#[cfg(feature = "std")]
use std::str::FromStr;

pub type SequenceNumber = u64;
pub type SignatureBytes = Vec<u8>;
pub type Gas = u64;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo))]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct Tx {
	pub body: Body,
	pub auth_info: AuthInfo,
	pub signatures: Vec<SignatureBytes>,
	pub hash: H256,
	pub len: u32,
}

#[cfg(feature = "std")]
impl Tx {
	pub fn decode(tx_bytes: &[u8], chain_id: &[u8]) -> Result<Self, DecodeTxError> {
		if tx_bytes.is_empty() {
			return Err(DecodeTxError::EmptyTxBytes);
		}

		let tx_origin =
			cosmrs::Tx::from_bytes(tx_bytes).map_err(|_| DecodeTxError::InvalidTxData)?;
		let signatures = tx_origin.signatures.to_vec();

		let chain_id = std::str::from_utf8(chain_id).unwrap();
		let sign_doc = match tx_origin
			.auth_info
			.signer_infos
			.first()
			.ok_or(DecodeTxError::InvalidTxData)?
			.mode_info
		{
			cosmrs::tx::ModeInfo::Single(single) => match single.mode {
				SignMode::Direct => {
					let chain_id = chain::Id::from_str(chain_id).unwrap();
					let sign_doc = cosmrs::tx::SignDoc::new(
						&tx_origin.body,
						&tx_origin.auth_info,
						&chain_id,
						0u64,
					)
					.map_err(|_| DecodeTxError::InvalidTxData)?;
					sign_doc.into_bytes().map_err(|_| DecodeTxError::InvalidSignDoc)?
				},
				SignMode::LegacyAminoJson => SignAminoDoc::new(&tx_origin, chain_id)?.to_bytes()?,
				_ => return Err(DecodeTxError::UnsupportedSignMode),
			},
			_ => return Err(DecodeTxError::UnsupportedSignMode),
		};
		let len = tx_bytes.len().try_into().map_err(|_| DecodeTxError::TooLongTxBytes)?;
		Ok(Self {
			body: tx_origin.body.try_into()?,
			auth_info: tx_origin.auth_info.try_into()?,
			signatures,
			hash: sha2_256(&sign_doc).into(),
			len,
		})
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo))]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct Body {
	pub messages: Vec<Any>,
	pub memo: Vec<u8>,
	pub timeout_height: u64,
}

#[cfg(feature = "std")]
impl TryFrom<cosmrs::tx::Body> for Body {
	type Error = DecodeTxError;

	fn try_from(body: cosmrs::tx::Body) -> Result<Self, Self::Error> {
		let mut messages: Vec<Any> = Vec::new();
		for msg in body.messages {
			messages.push(msg.into());
		}
		Ok(Self {
			messages,
			memo: body.memo.as_bytes().to_vec(),
			timeout_height: body.timeout_height.into(),
		})
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo))]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct Any {
	pub type_url: Vec<u8>,
	pub value: Vec<u8>,
}

#[cfg(feature = "std")]
impl From<cosmrs::Any> for Any {
	fn from(any: cosmrs::Any) -> Self {
		Any { type_url: any.type_url.as_bytes().to_vec(), value: any.value }
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo))]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct AuthInfo {
	pub signer_infos: Vec<SignerInfo>,
	pub fee: Fee,
}

#[cfg(feature = "std")]
impl TryFrom<cosmrs::tx::AuthInfo> for AuthInfo {
	type Error = DecodeTxError;

	fn try_from(auth_info: cosmrs::tx::AuthInfo) -> Result<Self, Self::Error> {
		let mut signer_infos: Vec<SignerInfo> = Vec::new();
		for signer_info in auth_info.signer_infos {
			signer_infos.push(signer_info.try_into()?);
		}
		Ok(Self { signer_infos, fee: auth_info.fee.try_into()? })
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo))]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct SignerInfo {
	pub public_key: Option<SignerPublicKey>,
	pub sequence: SequenceNumber,
}

#[cfg(feature = "std")]
impl TryFrom<cosmrs::tx::SignerInfo> for SignerInfo {
	type Error = DecodeTxError;

	fn try_from(signer_info: cosmrs::tx::SignerInfo) -> Result<Self, Self::Error> {
		let public_key = match signer_info.public_key {
			Some(pubkey) => match pubkey {
				cosmrs::tx::SignerPublicKey::Single(p) => match p.type_url() {
					cosmrs::crypto::PublicKey::ED25519_TYPE_URL => {
						let mut raw_bytes: [u8; 32] = [0u8; 32];
						raw_bytes.copy_from_slice(&p.to_bytes()[..]);
						Some(SignerPublicKey::Single(PublicKey::Ed25519(raw_bytes)))
					},
					cosmrs::crypto::PublicKey::SECP256K1_TYPE_URL => {
						let mut raw_bytes: [u8; 33] = [0u8; 33];
						raw_bytes.copy_from_slice(&p.to_bytes()[..]);
						Some(SignerPublicKey::Single(PublicKey::Secp256k1(raw_bytes)))
					},
					_ => return Err(DecodeTxError::UnsupportedSignerType),
				},
				_ => return Err(DecodeTxError::UnsupportedSignerType),
			},
			None => None,
		};
		Ok(Self { public_key, sequence: signer_info.sequence })
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo))]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub enum SignerPublicKey {
	/// Single singer.
	Single(PublicKey),
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo))]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub enum PublicKey {
	Ed25519([u8; 32]),
	Secp256k1([u8; 33]),
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo))]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct Fee {
	pub amount: Vec<Coin>,
	pub gas_limit: Gas,
	pub payer: Option<AccountId>,
	pub granter: Option<AccountId>,
}

#[cfg(feature = "std")]
impl TryFrom<cosmrs::tx::Fee> for Fee {
	type Error = DecodeTxError;

	fn try_from(fee: cosmrs::tx::Fee) -> Result<Self, Self::Error> {
		if fee.amount.is_empty() {
			return Err(DecodeTxError::EmptyFeeAmount);
		}
		let amount = fee.amount.iter().map(|c| c.into()).collect::<Vec<Coin>>();
		let payer = fee.payer.map(Into::into);
		let granter = fee.granter.map(Into::into);

		Ok(Self { amount, gas_limit: fee.gas_limit, payer, granter })
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo))]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct Account {
	pub sequence: SequenceNumber,
	pub amount: u128,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo))]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct Coin {
	pub denom: Vec<u8>,
	pub amount: u128,
}

#[cfg(feature = "std")]
impl From<&cosmrs::Coin> for Coin {
	fn from(coin: &cosmrs::Coin) -> Self {
		let denom = coin.denom.as_ref().as_bytes().to_vec();
		Self { denom, amount: coin.amount }
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo))]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct AccountId {
	pub hrp: Vec<u8>,
	pub address: H160,
}

#[cfg(feature = "std")]
impl From<cosmrs::AccountId> for AccountId {
	fn from(account_id: cosmrs::AccountId) -> Self {
		let hrp = account_id.prefix().as_bytes().to_vec();
		let address = H160::from_slice(&account_id.to_bytes());
		Self { hrp, address }
	}
}

#[cfg(test)]
mod tests {
	use crate::legacy::SignAminoDoc;
	use base64ct::{Base64, Encoding};
	use sp_core::hashing::sha2_256;

	#[test]
	fn test_sign_amino_doc_hash() {
		let tx_bytes =  "CpoBCpcBChwvY29zbW9zLmJhbmsudjFiZXRhMS5Nc2dTZW5kEncKLWNvc21vczFxZDY5bnV3ajk1Z3RhNGFramd5eHRqOXVqbXo0dzhlZG1xeXNxdxItY29zbW9zMW41amd4NjR6dzM4c3M3Nm16dXU0dWM3amV5cXcydmZqazYwZmR6GhcKBGFjZHQSDzEwMDAwMDAwMDAwMDAwMBJsCk4KRgofL2Nvc21vcy5jcnlwdG8uc2VjcDI1NmsxLlB1YktleRIjCiECChCRNB/lZkv6F4LV4Ed5aJBoyRawTLNl7DFTdVaE2aESBAoCCH8SGgoSCgRhY2R0EgoxMDQwMDAwMDAwEIDa8esEGkBgXIiPoBpecG7QpKDJPaztFogqvmxjDHF5ORfWBrOoSzf0+AAmch1CXrG4OmiKL0y8v9ITx0QzWYUc7ueXcdIm";
		let tx_bytes = Base64::decode_vec(tx_bytes).unwrap();
		let tx = cosmrs::Tx::from_bytes(&tx_bytes).unwrap();
		let sign_doc = SignAminoDoc::new(&tx, "dev").unwrap();
		let hash = sha2_256(&sign_doc.to_bytes().unwrap());
		assert_eq!(
			array_bytes::bytes2hex("", &hash),
			"714d4bdfdbd0bd630ebdf93b1f6eba7d3c752e92bbab6c9d3d9c93e1777348bb"
		);
	}
}
