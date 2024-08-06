// This file is part of Horizon.

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
	cosmos::tx::v1beta1::{ModeInfo, Tx},
	prost::alloc::string::String,
	Any,
};
use sp_std::vec::Vec;

#[derive(Clone)]
pub struct SignerData {
	pub address: String,
	pub chain_id: String,
	pub account_number: u64,
	pub sequence: u64,
	pub pub_key: Any,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SignModeHandlerError {
	EmptyAuthInfo,
	EmptyTxBody,
	EmptyFee,
	EmptyModeInfo,
	DecodeTxError,
	InvalidMsg,
	InvalidMode,
	SerializeError,
}

pub trait SignModeHandler {
	fn get_sign_bytes(
		mode: &ModeInfo,
		data: &SignerData,
		tx: &Tx,
	) -> Result<Vec<u8>, SignModeHandlerError>;
}
