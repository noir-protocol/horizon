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

// use crate::coin::Coin;
// use crate::error::DecodeError;
// #[cfg(feature = "std")]
// use crate::registry;
// use bech32::FromBase32;
// use cosmos_sdk_proto::prost::{alloc::string::String, Message};
#[cfg(feature = "with-codec")]
use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "with-codec")]
use scale_info::TypeInfo;
// use sp_core::H160;
// #[cfg(feature = "with-codec")]
// use sp_runtime_interface::pass_by::PassByCodec;
// #[cfg(not(feature = "std"))]
// use sp_std::vec::Vec;

pub type SequenceNumber = u64;
pub type SignatureBytes = Vec<u8>;
pub type Gas = u64;

// #[derive(Clone, Debug, PartialEq, Eq)]
// #[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo, PassByCodec))]
// pub struct Tx {
// 	pub body: Body,
// 	pub auth_info: AuthInfo,
// 	pub signatures: Vec<SignatureBytes>,
// 	pub raw: Vec<u8>,
// }

// #[cfg(feature = "std")]
// impl Tx {
// 	pub fn get_signers(&self) -> Result<Vec<String>, DecodeError> {
// 		let mut signers = Vec::<String>::new();
// 		for msg in &self.body.messages {
// 			let msg_signers = match registry::REGISTRY.get() {
// 				Some(reg) => reg.signers(msg),
// 				None => return Err(DecodeError::InvalidTypeUrl),
// 			}?;
// 			for msg_signer in msg_signers {
// 				if !signers.contains(&msg_signer) {
// 					signers.push(msg_signer);
// 				}
// 			}
// 		}
// 		if let Some(fee_payer) = &self.auth_info.fee.payer {
// 			if !signers.contains(fee_payer) {
// 				signers.push(fee_payer.clone());
// 			}
// 		}
// 		Ok(signers)
// 	}

// 	pub fn fee_payer(&self) -> Result<AccountId, DecodeError> {
// 		if let Some(fee_payer) = &self.auth_info.fee.payer {
// 			return Ok(fee_payer.clone());
// 		}
// 		self.get_signers()?.first().ok_or(DecodeError::InvalidTxData).cloned()
// 	}
// }

// impl Tx {
// 	pub fn decode(tx_bytes: &[u8]) -> Result<Self, DecodeError> {
// 		if tx_bytes.is_empty() {
// 			return Err(DecodeError::EmptyTxBytes);
// 		}
// 		let tx = cosmos_sdk_proto::cosmos::tx::v1beta1::Tx::decode(tx_bytes)
// 			.map_err(|_| DecodeError::InvalidTxData)?;

// 		Ok(Self {
// 			body: tx.body.map(Into::into).ok_or(DecodeError::InvalidTxData)?,
// 			auth_info: tx.auth_info.map(TryInto::try_into).ok_or(DecodeError::InvalidTxData)??,
// 			signatures: tx.signatures,
// 			raw: tx_bytes.to_vec(),
// 		})
// 	}
// }

// #[derive(Clone, Debug, PartialEq, Eq)]
// #[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo))]
// pub struct Body {
// 	pub messages: Vec<Any>,
// 	pub memo: Vec<u8>,
// 	pub timeout_height: u64,
// }

// impl From<cosmos_sdk_proto::cosmos::tx::v1beta1::TxBody> for Body {
// 	fn from(value: cosmos_sdk_proto::cosmos::tx::v1beta1::TxBody) -> Self {
// 		let messages = value.messages.into_iter().map(Into::into).collect::<Vec<Any>>();
// 		Self {
// 			messages,
// 			memo: value.memo.as_bytes().to_vec(),
// 			timeout_height: value.timeout_height,
// 		}
// 	}
// }

// #[derive(Clone, Debug, PartialEq, Eq)]
// #[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo, PassByCodec))]
// pub struct Any {
// 	pub type_url: Vec<u8>,
// 	pub value: Vec<u8>,
// }

// impl From<cosmos_sdk_proto::Any> for Any {
// 	fn from(value: cosmos_sdk_proto::Any) -> Self {
// 		Self { type_url: value.type_url.as_bytes().to_vec(), value: value.value }
// 	}
// }

// #[cfg(feature = "std")]
// impl From<cosmrs::Any> for Any {
// 	fn from(any: cosmrs::Any) -> Self {
// 		Any { type_url: any.type_url.as_bytes().to_vec(), value: any.value }
// 	}
// }

// #[derive(Clone, Debug, PartialEq, Eq)]
// #[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo))]
// pub struct AuthInfo {
// 	pub signer_infos: Vec<SignerInfo>,
// 	pub fee: Fee,
// }

// impl TryFrom<cosmos_sdk_proto::cosmos::tx::v1beta1::AuthInfo> for AuthInfo {
// 	type Error = DecodeError;

// 	fn try_from(
// 		value: cosmos_sdk_proto::cosmos::tx::v1beta1::AuthInfo,
// 	) -> Result<Self, Self::Error> {
// 		let mut signer_infos = Vec::<SignerInfo>::new();
// 		for signer in value.signer_infos.into_iter() {
// 			signer_infos.push(signer.try_into()?);
// 		}

// 		let fee = value.fee.map(TryInto::try_into).ok_or(DecodeError::InvalidTxData)??;

// 		Ok(Self { signer_infos, fee })
// 	}
// }

// #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
// #[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo))]
// #[repr(i32)]
// pub enum SignMode {
// 	Unspecified = 0,
// 	Direct = 1,
// 	Textual = 2,
// 	DirectAux = 3,
// 	LegacyAminoJson = 127,
// 	Eip191 = 191,
// }

// impl TryFrom<i32> for SignMode {
// 	type Error = DecodeError;

// 	fn try_from(mode: i32) -> Result<Self, Self::Error> {
// 		let mode = match mode {
// 			0i32 => SignMode::Unspecified,
// 			1i32 => SignMode::Direct,
// 			2i32 => SignMode::Textual,
// 			3i32 => SignMode::DirectAux,
// 			127i32 => SignMode::LegacyAminoJson,
// 			191i32 => SignMode::Eip191,
// 			_ => return Err(DecodeError::UnsupportedSigningMode),
// 		};

// 		Ok(mode)
// 	}
// }

// #[derive(Clone, Debug, PartialEq, Eq)]
// #[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo))]
// pub struct Single {
// 	pub mode: SignMode,
// }

// #[derive(Clone, Debug, PartialEq, Eq)]
// #[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo))]
// pub enum ModeInfo {
// 	Single(Single),
// }

// #[derive(Clone, Debug, PartialEq, Eq)]
// #[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo))]
// pub struct SignerInfo {
// 	pub public_key: Option<SignerPublicKey>,
// 	pub mode_info: ModeInfo,
// 	pub sequence: SequenceNumber,
// }

// impl TryFrom<cosmos_sdk_proto::cosmos::tx::v1beta1::SignerInfo> for SignerInfo {
// 	type Error = DecodeError;

// 	fn try_from(
// 		value: cosmos_sdk_proto::cosmos::tx::v1beta1::SignerInfo,
// 	) -> Result<Self, Self::Error> {
// 		let public_key = if let Some(any) = value.public_key {
// 			match any.type_url.as_str() {
// 				"/cosmos.crypto.secp256k1.PubKey" => {
// 					let pubkey =
// 						cosmos_sdk_proto::cosmos::crypto::secp256k1::PubKey::decode(&*any.value)
// 							.map_err(|_| DecodeError::InvalidTxData)?;
// 					let mut pk = [0u8; 33];
// 					pk.copy_from_slice(&pubkey.key[..]);
// 					Some(SignerPublicKey::Single(PublicKey::Secp256k1(pk)))
// 				},
// 				_ => return Err(DecodeError::UnsupportedSignerType),
// 			}
// 		} else {
// 			None
// 		};

// 		let mode_info = if let Some(mode_info) = value.mode_info {
// 			match mode_info.sum {
// 				Some(cosmos_sdk_proto::cosmos::tx::v1beta1::mode_info::Sum::Single(
// 					cosmos_sdk_proto::cosmos::tx::v1beta1::mode_info::Single { mode },
// 				)) => ModeInfo::Single(Single { mode: mode.try_into()? }),
// 				_ => return Err(DecodeError::UnsupportedSignerType),
// 			}
// 		} else {
// 			return Err(DecodeError::UnsupportedSignerType);
// 		};

// 		Ok(Self { public_key, mode_info, sequence: value.sequence })
// 	}
// }

// #[derive(Clone, Debug, PartialEq, Eq)]
// #[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo))]
// pub enum SignerPublicKey {
// 	/// Single singer.
// 	Single(PublicKey),
// }

// #[derive(Clone, Debug, PartialEq, Eq)]
// #[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo))]
// pub enum PublicKey {
// 	Secp256k1([u8; 33]),
// }

// #[derive(Clone, Debug, PartialEq, Eq)]
// #[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo))]
// pub struct Fee {
// 	pub amount: Vec<Coin>,
// 	pub gas_limit: Gas,
// 	pub payer: Option<Vec<u8>>,
// 	pub granter: Option<Vec<u8>>,
// }

// impl TryFrom<cosmos_sdk_proto::cosmos::tx::v1beta1::Fee> for Fee {
// 	type Error = DecodeError;

// 	fn try_from(value: cosmos_sdk_proto::cosmos::tx::v1beta1::Fee) -> Result<Self, Self::Error> {
// 		if value.amount.is_empty() {
// 			return Err(DecodeError::EmptyFeeAmount);
// 		}
// 		let mut amount = Vec::<Coin>::new();
// 		for amt in value.amount.into_iter() {
// 			amount.push(amt.try_into()?);
// 		}

// 		let payer = if value.payer.len() > 0 {
// 			let (hrp, data, _) = bech32::decode(&value.payer).unwrap();
// 			let data = Vec::<u8>::from_base32(&data).unwrap();
// 			let address = H160::from_slice(&data);

// 			Some(AccountId { hrp: hrp.into(), address, bech32: value.payer.as_bytes().to_vec() })
// 		} else {
// 			None
// 		};

// 		let granter = if value.granter.len() > 0 {
// 			let (hrp, data, _) = bech32::decode(&value.granter).unwrap();
// 			let data = Vec::<u8>::from_base32(&data).unwrap();
// 			let address = H160::from_slice(&data);

// 			Some(AccountId { hrp: hrp.into(), address, bech32: value.granter.as_bytes().to_vec() })
// 		} else {
// 			None
// 		};

// 		Ok(Self { amount, gas_limit: value.gas_limit, payer, granter })
// 	}
// }

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo))]
pub struct Account {
	pub sequence: SequenceNumber,
	pub amount: u128,
}
