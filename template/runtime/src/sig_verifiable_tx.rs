// This file is part of Hrozion.

// Copyright (C) 2023 Haderech Pte. Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use cosmos_sdk_proto::{
	cosmos::{bank::v1beta1::MsgSend, tx::v1beta1::Tx},
	prost::alloc::string::String,
	traits::Message,
};
use pallet_cosmos_x_auth_signing::sign_verifiable_tx::SigVerifiableTxError;
use pallet_cosmos_x_bank_types::msgs::msg_send;
use sp_std::vec::Vec;

pub struct SigVerifiableTx;

impl pallet_cosmos_x_auth_signing::sign_verifiable_tx::SigVerifiableTx for SigVerifiableTx {
	fn get_signers(tx: &Tx) -> Result<Vec<String>, SigVerifiableTxError> {
		let mut signers = Vec::<String>::new();

		let body = tx.body.clone().ok_or(SigVerifiableTxError::EmptyTxBody)?;
		for msg in body.messages.iter() {
			let msg_signers = match msg.type_url.as_str() {
				"/cosmos.bank.v1beta1.MsgSend" => {
					let msg =
						MsgSend::decode(&mut &*msg.value).map_err(SigVerifiableTxError::InvalidMsg);
					msg_send::get_signers(&msg)
				},
				_ => return Err(SigVerifiableTxError::InvalidMsg),
			};

			for msg_signer in msg_signers.iter() {
				if !signers.contains(msg_signer) {
					signers.push(msg_signer.clone());
				}
			}
		}

		let fee_payer = tx
			.auth_info
			.clone()
			.ok_or(SigVerifiableTxError::EmptyAuthInfo)?
			.fee
			.ok_or(SigVerifiableTxError::EmptyFee)?
			.payer;
		if !fee_payer.is_empty() && !signers.contains(&fee_payer) {
			signers.push(fee_payer.clone());
		}

		Ok(signers)
	}
}
