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

use pallet_cosmos_types::tx::AccountId;
use sp_runtime_interface::runtime_interface;
use sp_std::vec::Vec;

/// Interface for working with crypto-related, handling Cosmos transactions and messages.
#[runtime_interface]
pub trait Cosmos {
	/// Hash with ripemd160.
	fn ripemd160(msg: &[u8]) -> [u8; 20] {
		hp_crypto::ripemd160(msg)
	}

	/// Verify with secp256k1.
	fn secp256k1_ecdsa_verify(sig: &[u8], msg: &[u8], pub_key: &[u8]) -> bool {
		hp_crypto::secp256k1_ecdsa_verify(sig, msg, pub_key)
	}

	/// Decode raw type cosmos transaction
	fn decode_tx(tx_bytes: &[u8]) -> Option<pallet_cosmos_types::tx::Tx> {
		pallet_cosmos_types::tx::Tx::decode(tx_bytes).ok()
	}

	/// Get SignerDoc hash.
	fn get_sign_doc_bytes(
		tx_bytes: &[u8],
		chain_id: &[u8],
		account_number: u64,
	) -> Option<[u8; 32]> {
		pallet_cosmos_types::sign_doc::get_sign_doc_bytes(tx_bytes, chain_id, account_number).ok()
	}

	/// Get AminoSignerDoc hash.
	fn get_amino_sign_doc_bytes(
		tx_bytes: &[u8],
		chain_id: &[u8],
		account_number: u64,
		sequence: u64,
	) -> Option<[u8; 32]> {
		pallet_cosmos_types::sign_doc::get_amino_sign_doc_bytes(
			tx_bytes,
			chain_id,
			account_number,
			sequence,
		)
		.ok()
	}

	/// Converting a message from Protobuf encoding to Scale encoding
	fn protobuf_to_scale(type_url: &[u8], value: &[u8]) -> Option<Vec<u8>> {
		match pallet_cosmos_types::protobuf::TRANSCODER.get() {
			Some(f) => f(type_url, value).ok(),
			None => None,
		}
	}

	/// Converting a message from Protobuf encoding to Scale encoding
	fn get_msg_any_signers(_type_url: &[u8], _value: &[u8]) -> Option<Vec<AccountId>> {
		// TODO
		None
	}
}
