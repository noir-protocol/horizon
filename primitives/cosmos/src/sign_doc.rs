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

#[cfg(feature = "with-serde")]
use crate::legacy::AminoSignDoc;
use crate::{error::DecodeTxError, SequenceNumber};
use core::str::FromStr;
use cosmrs::tendermint::chain;
use sp_core::sha2_256;

pub fn get_signer_doc_bytes(
	tx_bytes: &[u8],
	chain_id: &[u8],
	account_number: u64,
) -> Result<[u8; 32], DecodeTxError> {
	let tx = cosmrs::Tx::from_bytes(tx_bytes).map_err(|_| DecodeTxError::InvalidTxData)?;
	let chain_id = core::str::from_utf8(chain_id).map_err(|_| DecodeTxError::InvalidChainId)?;
	let chain_id = chain::Id::from_str(chain_id).map_err(|_| DecodeTxError::InvalidChainId)?;
	let sign_doc = cosmrs::tx::SignDoc::new(&tx.body, &tx.auth_info, &chain_id, account_number)
		.map_err(|_| DecodeTxError::InvalidSignDoc)?;
	let sign_doc_bytes = sign_doc.into_bytes().map_err(|_| DecodeTxError::InvalidSignDoc)?;

	Ok(sha2_256(&sign_doc_bytes))
}

#[cfg(feature = "with-serde")]
pub fn get_amino_signer_doc_bytes(
	tx_bytes: &[u8],
	chain_id: &[u8],
	account_number: u64,
	sequence: SequenceNumber,
) -> Result<[u8; 32], DecodeTxError> {
	let tx = cosmrs::Tx::from_bytes(tx_bytes).map_err(|_| DecodeTxError::InvalidTxData)?;
	let chain_id =
		String::from_utf8(chain_id.to_vec()).map_err(|_| DecodeTxError::InvalidChainId)?;
	let sign_doc_bytes = AminoSignDoc::new(&tx, chain_id, sequence, account_number)
		.map_err(|_| DecodeTxError::InvalidSignDoc)?
		.bytes()?;

	Ok(sha2_256(&sign_doc_bytes))
}
