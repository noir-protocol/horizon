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

#[cfg(feature = "std")]
use cosmrs::{self, tx::MessageExt};
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
impl Tx {
	pub fn new(tx: cosmrs::tx::Tx, hash: [u8; 32]) -> Self {
		let signatures = tx.signatures.iter().map(|s| s.clone()).collect::<Vec<SignatureBytes>>();
		Self { body: tx.body.into(), auth_info: tx.auth_info.into(), signatures, hash }
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(codec::Encode, codec::Decode, scale_info::TypeInfo))]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Body {
	pub messages: Vec<Message>,
}

#[cfg(feature = "std")]
impl From<cosmrs::tx::Body> for Body {
	fn from(body: cosmrs::tx::Body) -> Self {
		let messages = body.messages.iter().map(|m| m.clone().into()).collect::<Vec<Message>>();
		Self { messages }
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(codec::Encode, codec::Decode, scale_info::TypeInfo))]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Message {
	MsgSend { from_address: H160, to_address: H160, amount: u128 },
}

#[cfg(feature = "std")]
impl From<cosmrs::Any> for Message {
	fn from(any: cosmrs::Any) -> Self {
		if any.type_url == "/cosmos.bank.v1beta1.MsgSend" {
			let type_msg = cosmrs::proto::cosmos::bank::v1beta1::MsgSend::from_any(&any).unwrap();
			let type_msg = cosmrs::bank::MsgSend::try_from(type_msg).unwrap();
			let amount = type_msg.amount[0].amount;
			let mut from_address: [u8; 20] = [0u8; 20];
			from_address.copy_from_slice(&type_msg.from_address.to_bytes()[..]);
			let mut to_address: [u8; 20] = [0u8; 20];
			to_address.copy_from_slice(&type_msg.to_address.to_bytes()[..]);

			Message::MsgSend {
				from_address: from_address.into(),
				to_address: to_address.into(),
				amount,
			}
		} else {
			// TODO: Handling error when decoding failed
			panic!();
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
pub struct LegacyAminoMultisig {
	/// Multisig threshold.
	pub threshold: u32,

	/// Public keys which comprise the multisig key.
	pub public_keys: Vec<PublicKey>,
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
