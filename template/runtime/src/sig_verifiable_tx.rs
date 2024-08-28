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

use alloc::string::String;
use cosmos_sdk_proto::{
	cosmos::{bank::v1beta1::MsgSend, tx::v1beta1::Tx},
	cosmwasm::wasm::v1::{
		MsgExecuteContract, MsgInstantiateContract2, MsgMigrateContract, MsgStoreCode,
		MsgUpdateAdmin,
	},
	prost::Message,
};
use pallet_cosmos_x_auth_signing::{any_match, sign_verifiable_tx::SigVerifiableTxError};
use pallet_cosmos_x_bank_types::msgs::msg_send;
use pallet_cosmos_x_wasm_types::tx::{
	msg_execute_contract, msg_instantiate_contract2, msg_migrate_contract, msg_store_code,
	msg_update_admin,
};
use sp_std::vec::Vec;

pub struct SigVerifiableTx;

impl pallet_cosmos_x_auth_signing::sign_verifiable_tx::SigVerifiableTx for SigVerifiableTx {
	fn get_signers(tx: &Tx) -> Result<Vec<String>, SigVerifiableTxError> {
		let mut signers = Vec::<String>::new();

		let body = tx.body.as_ref().ok_or(SigVerifiableTxError::EmptyTxBody)?;
		for msg in body.messages.iter() {
			let msg_signers = any_match!(
				msg, {
					MsgSend => MsgSend::decode(&mut &*msg.value).as_ref().map(msg_send::get_signers).map_err(|_| SigVerifiableTxError::InvalidMsg),
					MsgStoreCode => MsgStoreCode::decode(&mut &*msg.value).as_ref().map(msg_store_code::get_signers).map_err(|_| SigVerifiableTxError::InvalidMsg),
					MsgInstantiateContract2 => MsgInstantiateContract2::decode(&mut &*msg.value).as_ref().map(msg_instantiate_contract2::get_signers).map_err(|_| SigVerifiableTxError::InvalidMsg),
					MsgExecuteContract => MsgExecuteContract::decode(&mut &*msg.value).as_ref().map(msg_execute_contract::get_signers).map_err(|_| SigVerifiableTxError::InvalidMsg),
					MsgMigrateContract => MsgMigrateContract::decode(&mut &*msg.value).as_ref().map(msg_migrate_contract::get_signers).map_err(|_| SigVerifiableTxError::InvalidMsg),
					MsgUpdateAdmin => MsgUpdateAdmin::decode(&mut &*msg.value).as_ref().map(msg_update_admin::get_signers).map_err(|_| SigVerifiableTxError::InvalidMsg),
				},
				Err(SigVerifiableTxError::InvalidMsg)
			)?;

			for msg_signer in msg_signers.iter() {
				if !signers.contains(msg_signer) {
					signers.push(msg_signer.clone());
				}
			}
		}

		let fee_payer = &tx
			.auth_info
			.as_ref()
			.and_then(|auth_info| auth_info.fee.as_ref())
			.ok_or(SigVerifiableTxError::EmptyFee)?
			.payer;

		if !fee_payer.is_empty() && !signers.contains(fee_payer) {
			signers.push(fee_payer.clone());
		}

		Ok(signers)
	}

	fn fee_payer(tx: &Tx) -> Result<String, SigVerifiableTxError> {
		let fee = tx
			.auth_info
			.as_ref()
			.and_then(|auth_info| auth_info.fee.as_ref())
			.ok_or(SigVerifiableTxError::EmptyFee)?;

		let fee_payer = if fee.payer.is_empty() {
			Self::get_signers(tx)?
				.first()
				.ok_or(SigVerifiableTxError::EmptySigners)?
				.clone()
		} else {
			fee.payer.clone()
		};

		Ok(fee_payer)
	}

	fn sequence(tx: &Tx) -> Result<u64, SigVerifiableTxError> {
		let auth_info = tx.auth_info.as_ref().ok_or(SigVerifiableTxError::EmptyAuthInfo)?;
		let fee = auth_info.fee.as_ref().ok_or(SigVerifiableTxError::EmptyFee)?;

		let sequence = if !fee.payer.is_empty() {
			auth_info
				.signer_infos
				.first()
				.ok_or(SigVerifiableTxError::EmptySigners)?
				.sequence
		} else {
			auth_info
				.signer_infos
				.last()
				.ok_or(SigVerifiableTxError::EmptySigners)?
				.sequence
		};

		Ok(sequence)
	}
}
