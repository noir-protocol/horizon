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

use cosmos_sdk_proto::{cosmos::tx::v1beta1::Tx, prost::alloc::string::String};
use sp_std::vec::Vec;

#[derive(Clone, Debug)]
pub enum SigVerifiableTxError {
	EmptyAuthInfo,
	EmptyFee,
	EmptyTxBody,
	InvalidMsg,
}

pub trait SigVerifiableTx {
	fn get_signers(tx: &Tx) -> Result<Vec<String>, SigVerifiableTxError>;
}
