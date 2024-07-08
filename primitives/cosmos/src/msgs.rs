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

use crate::{error::DecodeMsgError, AccountId, Any, Coin};
#[cfg(feature = "std")]
use cosmrs::tx::Msg as _;
#[cfg(feature = "with-codec")]
use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "with-codec")]
use scale_info::TypeInfo;
#[cfg(not(feature = "std"))]
use sp_std::vec::Vec;

pub trait Msg {
	fn get_signers(&self) -> Vec<AccountId>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo))]
pub struct MsgSend {
	pub from_address: AccountId,
	pub to_address: AccountId,
	pub amount: Vec<Coin>,
}

#[cfg(feature = "std")]
impl Msg for MsgSend {
	fn get_signers(&self) -> Vec<AccountId> {
		vec![self.from_address.clone()]
	}
}

#[cfg(feature = "std")]
impl TryFrom<Any> for MsgSend {
	type Error = DecodeMsgError;

	fn try_from(msg: Any) -> Result<Self, Self::Error> {
		let type_url =
			String::from_utf8(msg.type_url).map_err(|_| DecodeMsgError::InvalidTypeUrl)?;
		if type_url != "/cosmos.bank.v1beta1.MsgSend" {
			return Err(DecodeMsgError::UnsupportedType);
		}

		let any = cosmrs::Any { type_url, value: msg.value };
		let msg_send =
			cosmrs::bank::MsgSend::from_any(&any).map_err(|_| DecodeMsgError::InvalidValue)?;

		let from_address = msg_send.from_address.into();
		let to_address = msg_send.to_address.into();
		let amount = msg_send.amount.iter().map(From::from).collect::<Vec<Coin>>();

		Ok(MsgSend { from_address, to_address, amount })
	}
}

#[cfg(all(feature = "std", feature = "with-codec"))]
pub fn to_scale(type_url: &[u8], value: &[u8]) -> Result<(Vec<u8>, Vec<u8>), DecodeMsgError> {
	match type_url {
		b"/cosmos.bank.v1beta1.MsgSend" => {
			let any = Any { type_url: type_url.to_vec(), value: value.to_vec() };
			let msg_send: MsgSend = any.try_into()?;
			Ok((type_url.to_vec(), msg_send.encode()))
		},
		_ => Err(DecodeMsgError::InvalidTypeUrl),
	}
}

#[cfg(all(feature = "std", feature = "with-codec"))]
pub fn get_msg_any_signers(
	type_url: &[u8],
	value: &[u8],
) -> Result<Vec<AccountId>, DecodeMsgError> {
	match type_url {
		b"/cosmos.bank.v1beta1.MsgSend" => {
			let any = Any { type_url: type_url.to_vec(), value: value.to_vec() };
			let msg_send: MsgSend = any.try_into()?;
			Ok(msg_send.get_signers())
		},
		_ => Err(DecodeMsgError::InvalidTypeUrl),
	}
}

#[cfg(test)]
#[cfg(feature = "with-codec")]
mod tests {
	use super::{to_scale, MsgSend};
	use crate::{Coin, Tx};
	use array_bytes::hex2bytes;
	use base64ct::{Base64, Encoding};
	use parity_scale_codec::Decode;
	use sp_core::H160;

	#[test]
	fn test_msg_to_scale_and_decode() {
		let tx_bytes =  "CpoBCpcBChwvY29zbW9zLmJhbmsudjFiZXRhMS5Nc2dTZW5kEncKLWNvc21vczFxZDY5bnV3ajk1Z3RhNGFramd5eHRqOXVqbXo0dzhlZG1xeXNxdxItY29zbW9zMW41amd4NjR6dzM4c3M3Nm16dXU0dWM3amV5cXcydmZqazYwZmR6GhcKBGFjZHQSDzEwMDAwMDAwMDAwMDAwMBJsCk4KRgofL2Nvc21vcy5jcnlwdG8uc2VjcDI1NmsxLlB1YktleRIjCiECChCRNB/lZkv6F4LV4Ed5aJBoyRawTLNl7DFTdVaE2aESBAoCCH8SGgoSCgRhY2R0EgoxMDQwMDAwMDAwEIDa8esEGkBgXIiPoBpecG7QpKDJPaztFogqvmxjDHF5ORfWBrOoSzf0+AAmch1CXrG4OmiKL0y8v9ITx0QzWYUc7ueXcdIm";
		let tx_bytes = Base64::decode_vec(tx_bytes).unwrap();
		let tx = Tx::decode(&tx_bytes).unwrap();

		let msg = tx.body.messages.first().unwrap();
		let (type_url, value) = to_scale(&msg.type_url, &msg.value).unwrap();

		assert_eq!(msg.type_url, type_url);

		let msg_send: MsgSend = Decode::decode(&mut &value[..]).unwrap();

		assert_eq!(
			msg_send.from_address.address,
			H160::from_slice(&hex2bytes("0x037459f1d22d10bed7b6920865c8bc96c5571f2d").unwrap())
		);
		assert_eq!(
			msg_send.to_address.address,
			H160::from_slice(&hex2bytes("0x9d24836aa2744f087b5b17395e63d2c900e53132").unwrap())
		);
		assert_eq!(
			msg_send.amount,
			vec![Coin { denom: vec![97, 99, 100, 116], amount: 100000000000000 }]
		);
	}
}
