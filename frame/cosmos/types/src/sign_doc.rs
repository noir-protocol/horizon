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

use crate::{error::DecodeError, legacy::StdSignDoc, tx::SequenceNumber};
use core::str::FromStr;
use cosmrs::tendermint::chain;

pub fn sign_bytes(
	tx_bytes: &[u8],
	chain_id: &[u8],
	account_number: u64,
) -> Result<Vec<u8>, DecodeError> {
	let tx = cosmrs::Tx::from_bytes(tx_bytes).map_err(|_| DecodeError::InvalidTxData)?;
	let chain_id = core::str::from_utf8(chain_id)
		.map(chain::Id::from_str)
		.map_err(|_| DecodeError::InvalidChainId)?
		.map_err(|_| DecodeError::InvalidChainId)?;

	cosmrs::tx::SignDoc::new(&tx.body, &tx.auth_info, &chain_id, account_number)
		.map_err(|_| DecodeError::InvalidSignDoc)?
		.into_bytes()
		.map_err(|_| DecodeError::InvalidSignDoc)
}

pub fn std_sign_bytes(
	tx_bytes: &[u8],
	chain_id: &[u8],
	account_number: u64,
	sequence: SequenceNumber,
) -> Result<Vec<u8>, DecodeError> {
	let tx = cosmrs::Tx::from_bytes(tx_bytes).map_err(|_| DecodeError::InvalidTxData)?;
	let chain_id = String::from_utf8(chain_id.to_vec()).map_err(|_| DecodeError::InvalidChainId)?;

	StdSignDoc::new(&tx, chain_id, sequence, account_number)
		.map_err(|_| DecodeError::InvalidSignDoc)?
		.to_bytes()
}

#[cfg(test)]
mod tests {
	use super::sign_bytes;
	use base64ct::{Base64, Encoding};
	use sp_core::sha2_256;

	#[test]
	fn test_sign_bytes() {
		let tx_raw = "CpMBCpABChwvY29zbW9zLmJhbmsudjFiZXRhMS5Nc2dTZW5kEnAKLWNvc21vczFxZDY5bnV3ajk1Z3RhNGFramd5eHRqOXVqbXo0dzhlZG1xeXNxdxItY29zbW9zMWdtajJleGFnMDN0dGdhZnBya2RjM3Q4ODBncm1hOW53ZWZjZDJ3GhAKBXVhdG9tEgcxMDAwMDAwEnEKTgpGCh8vY29zbW9zLmNyeXB0by5zZWNwMjU2azEuUHViS2V5EiMKIQIKEJE0H+VmS/oXgtXgR3lokGjJFrBMs2XsMVN1VoTZoRIECgIIARIfChUKBXVhdG9tEgw4ODY4ODAwMDAwMDAQgMDxxZSVFBpA9+DRmMYoIcxYF8jpNfUjMIMB4pgZ9diC8ySbnhc6YU84AA3b/0RsCr+nx9AZ27FwcrKJM/yBh8lz+/A9BFn3bg==";

		let tx_bytes = Base64::decode_vec(&tx_raw).unwrap();
		let expected_hash = sign_bytes(&tx_bytes, b"theta-testnet-001", 754989u64).unwrap();

		let sign_doc_raw = "CpMBCpABChwvY29zbW9zLmJhbmsudjFiZXRhMS5Nc2dTZW5kEnAKLWNvc21vczFxZDY5bnV3ajk1Z3RhNGFramd5eHRqOXVqbXo0dzhlZG1xeXNxdxItY29zbW9zMWdtajJleGFnMDN0dGdhZnBya2RjM3Q4ODBncm1hOW53ZWZjZDJ3GhAKBXVhdG9tEgcxMDAwMDAwEnEKTgpGCh8vY29zbW9zLmNyeXB0by5zZWNwMjU2azEuUHViS2V5EiMKIQIKEJE0H+VmS/oXgtXgR3lokGjJFrBMs2XsMVN1VoTZoRIECgIIARIfChUKBXVhdG9tEgw4ODY4ODAwMDAwMDAQgMDxxZSVFBoRdGhldGEtdGVzdG5ldC0wMDEgrYou";
		let hash = sha2_256(&Base64::decode_vec(&sign_doc_raw).unwrap());

		assert_eq!(expected_hash, hash);
	}
}
