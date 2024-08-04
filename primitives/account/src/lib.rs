// SPDX-License-Identifier: Apache-2.0
// This file is part of Frontier.
//
// Copyright (c) 2020-2023 Parity Technologies (UK) Ltd.
//
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

use hp_crypto::EcdsaExt;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use ripemd::Digest;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{de, Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use sp_core::{ecdsa, H160};
use sp_runtime::traits::IdentifyAccount;

#[derive(
	Copy,
	Clone,
	Eq,
	PartialEq,
	Ord,
	PartialOrd,
	Encode,
	Decode,
	MaxEncodedLen,
	TypeInfo,
	Serialize,
	Deserialize,
)]
pub struct CosmosSigner(pub ecdsa::Public);

impl IdentifyAccount for CosmosSigner {
	type AccountId = Self;

	fn into_account(self) -> Self {
		self
	}
}

impl From<ecdsa::Public> for CosmosSigner {
	fn from(value: ecdsa::Public) -> Self {
		CosmosSigner(value)
	}
}

impl From<[u8; 33]> for CosmosSigner {
	fn from(value: [u8; 33]) -> Self {
		CosmosSigner(ecdsa::Public(value))
	}
}

#[cfg(feature = "std")]
impl std::fmt::Display for CosmosSigner {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", hex::encode(self.0 .0))
	}
}

impl sp_std::fmt::Debug for CosmosSigner {
	fn fmt(&self, f: &mut sp_std::fmt::Formatter<'_>) -> sp_std::fmt::Result {
		write!(f, "{:?}", self.0)
	}
}

impl EcdsaExt for CosmosSigner {
	fn to_cosm_address(&self) -> Option<H160> {
		let mut hasher = ripemd::Ripemd160::new();
		hasher.update(&sp_io::hashing::sha2_256(&self.0 .0));
		let address = H160::from_slice(&hasher.finalize());

		Some(address)
	}
}

#[derive(Eq, PartialEq, Clone, Encode, Decode, sp_core::RuntimeDebug, TypeInfo)]
pub struct CosmosSignature(pub [u8; 64]);

#[cfg(feature = "std")]
impl Serialize for CosmosSignature {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(&array_bytes::bytes2hex("", self.0))
	}
}

impl TryFrom<&[u8]> for CosmosSignature {
	type Error = ();

	fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
		if data.len() == 64 {
			let mut inner = [0u8; 64];
			inner.copy_from_slice(data);
			Ok(CosmosSignature(inner))
		} else {
			Err(())
		}
	}
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for CosmosSignature {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let signature_hex = array_bytes::hex2bytes(String::deserialize(deserializer)?)
			.map_err(|e| de::Error::custom(format!("{:?}", e)))?;
		CosmosSignature::try_from(signature_hex.as_ref())
			.map_err(|e| de::Error::custom(format!("{:?}", e)))
	}
}

impl sp_runtime::traits::Verify for CosmosSignature {
	type Signer = CosmosSigner;

	fn verify<L: sp_runtime::traits::Lazy<[u8]>>(&self, mut msg: L, signer: &CosmosSigner) -> bool {
		hp_io::cosmos::secp256k1_ecdsa_verify(self.0.as_ref(), msg.get(), &signer.0 .0)
	}
}
