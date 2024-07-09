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
pub mod legacy;
pub mod msgs;
#[cfg(feature = "std")]
pub mod sign_doc;

#[cfg(feature = "std")]
use error::DecodeError;
#[cfg(feature = "with-codec")]
use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "with-codec")]
use scale_info::TypeInfo;
use sp_core::H160;
#[cfg(not(feature = "std"))]
use sp_std::vec::Vec;

pub type SequenceNumber = u64;
pub type SignatureBytes = Vec<u8>;
pub type Gas = u64;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo))]
pub struct Tx {
	pub body: Body,
	pub auth_info: AuthInfo,
	pub signatures: Vec<SignatureBytes>,
	pub raw: Vec<u8>,
}

#[cfg(feature = "std")]
impl Tx {
	pub fn decode(tx_bytes: &[u8]) -> Result<Self, DecodeError> {
		if tx_bytes.is_empty() {
			return Err(DecodeError::EmptyTxBytes);
		}
		let tx_origin = cosmrs::Tx::from_bytes(tx_bytes).map_err(|_| DecodeError::InvalidTxData)?;

		Ok(Self {
			body: tx_origin.body.try_into()?,
			auth_info: tx_origin.auth_info.try_into()?,
			signatures: tx_origin.signatures,
			raw: tx_bytes.to_vec(),
		})
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo))]
pub struct Body {
	pub messages: Vec<Any>,
	pub memo: Vec<u8>,
	pub timeout_height: u64,
}

#[cfg(feature = "std")]
impl TryFrom<cosmrs::tx::Body> for Body {
	type Error = DecodeError;

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
pub struct AuthInfo {
	pub signer_infos: Vec<SignerInfo>,
	pub fee: Fee,
}

#[cfg(feature = "std")]
impl TryFrom<cosmrs::tx::AuthInfo> for AuthInfo {
	type Error = DecodeError;

	fn try_from(auth_info: cosmrs::tx::AuthInfo) -> Result<Self, Self::Error> {
		let mut signer_infos: Vec<SignerInfo> = Vec::new();
		for signer_info in auth_info.signer_infos {
			signer_infos.push(signer_info.try_into()?);
		}
		Ok(Self { signer_infos, fee: auth_info.fee.try_into()? })
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo))]
#[repr(i32)]
pub enum SignMode {
	Unspecified = 0,
	Direct = 1,
	Textual = 2,
	DirectAux = 3,
	LegacyAminoJson = 127,
	Eip191 = 191,
}

#[cfg(feature = "std")]
impl From<cosmrs::tx::SignMode> for SignMode {
	fn from(sign_mode: cosmrs::tx::SignMode) -> Self {
		match sign_mode {
			cosmrs::tx::SignMode::Unspecified => SignMode::Unspecified,
			cosmrs::tx::SignMode::Direct => SignMode::Direct,
			cosmrs::tx::SignMode::Textual => SignMode::Textual,
			cosmrs::tx::SignMode::DirectAux => SignMode::DirectAux,
			cosmrs::tx::SignMode::LegacyAminoJson => SignMode::LegacyAminoJson,
			cosmrs::tx::SignMode::Eip191 => SignMode::Eip191,
		}
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo))]
pub struct Single {
	pub mode: SignMode,
}

#[cfg(feature = "std")]
impl From<cosmrs::tx::mode_info::Single> for Single {
	fn from(single: cosmrs::tx::mode_info::Single) -> Self {
		Single { mode: single.mode.into() }
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo))]
pub enum ModeInfo {
	Single(Single),
}

#[cfg(feature = "std")]
impl TryFrom<cosmrs::tx::ModeInfo> for ModeInfo {
	type Error = DecodeError;

	fn try_from(mode_info: cosmrs::tx::ModeInfo) -> Result<Self, Self::Error> {
		match mode_info {
			cosmrs::tx::ModeInfo::Single(single) => Ok(ModeInfo::Single(single.into())),
			_ => Err(DecodeError::UnsupportedSigningMode),
		}
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo))]
pub struct SignerInfo {
	pub public_key: Option<SignerPublicKey>,
	pub mode_info: ModeInfo,
	pub sequence: SequenceNumber,
}

#[cfg(feature = "std")]
impl TryFrom<cosmrs::tx::SignerInfo> for SignerInfo {
	type Error = DecodeError;

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
					_ => return Err(DecodeError::UnsupportedSignerType),
				},
				_ => return Err(DecodeError::UnsupportedSignerType),
			},
			None => None,
		};
		Ok(Self {
			public_key,
			mode_info: signer_info.mode_info.try_into()?,
			sequence: signer_info.sequence,
		})
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo))]
pub enum SignerPublicKey {
	/// Single singer.
	Single(PublicKey),
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo))]
pub enum PublicKey {
	Ed25519([u8; 32]),
	Secp256k1([u8; 33]),
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo))]
pub struct Fee {
	pub amount: Vec<Coin>,
	pub gas_limit: Gas,
	pub payer: Option<AccountId>,
	pub granter: Option<AccountId>,
}

#[cfg(feature = "std")]
impl TryFrom<cosmrs::tx::Fee> for Fee {
	type Error = DecodeError;

	fn try_from(fee: cosmrs::tx::Fee) -> Result<Self, Self::Error> {
		if fee.amount.is_empty() {
			return Err(DecodeError::EmptyFeeAmount);
		}
		let amount = fee.amount.iter().map(|c| c.into()).collect::<Vec<Coin>>();
		let payer = fee.payer.map(Into::into);
		let granter = fee.granter.map(Into::into);

		Ok(Self { amount, gas_limit: fee.gas_limit, payer, granter })
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo))]
pub struct Account {
	pub sequence: SequenceNumber,
	pub amount: u128,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo))]
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
