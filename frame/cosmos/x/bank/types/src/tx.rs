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

#[cfg(feature = "std")]
use cosmrs::tx::Msg as _;
use pallet_cosmos_types::{
	coin::Coin,
	error::DecodeError,
	tx::{AccountId, Any},
};
#[cfg(feature = "std")]
use pallet_cosmos_types::{
	legacy::{self, LegacyMsg},
	msgs::Msg,
};
#[cfg(feature = "with-codec")]
use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "with-codec")]
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde_json::json;
#[cfg(not(feature = "std"))]
use sp_std::vec::Vec;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo))]
pub struct MsgSend {
	pub from_address: AccountId,
	pub to_address: AccountId,
	pub amount: Vec<Coin>,
}

#[cfg(feature = "std")]
impl Msg for MsgSend {
	const TYPE_URL: &'static [u8] = b"/cosmos.bank.v1beta1.MsgSend";
	const AMINO_NAME: &'static [u8] = b"cosmos-sdk/MsgSend";

	fn get_signers(&self) -> Vec<AccountId> {
		vec![self.from_address.clone()]
	}

	fn legacy_msg(any: Any) -> Result<LegacyMsg, DecodeError> {
		let cosmrs::proto::cosmos::bank::v1beta1::MsgSend { from_address, to_address, amount } =
			cosmrs::Any::try_from(any)?.to_msg().map_err(|_| DecodeError::InvalidMsgData)?;
		let amount = amount
			.iter()
			.map(|amt| legacy::Coin { amount: amt.amount.to_string(), denom: amt.denom.clone() })
			.collect::<Vec<legacy::Coin>>();
		let value = serde_json::to_value(
			json!({ "from_address": from_address, "to_address": to_address, "amount": amount }),
		)
		.map_err(|_| DecodeError::InvalidMsgData)?;
		let r#type = String::from_utf8(MsgSend::AMINO_NAME.to_vec())
			.map_err(|_| DecodeError::InvalidMsgData)?;

		Ok(LegacyMsg { r#type, value })
	}
}

#[cfg(feature = "std")]
impl TryFrom<Any> for MsgSend {
	type Error = DecodeError;

	fn try_from(msg: Any) -> Result<Self, Self::Error> {
		let any = msg.try_into()?;
		let msg_send =
			cosmrs::bank::MsgSend::from_any(&any).map_err(|_| DecodeError::InvalidMsgData)?;

		let from_address = msg_send.from_address.into();
		let to_address = msg_send.to_address.into();
		let amount = msg_send.amount.iter().map(From::from).collect::<Vec<Coin>>();

		Ok(MsgSend { from_address, to_address, amount })
	}
}