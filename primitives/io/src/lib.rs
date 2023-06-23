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

//! I/O host interface for Horizon runtime.

#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

use k256::ecdsa::Signature;
use ripemd::{Digest, Ripemd160};
use signature::Verifier;
use sp_runtime_interface::runtime_interface;

/// Interfaces for working with crypto related types from within the runtime.
#[runtime_interface]
pub trait Crypto {
	/// Hash with ripemd160.
	fn ripemd160(msg: &[u8]) -> [u8; 20] {
		Ripemd160::digest(msg).into()
	}

	/// Verify with secp256k1.
	fn secp256k1_ecdsa_verify(pk: &[u8; 33], msg: &[u8], sig: &[u8]) -> bool {
		match (k256::ecdsa::VerifyingKey::from_sec1_bytes(pk), Signature::from_slice(sig)) {
			(Ok(verifying_key), Ok(signature)) => verifying_key.verify(msg, &signature).is_ok(),
			_ => false,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_ripemd160() {
		let msg = b"hello world";
		let hash = crypto::ripemd160(msg);
		assert_eq!(
			hash,
			[
				0x98, 0xc6, 0x15, 0x78, 0x4c, 0xcb, 0x5f, 0xe5, 0x93, 0x6f, 0xbc, 0x0c, 0xbe, 0x9d,
				0xfd, 0xb4, 0x08, 0xd9, 0x2f, 0x0f,
			]
		);
	}

	#[test]
	fn test_secp256k1_ecdsa_verify() {

	}
}
