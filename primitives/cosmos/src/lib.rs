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

use cosmrs::{self, tx::MessageExt};

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
}

impl From<cosmrs::tx::Tx> for Tx {
	fn from(tx: cosmrs::tx::Tx) -> Self {
		let signatures = tx.signatures.iter().map(|s| s.clone()).collect::<Vec<SignatureBytes>>();
		Self { body: tx.body.into(), auth_info: tx.auth_info.into(), signatures }
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(codec::Encode, codec::Decode, scale_info::TypeInfo))]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Body {
	pub messages: Vec<Message>,
	pub memo: String,
}

impl From<cosmrs::tx::Body> for Body {
	fn from(body: cosmrs::tx::Body) -> Self {
		let messages = body.messages.iter().map(|m| m.clone().into()).collect::<Vec<Message>>();
		Self { messages, memo: body.memo.into() }
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(codec::Encode, codec::Decode, scale_info::TypeInfo))]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Message {
	MsgSend { from_address: String, to_address: String, amount: Vec<Coin> },
}

impl From<cosmrs::Any> for Message {
	fn from(any: cosmrs::Any) -> Self {
		if any.type_url == "/cosmos.bank.v1beta1.MsgSend" {
			let type_msg = cosmrs::proto::cosmos::bank::v1beta1::MsgSend::from_any(&any).unwrap();
			let type_msg = cosmrs::bank::MsgSend::try_from(type_msg).unwrap();
			let amount = type_msg.amount.iter().map(|a| a.clone().into()).collect::<Vec<Coin>>();
			Message::MsgSend {
				from_address: type_msg.from_address.into(),
				to_address: type_msg.to_address.into(),
				amount,
			}
		} else {
			// TODO
			panic!();
		}
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(codec::Encode, codec::Decode, scale_info::TypeInfo))]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Coin {
	denum: String,
	amount: u128,
}

impl From<cosmrs::Coin> for Coin {
	fn from(coin: cosmrs::Coin) -> Self {
		Self { denum: coin.denom.to_string(), amount: coin.amount }
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(codec::Encode, codec::Decode, scale_info::TypeInfo))]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AuthInfo {
	pub signer_infos: Vec<SignerInfo>,
	pub fee: Fee,
}

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
	pub sequence: SequenceNumber,
}

impl From<cosmrs::tx::SignerInfo> for SignerInfo {
	fn from(signer_info: cosmrs::tx::SignerInfo) -> Self {
		Self { sequence: signer_info.sequence }
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(codec::Encode, codec::Decode, scale_info::TypeInfo))]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Fee {
	pub amount: Vec<Coin>,
	pub gas_limit: Gas,
}

impl From<cosmrs::tx::Fee> for Fee {
	fn from(fee: cosmrs::tx::Fee) -> Self {
		let amount = fee.amount.iter().map(|a| a.clone().into()).collect::<Vec<Coin>>();
		Self { amount, gas_limit: fee.gas_limit }
	}
}
